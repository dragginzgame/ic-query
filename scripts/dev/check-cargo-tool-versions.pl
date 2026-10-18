#!/usr/bin/env perl
# Shared companions: ci/tool-versions.env scripts/ci/StableVersion.pm scripts/ci/tool-catalog-records.pl
use strict;
use warnings;
use Cwd qw(abs_path);
use File::Basename qw(dirname);
use File::Temp qw(tempdir);
use FindBin qw($RealBin);
use lib "$RealBin/../ci";
use StableVersion qw(stable version_cmp);
use JSON::PP;
use POSIX qw(strftime);

my $root = abs_path("$RealBin/../..");
my ($consumer, $versions, $cargo_home, $json) = ($root, undef, $ENV{CARGO_HOME} // "$ENV{HOME}/.cargo", 0);
sub usage {
    return "Usage: check-cargo-tool-versions.pl [--consumer DIR] [--versions FILE] [--cargo-home DIR] [--json]\n" .
        "Compare shared Cargo pins, checkout selections and Cargo installation receipts with crates.io.\n" .
        "Requires Perl and curl >= 8.4; performs bounded HTTPS reads, never installs or changes selections.\n" .
        "Exit 0: current; 1: update/missing/yanked selection; 2: incomplete or invalid observation.\n";
}
sub read_file {
    my ($path) = @_;
    open my $fh, '<', $path or die "read $path: $!\n";
    local $/;
    return <$fh>;
}
sub receipt {
    my ($path) = @_;
    my $data = decode_json(read_file($path));
    ref($data) eq 'HASH' && ref($data->{installs}) eq 'HASH' or die "invalid Cargo receipt: $path\n";
    my %installed;
    for my $identity (sort keys %{$data->{installs}}) {
        $identity =~ /\A([a-zA-Z0-9][a-zA-Z0-9_-]*) (\S+) \(registry\+https:\/\/github.com\/rust-lang\/crates.io-index\)\z/
            or die "unsupported installation identity in $path: $identity\n";
        my ($name, $version) = ($1, $2);
        stable($version) && !exists $installed{$name} or die "non-stable or duplicate installation: $name\n";
        $installed{$name} = $version;
    }
    return \%installed;
}

my $status = eval {
    while (@ARGV) {
        my $arg = shift @ARGV;
        if ($arg eq '--help' || $arg eq '-h') { print usage(); exit 0; }
        if ($arg eq '--json') { $json = 1; next; }
        @ARGV && $arg =~ /\A--(?:consumer|versions|cargo-home)\z/ or die usage();
        my $value = shift @ARGV;
        if ($arg eq '--consumer') { $consumer = $value; }
        elsif ($arg eq '--versions') { $versions = $value; }
        else { $cargo_home = $value; }
    }
    $versions //= "$root/ci/tool-versions.env";
    for ($consumer, $cargo_home, $versions, $ENV{TMPDIR} // '/tmp') {
        length && !/[\r\n]/ or die "paths must not contain CR/LF or be empty\n";
        my $resolved = abs_path($_);
        !defined($resolved) || $resolved !~ /[\r\n]/ or die "resolved path contains CR/LF\n";
    }
    -d $consumer or die "consumer directory is missing: $consumer\n";
    # Reuse the exact same complete literal selection as the common installer.
    my %pins;
    open my $catalog, '-|', $^X, "$RealBin/../ci/tool-catalog-records.pl", 'rust', $versions
        or die "start catalog reader: $!\n";
    while (my $line = <$catalog>) {
        my ($name, $version) = split /\t/, $line;
        $name = lc $name; $name =~ tr/_/-/;
        $pins{$name} = $version;
    }
    close $catalog or die "invalid Cargo tool catalog: $versions\n";
    my $global_path = "$cargo_home/.crates2.json";
    my $global = -e $global_path ? receipt($global_path) : {};
    if (!-e $global_path && -e "$cargo_home/.crates.toml") { die "Cargo JSON receipt missing; cannot establish installed versions\n"; }
    my %names = (%pins, %$global);
    open my $curl, '-|', 'curl', '--disable', '--version' or die "start curl: $!\n";
    my $curl_version = do { local $/; <$curl> };
    close $curl or die "curl version probe failed\n";
    $curl_version =~ /^curl (\d+)\.(\d+)\./ && ($1 > 8 || ($1 == 8 && $2 >= 4))
        or die "bounded registry observations require curl >= 8.4\n";
    my $evidence = tempdir('cargo-tool-versions.XXXXXX', TMPDIR => 1, CLEANUP => 0);
    my (@rows, $result);
    $result = 0;
    for my $name (sort keys %names) {
        my $row = { tool => $name, pinned => $pins{$name}, global => $global->{$name}, local => undef };
        my $ok = eval {
            if (exists $pins{$name}) {
                my $binary = abs_path("$consumer/.tools/rust/common/bin/$name");
                if (defined $binary && -f $binary) {
                    my $installed = receipt(dirname(dirname($binary)) . '/.crates2.json');
                    exists $installed->{$name} or die "selected binary has no package receipt: $name\n";
                    $row->{local} = $installed->{$name};
                }
            }
            my $url = "https://crates.io/api/v1/crates/$name";
            my @command = ('curl', '--disable', '--silent', '--show-error', '--location',
                '--proto', '=https', '--proto-redir', '=https', '--tlsv1.2', '--connect-timeout', '10',
                '--max-time', '30', '--max-filesize', '8388608', '--user-agent',
                'dragginzgame-shared-tooling (https://github.com/dragginzgame/shared-tooling)',
                '--output', "$evidence/$name.json", '--stderr', "$evidence/$name.stderr",
                '--write-out', '%{http_code}', $url);
            open my $request, '-|', @command or die "start registry request: $!\n";
            my $http = do { local $/; <$request> };
            my $transport_ok = close $request;
            my $transport = $? >> 8;
            open my $observed, '>', "$evidence/$name.request" or die $!;
            print {$observed} "url=$url\nhttp=$http\ncurl_exit=$transport\n";
            close $observed or die $!;
            $transport_ok && $http eq '200' or die "registry unavailable (HTTP $http, curl $transport)\n";
            my $data = decode_json(read_file("$evidence/$name.json"));
            ref($data) eq 'HASH' && ref($data->{crate}) eq 'HASH' && ($data->{crate}{id} // '') eq $name &&
                ref($data->{versions}) eq 'ARRAY' or die "invalid registry identity\n";
            my %releases;
            for my $release (@{$data->{versions}}) {
                ref($release) eq 'HASH' && ($release->{crate} // '') eq $name &&
                    JSON::PP::is_bool($release->{yanked}) or die "invalid release metadata\n";
                next unless stable($release->{num});
                !exists $releases{$release->{num}} or die "duplicate release metadata\n";
                $releases{$release->{num}} = $release;
            }
            my @available = sort { version_cmp($a, $b) } grep { !$releases{$_}{yanked} } keys %releases;
            @available or die "no non-yanked stable release\n";
            $row->{latest} = $available[-1];
            $row->{rust_version} = $releases{$row->{latest}}{rust_version};
            my @findings;
            push @findings, 'local missing' if exists $pins{$name} && !defined $row->{local};
            for my $field (qw(pinned local global)) {
                my $version = $row->{$field}; next unless defined $version;
                exists $releases{$version} or die "selected $field version absent from registry: $version\n";
                push @findings, "$field yanked" if $releases{$version}{yanked};
                push @findings, "$field outdated" if version_cmp($version, $row->{latest}) < 0;
            }
            push @findings, 'local differs from pin' if defined $row->{local} && $row->{local} ne $row->{pinned};
            $row->{status} = @findings ? join(', ', @findings) : 'current';
            $result = 1 if @findings && !$result;
            1;
        };
        if (!$ok) { my $error = $@; chomp $error; $row->{status} = 'ERROR'; $row->{error} = $error; $result = 2; }
        push @rows, $row;
    }
    my $report = { observed_at => strftime('%Y-%m-%dT%H:%M:%SZ', gmtime), evidence => $evidence, tools => \@rows };
    if ($json) { print JSON::PP->new->canonical->pretty->encode($report); }
    else {
        my $format = "%-24s %-10s %-10s %-10s %-10s %s\n";
        printf $format, 'TOOL', 'PINNED', 'LOCAL', 'GLOBAL', 'LATEST', 'STATUS';
        for my $row (@rows) {
            printf $format, map { $row->{$_} // '-' } qw(tool pinned local global latest status);
            warn "$row->{tool}: $row->{error}\n" if exists $row->{error};
        }
        print "Checked $report->{observed_at}; registry evidence: $evidence\n";
        print "Versions come from installation receipts; use make rust-tools-check for byte/build admission.\n";
    }
    $result;
};
if ($@) { warn $@; exit 2; }
exit $status;
