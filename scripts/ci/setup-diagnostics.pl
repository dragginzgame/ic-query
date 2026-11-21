#!/usr/bin/env perl
# Query's setup commands only: retain sanitized failures, never caches or payloads.
use strict;
use warnings;
use Cwd qw(abs_path);
use File::Find qw(find);
use File::Basename qw(basename);
use Fcntl qw(O_RDONLY O_NOFOLLOW S_ISREG);
use FindBin qw($Bin);
use lib "$Bin/../../vendor/toko-tooling/scripts/ci";
use SetupDiagnostics qw(run_setup excerpt);

my ($phase, @command) = @ARGV;
die "usage: setup-diagnostics.pl PHASE COMMAND [ARG...]\n"
    unless defined($phase) && $phase =~ /\A[a-z0-9-]{1,48}\z/ && @command;
die "RUNNER_TEMP must name an existing absolute directory\n"
    unless defined($ENV{RUNNER_TEMP}) && $ENV{RUNNER_TEMP} =~ m{\A/} && -d $ENV{RUNNER_TEMP};
my $temporary = abs_path($ENV{RUNNER_TEMP}) // die "RUNNER_TEMP must exist\n";
my $max_attempt_logs = 8;
my $max_attempt_bytes = 16384;
my $max_root_entries = 1000;

sub write_file {
    my ($path, $body) = @_;
    open my $out, '>', $path or die "cannot write sanitized evidence\n";
    print {$out} $body or die "cannot write sanitized evidence\n";
    close $out or die "cannot close sanitized evidence\n";
}

# Route evidence labels through the same redactor as log lines. A single
# admitted label fits in the pipe before the reader starts.
sub evidence_label {
    my ($label) = @_;
    return '[label omitted]' unless $label =~ m{\A[A-Za-z0-9_./-]{1,500}\z};
    pipe(my $reader, my $writer) or die "cannot create evidence label pipe\n";
    print {$writer} "$label\n" or die "cannot write evidence label\n";
    close $writer or die "cannot close evidence label\n";
    my $safe = excerpt($reader);
    close $reader;
    chomp $safe;
    return $safe;
}

my $status = run_setup(sub {
    my ($failed_status, $command_log) = @_;
    print "Setup phase=$phase status=$failed_status; sanitized output\n";
    print "| $_\n" for split /\n/, substr($command_log, -8192);

    my $artifact = "$temporary/ic-query-setup-diagnostics";
    my $output = "$artifact/$phase";
    die "artifact root is a symlink\n" if -l $artifact;
    mkdir($artifact, 0700) or die "cannot create artifact root\n" unless -d $artifact;
    mkdir($output, 0700) or die "cannot create fresh phase directory\n";
    my $tool = evidence_label(basename($command[0]));
    write_file("$output/summary.txt", "phase=$phase\nstatus=$failed_status\ntool=$tool\nNo command arguments, environment dump, caches or payloads are retained.\n");
    write_file("$output/command.txt", $command_log);

    my $collected = 0;
    # Fixed consumer-owned roots, including hidden install attempts. Never glob homes.
    for my $root ("$temporary/ic-query-tool-store", "$temporary/ic-query-security-bin") {
        last if $collected >= $max_attempt_logs;
        next unless -d $root && !-l $root;
        my $entries = 0;
        find({ no_chdir => 1, follow => 0, wanted => sub {
            my $path = $File::Find::name;
            die "attempt selection limit reached; remaining logs omitted\n" if ++$entries > $max_root_entries;
            if ($collected >= $max_attempt_logs) { $File::Find::prune = 1; return; }
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
            die "attempt log changed while opening\n"
                unless @opened && $opened[0] == $before[0] && $opened[1] == $before[1];

            my $log = excerpt($in, $max_attempt_bytes);
            close $in;
            my $label = evidence_label(substr($path, length($root) + 1));
            write_file("$output/attempt-" . ++$collected . ".txt", "selected_attempt=$label\n$log");
            print "Selected attempt log: $label (sanitized excerpt)\n";
            print "| $_\n" for split /\n/, substr($log, 0, 1024);
        }}, $root);
    }
    write_file("$output/collection.txt", "selected_attempt_logs=$collected\n"
        . "At most $max_attempt_logs logs, $max_attempt_bytes source bytes each; traversal capped at $max_root_entries entries per root.\n"
        . ($collected ? '' : "No selected attempt logs were available; command output is retained.\n"));
    print "Sanitized setup evidence saved for phase=$phase (attempt logs=$collected)\n";
    print "No selected attempt logs were available.\n" unless $collected;
}, @command);

print "Setup phase=$phase status=0; setup completed\n" unless $status;
exit $status;
