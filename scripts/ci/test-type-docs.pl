#!/usr/bin/env perl
# Shared companions: scripts/ci/check-type-docs.pl
use strict;
use warnings;
use utf8;
use Encode qw(encode);
use POSIX qw(WIFEXITED WEXITSTATUS);
use File::Temp qw(tempdir);
use File::Path qw(remove_tree);
use FindBin qw($Bin);
my $root = tempdir('type-docs.XXXXXX', TMPDIR => 1, CLEANUP => 0);
my $complete = 0;
END { if ($complete) { remove_tree($root); } else { print STDERR "Type docs fixture retained: $root\n"; } }
my $checker = "$Bin/check-type-docs.pl";
my $file = "$root/source.rs";
my $attempt = 0;
sub run_checker {
    my ($expected, @selection) = @_;
    my $log = "$root/check-" . ++$attempt;
    my $pid = fork();
    defined $pid or die "cannot fork checker: $!\n";
    if (!$pid) {
        open STDOUT, '>:raw', "$log.stdout" or die $!;
        open STDERR, '>:raw', "$log.stderr" or die $!;
        exec $^X, $checker, @selection;
        die "cannot execute checker: $!\n";
    }
    waitpid($pid, 0) == $pid or die "cannot wait for checker: $!\n";
    my $status = $?;
    my @output;
    for my $stream (qw(stdout stderr)) {
        open my $handle, '<:raw', "$log.$stream" or die $!;
        local $/;
        push @output, <$handle> // '';
        close $handle or die $!;
    }
    my $normal = WIFEXITED($status);
    my $code = $normal ? WEXITSTATUS($status) : undef;
    unless ($normal && (defined $expected ? $code == $expected : $code != 0) &&
            $output[0] eq '' && ($code != 0 || $output[1] eq '')) {
        print STDERR @output;
        die "unexpected type-docs result (wait status $status); logs: $log.stdout, $log.stderr\n";
    }
    return $output[1];
}
sub check {
    my ($source, $expected) = @_;
    open my $handle, '>:encoding(UTF-8)', $file or die $!;
    print {$handle} $source;
    close $handle or die $!;
    return run_checker($expected, $file);
}
my $valid = <<'RUST';
mod nested {
    ///
    /// Current
    ///
    /// A current cross-module contract.
    ///

    #[derive(
        Debug,
    )]
    #[cfg_attr(any(), derive(Clone))]
    pub(crate) struct Current;
}
const RAW: &str = r###"
pub struct NotCode;
"###;
const BYTE_RAW: &[u8] = br##"
pub enum NotByteCode {}
"##;
const TEXT: &str = "escaped \" braces { }
pub trait NotStringCode {}";
const BYTE: u8 = b'{';
const CHAR: char = '}';
// pub enum NotCodeEither {}
/* outer /* nested */
pub trait NotCode {}
*/
RUST
check($valid, 0);
for my $name ('r#type', '型甲') {
    (my $logical_name = $name) =~ s/^r#//;
    my $documented = $valid;
    $documented =~ s/Current/$logical_name/g;
    $documented =~ s/struct \Q$logical_name\E/struct $name/;
    check($documented, 0);
    my $diagnostic = check("pub struct $name;\n", 1);
    index($diagnostic, "$file:1: " . encode('UTF-8', $logical_name) . "\n") >= 0
        or die "type diagnostic lost its identifier\n";
    $diagnostic !~ /Wide character/ or die "type diagnostic emitted an encoding warning\n";
}
my $original_file = $file;
$file = "$root/" . encode('UTF-8', '型.rs');
my $diagnostic = check("pub struct 型甲;\n", 1);
index($diagnostic, "$file:1: " . encode('UTF-8', '型甲') . "\n") >= 0
    or die "type diagnostic changed UTF-8 path or identifier bytes\n";
$diagnostic !~ /Wide character/ or die "type diagnostic emitted an encoding warning\n";
unlink $file or die $!;
$file = $original_file;
my $attribute_diagnostic = check("]\npub struct 型甲;\n", 255);
index($attribute_diagnostic, encode('UTF-8', '型甲')) >= 0
    or die "attribute diagnostic lost its Unicode identifier\n";
$attribute_diagnostic !~ /Wide character/ or die "attribute diagnostic emitted an encoding warning\n";
(my $missing = $valid) =~ s/    \/\/\/ A current cross-module contract[.]\n//;
check($missing, 1);
check("mod nested {\n    pub enum Missing {}\n}\n", 1);
(my $wrong_name = $valid) =~ s/\/\/\/ Current/\/\/\/ Another/;
check($wrong_name, 1);
check("pub fn helper() {}\nstruct Private;\n", 0);
check("const X: &str = r#\"unterminated", 255);
check("/* outer /* closed inner */", 255);
check("const X: &[u8] = br##\"unterminated\"#", 255);
run_checker(undef, "$root/missing");
run_checker(undef, $root, $file);
check($valid, 0);
run_checker(0, $root, $file);
# A readable sibling must not hide an unreadable selected subtree. Restore
# permissions before asserting, so a failed check still leaves inspectable data.
mkdir "$root/blocked" or die $!;
open my $blocked, '>', "$root/blocked/missing.rs" or die $!;
print {$blocked} "pub struct Missing;\n" or die $!;
close $blocked or die $!;
chmod 0000, "$root/blocked" or die $!;
if (-r "$root/blocked") {
    chmod 0700, "$root/blocked" or die $!;
    print "Unreadable-directory case skipped: current user bypasses permissions\n";
} else {
    # Restore permissions even when the child result is unexpected.
    my $checked = eval { run_checker(undef, $root); 1 };
    my $error = $@;
    chmod 0700, "$root/blocked" or die $!;
    die $error unless $checked;
}
$complete = 1;
print "Section-style documentation selection, syntax and refusal fixtures passed\n";
