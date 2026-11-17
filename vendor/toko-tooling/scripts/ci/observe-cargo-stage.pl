#!/usr/bin/env perl
# Shared companions:
use strict;
use warnings;
use Cwd qw(abs_path getcwd);
use File::Find;
use Digest::SHA qw(sha256_hex);
use JSON::PP;

# Exact prepared-candidate stage inventory. No version filtering occurs here:
# source, manifests and lock are observed exactly as compiled/tested.
my $json=JSON::PP->new->canonical->utf8;
sub command {
    open my $in,'-|',@_ or die $!; local $/; my $raw=<$in>; close $in or die "stage observation failed: @_\n";
    return $raw;
}
sub document {
    open my $in,'<:raw',$_[0] or die $!; local $/; my $raw=<$in>; close $in or die $!;
    return $json->decode($raw);
}
my $root=getcwd();
!-e "$root/.tools" && !-l "$root/.tools" or die "candidate tool roots must stay caller-owned\n";
my $tools=$ENV{CARGO_STAGE_TOOLS}//die "select bounded tool inputs\n";
my $identity=$ENV{CARGO_STAGE_IDENTITY}//die "select final candidate identity\n";
$identity =~ /\A[0-9a-f]{64}\z/ or die "invalid candidate identity\n";
# A stale projection label must not bless source modified between leaf stages.
command('git','status','--porcelain=v1','--untracked-files=all') eq ''
    or die "candidate observation requires clean qualified source\n";
my (%source_files,%source_modes);
for my $entry (split /\0/,command('git','ls-tree','-rz','--full-tree','HEAD')) {
    $entry =~ /\A(100644|100755) blob [0-9a-f]{40,64}\t([^\x00\r\n]+)\z/
        or die "candidate observation needs regular tracked files\n";
    my ($mode,$path)=($1,$2);
    -f $path && !-l $path && -s $path <= 8388608 or die "invalid candidate source file\n";
    open my $in,'<:raw',$path or die $!; my $digest=Digest::SHA->new(256);
    $digest->addfile($in); close $in or die $!;
    (((stat $path)[2] & 0111) != 0) == ($mode eq '100755') or die "candidate mode drift\n";
    $source_files{$path}=$digest->hexdigest; $source_modes{$path}=$mode;
}
sha256_hex($json->encode({files=>\%source_files,modes=>\%source_modes})) eq $identity
    or die "candidate content no longer matches qualified identity\n";
my %files=map {$_=>1} keys %source_files;
$files{$tools}=1;
for my $path (@{document($tools)->{files}}) { $files{$path}=1; }
my (%trees,%links);
sub tree {
    my ($directory) = @_;
    return if $trees{$directory}++;
    my @linked;
    -d $directory or die "missing stage input tree: $directory\n";
    find({no_chdir=>1,wanted=>sub {
        if (-l $_ && -d $_) {
            my $resolved=abs_path($_)//die "unresolved stage input tree\n";
            $links{$_}=$resolved; push @linked,$resolved; return;
        }
        return if -d $_;
        -f $_ or die "nonregular stage input: $_\n";
        $files{$_}=1;
    }},$directory);
    tree($_) for @linked;
}
my @manifests=('Cargo.toml');
my $policy=document('ci/cargo-release-groups.json');
if (exists $policy->{qualification_workspace}) {
    $policy->{qualification_workspace} eq 'qualification'
        or die "unsupported qualification workspace\n";
    push @manifests,'qualification/Cargo.toml';
}
for my $manifest (@manifests) {
    my $metadata=$json->decode(command('cargo','metadata','--offline','--locked','--all-features','--format-version','1','--manifest-path',$manifest));
    for my $package (@{$metadata->{packages}}) {
        my $package_manifest=$package->{manifest_path};
        # Local tracked source is already enumerated. External package directories
        # include expanded registry sources, not just their lockfile checksums.
        next if index($package_manifest,"$root/")==0;
        $package_manifest =~ s{/Cargo\.toml\z}{} or die "invalid dependency manifest path\n";
        tree($package_manifest);
    }
}
my $sysroot=command('rustc','--print','sysroot'); $sysroot =~ s/\n\z//;
tree("$sysroot/bin");
tree("$sysroot/lib");
# Current native candidate reuse is qualified on Linux; other hosts execute
# their ordinary stages until this native-input contract is qualified there.
my $os=command('uname','-s');
$os eq "Linux\n" or die "native input observer not qualified for this host\n";
tree('/usr/include');
for my $name (qw(include include-fixed libgcc.a libgcc_s.so.1 libstdc++.so libc.so)) {
    my $path=command('cc',"-print-file-name=$name"); $path =~ s/\n\z//;
    next if $path eq $name;
    if (-d $path) { tree(abs_path($path)); }
    elsif (-f $path) { $files{$path}=1; }
    else { die "missing native compiler input: $path\n"; }
}
for my $path (glob('/lib/*/libc.so.*'),glob('/lib/*/libm.so.*'),glob('/lib/*/libpthread.so.*'),glob('/lib/*/libdl.so.*')) {
    $files{$path}=1 if -f $path;
}
$files{abs_path($_)}=1 for values %INC;
print $json->encode({schema=>1,files=>[sort keys %files],configuration=>{
    contract=>'cargo-final-candidate/1',identity=>$identity,links=>\%links,
    compiler=>command('rustc','-vV'),cargo=>command('cargo','-vV'),cc=>command('cc','--version'),
}}),"\n";
