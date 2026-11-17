#!/usr/bin/env perl
# Shared companions: scripts/ci/run-validation-stage.pl
use strict;
use warnings;
use Cwd qw(abs_path);
use Digest::SHA qw(sha256_hex);
use JSON::PP;
my $json=JSON::PP->new->canonical->utf8;
@ARGV==4 or die "usage: seal-validation-stage.pl ADMISSION SOURCE GROUP CANDIDATE_ID\n";
my ($path,$source,$group,$identity)=@ARGV;
sub bytes {
    my ($path)=@_;
    $path =~ m{\A/} && $path !~ m{(?:\A|/)\.{1,2}(?:/|\z)|//|[\x00\r\n]} or die "invalid stage evidence route\n";
    my $cursor='';
    for my $part (grep {length} split m{/},$path) { $cursor.="/$part"; !-l $cursor or die "redirected stage evidence\n"; }
    -f $path && -s $path <= 536870912 or die "missing or oversized stage evidence\n";
    open my $in,'<:raw',$path or die $!; local $/; my $raw=<$in>; close $in or die $!; return $raw;
}
sub document {
    my $raw=bytes($_[0]); my $value=$json->decode($raw);
    $raw eq $json->encode($value)."\n" or die "noncanonical stage evidence\n";
    return $value;
}
my $admission=document($path);
$source =~ /\A(?:[0-9a-f]{40}|[0-9a-f]{64})\z/ && $group =~ /\A[A-Za-z0-9][A-Za-z0-9_-]*\z/
    && $identity =~ /\A[0-9a-f]{64}\z/ or die "invalid fresh stage identity\n";
($admission->{schema}//0)==1 && ($admission->{kind}//'') eq 'release-stage-admission'
    && ($admission->{source}//'') eq $source && ($admission->{group}//'') eq $group
    && ($admission->{mode}//'') =~ /\A(?:executed|reused)\z/
    or die "mismatched stage admission\n";
my $proof_path=$admission->{execution_proof};
my $proof_raw=bytes($proof_path);
sha256_hex($proof_raw) eq $admission->{execution_proof_sha256} or die "changed original execution proof\n";
my $proof=document($proof_path);
($proof->{schema}//0)==1 && ($proof->{kind}//'') eq 'release-stage-execution'
    && ($proof->{result}//1)==0 && $proof->{context}{stage} eq $admission->{stage}
    && $proof->{context}{contract} eq $admission->{contract}
    && sha256_hex($json->encode($proof->{context})) eq $admission->{context_sha256}
    && ($proof->{context}{observation}{configuration}{identity}//'') eq $identity
    or die "stage execution context differs\n";
my %files=($path=>sha256_hex(bytes($path)),$proof_path=>sha256_hex($proof_raw));
for my $stream (qw(stdout stderr)) {
    my $log=$proof->{logs}{$stream};
    $log->{path} eq "$proof->{origin}{attempt}/command.$stream" or die "wrong original stage log\n";
    $files{$log->{path}}=sha256_hex(bytes($log->{path}));
    $files{$log->{path}} eq $log->{sha256} or die "changed original stage log\n";
}
print $json->encode({schema=>1,kind=>'sealed-cargo-stage-admission',admission=>$admission,
    candidate_identity=>$identity,origin=>$proof->{origin},files=>\%files,
    environment_sha256=>sha256_hex($json->encode($proof->{context}{environment})),
    command=>$proof->{context}{command},cwd=>$proof->{context}{cwd},
    host=>$proof->{context}{host},compiler=>$proof->{context}{observation}{configuration}{compiler}}),"\n";
