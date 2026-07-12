#!/usr/bin/env perl

use strict;
use warnings;
use File::Find qw(find);

my @rust_files;
find(
    sub {
        return unless -f $_ && /[.]rs\z/;
        push @rust_files, $File::Find::name;
    },
    'crates',
);

my @violations;
for my $file (sort @rust_files) {
    open my $handle, '<', $file or die "error: cannot read $file: $!\n";
    local $/;
    my $source = <$handle>;
    close $handle or die "error: cannot close $file: $!\n";

    while ($source =~ /^(?:pub(?:\([^)]*\))?)\s+(?:struct|enum|trait)\s+([A-Za-z_][A-Za-z0-9_]*)/mg) {
        my $name = $1;
        my $declaration_offset = $-[0];
        my $prefix = substr $source, 0, $declaration_offset;
        my $valid = $prefix =~ m{
            (?:\A|\n)
            ///\n
            ///[ ]\Q$name\E\n
            ///\n
            (?:///[ ][^\n]+\n)+
            ///\n
            \n
            (?:\#[[][^]]*[]]\n)*
            \z
        }xms;
        next if $valid;

        my $line = 1 + (substr($source, 0, $declaration_offset) =~ tr/\n//);
        push @violations, "$file:$line: $name";
    }
}

if (@violations) {
    print STDERR "error: cross-module types must use the complete section-style doc block:\n";
    print STDERR "  empty line, type name, empty line, description, empty line\n";
    print STDERR "$_\n" for @violations;
    exit 1;
}

