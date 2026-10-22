#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd -P "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
# Cargo resolves workspace inheritance without fetching or compiling dependencies.
metadata="$(CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0 cargo metadata \
  --manifest-path "$repo_root/Cargo.toml" --no-deps --format-version 1 --locked --offline)" || exit "$?"
printf '%s\n' "$metadata" | perl -MJSON::PP -0777 -e '
  my $metadata = decode_json(<>);
  my %members = map { $_ => 1 } @{$metadata->{workspace_members}};
  my %versions;
  for my $package (@{$metadata->{packages}}) {
    next unless $members{$package->{id}};
    my $version = $package->{rust_version} // "";
    $version =~ /\A(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)(?:\.(0|[1-9][0-9]*))?\z/
      or die "workspace members must declare a Rust version\n";
    $versions{defined($3) ? $version : "$version.0"} = 1;
  }
  keys(%versions) == 1 or die "workspace members must share one Rust version\n";
  print((keys %versions)[0], "\n");
'
