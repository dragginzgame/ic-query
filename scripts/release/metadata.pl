#!/usr/bin/env perl
# Shared companions: scripts/ci/cargo-release-group.sh scripts/ci/finalize-release-changelog.awk
use strict;
use warnings;
use File::Basename qw(dirname);
use File::Path qw(make_path);
use File::Temp qw(tempfile tempdir);
use Cwd qw(getcwd);
use JSON::PP qw(encode_json decode_json);

# Consumer-owned metadata rendering and source admission; no Git mutations or setup.
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
my $group = $ENV{RELEASE_GROUP} // '';
$group =~ /\A(?:query|cli)\z/ or die "select RELEASE_GROUP=query or RELEASE_GROUP=cli\n";
my $source_selection = $mode eq 'preflight' ? undef : $ENV{RELEASE_SOURCE};
my $plan;
{
    local $ENV{RELEASE_COMMIT} = '';
    local $ENV{CARGO_NET_OFFLINE} = 'true';
    local $ENV{RUSTUP_AUTO_INSTALL} = '0';
    my $root = getcwd();
    my $temporary;
    if (defined $source_selection) {
        $source_selection =~ /\A[0-9a-f]{40,64}\z/ or die "invalid validated source identity\n";
        $temporary = tempdir('ic-query-group-source.XXXXXX', TMPDIR => 1, CLEANUP => 1);
        open my $archive, '-|', 'git', 'archive', '--format=tar', $source_selection or die "cannot read source archive\n";
        open my $tar, '|-', 'tar', '-xf', '-', '-C', $temporary or die "cannot extract source archive\n";
        my $buffer;
        while (read($archive, $buffer, 65536)) { print {$tar} $buffer or die "cannot retain source archive\n"; }
        close $archive or die "source archive failed\n";
        close $tar or die "source extraction failed\n";
        chdir $temporary or die "cannot select source archive\n";
    }
    open my $reader, '-|', 'bash', "$root/scripts/ci/cargo-release-group.sh", 'plan', $group
        or die "cannot observe canonical release group: $!\n";
    my $json = do { local $/; <$reader> };
    close $reader or die "release group admission failed\n";
    chdir $root or die "cannot restore release root\n";
    $plan = decode_json($json);
}
my @files = @{$plan->{files}};
my %manifests = map { $_->{path} => $_ } @{$plan->{manifests}};
my $ledger = $plan->{changelog};
my $detail = "releases/$group/$minor.md";
grep($_ eq $detail, @files) or die "group policy must own detailed release notes $detail\n";
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
    $plan->{previous} eq $previous or die "group differs from selected base version\n";
    check_pending_heading($ledger, read_file($ledger), "[$version]");
    check_pending_heading($detail, read_file($detail), $version);
    exit;
}
my $evidence = shift @ARGV // die "missing validation evidence directory\n";
my ($editor, $max_bytes) = @ENV{qw(CARGO_RELEASE_METADATA_EDITOR CARGO_RELEASE_METADATA_MAX_BYTES)};
defined($editor) && $editor =~ m{\A/} && -f $editor && -x $editor && !-l $editor
    or die "select an absolute prepared IC Build release metadata editor\n";
defined($max_bytes) && $max_bytes =~ /\A[1-9][0-9]*\z/
    or die "select a positive Cargo release metadata input byte allowance\n";
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

my $policy_path = 'ci/cargo-release-groups.json';
my $policy_before = read_git_file("$source:$policy_path");
my $policy_current = $mode eq 'check-commit' ? read_git_file("$ENV{RELEASE_COMMIT}:$policy_path")
    : $mode eq 'check-index' ? read_git_file(":$policy_path") : read_file($policy_path);
$policy_before eq $policy_current or die "release group policy differs from validated source\n";

sub validated_before {
    my ($path) = @_;
    my $before = read_file("$evidence/before/$path");
    my $committed = read_git_file("$source:$path");
    $before eq $committed or die "retained inputs differ from validated source: $path\n";
    return $before;
}

sub expected {
    my ($path, $text) = @_;
    if (exists $manifests{$path}) {
        # Render an owned copy of the authenticated bytes, not a reopened input.
        my ($manifest, $manifest_path) = tempfile('ic-query-render-source.XXXXXX', TMPDIR => 1, UNLINK => 1);
        binmode $manifest or die "cannot select manifest byte mode: $!\n";
        print {$manifest} $text or die "cannot retain manifest input: $!\n";
        close $manifest or die "cannot close manifest input: $!\n";
        my ($selection, $selection_path) = tempfile('ic-query-render-selection.XXXXXX', TMPDIR => 1, UNLINK => 1);
        print {$selection} encode_json({
            version => $manifests{$path}->{version},
            dependencies => $manifests{$path}->{dependencies},
        }) or die "cannot retain manifest selection: $!\n";
        close $selection or die "cannot close manifest selection: $!\n";
        open my $renderer, '-|', $editor, '--max-input-bytes', $max_bytes,
            'group-manifest', $manifest_path, $previous, $version, $selection_path
            or die "cannot run IC Build manifest editor: $!\n";
        $text = do { local $/; <$renderer> };
        close $renderer or die "workspace version rendering failed\n";
    } elsif ($path eq 'Cargo.lock') {
        open my $lock, '-|', $editor, '--max-input-bytes', $max_bytes,
            'lock', "$evidence/before/$path", $previous, $version, @{$plan->{members}}
            or die "cannot run IC Build lockfile editor: $!\n";
        local $/;
        $text = <$lock>;
        close $lock or die "local lockfile transformation failed\n";
    } elsif ($path eq $ledger) {
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
    my ($primary) = grep { /\Q$plan->{primary}\E\/Cargo\.toml\z/ } @files;
    defined $primary or die "selected primary manifest is missing\n";
    replace_file($_, $prepared{$_}) for (grep($_ ne $primary, @files), $primary);
}
