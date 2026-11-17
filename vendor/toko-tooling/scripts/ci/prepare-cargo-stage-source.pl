#!/usr/bin/env perl
# Shared companions: scripts/ci/cargo-release-group.sh
use strict;
use warnings;
use Cwd qw(abs_path getcwd);
use Digest::SHA qw(sha256_hex);
use File::Basename qw(dirname);
use File::Path qw(make_path);
use File::Temp qw(tempdir);
use Getopt::Long qw(GetOptions);
use JSON::PP;

# Qualification source only. Never prepare a live release, rebind an old gate,
# project old executable evidence, or change the caller's files/index/versions.
my ($root,$state,$source,$editor,$verify_projection);
GetOptions('root=s'=>\$root,'state=s'=>\$state,'source=s'=>\$source,'editor=s'=>\$editor,'verify-projection=s'=>\$verify_projection)
    && !@ARGV && (defined($verify_projection) || !grep { !defined($_) || !length($_) } ($root,$state,$source,$editor))
    or die "usage: prepare-cargo-stage-source.pl --root DIR --state DIR --source COMMIT --editor ABSOLUTE\n";
my $json = JSON::PP->new->canonical->utf8;
sub regular {
    my ($path) = @_;
    my $cursor = '';
    $path =~ m{\A/} && $path !~ m{[\x00\r\n]|(?:\A|/)\.{1,2}(?:/|\z)|//}
        or die "noncanonical candidate route: $path\n";
    for my $part (grep {length} split m{/},$path) {
        $cursor .= "/$part";
        !-l $cursor or die "redirected candidate route: $cursor\n";
    }
}
sub bytes {
    my ($path) = @_;
    -f $path && !-l $path && -s $path <= 8388608 or die "missing or oversized candidate file: $path\n";
    open my $in,'<:raw',$path or die "$path: $!\n";
    local $/; my $raw=<$in>; close $in or die $!; return $raw;
}
sub tool_digest {
    my ($path) = @_;
    -f $path && -s $path <= 268435456 or die "oversized candidate tool\n";
    open my $in,'<:raw',$path or die $!;
    my $hash=Digest::SHA->new(256); $hash->addfile($in); close $in or die $!;
    return $hash->hexdigest;
}
sub write_file {
    my ($path,$raw) = @_;
    open my $out,'>:raw',$path or die "$path: $!\n";
    print {$out} $raw or die $!; close $out or die $!;
}
sub observe {
    open my $in,'-|',@_ or die "start candidate observation: $!\n";
    local $/; my $raw=<$in>; close $in or die "candidate observation failed: @_\n";
    length($raw) <= 16777216 or die "oversized candidate observation\n";
    return $raw;
}
sub run {
    my ($log,@command) = @_;
    my $pid=fork(); defined $pid or die $!;
    if (!$pid) {
        open STDOUT,'>>:raw',$log or exit 125;
        open STDERR,'>&',\*STDOUT or exit 125;
        exec {$command[0]} @command or exit 127;
    }
    waitpid($pid,0)==$pid or die $!;
    $? == 0 or die "candidate preparation failed; retained log: $log\n";
}
# Read-only admission of already-qualified candidate source. This checks the
# sealed inventory, not today's toolchain; callers still own publication source,
# receipt and exact primary-closure admission. No rendering/setup/execution.
if (defined $verify_projection) {
    !grep {defined $_} ($root,$state,$source,$editor) or die "projection verification has no preparation options\n";
    regular($verify_projection);
    my $raw=bytes($verify_projection); my $projection=$json->decode($raw);
    $raw eq $json->encode($projection)."\n"
        && ($projection->{schema}//0)==1 && ($projection->{kind}//'') eq 'cargo-final-candidate-source'
        && ($projection->{source}//'') =~ /\A(?:[0-9a-f]{40}|[0-9a-f]{64})\z/
        && ($projection->{identity}//'') =~ /\A[0-9a-f]{64}\z/
        or die "invalid retained candidate projection\n";
    my $directory=$projection->{directory}; regular($directory);
    !-e "$directory/.tools" && !-l "$directory/.tools" or die "candidate tool roots must stay caller-owned\n";
    chdir $directory or die "missing retained candidate\n";
    observe('git','status','--porcelain=v1','--untracked-files=all') eq ''
        or die "retained candidate source is dirty\n";
    my (%files,%modes);
    for my $entry (split /\0/,observe('git','ls-tree','-rz','--full-tree','HEAD')) {
        $entry =~ /\A(100644|100755) blob [0-9a-f]{40,64}\t([^\x00\r\n]+)\z/
            or die "retained candidate needs regular tracked files\n";
        my ($mode,$path)=($1,$2); regular("$directory/$path");
        $files{$path}=sha256_hex(bytes("$directory/$path"));
        (((stat "$directory/$path")[2] & 0111) != 0) == ($mode eq '100755')
            or die "retained candidate mode drift\n";
        $modes{$path}=$mode;
    }
    keys(%files) && sha256_hex($json->encode({files=>\%files,modes=>\%modes})) eq $projection->{identity}
        or die "retained candidate inventory drift\n";
    print "Retained candidate inventory matches qualified identity: $projection->{identity}\n";
    exit;
}
$root=abs_path($root)//die "missing source root\n";
regular($root);
my $tooling_root = -d "$root/vendor/toko-tooling" ? "$root/vendor/toko-tooling" : $root;
regular($tooling_root);
$state = "$root/$state" unless $state =~ m{\A/};
regular($state);
$editor =~ m{\A/} && -f $editor && -x $editor or die "select the admitted absolute canonical editor\n";
$source =~ /\A(?:[0-9a-f]{40}|[0-9a-f]{64})\z/ or die "exact source commit required\n";
chdir $root or die $!;
observe('git','rev-parse','HEAD') eq "$source\n" or die "candidate source moved\n";
observe('git','status','--porcelain=v1','--untracked-files=all') eq '' or die "candidate source/index must be clean\n";
# Ambient Cargo overrides require ordinary execution, never silent sanitization.
for my $name (keys %ENV) {
    next unless $name =~ /\A(?:RUSTFLAGS|RUSTDOCFLAGS|CARGO_ENCODED_RUSTFLAGS|CARGO_BUILD_|CARGO_TARGET_.*(?:LINKER|RUSTFLAGS)|RUSTC_WRAPPER|RUSTC_WORKSPACE_WRAPPER)/;
    !length($ENV{$name}) or die "candidate source does not admit build override: $name\n";
}
for (my $directory=$root;;$directory=dirname($directory)) {
    !-e "$directory/.cargo/config" && !-e "$directory/.cargo/config.toml"
        or die "candidate source requires ordinary execution with ancestor Cargo configuration\n";
    last if $directory eq '/';
}
my $cargo_home=$ENV{CARGO_HOME}//"$ENV{HOME}/.cargo";
!-e "$cargo_home/config" && !-e "$cargo_home/config.toml"
    or die "candidate source requires ordinary execution with Cargo home configuration\n";
my @entries = split /\0/,observe('git','ls-tree','-rz','--full-tree',$source);
my (@paths,%modes);
for my $entry (@entries) {
    $entry =~ /\A(100644|100755) blob [0-9a-f]{40,64}\t([^\x00\r\n]+)\z/
        or die "candidate source requires regular tracked files\n";
    my ($mode,$path)=($1,$2);
    $path !~ m{(?:\A|/)\.{1,2}(?:/|\z)|//|\A/} or die "unsafe candidate path\n";
    push @paths,$path; $modes{$path}=$mode;
}
@paths or die "empty candidate source\n";
make_path("$state/attempts","$state/sources");
my $attempt=tempdir('candidate.XXXXXX',DIR=>"$state/attempts",CLEANUP=>0);
my $tree="$attempt/source"; make_path($tree);
my $archive="$attempt/source.tar";
my $pid=fork(); defined($pid) or die $!;
if (!$pid) { open STDOUT,'>:raw',$archive or exit 125; exec 'git','archive','--format=tar',$source or exit 127; }
waitpid($pid,0)==$pid && $?==0 or die "candidate source export failed\n";
run("$attempt/preparation.log",'tar','-xf',$archive,'-C',$tree);
my %original=map {$_=>sha256_hex(bytes("$tree/$_"))} @paths;
my $policy=$json->decode(bytes("$tree/ci/cargo-release-groups.json"));
ref($policy->{groups}) eq 'ARRAY' && @{$policy->{groups}} or die "missing candidate group policy\n";
if (-e "$tree/qualification/Cargo.toml" && !exists $policy->{qualification_workspace}) {
    die "qualification workspace requires declared multi-lock admission; use fresh execution\n";
}
my %versions;
chdir $tree or die $!;
local $ENV{CARGO_TARGET_DIR}="$tree/target";
local $ENV{CARGO_NET_OFFLINE}='true';
local $ENV{CARGO_RELEASE_METADATA_EDITOR}=$editor;
local $ENV{CARGO_RELEASE_METADATA_MAX_BYTES}='8388608';
delete @ENV{qw(RELEASE_COMMIT RELEASE_GROUP RELEASE_PREVIOUS RELEASE_VERSION RELEASE_DATE RELEASE_SOURCE BLOB_RELEASE_SOURCE_ROOT BLOB_RELEASE_VALIDATION_PLAN)};
for my $group (@{$policy->{groups}}) {
    my $name=$group->{name};
    $name =~ /\A[A-Za-z0-9][A-Za-z0-9_-]*\z/ or die "invalid candidate group\n";
    my $plan=$json->decode(observe('bash',"$tooling_root/scripts/ci/cargo-release-group.sh",'plan',$name));
    my $previous=$plan->{previous};
    my $notes=bytes("$tree/$group->{changelog}");
    # Never search past the first release heading for an older usable version.
    my ($heading) = $notes =~ /^(##[ \t]+[^\n]*)/m;
    defined($heading) or die "candidate group has no release heading\n";
    $heading =~ s/[ \t]+$//;
    $heading =~ s/[ \t]+/ /g;
    $heading =~ /\A## \[(\d+\.\d+\.\d+)\](?: - (Unreleased|\d{4}-\d{2}-\d{2}))?\z/
        or die "candidate group needs its exact latest release heading\n";
    my ($version,$date)=($1,$2 // 'Unreleased');
    if ($date eq 'Unreleased') {
        $version ne $previous or die "pending candidate must advance its group\n";
        local @ENV{qw(RELEASE_PREVIOUS RELEASE_VERSION RELEASE_DATE)}=($previous,$version,'2000-01-01');
        run("$attempt/preparation.log",'bash',"$tooling_root/scripts/ci/cargo-release-group.sh",'prepare',$name);
    } else {
        $version eq $previous or die "finalized group and package version differ\n";
        run("$attempt/preparation.log",'awk','-v',"version=$version",'-v',"previous=$previous",
            '-v',"date=$date",'-v','allow_finalized=1','-v','verify_current=1',
            '-f',"$tooling_root/scripts/ci/finalize-release-changelog.awk","$tree/$group->{changelog}");
    }
    $versions{$name}={version=>$version,primary=>$plan->{primary},members=>$plan->{members}};
}
# These are stage-only inputs, not replacement release records. The consumer
# must separately qualify actual ledgers/receipts and never cache their admission.
for my $group (@{$policy->{groups}}) {
    write_file("$tree/$group->{changelog}","# Candidate stage only\n\n## [$versions{$group->{name}}{version}] - Unreleased\n");
    for my $path (@{$group->{metadata_files}//[]}) { write_file("$tree/$path","{}\n"); }
}
my %projected=map {$_=>sha256_hex(bytes("$tree/$_"))} @paths;
my $identity=sha256_hex($json->encode({files=>\%projected,modes=>\%modes}));
my $destination="$state/sources/$identity";
regular($destination);
if (-e $destination) {
    -d $destination or die "occupied candidate source\n";
    for my $path (@paths) {
        regular("$destination/$path");
        sha256_hex(bytes("$destination/$path")) eq $projected{$path}
            && (((stat "$destination/$path")[2] & 0111) != 0) == ($modes{$path} eq '100755')
            or die "retained candidate source drift: $path\n";
    }
    chdir $destination or die $!;
    observe('git','status','--porcelain=v1','--untracked-files=all') eq '' or die "retained candidate source is dirty\n";
} else {
    rename $tree,$destination or die "publish candidate source: $!\n";
    chdir $destination or die $!;
    local @ENV{qw(GIT_CONFIG_GLOBAL GIT_CONFIG_NOSYSTEM GIT_AUTHOR_DATE GIT_COMMITTER_DATE)}=('/dev/null','1','2000-01-01T00:00:00Z','2000-01-01T00:00:00Z');
    run("$attempt/preparation.log",'git','init','-q');
    run("$attempt/preparation.log",'git','config','user.name','Validation source');
    run("$attempt/preparation.log",'git','config','user.email','validation@example.invalid');
    run("$attempt/preparation.log",'git','config','core.hooksPath','/dev/null');
    run("$attempt/preparation.log",'git','add','--',@paths);
    run("$attempt/preparation.log",'git','-c','commit.gpgsign=false','commit','-qm','Frozen final candidate qualification source');
    # Managed tool selections remain caller-owned. Consumers pass an explicit
    # read-only tools root; physical activation roots are never redirected.
}
chdir $root or die $!;
observe('git','rev-parse','HEAD') eq "$source\n" && observe('git','status','--porcelain=v1','--untracked-files=all') eq ''
    or die "candidate source/index drifted during rendering\n";
my %metadata=map {$_=>1} grep {m{(?:\A|/)Cargo\.(?:toml|lock)\z}} @paths;
for my $group (@{$policy->{groups}}) {
    $metadata{$group->{changelog}}=1; $metadata{$_}=1 for @{$group->{metadata_files}//[]};
}
my $result={schema=>1,kind=>'cargo-final-candidate-source',source=>$source,root=>$root,
    directory=>$destination,identity=>$identity,versions=>\%versions,original=>{map {$_=>$original{$_}} sort keys %metadata},
    projected=>{map {$_=>$projected{$_}} sort keys %metadata},
    editor=>{path=>$editor,sha256=>tool_digest($editor)}};
write_file("$attempt/projection.json",$json->encode($result)."\n");
print $json->encode($result),"\n";
