#!/usr/bin/env perl
# Shared companions: scripts/ci/check-public-docs.pl
use strict;
use warnings;
use utf8;
use Encode qw(encode);
use JSON::PP qw(encode_json decode_json);
use File::Temp qw(tempdir);
use File::Path qw(remove_tree);
use FindBin qw($Bin);
use Test::More;
require "$Bin/check-public-docs.pl";
my $root = tempdir('rustdoc-identities.XXXXXX', TMPDIR => 1, CLEANUP => 0);
my $complete = 0;
END { if ($complete) { remove_tree($root); } else { print STDERR "Rustdoc fixture retained: $root\n"; } }
sub write_bytes {
    my ($path, $bytes) = @_;
    open my $fh, '>:raw', $path or die $!;
    print {$fh} $bytes or die $!;
    close $fh or die $!;
}
sub message {
    my ($source, $declaration, $kind) = @_;
    $kind //= 'a struct field';
    write_bytes("$root/source.rs", encode('UTF-8', $source));
    my $start = index($source, $declaration);
    die "fixture declaration absent" if $start < 0;
    my $byte_start = length encode('UTF-8', substr($source, 0, $start));
    return { message => "missing documentation for $kind", code => { code => 'missing_docs' },
        spans => [{ is_primary => JSON::PP::true, expansion => undef, file_name => "$root/source.rs",
            byte_start => $byte_start, byte_end => $byte_start + length encode('UTF-8', $declaration) }] };
}
sub key {
    my $key = item_key(message(@_));
    $key =~ s/^\Q$root\/source.rs\E:://;
    return $key;
}
my $source = 'pub struct Request { pub endpoint: String }';
is(key($source, 'pub endpoint: String'), key("// { Unicode é }\n$source", 'pub endpoint: String'), 'line/UTF-8 movement preserves identity');
isnt(key($source, 'pub endpoint: String'), key('pub struct Request { pub timeout: u64 }', 'pub timeout: u64'), 'unrelated fields cannot offset');
isnt(key('pub struct A { pub value: u64 }', 'pub value: u64'), key('pub struct B { pub value: u64 }', 'pub value: u64'), 'owner namespaces');
is(key('pub enum Error { Invalid { value: String } }', 'value: String'), 'enum Error/variant Invalid::a struct field::value', 'named variant');
isnt(key('pub enum Error { A(String) }', 'String'), key('pub enum Error { B(String) }', 'String'), 'tuple variant namespaces');
like(key('impl<T, E> Request<T, E> { pub fn new() -> Self {} }', 'pub fn new()', 'an associated function'), qr/Request/, 'generic impl owner');
for my $pair (['struct','a struct'], ['enum','an enum'], ['trait','a trait'], ['type','a type alias'], ['union','a union'], ['mod','a module']) {
    my ($keyword, $kind) = @$pair;
    isnt(key("pub $keyword First {}", "pub $keyword First {}", $kind), key("pub $keyword Second {}", "pub $keyword Second {}", $kind), "$keyword uses declared name");
}
like(key('pub static mut COUNTER: u64 = 0;', 'pub static mut COUNTER: u64 = 0;', 'a static'), qr/::COUNTER$/, 'static name');
isnt(key('pub struct r#type;', 'pub struct r#type;', 'a struct'), key('pub struct r#match;', 'pub struct r#match;', 'a struct'), 'raw type names remain distinct');
is(key('pub struct r#Request { pub r#type: u64 }', 'pub r#type: u64'), 'struct Request::a struct field::type', 'raw owner and field names');
is(key('impl r#Request { pub fn r#match() {} }', 'pub fn r#match()', 'a method'), key('impl Request { pub fn r#match() {} }', 'pub fn r#match()', 'a method'), 'raw impl spelling does not change owner identity');
isnt(key('pub enum Error { r#type { value: u64 } }', 'value: u64'), key('pub enum Error { r#match { value: u64 } }', 'value: u64'), 'raw lowercase variants retain namespaces');
isnt(key('pub enum Error { first { value: u64 } }', 'value: u64'), key('pub enum Error { second { value: u64 } }', 'value: u64'), 'lowercase variants retain namespaces');
isnt(key('pub struct 型甲 { pub value: u64 }', 'pub value: u64'), key('pub struct 型乙 { pub value: u64 }', 'pub value: u64'), 'Unicode owners retain their full names');
isnt(key('pub enum Error { 型甲 { value: u64 } }', 'value: u64'), key('pub enum Error { 型乙 { value: u64 } }', 'value: u64'), 'Unicode variants retain namespaces');
like(key('macro_rules! exported { () => {} }', 'macro_rules! exported { () => {} }', 'a macro'), qr/::exported$/, 'macro name');
like(key('impl Request { pub const fn with_sort() {} }', 'pub const fn with_sort()', 'a method'), qr/::with_sort$/, 'const method');
is(key('const X: &str = r#"{fake}"#; /* outer /* {nested} */ end */ pub struct Real { pub value: u64 }', 'pub value: u64'), 'struct Real::a struct field::value', 'raw literals and nested comments');
my $msg = message($source, 'pub endpoint: String');
my $record = encode_json({message => $msg}) . "\n";
write_bytes("$root/input", $record x 2);
is_deeply([values %{missing_items("$root/input")}], [2], 'multiplicity retained');
$msg->{spans}[0]{expansion} = {macro_decl_name => 'fixture!'};
ok(!eval { item_key($msg); 1 }, 'expanded identity refused');
$msg->{spans}[0]{expansion} = undef;
$msg->{spans}[0]{byte_end} = 9999;
ok(!eval { item_key($msg); 1 }, 'out-of-range span refused');
$msg->{spans}[0]{byte_end} = 0;
ok(!eval { item_key($msg); 1 }, 'reversed span refused');
sub cli {
    my (@extra) = @_;
    my $pid = fork();
    die $! unless defined $pid;
    if (!$pid) {
        open STDOUT, '>', "$root/stdout" or die $!;
        open STDERR, '>', "$root/stderr" or die $!;
        exec $^X, "$Bin/check-public-docs.pl", "$root/input", "$root/baseline", @extra;
        die $!;
    }
    waitpid($pid, 0);
    die "checker killed by signal" if $? & 127;
    return $? >> 8;
}
is(cli('--write-baseline'), 0, 'explicit baseline creation');
my $saved = read_bytes("$root/baseline");
is(cli(), 0, 'matching debt');
write_bytes("$root/input", $record);
is(cli(), 1, 'duplicate count change detected');
write_bytes("$root/baseline", encode_json(['unrelated item']));
my $before = read_bytes("$root/baseline");
is(cli(), 1, 'same count cannot offset unrelated identity');
like(read_bytes("$root/stderr"), qr/newly undocumented item/, 'added diagnostic');
like(read_bytes("$root/stderr"), qr/remove resolved baseline item/, 'resolved diagnostic');
is(read_bytes("$root/baseline"), $before, 'comparison preserves baseline');
write_bytes("$root/baseline", $saved);
write_bytes("$root/input", '');
is(cli(), 1, 'resolved debt requires explicit removal');
write_bytes("$root/input", $record . "\n");
ok(cli("--write-baseline") != 0, "blank diagnostic record refused");
is(read_bytes("$root/baseline"), $saved, "blank record preserves baseline");
write_bytes("$root/input", '{incomplete');
ok(cli('--write-baseline') != 0, 'malformed input refuses baseline replacement');
is(read_bytes("$root/baseline"), $saved, 'malformed input preserves baseline');
write_bytes("$root/input", $record);
write_bytes("$root/baseline", '{}');
ok(cli() != 0, 'malformed baseline refused');
unlink "$root/source.rs" or die $!;
ok(cli('--write-baseline') != 0, 'missing source refused');
is(read_bytes("$root/baseline"), '{}', 'missing source preserves baseline');
done_testing();
$complete = Test::More->builder->is_passing;
