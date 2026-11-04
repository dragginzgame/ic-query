#!/usr/bin/env perl
# Shared companions: scripts/ci/ValidationPlan.pm ci/release-validation.json ci/release-validation-scopes.json ci/release-validation-hosts.json ci/cargo-release-groups.json
use strict;
use warnings;
use FindBin qw($RealBin);
use lib "$RealBin/../ci";
use ValidationPlan qw(json digest document policy relative_path keys_are);
use JSON::PP;
use Digest::SHA qw(sha256_hex);
use Cwd qw(abs_path);

my ($operation, $root, @args) = @ARGV;
$root = abs_path($root // '') // die "missing Query source root\n";
chdir $root or die "change source root: $!\n";
sub capture {
    open my $in, '-|', @_ or die "start observation: $!\n";
    binmode $in;
    my $raw = '';
    while (1) {
        my $size = read $in, my $chunk, 65536;
        defined $size or die "read observation: $!\n";
        last unless $size;
        $raw .= $chunk;
        length($raw) <= 4194304 or die "Query observation exceeds 4 MiB\n";
    }
    close $in or die "Query observation failed: @_\n";
    return $raw;
}
sub inventory { split /\0/, capture('git', 'ls-files', '-z', '--') }
sub mapping {
    return sha256_hex(join '', map { "$_\0".digest($root, $_)."\n" }
        grep { $_ ne 'ci/release-validation-hosts.json' &&
            m{\A(?:Makefile\z|rust-toolchain\.toml\z|\.shared-tooling\.snapshot\z|ci/|scripts/|make/|\.github/)} }
        inventory());
}
if ($operation eq 'context') {
    my ($purpose) = @args;
    $purpose =~ /\A(?:release|qualification)\z/ or die "invalid validation purpose\n";
    my $host = capture('uname', '-s'); chomp $host;
    my $arch = capture('uname', '-m'); chomp $arch;
    if ($host eq 'Darwin') {
        my $version = capture('sw_vers', '-productVersion');
        $version =~ /\A(\d+)\./ or die "cannot observe macOS major\n";
        $host .= "-$1";
    }
    my %environment = map { $_ => $ENV{$_} } grep {
        /\A(?:RUSTFLAGS|RUSTDOCFLAGS|RUSTUP_TOOLCHAIN|RUSTC|RUSTC_WRAPPER|RUSTC_WORKSPACE_WRAPPER|CARGO_ENCODED_RUSTFLAGS|CARGO_BUILD_.*|CARGO_TARGET_.*|CARGO_PROFILE_.*|IC_TOOL_PINS|IC_TOOLS|FORMAT_CARGO|SHARED_TOOLING_ROOT)\z/
    } keys %ENV;
    print json({schema=>1, purpose=>$purpose, host=>"$host/$arch", mapping=>mapping(),
        source=>$ENV{RELEASE_SOURCE}, group=>$ENV{RELEASE_GROUP}, environment=>\%environment,
        umask=>capture('sh', '-c', 'umask'), cargo=>capture('cargo', '-vV'), rustc=>capture('rustc', '-vV')});
} elsif ($operation eq 'plan') {
    my ($directory, $mode) = @args;
    $mode =~ /\A(?:release|qualification|workspace)\z/ or die "invalid plan mode\n";
    my $context = document($root, "$directory/context.json");
    my $baseline = document($root, "$directory/baseline.json");
    my $hosts = document($root, 'ci/release-validation-hosts.json');
    keys_are($hosts, qw(schema hosts));
    json($hosts->{schema}) eq "1\n" && ref($hosts->{hosts}) eq 'HASH' or die "invalid host qualification policy\n";
    my $roster = policy($root, 'ci/release-validation.json');
    my $scopes = document($root, 'ci/release-validation-scopes.json');
    keys_are($scopes, qw(schema groups));
    json($scopes->{schema}) eq "1\n" or die "invalid Query scope schema\n";
    keys_are($scopes->{groups}, @{$roster->{groups}});
    my %targets = map { $_=>1 } @{$roster->{stages}};
    for my $selection (values %{$scopes->{groups}}) {
        keys_are($selection, 'omit');
        ref($selection->{omit}) eq 'HASH' or die "invalid Query omissions\n";
        for my $target (keys %{$selection->{omit}}) {
            $targets{$target} or die "unknown Query omission: $target\n";
            my $reason = $selection->{omit}{$target};
            defined($reason) && !ref($reason) && $reason =~ /\S/ && $reason !~ /[\x00-\x1f\x7f]/
                or die "Query omission requires a single-line reason\n";
        }
    }
    my $group = $ENV{RELEASE_GROUP} // '';
    grep($_ eq $group, @{$roster->{groups}}) or die "unknown Query release group\n";
    $context->{source} eq $ENV{RELEASE_SOURCE} && $context->{group} eq $group &&
        $context->{purpose} eq ($mode eq 'release' ? 'release' : 'qualification')
        or die "validation context differs from intent\n";
    my $config = JSON::PP->new->decode(capture('git', 'show', "$ENV{RELEASE_SOURCE}:ci/cargo-release-groups.json"));
    my ($owner) = grep { $_->{name} eq $group } @{$config->{groups}};
    defined $owner or die "group absent from Cargo policy\n";
    my $members = join '|', map { quotemeta $_ } @{$owner->{members}};
    my @configuration_inputs;
    my $configuration = $context->{environment};
    if (defined(my $pins = $configuration->{IC_TOOL_PINS})) {
        $pins =~ s{\A\Q$root\E/}{};
        relative_path($pins);
        digest($root, $pins);
        push @configuration_inputs, $pins;
    }
    my $reason = 'Complete product and reverse-consumer qualification';
    my $narrow = $mode eq 'qualification';
    if ($mode eq 'release') {
        my $qualified = ($hosts->{hosts}{$context->{host}} // '') eq $context->{mapping};
        if (!$qualified) { $reason = 'Full gate: this host has no matching mapping qualification'; }
        elsif (!$baseline->{available}) { $reason = 'Full gate: no annotated ancestor release baseline'; }
        else {
            $narrow = 1;
            for my $path (@{$baseline->{changes}}) {
                next if $path =~ m{\Acrates/(?:$members)/(?:src/|tests/|README\.md\z)} ||
                    $path =~ m{\Areleases/\Q$group\E/(?:CHANGELOG\.md|\d+\.\d+\.md)\z};
                $narrow = 0;
                $reason = "Full gate: shared, dependency, other-owner or unknown input changed ($path)";
                last;
            }
        }
        for my $name (keys %{$context->{environment}}) {
            if ($name =~ /\A(?:RUSTFLAGS|RUSTDOCFLAGS|RUSTC(?:_.*)?|CARGO_ENCODED_RUSTFLAGS|CARGO_BUILD_.*|CARGO_TARGET_.*|CARGO_PROFILE_.*)\z/ &&
                length($context->{environment}{$name})) {
                $narrow = 0;
                $reason = "Full gate: nondefault compiler/build configuration ($name)";
            }
        }
        my %defaults = (IC_TOOL_PINS=>"$root/ci/ic-tools.tsv",
            IC_TOOLS=>'didc ic-wasm icp quill wasm-opt', FORMAT_CARGO=>'cargo', SHARED_TOOLING_ROOT=>$root);
        for my $name (sort keys %defaults) {
            if (defined($configuration->{$name}) && $configuration->{$name} ne $defaults{$name}) {
                $narrow = 0;
                $reason = "Full gate: nondefault tool selection ($name)";
            }
        }
        if ($context->{umask} !~ /\A00[02]2\n\z/) {
            $narrow = 0;
            $reason = 'Full gate: nondefault process umask';
        }
    }
    $narrow = 0 if $mode eq 'workspace';
    my $omissions = $scopes->{groups}{$group}{omit};
    my @stages = map {
        my $omit = $narrow && exists $omissions->{$_};
        {target=>$_, required=>$omit ? JSON::PP::false : JSON::PP::true,
         reason=>$omit ? $omissions->{$_} : $reason}
    } @{$roster->{stages}};
    my @inputs = (inventory(), @configuration_inputs, "$directory/context.json", "$directory/baseline.json");
    my %seen;
    @inputs = grep { !$seen{$_}++ } @inputs;
    relative_path($_) for @inputs;
    print json({schema=>1, source=>$ENV{RELEASE_SOURCE}, group=>$group,
        scope=>$narrow ? 'group' : 'workspace', policy=>'ci/release-validation.json',
        inputs=>\@inputs, stages=>\@stages});
} elsif ($operation eq 'custody-paths') {
    my ($directory) = @args;
    my $proof = document($root, "$directory/proof.json");
    my @paths = (map { "$directory/$_" } qw(plan.json context.json baseline.json proof.json));
    push @paths, map { "$proof->{run}/$_" } sort keys %{$proof->{files}};
    for my $path (@paths) {
        digest($root, $path); # Shared checks regular contained routes before shell custody.
        print "$path\0";
    }
} else { die "unknown Query validation operation\n"; }
