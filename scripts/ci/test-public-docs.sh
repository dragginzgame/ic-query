#!/usr/bin/env bash
# Query-owned compiler qualification; lifecycle support stays in the shared snapshot.
set -euo pipefail
root="${BASH_SOURCE[0]}"
[[ "$root" == /* ]] || root="$PWD/$root"
root="$(CDPATH='' cd -P -- "${root%/*}/../.." && printf '%s/.' "$PWD")"
root="${root%/.}"
fixture="$(mktemp -d "${TMPDIR:-/tmp}/rustdoc-debt.XXXXXX")"
fixture="$(CDPATH='' cd -P -- "$fixture" && printf '%s/.' "$PWD")"
fixture="${fixture%/.}"
fixture_complete=false
# shellcheck source=scripts/ci/fixture-lifecycle.sh
source "$root/scripts/ci/fixture-lifecycle.sh"
trap 'fixture_finish "$?" "$fixture" "Rustdoc debt compiler fixture retained"' EXIT
export CARGO_TARGET_DIR="$fixture/target" RUSTDOCFLAGS='-W missing-docs'
export RUSTUP_AUTO_INSTALL=0
mkdir "$fixture/src"
cat > "$fixture/Cargo.toml" <<'MANIFEST'
[package]
name = "rustdoc-debt-fixture"
version = "0.0.0"
edition = "2024"
[workspace]
MANIFEST
cat > "$fixture/src/lib.rs" <<'RUST'
//! Fixture crate.
pub struct First;
pub struct Second;
pub struct r#type;
RUST
cd "$fixture"
cargo generate-lockfile --offline
cargo doc --locked --offline --no-deps --message-format=json > before.jsonl 2> before.log
perl "$root/scripts/ci/check-public-docs.pl" before.jsonl baseline.json --write-baseline
perl "$root/scripts/ci/check-public-docs.pl" before.jsonl baseline.json
cp baseline.json saved-baseline.json
cat > src/lib.rs <<'RUST'
//! Fixture crate.
/// Documented item.
pub struct First;
pub struct Second;
pub struct Third;
pub struct r#match;
RUST
cargo doc --locked --offline --no-deps --message-format=json > after.jsonl 2> after.log
status=0
perl "$root/scripts/ci/check-public-docs.pl" after.jsonl baseline.json > comparison.log 2>&1 || status=$?
[[ "$status" == 1 ]] || { cat comparison.log >&2; exit 1; }
grep -Fq 'newly undocumented item' comparison.log
grep -Fq '::a struct::Third' comparison.log
grep -Fq 'remove resolved baseline item' comparison.log
grep -Fq '::a struct::First' comparison.log
grep -Fq '::a struct::match' comparison.log
grep -Fq '::a struct::type' comparison.log
cmp saved-baseline.json baseline.json
fixture_complete=true
printf 'Real rustdoc declaration identity and unchanged-count debt fixtures passed\n'
