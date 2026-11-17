#!/usr/bin/env perl
# Query's setup commands only: retain sanitized failures, never caches or payloads.
use strict;
use warnings;
use Cwd qw(abs_path);
use File::Find qw(find);
use File::Basename qw(basename);
use Fcntl qw(O_RDONLY O_NOFOLLOW S_ISREG);

my ($phase, @command) = @ARGV;
die "usage: setup-diagnostics.pl PHASE COMMAND [ARG...]\n"
    unless defined($phase) && $phase =~ /\A[a-z0-9-]{1,48}\z/ && @command;
die "RUNNER_TEMP must name an existing absolute directory\n"
    unless defined($ENV{RUNNER_TEMP}) && $ENV{RUNNER_TEMP} =~ m{\A/} && -d $ENV{RUNNER_TEMP};
my $temporary = abs_path($ENV{RUNNER_TEMP}) // die "RUNNER_TEMP must exist\n";
my @secrets = sort { length($b) <=> length($a) }
    grep { defined($_) && length($_) >= 4 }
    map { $ENV{$_} } grep { /TOKEN|SECRET|PASSWORD|CREDENTIAL|PRIVATE_KEY|ACCESS_KEY/i } keys %ENV;
my $limit = 65536;
my ($tail, $omitted, $in_key) = ('', 0, 0);
sub sanitize {
    my ($line) = @_;
    $line =~ s/\e\[[0-?]*[ -\/]*[\@-~]//g;
    if ($line =~ /-----BEGIN .*PRIVATE KEY-----/) { $in_key = 1; }
    if ($in_key) {
        $in_key = 0 if $line =~ /-----END .*PRIVATE KEY-----/;
        return "[private key material omitted]\n";
    }
    for my $secret (@secrets) { $line =~ s/\Q$secret\E/[redacted]/g; }
    return "[sensitive field or environment assignment omitted]\n"
        if $line =~ /(?:authorization|cookie|password|passwd|secret|credential|token|private[_ -]?key|access[_ -]?key)\s*["']?\s*[:=]/i
        || $line =~ /^\s*[A-Za-z_][A-Za-z0-9_]*=/;
    $line =~ s/\b(?:gh[pousr]_[A-Za-z0-9_]+|github_pat_[A-Za-z0-9_]+|AKIA[A-Z0-9]{16})\b/[redacted]/g;
    $line =~ s/\bBearer\s+\S+/Bearer [redacted]/ig;
    $line =~ s{(https?://)[^\s/\@]+\@}{${1}[redacted]\@}g;
    $line =~ s{(https?://[^\s?]+)\?\S+}{$1?[query omitted]}g;
    $line =~ s/[\x00-\x08\x0b-\x1f\x7f]/[control]/g;
    return $line;
}
sub retain {
    my ($line) = @_;
    $tail .= sanitize($line);
    if (length($tail) > $limit) {
        $tail = substr($tail, -$limit);
        $tail =~ s/^[^\n]*\n//;
        $omitted = 1;
    }
}
# Consume the entire stream to preserve child status, retaining bounded complete lines.
# Oversized lines are dropped as a whole, so partial credentials cannot leak.
sub consume {
    my ($handle) = @_;
    my ($pending, $dropping) = ('', 0);
    while (1) {
        my $count = sysread($handle, my $chunk, 4096);
        die "diagnostic stream read failed\n" unless defined $count;
        last unless $count;
        for my $piece (split /(?<=\n)/, $chunk) {
            $pending .= $piece unless $dropping;
            if (length($pending) > 8192) { $pending = ''; $dropping = 1; }
            if ($piece =~ /\n\z/) {
                if ($dropping) { retain("[oversized diagnostic line omitted]\n"); }
                else { retain($pending); }
                ($pending, $dropping) = ('', 0);
            }
        }
    }
    retain($dropping ? "[oversized diagnostic line omitted]\n" : "$pending\n") if $dropping || length($pending);
}
pipe(my $reader, my $writer) or die "cannot create diagnostic pipe\n";
my $pid = fork();
die "cannot start setup command\n" unless defined $pid;
if (!$pid) {
    close $reader;
    open STDOUT, '>&', $writer or exit 125;
    open STDERR, '>&', $writer or exit 125;
    close $writer;
    exec { $command[0] } @command or do { print STDERR "setup executable could not start: $!\n"; exit 127; };
}
close $writer;
my $capture_error = '';
eval { consume($reader); 1 } or $capture_error = 'diagnostic capture failed';
close $reader;
waitpid($pid, 0);
my $wait = $?;
my $status = ($wait & 127) ? 128 + ($wait & 127) : $wait >> 8;
print "Setup phase=$phase status=$status; sanitized output", ($omitted ? ' (tail truncated)' : ''), "\n";
print "| $_\n" for split /\n/, substr($tail, -8192);
print "| [no command output]\n" unless length($tail);
print STDERR "$capture_error\n" if $capture_error;
exit($capture_error ? 1 : 0) unless $status;

my $artifact = "$temporary/ic-query-setup-diagnostics";
my $output = "$artifact/$phase";
my $collected = 0;
my $collection_error = '';
eval {
    die "artifact root is a symlink\n" if -l $artifact;
    mkdir($artifact, 0700) or die "cannot create artifact root\n" unless -d $artifact;
    mkdir($output, 0700) or die "cannot create fresh phase directory\n";
    sub write_file {
        my ($path, $body) = @_;
        open my $out, '>', $path or die "cannot write sanitized evidence\n";
        print {$out} $body or die "cannot write sanitized evidence\n";
        close $out or die "cannot close sanitized evidence\n";
    }
    write_file("$output/summary.txt", "phase=$phase\nstatus=$status\ntool=" . basename($command[0]) . "\ncommand_output_truncated=$omitted\nNo command arguments, environment dump, caches or payloads are retained.\n");
    write_file("$output/command.txt", length($tail) ? $tail : "[no command output]\n");
    # Fixed consumer-owned roots, including hidden install attempts. Never glob homes.
    for my $root ("$temporary/ic-query-tool-store", "$temporary/ic-query-security-bin") {
        next unless -d $root && !-l $root;
        my $entries = 0;
        find({ no_chdir => 1, follow => 0, wanted => sub {
            my $path = $File::Find::name;
            die "attempt selection limit reached; remaining logs omitted\n" if ++$entries > 1000;
            if ($collected >= 8) { $File::Find::prune = 1; return; }
            return if -l $path;
            my $name = basename($path);
            if (-d $path) {
                $File::Find::prune = 1 if $name =~ /\A(?:build|target|payload|registry|src|bin|\.git)\z/;
                return;
            }
            return unless $name =~ /\A(?:install\.log|download\.stderr|failure\.txt|source\.log|status)\z/;
            my @before = lstat($path);
            return unless @before && S_ISREG($before[2]);
            my $physical = abs_path($path) // return;
            die "attempt log escaped selected root\n" unless index($physical, "$root/") == 0;
            sysopen(my $in, $path, O_RDONLY | O_NOFOLLOW) or die "cannot open selected attempt log\n";
            my @opened = stat($in);
            die "attempt log changed while opening\n" unless $opened[0] == $before[0] && $opened[1] == $before[1];
            my $size = $opened[7];
            # A leading window avoids beginning in the middle of a secret/PEM block.
            my $n = sysread($in, my $raw, 16384);
            die "cannot read selected attempt log\n" unless defined $n;
            close $in;
            $raw =~ s/[^\n]*\z// if $size > $n;
            local $/ = "\n";
            ($tail, $omitted, $in_key) = ('', 0, 0);
            for my $line (split /(?<=\n)/, $raw) {
                retain(length($line) > 8192 ? "[oversized diagnostic line omitted]\n" : $line);
            }
            $tail .= "[attempt log truncated at 16384 bytes]\n" if $size > $n;
            my $relative = substr($path, length($root) + 1);
            $relative =~ s/[^A-Za-z0-9_.\/-]/_/g;
            $relative = sanitize($relative);
            write_file("$output/attempt-" . ++$collected . ".txt", "selected_attempt=$relative\n$tail");
            print "Selected attempt log: $relative (sanitized excerpt)\n";
            print "| $_\n" for split /\n/, substr($tail, 0, 1024);
        }}, $root);
    }
    write_file("$output/collection.txt", "selected_attempt_logs=$collected\nAt most 8 logs, 16384 source bytes each; traversal capped at 1000 entries per root.\n" . ($collected ? '' : "No selected attempt logs were available; command output is retained.\n"));
    1;
} or $collection_error = $@ || 'unknown collector failure';
if ($collection_error) {
    print STDERR "Setup diagnostics collection failed: ", sanitize($collection_error);
} else {
    print "Sanitized setup evidence: $output (attempt logs=$collected)\n";
    print "No selected attempt logs were available.\n" unless $collected;
}
exit $status;
