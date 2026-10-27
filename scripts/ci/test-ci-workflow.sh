#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
yq="$repo_root/.tools/host/bin/yq"
jq="$repo_root/.tools/host/bin/jq"

# shellcheck disable=SC2016 # Dollar-prefixed names belong to jq.
"$yq" -o=json '.' "$repo_root/.github/workflows/ci.yml" | "$jq" -e '
  def selected($event; $ref):
    .on as $triggers |
    if $event == "push" then
      if $ref | startswith("refs/heads/") then
        any($triggers.push.branches[]?; . == ($ref | ltrimstr("refs/heads/")))
      else
        any($triggers.push.tags[]?;
          . == ($ref | ltrimstr("refs/tags/")) or
          . == "*" or (. == "v*" and ($ref | startswith("refs/tags/v"))))
      end
    else $triggers | has($event)
    end;

  . as $workflow |
  [
    ["pull_request", "refs/pull/1/merge", true],
    ["push", "refs/heads/main", true],
    ["push", "refs/heads/topic", false],
    ["push", "refs/tags/v0.52.2", false],
    ["workflow_dispatch", "refs/heads/main", true],
    ["workflow_dispatch", "refs/heads/release", true],
    ["workflow_dispatch", "refs/tags/v0.52.2", true]
  ] |
  all(.[]; . as $case | ($workflow | selected($case[0]; $case[1])) == $case[2]) and
  ($workflow.on.push == {branches:["main"]}) and
  ($workflow.jobs | keys == ["canister", "checks", "msrv"]) and
  ($workflow.jobs | all(.[];
    (has("if") | not) and
    (.strategy.matrix.host | sort == ["macos-15", "macos-15-intel", "ubuntu-24.04"]) and
    all(.steps[]; .with.ref == null))) and
  ($workflow.jobs.msrv.steps | any(.[]; .id == "msrv" and .shell == "bash")) and
  ($workflow.jobs.msrv.steps | any(.[];
    ((.uses // "") | startswith("dtolnay/rust-toolchain@")) and
    .with.toolchain == "${{ steps.msrv.outputs.version }}")) and
  ($workflow.jobs.msrv.steps | any(.[];
    .env.MSRV == "${{ steps.msrv.outputs.version }}" and
    .run == "echo \"RUSTUP_TOOLCHAIN=$MSRV\" >> \"$GITHUB_ENV\"")) and
  ($workflow.jobs.canister.steps | any(.[]; .if == "always()" and ((.uses // "") | startswith("actions/upload-artifact@")))) and
  ($workflow.jobs.checks.steps | any(.[]; .if == "failure()" and ((.uses // "") | startswith("actions/upload-artifact@")))) and
  ($workflow.jobs.checks.steps | all(.[] | select((.uses // "") | startswith("actions/upload-artifact@"));
    (.with.path | split("\n") | map(select(length > 0))) == ["${{ runner.temp }}/ic-query-tool-evidence.*/"])) and
  ([$workflow.jobs.checks, $workflow.jobs.canister] | all(.[];
    any(.steps[]; ((.uses // "") | startswith("actions/upload-artifact@")) and
      ((.with.path // "") | contains("ic-query-tool-evidence.*/")) and
      .with["include-hidden-files"] == true and .with["retention-days"] == 30)))
' > /dev/null
echo 'CI event selection and nine-job native coverage fixtures passed'

fixture="$(mktemp -d "${TMPDIR:-/tmp}/ic-query-ci-retention.XXXXXX")"
fixture_complete=false
finish() {
  local status=$?
  [[ "$fixture_complete" == true || "$status" != 0 ]] || status=1
  if [[ "$status" == 0 ]]; then rm -rf -- "$fixture"
  else echo "CI retention fixtures retained: $fixture" >&2; fi
  exit "$status"
}
trap finish EXIT
fixture="$(cd -P "$fixture" && printf '%s/.' "$PWD")"
fixture="${fixture%/.}"
mkdir -p "$fixture/msrv/bin"
"$yq" -o=json '.' "$repo_root/.github/workflows/ci.yml" | "$jq" -er \
  '.jobs.msrv.steps[] | select(.id == "msrv") | .run' > "$fixture/msrv/select.sh"
cat > "$fixture/msrv/bin/rustup" <<'STUB'
#!/usr/bin/env bash
[[ "$*" == default ]] || exit 60
printf 'fixture-bootstrap (default)\n'
STUB
cat > "$fixture/msrv/bin/make" <<'STUB'
#!/usr/bin/env bash
[[ "$*" == '--no-print-directory -s msrv-version' ]] || exit 61
[[ "$RUSTUP_TOOLCHAIN" == fixture-bootstrap && "$RUSTUP_AUTO_INSTALL" == 0 ]] || exit 62
printf '1.87.0\n'
STUB
chmod +x "$fixture/msrv/bin/"{rustup,make}
PATH="$fixture/msrv/bin:$PATH" RUSTUP_TOOLCHAIN=unprepared-repository-toolchain \
  GITHUB_OUTPUT="$fixture/msrv/output" "$BASH" "$fixture/msrv/select.sh"
grep -Fxq 'version=1.87.0' "$fixture/msrv/output"
echo 'CI bootstraps MSRV selection with an installed compiler and preserves the declared floor'
# shellcheck disable=SC2016 # Dollar-prefixed names belong to jq.
"$yq" -o=json '.' "$repo_root/.github/workflows/ci.yml" | "$jq" -er '
  [.jobs.checks.steps[], .jobs.canister.steps[] | select(.name == "Archive retained tool evidence")]
  | . as $collectors
  | if length == 2 and all(.[]; .if == "failure()" and .run == $collectors[0].run) then .[0].run
    else error("missing tool evidence owner") end
' > "$fixture/collect.sh"
mkdir -p "$fixture/repository/.tools/rust/slot/installed" "$fixture/temp" "$fixture/outside"
server_attempt="$fixture/repository/.tools/testkit-server/.setup-v1-fixture"
mkdir -p "$server_attempt"
printf 'curl: (28) connection timed out\n' > "$server_attempt/download.stderr"
printf 'partial archive bytes\n' > "$server_attempt/archive.gz"
chmod 640 "$server_attempt/download.stderr"
ln -s "$fixture/outside" "$server_attempt/external-link"
printf 'retained build\n' > "$fixture/repository/.tools/rust/slot/build.log"
printf 'retained receipt\n' > "$fixture/repository/.tools/rust/slot/installed/selection.json"
printf 'private outside bytes\n' > "$fixture/outside/private"
ln -s "$fixture/outside" "$fixture/repository/.tools/rust/external-link"
export SHARED_TOOLING_STORE="$fixture/store"
host_slot="host-$(printf '%064d' 0)"
mkdir -p "$fixture/store/$host_slot/set.fixture" "$fixture/store/unrelated" \
  "$fixture/repository/.tools/ic-set.retained"
chmod 0700 "$fixture/store" "$fixture/store/$host_slot" "$fixture/store/$host_slot/set.fixture"
printf 'shared bundle diagnostics\n' > "$fixture/store/$host_slot/set.fixture/install.log"
printf 'retained local IC diagnostics\n' > "$fixture/repository/.tools/ic-set.retained/install.log"
printf 'unrelated checkout bytes\n' > "$fixture/store/unrelated/private"
ln -s "$fixture/store/$host_slot/set.fixture" "$fixture/repository/.tools/host-set.fixture"
ln -s host-set.fixture "$fixture/repository/.tools/host"
ln -s "$fixture/outside" "$fixture/repository/.tools/host-set.external"
rust_slot="rust-$(printf '%064d' 1)"
mkdir -p "$fixture/store/$rust_slot/set.complete" "$fixture/store/$rust_slot/set.failed/build"
chmod 0700 "$fixture/store/$rust_slot" "$fixture/store/$rust_slot/set.complete" "$fixture/store/$rust_slot/set.failed"
printf 'shared Cargo receipt\n' > "$fixture/store/$rust_slot/set.complete/selection.json"
printf 'failed Cargo build\n' > "$fixture/store/$rust_slot/set.failed/build/install.log"
ln -s "$fixture/store/$rust_slot/set.complete" "$fixture/repository/.tools/rust/rust-set.complete"
ln -s "$fixture/store/$rust_slot/set.failed" "$fixture/repository/.tools/rust/rust-set.failed"
ln -s "$fixture/outside" "$fixture/repository/.tools/rust/rust-set.external"
(
  cd "$repo_root"
  GITHUB_WORKSPACE="$fixture/repository" RUNNER_TEMP="$fixture/temp" "$BASH" "$fixture/collect.sh"
)
archives=("$fixture/temp/"ic-query-tool-evidence.*/tools.tar.gz)
[[ "${#archives[@]}" == 1 && -f "${archives[0]}" ]] || exit 1
mkdir "$fixture/unpacked"
tar -xzf "${archives[0]}" -C "$fixture/unpacked"
for name in download.stderr archive.gz; do
  cmp "$server_attempt/$name" "$fixture/unpacked/.tools/testkit-server/.setup-v1-fixture/$name" || exit 1
done
[[ "$(perl -e 'printf "%o", (stat($ARGV[0]))[2] & 0777' "$fixture/unpacked/.tools/testkit-server/.setup-v1-fixture/download.stderr")" == 640 ]] || exit 1
[[ -L "$fixture/unpacked/.tools/testkit-server/.setup-v1-fixture/external-link" ]] || exit 1
cmp "$fixture/repository/.tools/rust/slot/build.log" "$fixture/unpacked/.tools/rust/slot/build.log" || exit 1
cmp "$fixture/repository/.tools/rust/slot/installed/selection.json" "$fixture/unpacked/.tools/rust/slot/installed/selection.json" || exit 1
[[ -L "$fixture/unpacked/.tools/rust/external-link" ]] || exit 1
[[ -L "$fixture/unpacked/.tools/host-set.fixture" && -L "$fixture/unpacked/.tools/host-set.external" ]] || exit 1
cmp "$fixture/store/$host_slot/set.fixture/install.log" "$fixture/unpacked/$host_slot/set.fixture/install.log" || exit 1
cmp "$fixture/repository/.tools/ic-set.retained/install.log" "$fixture/unpacked/.tools/ic-set.retained/install.log" || exit 1
cmp "$fixture/store/$rust_slot/set.complete/selection.json" "$fixture/unpacked/$rust_slot/set.complete/selection.json" || exit 1
cmp "$fixture/store/$rust_slot/set.failed/build/install.log" "$fixture/unpacked/$rust_slot/set.failed/build/install.log" || exit 1
[[ -L "$fixture/unpacked/.tools/rust/rust-set.external" ]] || exit 1
tar -tzf "${archives[0]}" > "$fixture/archive-paths"
if grep -Eq '/external-link/|/host-set.external/|/rust-set.external/|unrelated|/private$' "$fixture/archive-paths"; then exit 1; fi
# A redirected Rust root must never select its target for archiving.
mv "$fixture/repository/.tools/rust" "$fixture/retained-rust"
ln -s "$fixture/retained-rust" "$fixture/repository/.tools/rust"
(
  cd "$repo_root"
  GITHUB_WORKSPACE="$fixture/repository" RUNNER_TEMP="$fixture/temp" "$BASH" "$fixture/collect.sh"
)
remaining=("$fixture/temp/"ic-query-tool-evidence.*/tools.tar.gz)
[[ "${#remaining[@]}" == 2 ]] || exit 1
for archive in "${remaining[@]}"; do
  [[ "$archive" != "${archives[0]}" ]] || continue
  tar -tzf "$archive" > "$fixture/redirected-paths"
  if grep -Fq '.tools/rust/' "$fixture/redirected-paths"; then exit 1; fi
done
# A rejected store still retains safe local bytes and its original diagnostic.
rm "$fixture/repository/.tools/rust"
mv "$fixture/retained-rust" "$fixture/repository/.tools/rust"
ln -s "$fixture/store" "$fixture/rejected-store"
if (
  cd "$repo_root"
  SHARED_TOOLING_STORE="$fixture/rejected-store" GITHUB_WORKSPACE="$fixture/repository" \
    RUNNER_TEMP="$fixture/temp" "$BASH" "$fixture/collect.sh"
) > "$fixture/rejected-store.log" 2>&1; then exit 1; fi
remaining=("$fixture/temp/"ic-query-tool-evidence.*/tools.tar.gz)
[[ "${#remaining[@]}" == 3 ]] || exit 1
diagnostics=("$fixture/temp/"ic-query-tool-evidence.*/diagnostics/selection.log)
for diagnostic in "${diagnostics[@]}"; do
  grep -Fq 'tool store root may not be a symlink' "$diagnostic" || continue
  mkdir "$fixture/partial"
  grep -Fq 'Shared tool evidence incomplete:' "$diagnostic" || exit 1
  tar -xzf "${diagnostic%/diagnostics/*}/tools.tar.gz" -C "$fixture/partial"
done
cmp "$fixture/repository/.tools/rust/slot/build.log" "$fixture/partial/.tools/rust/slot/build.log" || exit 1
cmp "$fixture/repository/.tools/rust/slot/installed/selection.json" "$fixture/partial/.tools/rust/slot/installed/selection.json" || exit 1
cmp "$server_attempt/download.stderr" "$fixture/partial/.tools/testkit-server/.setup-v1-fixture/download.stderr" || exit 1
cmp "$server_attempt/archive.gz" "$fixture/partial/.tools/testkit-server/.setup-v1-fixture/archive.gz" || exit 1
[[ ! -e "$fixture/partial/$rust_slot" && ! -e "$fixture/partial/$host_slot" ]] || exit 1
# A redirected checkout tool root cannot select any outside tree.
mv "$fixture/repository/.tools" "$fixture/retained-tools"
ln -s "$fixture/retained-tools" "$fixture/repository/.tools"
(
  cd "$repo_root"
  GITHUB_WORKSPACE="$fixture/repository" RUNNER_TEMP="$fixture/temp" "$BASH" "$fixture/collect.sh"
)
remaining=("$fixture/temp/"ic-query-tool-evidence.*/tools.tar.gz)
[[ "${#remaining[@]}" == 3 ]] || exit 1
cmp <(printf 'private outside bytes\n') "$fixture/outside/private" || exit 1
cmp <(printf 'unrelated checkout bytes\n') "$fixture/store/unrelated/private" || exit 1
# Early setup failure may leave no Testkit directory. Its absence must not
# prevent collecting the common tool evidence selected by this workflow.
rm "$fixture/repository/.tools"
mv "$fixture/retained-tools" "$fixture/repository/.tools"
mv "$fixture/repository/.tools/testkit-server" "$fixture/retained-testkit"
mkdir "$fixture/early-temp" "$fixture/early-unpacked"
(
  cd "$repo_root"
  GITHUB_WORKSPACE="$fixture/repository" RUNNER_TEMP="$fixture/early-temp" "$BASH" "$fixture/collect.sh"
)
early=("$fixture/early-temp/"ic-query-tool-evidence.*/tools.tar.gz)
[[ "${#early[@]}" == 1 && -f "${early[0]}" ]] || exit 1
tar -xzf "${early[0]}" -C "$fixture/early-unpacked"
cmp "$fixture/repository/.tools/rust/slot/build.log" "$fixture/early-unpacked/.tools/rust/slot/build.log" || exit 1
[[ ! -e "$fixture/early-unpacked/.tools/testkit-server" ]] || exit 1
echo 'CI retains Testkit setup failures, local Rust/IC evidence and selected shared bundles without traversing unknown links or unrelated storage'
fixture_complete=true
