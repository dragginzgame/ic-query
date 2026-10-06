#!/usr/bin/env perl
use strict;
use warnings;
use File::Basename qw(dirname);
use File::Path qw(make_path);
use File::Temp qw(tempfile);

# Consumer-owned metadata transformations; no dependency resolution or Git effects.
sub read_file {
    my ($path) = @_;
    open my $file, '<', $path or die "cannot read $path: $!\n";
    local $/;
    return <$file>;
}

sub check_pending_heading {
    my ($path, $text, $heading) = @_;
    $text =~ /^## (.+)$/m && $1 eq $heading
        or die "top pending heading in $path must select $heading\n";
}

my $mode = shift @ARGV // '';
my ($previous, $version, $date) = @ENV{qw(RELEASE_PREVIOUS RELEASE_VERSION RELEASE_DATE)};
for ($previous, $version) {
    defined && /\A(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\z/
        or die "missing or invalid release version selection\n";
}
defined($date) && $date =~ /\A[0-9]{4}-[0-9]{2}-[0-9]{2}\z/
    or die "missing or invalid UTC release date\n";
my $minor = $version =~ s/\.[0-9]+\z//r;
my $previous_minor = $previous =~ s/\.[0-9]+\z//r;
my @files = ('Cargo.toml', 'Cargo.lock', 'README.md', 'docs/library-usage.md',
    'CHANGELOG.md', "docs/changelog/$minor.md");
if ($mode eq 'files') {
    print "$_\0" for @files;
    exit;
}
if ($mode eq 'check-paths') {
    my %owned = map { $_ => 1 } @files;
    local $/ = "\0";
    while (my $path = <STDIN>) {
        $path =~ s/\0\z// or die "unterminated changed path\n";
        exists $owned{$path} or die "unrelated changed path: $path\n";
    }
    exit;
}
if ($mode eq 'preflight') {
    open my $reader, '-|', 'bash', 'scripts/ci/read-cargo-workspace-version.sh',
        '--stable', 'Cargo.toml' or die "cannot run shared workspace version reader: $!\n";
    my $current = do { local $/; <$reader> };
    close $reader or die "workspace version observation failed\n";
    $current eq "$previous\n"
        or die "manifest differs from the selected base version\n";
    check_pending_heading('CHANGELOG.md', read_file('CHANGELOG.md'), "[$version]");
    my $detail = "docs/changelog/$minor.md";
    check_pending_heading($detail, read_file($detail), $version);
    exit;
}
my $evidence = shift @ARGV // die "missing validation evidence directory\n";
my $source = $ENV{RELEASE_SOURCE} // die "missing validated source identity\n";
$source =~ /\A[0-9a-f]{40,64}\z/ or die "invalid validated source identity\n";

sub read_git_file {
    my ($reference) = @_;
    open my $git, '-|', 'git', 'show', $reference
        or die "cannot read $reference: $!\n";
    local $/;
    my $committed = <$git>;
    close $git or die "Git does not contain $reference\n";
    return $committed;
}

sub validated_before {
    my ($path) = @_;
    my $before = read_file("$evidence/before/$path");
    my $committed = read_git_file("$source:$path");
    $before eq $committed or die "retained inputs differ from validated source: $path\n";
    return $before;
}

sub expected {
    my ($path, $text) = @_;
    if ($path eq 'Cargo.toml') {
        my $count = $text =~ s/(^\[workspace\.package\]\n(?:(?!^\[).)*?^version = ")\Q$previous\E("$)/$1$version$2/ms;
        $count == 1 or die "ambiguous workspace package version\n";
        $count = $text =~ s/(^ic-query = \{ path = "crates\/ic-query", version = ")\Q$previous\E(")/$1$version$2/m;
        $count == 1 or die "missing owned workspace dependency version\n";
    } elsif ($path eq 'Cargo.lock') {
        open my $lock, '-|', $^X, 'scripts/ci/rewrite-local-lock-versions.pl',
            "$evidence/before/$path", $previous, $version, 'ic-query', 'ic-query-cli'
            or die "cannot run shared lockfile transformer: $!\n";
        local $/;
        $text = <$lock>;
        close $lock or die "local lockfile transformation failed\n";
    } elsif ($path eq 'README.md' || $path eq 'docs/library-usage.md') {
        ($text =~ s/(^ic-query = \{ version = ")\Q$previous_minor\E(")/$1$minor$2/gm) > 0
            or die "missing current dependency examples in $path\n";
    } elsif ($path eq 'CHANGELOG.md') {
        check_pending_heading($path, $text, "[$version]");
        my $baseline = "$evidence/before/$path";
        open my $awk, '-|', 'awk', '-v', "version=$version", '-v', "date=$date",
            '-f', 'scripts/ci/finalize-release-changelog.awk', $baseline
            or die "cannot run canonical changelog finalizer: $!\n";
        local $/;
        $text = <$awk>;
        close $awk or die "canonical changelog finalization failed\n";
    } else {
        check_pending_heading($path, $text, $version);
        ($text =~ s/^## \Q$version\E$/## $version - $date/m) == 1
            or die "ambiguous detailed changelog candidate\n";
    }
    return $text;
}

sub replace_file {
    my ($path, $text) = @_;
    my ($file, $temporary) = tempfile('.ic-query-release.XXXXXX', DIR => dirname($path));
    my $success = eval {
        print {$file} $text or die "cannot write $temporary: $!\n";
        close $file or die "cannot close $temporary: $!\n";
        chmod((stat($path))[2] & 0777, $temporary) or die "cannot preserve $path mode: $!\n";
        rename $temporary, $path or die "cannot replace $path: $!\n";
        1;
    };
    my $error = $@;
    unlink $temporary if !$success && -e $temporary;
    die $error unless $success;
}

if ($mode eq 'capture') {
    for my $path (@files) {
        my $target = "$evidence/before/$path";
        make_path(dirname($target));
        open my $file, '>', $target or die "cannot retain $target: $!\n";
        print {$file} read_file($path) or die "cannot write $target: $!\n";
        close $file or die "cannot close $target: $!\n";
    }
    expected($_, validated_before($_)) for @files;
    exit;
}
$mode =~ /\A(?:prepare|check|check-commit|check-index|admit)\z/ or die "unknown metadata operation $mode\n";
my $release_commit;
if ($mode eq 'check-commit') {
    $release_commit = $ENV{RELEASE_COMMIT} // '';
    $release_commit =~ /\A[0-9a-f]{40,64}\z/ or die "missing selected release commit\n";
}
my %prepared;
for my $path (@files) {
    my $before = validated_before($path);
    $prepared{$path} = expected($path, $before);
    my $current = $mode eq 'check-commit' ? read_git_file("$release_commit:$path")
        : $mode eq 'check-index' ? read_git_file(":$path") : read_file($path);
    if ($mode =~ /\Acheck(?:-commit|-index)?\z/) {
        $current eq $prepared{$path} or die "prepared metadata differs from validated inputs: $path\n";
    } else {
        $current eq $before || $current eq $prepared{$path}
            or die "release payload conflict: $path\n";
    }
}
if ($mode eq 'prepare') {
    # Publish the canonical version last. An interrupted earlier file can be
    # reconciled only against the retained validated before/after payloads.
    replace_file($_, $prepared{$_}) for (grep($_ ne 'Cargo.toml', @files), 'Cargo.toml');
}
