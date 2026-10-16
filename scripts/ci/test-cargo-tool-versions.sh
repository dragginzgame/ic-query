#!/usr/bin/env bash
# Shared companions: scripts/dev/check-cargo-tool-versions.pl
set -euo pipefail
ROOT="${BASH_SOURCE[0]}"
[[ "$ROOT" == /* ]] || ROOT="$PWD/$ROOT"
ROOT="$(cd -P "${ROOT%/*}/../.." && printf '%s/.' "$PWD")"; ROOT="${ROOT%/.}"
fixture="$(mktemp -d "${TMPDIR:-/tmp}/cargo-version-test.XXXXXX")"
fixture_complete=false
finish() {
    local status=$?
    [[ "$fixture_complete" == true || "$status" != 0 ]] || status=1
    if [[ "$status" == 0 ]]; then rm -rf "$fixture"
    else printf 'Cargo version fixture retained: %s\n' "$fixture" >&2; fi
    exit "$status"
}
trap finish EXIT
export VERSION_FIXTURE="$fixture"
mkdir -p "$fixture/bin" "$fixture/home" "$fixture/consumer/ci" "$fixture/consumer/.tools/rust/common/bin" "$fixture/bundle/bin" "$fixture/evidence"
export TMPDIR="$fixture/evidence"
cat > "$fixture/bin/curl" <<'CURL'
#!/usr/bin/env bash
set -eu
if [[ "$*" == '--disable --version' ]]; then echo "curl ${VERSION_CURL:-8.4.0} fixture"; exit 0; fi
url="${!#}"; package="${url##*/}"
printf '%s\n' "$*" >> "$VERSION_FIXTURE/calls"
while [[ $# -gt 0 ]]; do
    case "$1" in
        --output) cp "$VERSION_FIXTURE/$package.json" "$2"; shift 2 ;;
        --stderr) : > "$2"; shift 2 ;;
        *) shift ;;
    esac
done
printf '%s' "${VERSION_HTTP:-200}"
exit "${VERSION_TRANSPORT:-0}"
CURL
chmod +x "$fixture/bin/curl"
export PATH="$fixture/bin:$PATH"
mkdir -p "$fixture/snapshot/scripts/ci" "$fixture/snapshot/scripts/dev" "$fixture/snapshot/ci"
cp "$ROOT/scripts/ci/StableVersion.pm" "$fixture/snapshot/scripts/ci/"
cp "$ROOT/scripts/dev/check-cargo-tool-versions.pl" "$fixture/snapshot/scripts/dev/"
printf 'export SHARED_TOOLING_CARGO_SORT_VERSION=2.1.4\n' > "$fixture/consumer/ci/tool-versions.env"
cp "$fixture/consumer/ci/tool-versions.env" "$fixture/snapshot/ci/tool-versions.env"
printf 'fixture binary; never execute\n' > "$fixture/bundle/bin/cargo-sort"
ln -s "$fixture/bundle/bin/cargo-sort" "$fixture/consumer/.tools/rust/common/bin/cargo-sort"
jq -n '{installs:{"cargo-sort 2.1.4 (registry+https://github.com/rust-lang/crates.io-index)":{}}}' > "$fixture/bundle/.crates2.json"
cp "$fixture/bundle/.crates2.json" "$fixture/home/.crates2.json"
jq '.installs["twiggy 0.8.0 (registry+https://github.com/rust-lang/crates.io-index)"]={}' "$fixture/home/.crates2.json" > "$fixture/global-receipt"
cp "$fixture/global-receipt" "$fixture/home/.crates2.json"
jq -n '{crate:{id:"twiggy"},versions:[{crate:"twiggy",num:"0.8.0",yanked:false}]}' > "$fixture/twiggy.json"
jq -n '{crate:{id:"cargo-sort"},versions:[
 {crate:"cargo-sort",num:"2.1.4",yanked:false,rust_version:"1.82"},
 {crate:"cargo-sort",num:"3.0.0-rc.1",yanked:false},
 {crate:"cargo-sort",num:"4.0.0",yanked:true}]}' > "$fixture/cargo-sort.json"
run() {
    local expected="$1" status=0
    perl "$fixture/snapshot/scripts/dev/check-cargo-tool-versions.pl" --consumer "$fixture/consumer" \
        --versions "$fixture/consumer/ci/tool-versions.env" --cargo-home "$fixture/home" --json > "$fixture/report.json" 2> "$fixture/stderr" || status=$?
    [[ "$status" == "$expected" ]] || { cat "$fixture/report.json" "$fixture/stderr"; exit 1; }
}
run 0
# Default selection comes from the executing snapshot, never the consumer file.
printf 'not a pin catalog\n' > "$fixture/consumer/ci/tool-versions.env"
perl "$fixture/snapshot/scripts/dev/check-cargo-tool-versions.pl" --consumer "$fixture/consumer" \
    --cargo-home "$fixture/home" --json > "$fixture/default.json"
jq -e '.tools[0].pinned=="2.1.4"' "$fixture/default.json" >/dev/null
cp "$fixture/snapshot/ci/tool-versions.env" "$fixture/consumer/ci/tool-versions.env"
jq -e '.tools[0] | .status=="current" and .latest=="2.1.4" and .local=="2.1.4" and .global=="2.1.4"' "$fixture/report.json" > /dev/null
jq -e '.tools[1] | .tool=="twiggy" and .pinned==null and .local==null and .status=="current"' "$fixture/report.json" > /dev/null
# Compute numeric latest from eligible versions, independently of ordering or max fields.
jq '.versions += [{crate:"cargo-sort",num:"2.1.10",yanked:false},{crate:"cargo-sort",num:"2.1.9",yanked:false}]' "$fixture/cargo-sort.json" > "$fixture/new.json"
cp "$fixture/new.json" "$fixture/cargo-sort.json"
run 1
jq -e '.tools[0] | .latest=="2.1.10" and (.status|contains("pinned outdated"))' "$fixture/report.json" > /dev/null
evidence="$(jq -r .evidence "$fixture/report.json")"
cmp "$fixture/cargo-sort.json" "$evidence/cargo-sort.json"
[[ -s "$evidence/cargo-sort.request" ]]
# A separately selected local version is visible, as is a missing local binary.
sed 's/2.1.4/2.1.10/g' "$fixture/bundle/.crates2.json" > "$fixture/changed-receipt"
cp "$fixture/changed-receipt" "$fixture/bundle/.crates2.json"
run 1
jq -e '.tools[0].status | contains("local differs from pin")' "$fixture/report.json" > /dev/null
rm "$fixture/consumer/.tools/rust/common/bin/cargo-sort"
run 1
jq -e '.tools[0].status | contains("local missing")' "$fixture/report.json" > /dev/null
# Unavailable observations cannot become a false current result.
for http in 404 429 503; do VERSION_HTTP="$http" run 2; done
VERSION_HTTP=200 VERSION_TRANSPORT=28 run 2
jq -e '.tools[0].status=="ERROR"' "$fixture/report.json" > /dev/null
cp "$fixture/cargo-sort.json" "$fixture/valid.json"
for malformed in identity invalid yanked; do
    case "$malformed" in
        identity) jq '.crate.id="other"' "$fixture/valid.json" > "$fixture/cargo-sort.json" ;;
        invalid) printf 'not JSON\n' > "$fixture/cargo-sort.json" ;;
        yanked) jq '.versions[].yanked="false"' "$fixture/valid.json" > "$fixture/cargo-sort.json" ;;
    esac
    run 2
    jq -e '.tools[1].status=="current"' "$fixture/report.json" > /dev/null
done
cp "$fixture/valid.json" "$fixture/cargo-sort.json"
VERSION_CURL=8.3.0 run 2
cat > "$fixture/consumer/ci/tool-versions.env" <<'PINS'
SHARED_TOOLING_CARGO_SORT_VERSION=$(touch "$VERSION_FIXTURE/unexpected") # pin=2.1.4
PINS
run 2
[[ ! -e "$fixture/unexpected" ]]
# Unsupported registry identities and malformed receipts are incomplete checks.
printf 'SHARED_TOOLING_CARGO_SORT_VERSION=2.1.4\n' > "$fixture/consumer/ci/tool-versions.env"
printf '{"installs":{"twiggy 0.8.0 (git+https://example.invalid/repo)":{}}}\n' > "$fixture/home/.crates2.json"
run 2
printf 'invalid JSON\n' > "$fixture/home/.crates2.json"
run 2
# Registry ordering uses the same exact comparator as release admission, even
# beyond native integer precision. Neither release order nor input order wins.
cp "$fixture/bundle/.crates2.json" "$fixture/home/.crates2.json"
jq '.versions += [
 {crate:"cargo-sort",num:"18446744073709551617.0.0",yanked:false},
 {crate:"cargo-sort",num:"18446744073709551616.0.0",yanked:false}]' \
    "$fixture/valid.json" > "$fixture/cargo-sort.json"
run 1
jq -e '.tools[0].latest=="18446744073709551617.0.0"' "$fixture/report.json" >/dev/null
echo 'Cargo version freshness, separate selections and incomplete observation tests passed (substitute registry)'
fixture_complete=true
