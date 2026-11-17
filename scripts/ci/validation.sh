#!/usr/bin/env bash
# Query policy over the shared roster, baseline and evidence interfaces.
set -euo pipefail
root="$(git rev-parse --show-toplevel)"
[[ "$PWD" == "$root" ]] || exit 2
shared="$root/vendor/toko-tooling/scripts/ci"
export PATH="$root/.tools/host/bin:$root/.tools/rust/common/bin:$PATH"
mode="${1:?select full, native, or plan}"
mkdir -p target/validation
work="$(mktemp -d target/validation/selection.XXXXXX)"
jq -cS '[.groups[].name]' ci/cargo-release-groups.json > "$work/groups.json"
perl "$shared/validation-evidence.pl" roster "$root" ci/release-validation.json "$work/groups.json" > "$work/roster"
case "$mode" in
    full|native)
        targets=()
        while IFS= read -r target; do
            [[ "$mode" != native || "$target" != validate-common ]] || continue
            targets+=("$target")
        done < "$work/roster"
        VALIDATION_REPOSITORY_ROOT="$root" bash "$shared/run-validation-targets.sh" --fail-fast "${targets[@]}"
        ;;
    plan)
        [[ $# == 2 && -d "$2" && ! -L "$2" ]] || exit 2
        destination="$2"
        source="${RELEASE_SOURCE:?}"
        group="${RELEASE_GROUP:?}"
        previous="${RELEASE_PREVIOUS:?}"
        [[ "$(git rev-parse HEAD)" == "$source" ]] || exit 1
        perl "$shared/validation-evidence.pl" baseline "$root" "$source" \
            "refs/tags/$group-v$previous" > "$destination/baseline.json"
        jq -cnS --arg host "$(uname -sm)" --arg rust "$(rustc -Vv)" \
            --arg cargo "$(cargo -V)" --arg source "$source" \
            '{host:$host,rust:$rust,cargo:$cargo,source:$source,contract:1}' > "$destination/context.json"
        # Bind the complete tracked inventory, including helpers and recipes.
        git ls-files -z | jq -RscS 'split("\u0000") | map(select(length > 0))' > "$destination/inventory.json"
        jq -cS --arg directory "$destination" \
            '. + [$directory+"/inventory.json",$directory+"/baseline.json",$directory+"/context.json"]' \
            "$destination/inventory.json" > "$work/inputs.json"
        if [[ -f "$destination/projection.json" ]]; then
            jq -cS --arg directory "$destination" '. + [$directory+"/projection.json",$directory+"/reuse-environment.json"]' \
                "$work/inputs.json" > "$work/reuse-inputs.json"
            mv "$work/reuse-inputs.json" "$work/inputs.json"
        fi
        jq -cnS --arg source "$source" --arg group "$group" \
            --slurpfile baseline "$destination/baseline.json" \
            --slurpfile policy ci/release-validation.json --slurpfile inputs "$work/inputs.json" \
            -f scripts/ci/select-validation.jq > "$destination/plan.json"
        jq -r '.scope, (.stages[] | "\(.target): \(.required) — \(.reason)")' "$destination/plan.json" >&2
        ;;
    *) exit 2 ;;
esac
