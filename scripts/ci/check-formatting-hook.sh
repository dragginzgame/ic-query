#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd -P "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
work_dir="$(mktemp -d "${TMPDIR:-/tmp}/ic-query-formatting-hook.XXXXXX")"
fixture_complete=false
finish() {
  local status=$?
  [[ "$fixture_complete" == true || "$status" != 0 ]] || status=1
  if [[ "$status" == 0 ]]; then rm -rf -- "$work_dir"
  else
    echo "Formatting hook fixtures retained: $work_dir" >&2
    for log in "$work_dir/"*.log "$work_dir/formatting."*; do
      [[ -f "$log" ]] || continue
      printf '\nFormatting fixture log: %s\n' "$log" >&2 || :
      cat "$log" >&2 || :
    done
  fi
  exit "$status"
}
trap finish EXIT
work_dir="$(cd -P "$work_dir" && printf '%s/.' "$PWD")"
work_dir="${work_dir%/.}"
export TMPDIR="${work_dir%/*}" RUNNER_TEMP="$work_dir" CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0
unset GIT_DIR GIT_WORK_TREE GIT_INDEX_FILE GIT_COMMON_DIR GIT_OBJECT_DIRECTORY GIT_ALTERNATE_OBJECT_DIRECTORIES GIT_NAMESPACE
mkdir "$work_dir/templates"
export GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null GIT_TEMPLATE_DIR="$work_dir/templates"

fixture="$work_dir/repository"
git clone --quiet --shared "$repo_root" "$fixture"
cp "$repo_root/Makefile" "$repo_root/.shared-tooling.snapshot" "$fixture/"
for input in Cargo.toml Cargo.lock rust-toolchain.toml crates/ic-query/Cargo.toml crates/ic-query-cli/Cargo.toml; do
  cp "$repo_root/$input" "$fixture/$input"
  git -C "$fixture" add -- "$input"
done
bash "$repo_root/scripts/ci/verify-shared-tooling-snapshot.sh" \
  --read-manifest "$fixture/.shared-tooling.snapshot" > "$work_dir/shared-records"
while IFS=$'\t' read -r kind _digest _mode path; do
  [[ "$kind" == file ]] || continue
  mkdir -p "$fixture/$(dirname "$path")"
  cp -p "$repo_root/$path" "$fixture/$path"
  git -C "$fixture" add -- "$path"
done < "$work_dir/shared-records"
git -C "$fixture" add -- Makefile .shared-tooling.snapshot

# Explicit fixture preparation must reuse the checkout's verified payloads.
bash "$fixture/scripts/dev/install-rust-tools.sh" --consumer "$fixture" \
  --versions "$fixture/ci/tool-versions.env" > "$work_dir/preparation.log" 2>&1
for tool in cargo-sort cargo-sort-derives; do
  original="$(perl -MCwd=abs_path -e 'print abs_path(shift) // die "missing prepared tool\n"' "$repo_root/.tools/rust/common/bin/$tool")"
  reused="$(perl -MCwd=abs_path -e 'print abs_path(shift) // die "missing fixture tool\n"' "$fixture/.tools/rust/common/bin/$tool")"
  [[ "$original" == "$reused" ]] || { echo "fixture did not reuse prepared $tool" >&2; exit 1; }
done
cd "$fixture"
make --no-print-directory fmt-check > "$work_dir/baseline.log" 2>&1
make --no-print-directory install-hooks > "$work_dir/install.log" 2>&1
[[ "$(git config --local --get core.hooksPath)" == .githooks ]]
cp Cargo.toml "$work_dir/manifest"
cp Cargo.lock "$work_dir/lock"

selected=crates/ic-query/src/lib.rs
printf '\npub fn shared_tooling_hook_fixture( ) { }\n' >> "$selected"
perl -0777 -pi -e 's/^(ic-host-artifacts[^\n]*\n)(ic-host-fs[^\n]*\n)/$2$1/m or die "missing sortable fixture pair\n"' Cargo.toml
git add -- "$selected" Cargo.toml
printf '\nUnrelated working evidence.\n' >> README.md
cp README.md "$work_dir/readme"
printf 'untracked evidence\n' > hook-untracked.txt
bash .githooks/pre-commit > "$work_dir/hook.log" 2>&1
grep -Fx 'pub fn shared_tooling_hook_fixture() {}' "$selected" > /dev/null
cmp "$work_dir/manifest" Cargo.toml
cmp "$work_dir/lock" Cargo.lock
cmp "$work_dir/readme" README.md
git diff --cached --exit-code HEAD -- README.md > "$work_dir/readme-index.diff"
[[ "$(cat hook-untracked.txt)" == 'untracked evidence' && -z "$(git ls-files -- hook-untracked.txt)" ]]
git diff --exit-code -- "$selected" Cargo.toml > "$work_dir/selected.diff"
make --no-print-directory fmt-check > "$work_dir/check.log" 2>&1

# A partial selection must refuse before formatting or changing the payload.
printf '\n' >> "$selected"
git add -- "$selected"
printf '\nUnstaged fixture evidence.\n' >> "$selected"
cp "$selected" "$work_dir/partial"
tree="$(git write-tree)"
if bash .githooks/pre-commit > "$work_dir/partial.log" 2>&1; then
  echo 'formatting hook admitted a partially staged file' >&2; exit 1
fi
[[ "$(git write-tree)" == "$tree" ]]
cmp "$work_dir/partial" "$selected"
cmp "$work_dir/lock" Cargo.lock
cmp "$work_dir/readme" README.md
echo 'Query formatting hook, receipt admission, prepared-tool reuse and selection preservation passed'
fixture_complete=true
