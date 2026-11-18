#!/usr/bin/env perl
# Shared companions: scripts/ci/run-validation-stage.pl
use strict;
use warnings;
use Digest::SHA qw(sha256_hex);
use JSON::PP;
my $json=JSON::PP->new->canonical->utf8;
my $generic = @ARGV && $ARGV[0] eq '--generic';
shift if $generic;
@ARGV == ($generic ? 5 : 4) or die "usage: seal-validation-stage.pl ADMISSION SOURCE GROUP CANDIDATE_ID or --generic ADMISSION SOURCE GROUP STAGE CONTRACT\n";
my ($path,$source,$group,$identity,$contract)=@ARGV;
sub bytes {
    my ($path)=@_;
    $path =~ m{\A/} && $path !~ m{(?:\A|/)\.{1,2}(?:/|\z)|//|[\x00\r\n]} or die "invalid stage evidence route\n";
    my $cursor='';
    for my $part (grep {length} split m{/},$path) { $cursor.="/$part"; !-l $cursor or die "redirected stage evidence\n"; }
    -f $path && -s $path <= 536870912 or die "missing or oversized stage evidence\n";
    open my $in,'<:raw',$path or die $!; local $/; my $raw=<$in>; close $in or die $!; return $raw;
}
sub document {
    my ($raw)=@_; my $value=$json->decode($raw);
    $raw eq $json->encode($value)."\n" or die "noncanonical stage evidence\n";
    return $value;
}
my $admission_raw=bytes($path);
my $admission=document($admission_raw);
$source =~ /\A(?:[0-9a-f]{40}|[0-9a-f]{64})\z/ && $group =~ /\A[A-Za-z0-9][A-Za-z0-9_-]*\z/
    && ($generic ? ($identity =~ /\A[A-Za-z0-9][A-Za-z0-9_-]*\z/ && $contract =~ m{\A[A-Za-z0-9][A-Za-z0-9_./-]*\z}) : $identity =~ /\A[0-9a-f]{64}\z/) or die "invalid fresh stage identity\n";
$json->encode($admission->{schema}) eq '1' && ($admission->{kind}//'') eq 'release-stage-admission'
    && ($admission->{source}//'') eq $source && ($admission->{group}//'') eq $group
    && ($admission->{mode}//'') =~ /\A(?:executed|reused)\z/
    or die "mismatched stage admission\n";
my $proof_path=$admission->{execution_proof};
my $proof_raw=bytes($proof_path);
sha256_hex($proof_raw) eq $admission->{execution_proof_sha256} or die "changed original execution proof\n";
my $proof=document($proof_raw);
$json->encode($proof->{schema}) eq '1' && ($proof->{kind}//'') eq 'release-stage-execution'
    && $json->encode($proof->{result}) eq '0' && $proof->{context}{stage} eq $admission->{stage}
    && $proof->{context}{contract} eq $admission->{contract}
    && sha256_hex($json->encode($proof->{context})) eq $admission->{context_sha256}
    && ($generic ? ($admission->{stage} eq $identity && $admission->{contract} eq $contract)
        : ($proof->{context}{observation}{configuration}{identity}//'') eq $identity)
    or die "stage execution context differs\n";
sub exact_keys {
    my ($record,@keys)=@_;
    ref($record) eq 'HASH' && keys(%$record)==@keys && !grep { !exists $record->{$_} } @keys
        or die "unexpected stage record fields\n";
}
exact_keys($admission,qw(schema kind mode source group stage contract context_sha256 execution_proof execution_proof_sha256 elapsed_seconds));
exact_keys($proof,qw(schema kind result context origin logs elapsed_seconds));
exact_keys($proof->{context},qw(schema stage contract cwd command environment host observation observer program runner));
exact_keys($proof->{origin},qw(source group attempt));
exact_keys($proof->{logs},qw(stdout stderr));
my $origin=$proof->{origin};
my $ctx=$proof->{context};
$json->encode($ctx->{schema}) eq '1' && ref($ctx->{command}) eq 'ARRAY' && @{$ctx->{command}}
    && ref($ctx->{environment}) eq 'HASH' && ref($ctx->{host}) eq 'ARRAY' && @{$ctx->{host}}
    && ref($ctx->{observation}{configuration}) eq 'HASH' && ref($ctx->{observation}{files}) eq 'ARRAY'
    && $origin->{source} =~ /\A(?:[0-9a-f]{40}|[0-9a-f]{64})\z/
    && $origin->{group} =~ /\A[A-Za-z0-9][A-Za-z0-9_-]*\z/
    or die "malformed execution context or origin\n";
for my $record ($admission,$proof) {
    $json->encode($record->{elapsed_seconds}) =~ /\A(?:0|[1-9][0-9]*)(?:\.[0-9]+)?(?:e[+-]?[0-9]+)?\z/i
        or die "invalid elapsed time\n";
}
exact_keys($ctx->{observation},qw(configuration files));
exact_keys($ctx->{observer},qw(path sha256));
exact_keys($ctx->{runner},qw(sha256 perl modules json digest));
for my $file (@{$ctx->{observation}{files}}) {
    exact_keys($file,qw(path resolved sha256));
    $file->{sha256} =~ /\A[0-9a-f]{64}\z/ && $file->{resolved} =~ m{\A/} && length($file->{path})
        or die "invalid observed file\n";
}
my ($state,$suffix)=$origin->{attempt} =~ m{\A(.+)/attempts/\Q$admission->{stage}-$origin->{group}\E\.([A-Za-z0-9_-]{6})\z};
defined($state) && $proof_path eq "$state/proofs/$admission->{context_sha256}.$suffix.json"
    or die "wrong original proof route\n";
$admission->{mode} ne 'executed' || ($source eq $origin->{source} && $group eq $origin->{group})
    or die "executed admission differs from origin\n";
my %files=($path=>sha256_hex($admission_raw),$proof_path=>sha256_hex($proof_raw));
for my $stream (qw(stdout stderr)) {
    my $log=$proof->{logs}{$stream};
    exact_keys($log,qw(path sha256));
    $log->{path} eq "$proof->{origin}{attempt}/command.$stream" or die "wrong original stage log\n";
    $files{$log->{path}}=sha256_hex(bytes($log->{path}));
    $files{$log->{path}} eq $log->{sha256} or die "changed original stage log\n";
}
print $json->encode({schema=>1,kind=>($generic ? 'sealed-stage-admission' : 'sealed-cargo-stage-admission'),admission=>$admission,
    origin=>$proof->{origin},files=>\%files,
    environment_sha256=>sha256_hex($json->encode($proof->{context}{environment})),
    command=>$proof->{context}{command},cwd=>$proof->{context}{cwd},
    host=>$proof->{context}{host},($generic ? () : (candidate_identity=>$identity,compiler=>$proof->{context}{observation}{configuration}{compiler}))}),"\n";
