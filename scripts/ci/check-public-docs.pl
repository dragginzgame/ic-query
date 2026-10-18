#!/usr/bin/env perl
# Query-owned item identities for its temporary public-documentation debt.
use strict;
use warnings;
use JSON::PP qw(decode_json encode_json);
use Encode qw(decode FB_CROAK LEAVE_SRC);
use Getopt::Long qw(GetOptionsFromArray);

sub read_bytes {
    my ($path) = @_;
    open my $fh, '<:raw', $path or die "read $path: $!\n";
    local $/;
    my $bytes = <$fh>;
    close $fh or die "close $path: $!\n";
    return $bytes;
}

# Skip Rust literals and nested comments before interpreting owner delimiters.
sub tokens {
    my ($source) = @_;
    my @tokens;
    while ($source =~ m{//[^\n]*|/\*|(?:br|r)(\#*)"|b?"(?:\\.|[^"\\])*"|b?'(?:\\.|[^'\\])'|(?:r\#)?[_\p{XID_Start}][\p{XID_Continue}]*|[^\s]}sg) {
        my ($token, $hashes) = ($&, $1);
        next if $token =~ m{^//};
        if ($token eq '/*') {
            my ($depth, $offset) = (1, pos($source));
            while ($depth) {
                my $opening = index($source, '/*', $offset);
                my $closing = index($source, '*/', $offset);
                die "unterminated Rust block comment\n" if $closing < 0;
                if ($opening >= 0 && $opening < $closing) { ++$depth; $offset = $opening + 2; }
                else { --$depth; $offset = $closing + 2; }
            }
            pos($source) = $offset;
            next;
        }
        if (defined $hashes) {
            my $closing = index($source, '"' . $hashes, pos($source));
            die "unterminated Rust raw string\n" if $closing < 0;
            pos($source) = $closing + 1 + length($hashes);
            next;
        }
        next if index($token, '"') >= 0 || $token =~ /^'.*'$/s;
        push @tokens, $token;
    }
    return @tokens;
}

sub owner_path {
    my ($source) = @_;
    my (@scopes, @header);
    for my $token (tokens($source)) {
        if ($token eq '{' || $token eq '(') {
            my $owner;
            for my $kind ($token eq '{' ? qw(struct enum trait mod) : ('struct')) {
                for my $i (0 .. $#header - 1) {
                    if ($header[$i] eq $kind) {
                        (my $name = $header[$i + 1]) =~ s/^r#//;
                        $owner = "$kind $name";
                        last;
                    }
                }
                last if defined $owner;
            }
            if (!defined $owner && $token eq '{') {
                for my $i (0 .. $#header) {
                    if ($header[$i] eq 'impl') {
                        $owner = join ' ', map { my $part = $_; $part =~ s/^r#//; $part } @header[$i .. $#header];
                        last;
                    }
                }
            }
            if (!defined $owner && @scopes && defined $scopes[-1] && $scopes[-1] =~ /^enum /) {
                my @names = map { my $name = $_; $name =~ s/^r#//; $name } grep /^(?:r#)?[_\p{XID_Start}][\p{XID_Continue}]*$/, @header;
                $owner = "variant $names[-1]" if @names;
            }
            push @scopes, $owner;
            @header = ();
        } elsif ($token eq '}' || $token eq ')') {
            die "unbalanced Rust item scopes\n" unless @scopes;
            pop @scopes;
            @header = ();
        } elsif ($token eq ';' || ($token eq ',' && !grep { $_ eq 'impl' } @header)) {
            @header = ();
        } else { push @header, $token; }
    }
    return join '/', grep { defined && length } @scopes;
}

sub item_key {
    my ($message) = @_;
    die "missing diagnostic spans\n" unless ref($message->{spans}) eq 'ARRAY';
    my @primary = grep { ref($_) eq 'HASH' && $_->{is_primary} } @{$message->{spans}};
    die "expected one primary diagnostic span\n" unless @primary == 1;
    my $span = $primary[0];
    die "expanded missing-doc diagnostic needs an explicit item identity\n" if $span->{expansion};
    die "missing source path\n" unless defined $span->{file_name} && !ref($span->{file_name});
    my $source = read_bytes($span->{file_name});
    for (qw(byte_start byte_end)) {
        die "invalid diagnostic byte range\n" unless defined $span->{$_} && !ref($span->{$_}) && $span->{$_} =~ /\A(?:0|[1-9][0-9]*)\z/ && $span->{$_} <= length($source);
    }
    die "reversed diagnostic byte range\n" if $span->{byte_end} < $span->{byte_start};
    my $prefix = decode('UTF-8', substr($source, 0, $span->{byte_start}), FB_CROAK | LEAVE_SRC);
    my $declaration = decode('UTF-8', substr($source, $span->{byte_start}, $span->{byte_end} - $span->{byte_start}), FB_CROAK | LEAVE_SRC);
    my $kind = $message->{message} // '';
    $kind =~ s/^missing documentation for //;
    my %keywords = ('a function'=>'fn', 'a method'=>'fn', 'an associated function'=>'fn',
        'a constant'=>'const', 'a static'=>'static', 'a module'=>'mod', 'a struct'=>'struct',
        'an enum'=>'enum', 'a trait'=>'trait', 'a union'=>'union', 'a type alias'=>'type', 'an associated type'=>'type');
    my $symbol;
    if (my $keyword = $keywords{$kind}) { ($symbol) = $declaration =~ /\b\Q$keyword\E\s+(?:mut\s+)?(?:r#)?(\w+)/; }
    elsif ($kind eq 'a macro') { ($symbol) = $declaration =~ /\b(?:macro_rules!|macro)\s+(?:r#)?(\w+)/; }
    elsif ($kind eq 'a struct field' || $kind eq 'a variant') { ($symbol) = $declaration =~ /(?:pub(?:\([^)]*\))?\s+)?(?:r#)?(\w+)/; }
    else { die "unsupported missing-doc item kind: $kind\n"; }
    die "missing-doc declaration has no supported item name\n" unless defined $symbol;
    return join '::', $span->{file_name}, owner_path($prefix), $kind, $symbol;
}

sub missing_items {
    my ($path) = @_;
    my %items;
    my @lines = split /\n/, read_bytes($path), -1;
    pop @lines if @lines && $lines[-1] eq "";
    for my $line (@lines) {
        my $record = decode_json($line);
        die "diagnostic record must be an object\n" unless ref($record) eq 'HASH';
        my $message = $record->{message};
        next unless ref($message) eq 'HASH' && ref($message->{code}) eq 'HASH' && ($message->{code}{code} // '') eq 'missing_docs';
        ++$items{item_key($message)};
    }
    return \%items;
}

sub main {
    my (@args) = @_;
    my $write = 0;
    Getopt::Long::Configure(qw(no_auto_abbrev no_ignore_case));
    GetOptionsFromArray(\@args, 'write-baseline' => \$write) && @args == 2
        or die "usage: check-public-docs.pl DIAGNOSTICS BASELINE [--write-baseline]\n";
    my ($diagnostics, $baseline) = @args;
    my $actual = missing_items($diagnostics);
    if ($write) {
        my @items = map { ($_) x $actual->{$_} } sort keys %$actual;
        my $json = JSON::PP->new->utf8->pretty->encode(\@items);
        open my $fh, '>:raw', $baseline or die "write $baseline: $!\n";
        print {$fh} $json or die "write $baseline: $!\n";
        close $fh or die "close $baseline: $!\n";
        return 0;
    }
    my $baseline_items = decode_json(read_bytes($baseline));
    die "baseline must be an array of item strings\n" unless ref($baseline_items) eq 'ARRAY' && !grep { !defined || ref } @$baseline_items;
    my %expected;
    ++$expected{$_} for @$baseline_items;
    my ($changed, $count) = (0, 0);
    my %keys = map { $_ => 1 } (keys %$actual, keys %expected);
    for my $key (sort keys %keys) {
        my $difference = ($actual->{$key} // 0) - ($expected{$key} // 0);
        $count += $actual->{$key} // 0;
        next unless $difference;
        my $label = $difference > 0 ? 'newly undocumented item' : 'remove resolved baseline item';
        print STDERR "error: $label (" . abs($difference) . "): $key\n";
        $changed = 1;
    }
    return 1 if $changed;
    print "public documentation debt: $count known items (no new undocumented items)\n";
    return 0;
}
exit main(@ARGV) unless caller;
1;
