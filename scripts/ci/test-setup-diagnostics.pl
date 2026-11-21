#!/usr/bin/env perl
use strict;
use warnings;
use Test::More;
use File::Temp qw(tempdir);
use File::Path qw(make_path remove_tree);
use File::Find qw(find);
use Cwd qw(abs_path);
my $root = tempdir('query-setup-fixture.XXXXXX', TMPDIR => 1, CLEANUP => 0);
my $helper = abs_path('scripts/ci/setup-diagnostics.pl');
sub read_file {
    my ($path) = @_;
    open my $in, '<', $path or die "read fixture: $!";
    local $/; return <$in> // '';
}
sub run_case {
    my ($name, $program, $prepare) = @_;
    my $dir = "$root/$name";
    make_path($dir);
    $prepare->($dir) if $prepare;
    my $pid = fork(); die "fork: $!" unless defined $pid;
    if (!$pid) {
        $ENV{RUNNER_TEMP} = $dir;
        $ENV{QUERY_TEST_TOKEN} = 'fixture-known-sensitive-value';
        open STDOUT, '>', "$dir/output.txt" or die $!;
        open STDERR, '>&', \*STDOUT or die $!;
        exec $^X, $helper, 'early-setup', $^X, '-e', $program;
        die "exec: $!";
    }
    waitpid($pid, 0);
    return ($? >> 8, $dir, read_file("$dir/output.txt"));
}
my ($status, $dir, $console) = run_case('failure', q{
    use File::Path qw(make_path);
    $| = 1;
    my $base = "$ENV{RUNNER_TEMP}/ic-query-security-bin/.hidden-attempt";
    make_path($base);
    open my $log, '>', "$base/download.stderr" or die $!;
    print {$log} ("attempt warmup " x 750), "\n";
    print {$log} "curl: TLS handshake failed\nAuthorization: Bearer dummy-auth\n";
    print {$log} "https://user:dummy-password\@example.test/file?token=dummy-query\n";
    print {$log} "-----BEGIN PRIVATE KEY-----\ndummy-pem-material\n-----END PRIVATE KEY-----\n";
    print {$log} "$ENV{QUERY_TEST_TOKEN}\n";
    print {$log} ("attempt filler " x 425), "\nsetup output beyond the read ceiling\n";
    close $log;
    my $sensitive_dir = "$ENV{RUNNER_TEMP}/ic-query-tool-store/$ENV{QUERY_TEST_TOKEN}";
    make_path($sensitive_dir);
    open my $other, '>', "$sensitive_dir/failure.txt" or die $!;
    print {$other} "second selected attempt\n"; close $other;
    open my $private, '>', "$base/payload.txt" or die $!;
    print {$private} "unselected-payload-marker"; close $private;
    open my $outside, '>', "$ENV{RUNNER_TEMP}/outside.txt" or die $!;
    print {$outside} "symlink-target-marker"; close $outside;
    symlink "$ENV{RUNNER_TEMP}/outside.txt", "$base/install.log" or die $!;
    print "tool=fixture phase=download\n";
    print "ordinary diagnostic filler line\n" x 5000;
    print "x" x 9000, "fixture-long-secret\n";
    print STDERR "curl: request failed; see retained attempt\n";
    print STDERR "password=dummy-console-password\n";
    print STDERR "Bearer dummy-bare-bearer\n";
    print STDERR qq|{"payload":"dummy-labelled-payload"}\n|;
    print STDERR "::warning::fixture workflow directive\n";
    exit 37;
});
is($status, 37, 'original setup failure status survives collection');
like($console, qr/TLS handshake failed/, 'hidden attempt failure excerpt is surfaced');
like($console, qr/phase=early-setup status=37/, 'phase and original status are visible');
my $command = read_file("$dir/ic-query-setup-diagnostics/early-setup/command.txt");
like($command, qr/earlier output or oversized lines omitted/, 'command truncation is disclosed');
ok(length($command) <= 32100, 'shared command excerpt is bounded');
unlike($console . $command, qr/::warning::/, 'workflow directive is neutralized');
my @files;
find({no_chdir => 1, wanted => sub { push @files, $File::Find::name if -f $_ }}, "$dir/ic-query-setup-diagnostics");
is(scalar @files, 5, 'only summary, command, two selected logs and collection record retained');
my $body = join '', map { read_file($_) } @files;
like($body, qr/\.hidden-attempt\/download.stderr/, 'hidden attempt identity retained');
like($body, qr/selected_attempt=\[sensitive line omitted\]\nsecond selected attempt/, 'sensitive attempt label is omitted');
like($body, qr/selected_attempt_logs=2/, 'symlink log excluded');
like($body, qr/input truncated at byte limit/, 'attempt read ceiling is disclosed');
unlike($body . $console, qr/setup output beyond the read ceiling/, 'attempt output beyond the ceiling is excluded');
for my $private (qw(dummy-auth dummy-password dummy-query dummy-pem-material fixture-known-sensitive-value unselected-payload-marker symlink-target-marker fixture-long-secret dummy-console-password dummy-bare-bearer dummy-labelled-payload)) {
    ok(index($body . $console, $private) < 0, "private fixture marker excluded: $private");
}
ok(length($body) < 200000, 'artifact payload is bounded');
($status, $dir, $console) = run_case('missing', 'print STDERR "setup source unavailable\\n"; exit 19;');
is($status, 19, 'missing attempt logs preserve original status');
like(read_file("$dir/ic-query-setup-diagnostics/early-setup/collection.txt"), qr/No selected attempt logs/, 'missing attempt evidence explicit');
like($console, qr/setup source unavailable/, 'command diagnostic survives missing attempt logs');
($status, $dir, $console) = run_case('full-selection', 'exit 31;', sub {
    my ($dir) = @_;
    for my $index (1 .. 8) {
        my $attempt = "$dir/ic-query-tool-store/attempt-$index";
        make_path($attempt);
        open my $log, '>', "$attempt/install.log" or die $!;
        print {$log} "selected attempt $index\n";
        close $log or die $!;
    }
    my $unused = "$dir/ic-query-security-bin";
    make_path($unused);
    for my $index (1 .. 1001) {
        open my $entry, '>', "$unused/entry-$index" or die $!;
        close $entry or die $!;
    }
});
is($status, 31, 'full attempt selection preserves original failure');
like(read_file("$dir/ic-query-setup-diagnostics/early-setup/collection.txt"), qr/selected_attempt_logs=8/, 'full selection is recorded without scanning the next root');
unlike($console, qr/Diagnostic collection failed/, 'unused root cannot fail an already full selection');
($status, $dir, $console) = run_case('collector-failure', 'print STDERR "underlying setup failed\\n"; exit 23;', sub {
    my ($dir) = @_;
    open my $conflict, '>', "$dir/ic-query-setup-diagnostics" or die $!;
    close $conflict;
});
is($status, 23, 'collector failure never masks original status');
like($console, qr/Diagnostic collection failed; original setup status remains 23/, 'collector failure reported separately');
like($console, qr/underlying setup failed/, 'original diagnostic survives collector failure');
($status, $dir, $console) = run_case('success', 'print "successful command output\\n" x 5000; exit 0;');
is($status, 0, 'success remains success');
is($console, "Setup phase=early-setup status=0; setup completed\n", 'successful setup emits only the bounded phase summary');
ok(!-e "$dir/ic-query-setup-diagnostics", 'successful setup creates no failure artifact');
($status, $dir, $console) = run_case('signal', 'kill 15, $$; sleep 1;');
is($status, 143, 'signal failure is retained');

my $workflow = read_file('.github/workflows/ci.yml');
my $setup_steps = join "\n", grep { /^        id: setup_artifact$/m }
    split /(?=^      - )/m, $workflow;
my $uploads = () = $setup_steps =~ /uses: actions\/upload-artifact\@cf430e030ddbb5b0abf93d22962f4752f3646cd9/g;
is($uploads, 2, 'scaffold and native jobs retain their own artifacts');
my $retention = () = $setup_steps =~ /retention-days: 7/g;
is($retention, 2, 'both uploads have finite retention');
my $missing = () = $setup_steps =~ /if-no-files-found: error/g;
is($missing, 2, 'missing upload evidence cannot silently pass');
unlike($workflow, qr/continue-on-error:/, 'workflow does not mask setup/collection/upload failures');
like($workflow, qr/if: failure\(\).*steps\.host_setup\.outcome/, 'upload is reachable after early setup failure');
like($workflow, qr/artifact-url/, 'CI summary links the retained artifact when uploaded');

done_testing();
if (Test::More->builder->is_passing) { remove_tree($root); }
else { diag("Failure fixture retained: $root"); }
