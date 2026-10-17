package RustSource;
use strict;
use warnings;
use Exporter 'import';
our @EXPORT_OK = qw(code_mask tokens);

# Preserve offsets/newlines in decoded source while excluding literals and
# comments. Documentation is checked against the original, unmodified source.
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
        my $length = $end - $start;
        my $hidden = substr($source, $start, $length);
        $hidden =~ s/[^\n]/ /g;
        substr($mask, $start, $length) = $hidden;
        pos($source) = $end;
    }
    return $mask;
}

sub tokens {
    my $mask = code_mask($_[0]);
    return $mask =~ /(?:r\#)?[_\p{XID_Start}][\p{XID_Continue}]*|[^\s]/sg;
}
1;
