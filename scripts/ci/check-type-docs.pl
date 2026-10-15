#!/usr/bin/env perl
# Shared companions: docs/type-docs.md
use strict;
use warnings;
use File::Find qw(find);
use Encode qw(decode FB_CROAK);

@ARGV or die "usage: check-type-docs.pl <source-directory-or-rust-file>...\n";
my %rust_files;
for my $root (@ARGV) {
    -l $root and die "error: source roots must not be symlinks: $root\n";
    if (-f $root && $root =~ /[.]rs\z/) { $rust_files{$root} = 1; next; }
    -d $root or die "error: source root is not a directory or Rust file: $root\n";
    # File::Find warns and continues when a subtree cannot be read. A partial
    # source inventory cannot establish that the selected documentation passes.
    local $SIG{__WARN__} = sub { die "error: source traversal failed: @_" };
    find({ no_chdir => 1, wanted => sub {
        return if -l $_;
        $rust_files{$_} = 1 if -f $_ && /[.]rs\z/;
    }}, $root);
}
keys %rust_files or die "error: source selection contains no Rust files\n";

# Preserve offsets/newlines while excluding literals and comments from declaration
# discovery. Documentation is checked against the original, unmodified source.
sub code_mask {
    my ($source) = @_;
    my $mask = $source;
    pos($source) = 0;
    while ($source =~ m{(//[^\n]*|/\*|(?:br|r)(\#*)"|b?"(?:\\.|[^"\\])*"|b?'(?:\\.|[^'\\])')}gcs) {
        my ($token, $hashes, $start, $end) = ($1, $2, $-[0], $+[0]);
        if ($token eq '/*') {
            my $depth = 1;
            while ($depth) {
                my $open = index($source, '/*', $end);
                my $close = index($source, '*/', $end);
                $close >= 0 or die "error: unterminated Rust block comment\n";
                if ($open >= 0 && $open < $close) { $depth++; $end = $open + 2; }
                else { $depth--; $end = $close + 2; }
            }
        } elsif (defined $hashes) {
            my $close = index($source, '"' . $hashes, $end);
            $close >= 0 or die "error: unterminated Rust raw string\n";
            $end = $close + 1 + length($hashes);
        }
        my $hidden = substr($source, $start, $end - $start);
        $hidden =~ s/[^\n]/ /g;
        substr($mask, $start, $end - $start) = $hidden;
        pos($source) = $end;
    }
    return $mask;
}

my @violations;
for my $file (sort keys %rust_files) {
    open my $handle, '<:raw', $file or die "error: cannot read $file: $!\n";
    local $/;
    my $source = <$handle>;
    close $handle or die "error: cannot close $file: $!\n";
    $source = decode('UTF-8', $source, FB_CROAK);
    my $mask = code_mask($source);
    while ($mask =~ /^[ \t]*pub(?:\([^)]*\))?[ \t]+(?:struct|enum|trait)[ \t]+(?:r#)?([_\p{XID_Start}][\p{XID_Continue}]*)/mg) {
        my $name = $1;
        my $offset = $-[0];
        my $prefix = substr($source, 0, $offset);
        # Nested attributes may span several lines. Walk balanced brackets in
        # the masked prefix so strings/comments cannot change attribute depth.
        my $before = substr($mask, 0, $offset);
        $before =~ /\s*\z/;
        my $cursor = $-[0];
        while ($cursor > 0 && substr($before, $cursor - 1, 1) eq ']') {
            my $end = $cursor;
            my $depth = 1;
            $cursor--;
            while ($cursor > 0 && $depth) {
                my $char = substr($before, --$cursor, 1);
                $depth++ if $char eq ']';
                $depth-- if $char eq '[';
            }
            $depth == 0 && $cursor > 0 && substr($before, $cursor - 1, 1) eq '#'
                or die "error: unsupported Rust attribute before $name in $file\n";
            $cursor--;
            $prefix = substr($source, 0, $cursor);
            $before = substr($before, 0, $cursor);
            $before =~ /\s*\z/;
            $cursor = $-[0];
        }
        my $valid = $prefix =~ m{
            (?:\A|\n)[ \t]*///\n
            [ \t]*///[ ]\Q$name\E\n
            [ \t]*///\n
            (?:[ \t]*///[ ][^\n]+\n)+
            [ \t]*///\n
            [ \t]*\n[ \t]*\z
        }xms;
        next if $valid;
        my $line = 1 + (substr($source, 0, $offset) =~ tr/\n//);
        push @violations, "$file:$line: $name";
    }
}
if (@violations) {
    print STDERR "error: selected cross-module types require section-style doc blocks:\n";
    print STDERR "$_\n" for @violations;
    exit 1;
}
