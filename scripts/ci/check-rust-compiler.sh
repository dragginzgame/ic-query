#!/usr/bin/env bash
set -euo pipefail

repo_root="$(CDPATH='' cd -P -- "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
# Resolve inherited package policy without fetching or compiling dependencies.
metadata="$(CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0 cargo metadata \
  --manifest-path "$repo_root/Cargo.toml" --no-deps --format-version 1 --locked --offline)"
version="$(printf '%s\n' "$metadata" | perl -MJSON::PP -0777 -e '
  my $metadata = decode_json(<>);
  my %members = map { $_ => 1 } @{$metadata->{workspace_members}};
  my %versions;
  for my $package (@{$metadata->{packages}}) {
    next unless $members{$package->{id}};
    my $version = $package->{rust_version} // "";
    $version =~ /\A(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\z/
      or die "workspace members must declare an exact Rust version\n";
    $versions{$version} = 1;
  }
  keys(%versions) == 1 or die "workspace members must share one Rust version\n";
  print((keys %versions)[0], "\n");
')"
cargo_identity="$(cargo --version)"
rustc_identity="$(rustc --version)"
printf 'Rust compiler policy: %s\n%s\n%s\n' "$version" "$cargo_identity" "$rustc_identity"
if [[ "$cargo_identity" != "cargo $version "* || "$rustc_identity" != "rustc $version "* ]]; then
  echo 'error: Cargo and rustc must match the declared compiler; prepare rust-toolchain.toml and remove conflicting overrides' >&2
  exit 1
fi
