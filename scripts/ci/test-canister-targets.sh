#!/usr/bin/env bash
set -euo pipefail

repo_root="$(CDPATH='' cd -P -- "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
make_bin="$(command -v make)"
# shellcheck source=scripts/ci/fixture-lifecycle.sh
source "$repo_root/scripts/ci/fixture-lifecycle.sh"
mkdir -p "${TMPDIR:-$repo_root/target/tmp}"
work_dir="$(fixture_create "${TMPDIR:-$repo_root/target/tmp}/ic-query-canister-targets.XXXXXX")"
fixture_complete=false
trap 'fixture_finish "$?" "$work_dir" "Canister target fixtures retained"' EXIT

fail() {
  echo "error: $*" >&2
  exit 1
}

canister_case="${work_dir}/canister-preparation"
mkdir -p "$canister_case/bin"
bash "$repo_root/scripts/ci/prepare-script-fixture.sh" "$repo_root" "$canister_case" Makefile
printf 'selected dependency graph\n' > "$canister_case/Cargo.lock"
cp "$canister_case/Cargo.lock" "$canister_case/original-lock"
cat > "$canister_case/bin/cargo" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$*" >> "$TRACE_FILE"
case "$1" in
  fetch)
    [[ "$*" == 'fetch --locked' ]] || exit 58
    [[ "${CARGO_NET_OFFLINE:-}" != true ]] || exit 43
    [[ "${CANISTER_FETCH_FAIL:-}" != yes ]] || exit 47
    : > prepared-cache ;;
  clippy)
    [[ -f prepared-cache ]] || exit 59
    case " $* " in *' --locked --offline -- -D warnings '*) ;; *) exit 58 ;; esac
    [[ "${CANISTER_LINT_FAIL:-}" != "$2" ]] || exit 61 ;;
  build|test)
    [[ -f prepared-cache ]] || exit 59
    case " $* " in *' --locked --offline '*) ;; *) exit 58 ;; esac ;;
  run)
    [[ -f prepared-cache ]] || exit 59
    case " $* " in *' --locked --offline -- '*) ;; *) exit 58 ;; esac ;;
  *) exit 60 ;;
esac
EOF
chmod +x "$canister_case/bin/cargo"
for selection in 'canister-build build' 'canister-bundle bundle' 'canister-smoke local'; do
  read -r target operation <<< "$selection"
  : > "$canister_case/trace"
  PATH="$canister_case/bin:$PATH" TRACE_FILE="$canister_case/trace" CARGO_NET_OFFLINE=false \
    "$make_bin" --no-print-directory -C "$canister_case" "$target" > "$canister_case/output" 2>&1 \
    || fail "$target did not prepare dependencies before offline runner dispatch"
  printf '%s\n' 'fetch --locked' \
    'clippy -p ic-query --all-targets --no-default-features --locked --offline -- -D warnings' \
    'clippy --workspace --all-targets --all-features --locked --offline -- -D warnings' \
    "run -p ic-query-cli --example governance_smoke --locked --offline -- $operation" > "$canister_case/expected"
  cmp "$canister_case/trace" "$canister_case/expected"
  rm "$canister_case/prepared-cache"
  for refusal in offline network; do
    : > "$canister_case/trace"
    offline=false fetch_fail=no
    if [[ "$refusal" == offline ]]; then offline=true; else fetch_fail=yes; fi
    if PATH="$canister_case/bin:$PATH" TRACE_FILE="$canister_case/trace" \
      CARGO_NET_OFFLINE="$offline" CANISTER_FETCH_FAIL="$fetch_fail" \
      "$make_bin" --no-print-directory -C "$canister_case" "$target" > "$canister_case/output" 2>&1; then
      fail "$target ignored $refusal preparation failure"
    fi
    printf '%s\n' 'fetch --locked' > "$canister_case/expected"
    cmp "$canister_case/trace" "$canister_case/expected"
    [[ ! -e "$canister_case/prepared-cache" ]] || fail 'failed preparation changed cache'
  done
  for lint_scope in -p --workspace; do
    : > "$canister_case/trace"
    if PATH="$canister_case/bin:$PATH" TRACE_FILE="$canister_case/trace" \
      CARGO_NET_OFFLINE=false CANISTER_LINT_FAIL="$lint_scope" \
      "$make_bin" --no-print-directory -j4 -C "$canister_case" "$target" > "$canister_case/output" 2>&1; then
      fail "$target ignored failed Clippy gate"
    fi
    printf '%s\n' 'fetch --locked' \
      'clippy -p ic-query --all-targets --no-default-features --locked --offline -- -D warnings' > "$canister_case/expected"
    if [[ "$lint_scope" == --workspace ]]; then
      printf '%s\n' 'clippy --workspace --all-targets --all-features --locked --offline -- -D warnings' >> "$canister_case/expected"
    fi
    cmp "$canister_case/trace" "$canister_case/expected"
    rm "$canister_case/prepared-cache"
  done
  cmp "$canister_case/Cargo.lock" "$canister_case/original-lock"
done

cat > "$canister_case/bin/select-testkit" <<'EOF'
#!/usr/bin/env bash
printf '%s/bin/testkit\n' "$PWD"
EOF
cat > "$canister_case/bin/testkit" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf 'testkit %s\n' "$*" >> "$TRACE_FILE"
case "$1" in
  check) ;;
  run)
    [[ "$IC_QUERY_GOVERNANCE_PROBE_WASM" == "$PWD/target/testkit/wasm32-unknown-unknown/release/examples/governance_probe.wasm" ]] || exit 62
    while [[ "$1" != -- ]]; do shift; done
    shift
    "$@" ;;
  *) exit 63 ;;
esac
EOF
chmod +x "$canister_case/bin/"{select-testkit,testkit}
export PATH="$canister_case/bin:$PATH" TRACE_FILE="$canister_case/trace" CARGO_NET_OFFLINE=true
: > "$canister_case/prepared-cache"
: > "$TRACE_FILE"
"$make_bin" --no-print-directory -C "$canister_case" clippy > "$canister_case/output" 2>&1
(cd "$canister_case" && cargo clippy -p ic-query --example governance_probe --target wasm32-unknown-unknown --no-default-features --features canister --locked --offline -- -D warnings)
for target in canister-test-check canister-smoke-check canister-bundle-check; do
  "$make_bin" --no-print-directory -C "$canister_case" "$target" \
    TESTKIT_TOOL=bin/select-testkit >> "$canister_case/output" 2>&1
done
printf '%s\n' \
  'clippy -p ic-query --all-targets --no-default-features --locked --offline -- -D warnings' \
  'clippy --workspace --all-targets --all-features --locked --offline -- -D warnings' \
  'clippy -p ic-query --example governance_probe --target wasm32-unknown-unknown --no-default-features --features canister --locked --offline -- -D warnings' > "$canister_case/lint-expected"
printf '%s\n' \
  "testkit check --directory $canister_case/.tools/testkit-server" \
  "build -p ic-query --example governance_probe --target wasm32-unknown-unknown --target-dir $canister_case/target/testkit --release --no-default-features --features canister --locked --offline" \
  'test -p ic-query-testkit --test governance --locked --offline --no-run' \
  "testkit run --directory $canister_case/.tools/testkit-server --ttl 300 --idle-ttl 300 -- cargo test -p ic-query-testkit --test governance --locked --offline -- --ignored" \
  'test -p ic-query-testkit --test governance --locked --offline -- --ignored' > "$canister_case/test-expected"
cat "$canister_case/lint-expected" "$canister_case/test-expected" > "$canister_case/expected"
printf '%s\n' \
  'run -p ic-query-cli --example governance_smoke --locked --offline -- local' \
  'run -p ic-query-cli --example governance_smoke --locked --offline -- bundle' >> "$canister_case/expected"
cmp "$TRACE_FILE" "$canister_case/expected"

: > "$TRACE_FILE"
"$make_bin" --no-print-directory -j4 -C "$canister_case" canister-test \
  TESTKIT_TOOL=bin/select-testkit > "$canister_case/output" 2>&1
head -n 2 "$canister_case/lint-expected" > "$canister_case/expected"
cat "$canister_case/test-expected" >> "$canister_case/expected"
cmp "$TRACE_FILE" "$canister_case/expected"
for lint_scope in -p --workspace; do
  : > "$TRACE_FILE"
  if CANISTER_LINT_FAIL="$lint_scope" "$make_bin" --no-print-directory -j4 -C "$canister_case" \
    canister-test TESTKIT_TOOL=bin/select-testkit > "$canister_case/output" 2>&1; then
    fail 'canister-test ignored failed Clippy gate'
  fi
  count=1
  if [[ "$lint_scope" == --workspace ]]; then count=2; fi
  head -n "$count" "$canister_case/lint-expected" > "$canister_case/expected"
  cmp "$TRACE_FILE" "$canister_case/expected"
done
cmp "$canister_case/Cargo.lock" "$canister_case/original-lock"

fixture_complete=true
echo "Canister CI lints once; standalone preparation and lint failures gate product execution"
