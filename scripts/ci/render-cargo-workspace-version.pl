#!/usr/bin/env perl
use strict;
use warnings;

# Pure literal rendering, not TOML admission or a release transaction. Callers
# admit the source and independently validate the candidate before installing it.
@ARGV == 3 or die "usage: render-cargo-workspace-version.pl SOURCE PREVIOUS CANDIDATE\n";
my ($path, $previous, $candidate) = @ARGV;
my $semver = qr/(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)/;
for ($previous, $candidate) {
    /\A$semver\z/ or die "expected canonical stable version\n";
}
-f $path && !-l $path or die "manifest must be a regular file\n";
open my $input, '<', $path or die "$path: $!\n";
binmode $input or die "$path: $!\n";
my $text = do { local $/; <$input> };
close $input or die "$path: $!\n";
defined $text or die "empty manifest\n";

my ($in_package, $tables, $fields, $offset) = (0, 0, 0, 0);
my ($version, $start);
for my $line (split /(?<=\n)/, $text) {
    if ($line =~ /^[ \t]*\[/) {
        $in_package = $line =~ /^[ \t]*\[[ \t]*workspace\.package[ \t]*\][ \t]*(?:#[^\r\n]*)?(?:\r?\n|\z)/;
        $tables++ if $in_package;
    } elsif ($in_package && $line =~ /^[ \t]*version[ \t]*=/) {
        $fields++;
        $line =~ /^([ \t]*version[ \t]*=[ \t]*")($semver)("[ \t]*(?:#[^\r\n]*)?(?:\r?\n|\z))/
            or die "workspace.package.version must be a literal canonical stable version\n";
        ($version, $start) = ($2, $offset + length($1));
    }
    $offset += length($line);
}
$tables == 1 && $fields == 1
    or die "expected exactly one literal workspace.package table and version\n";
$version eq $previous
    or die "workspace version changed: expected $previous, found $version\n";
substr($text, $start, length($version), $candidate);
binmode STDOUT or die "stdout: $!\n";
print $text or die "manifest output: $!\n";
close STDOUT or die "manifest output: $!\n";
