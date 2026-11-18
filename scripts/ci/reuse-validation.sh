#!/usr/bin/env bash
# Query's candidate-stage policy; Tooling owns projection and exact-input proofs.
set -euo pipefail
root="$(git rev-parse --show-toplevel)"
shared="$root/vendor/toko-tooling/scripts/ci"
tools_root="${QUERY_VALIDATION_TOOLS_ROOT:-$root}"
export PATH="$tools_root/.tools/host/bin:$tools_root/.tools/rust/common/bin:$PATH"
source "$shared/release-receipts.sh"
source "$shared/release-timing.sh"
trap 'release_timing_end "$?"' EXIT
mode="${1:?prepare, products or stage}"
case "$mode" in
    prepare)
        attempt="${2:?attempt directory}"
        # The published native observer is Linux-only. Build overrides/configs
        # need ordinary current-source checks, not silently sanitized builds.
        if [[ "$(uname -s)" != Linux ]] || env | rg '^(RUSTFLAGS|RUSTDOC[^=]*|RUSTUP_TOOLCHAIN|CARGO_ENCODED_[^=]*|CARGO_BUILD_[^=]*|CARGO_TARGET_[^=]*|CARGO_PROFILE_[^=]*|RUSTC[^=]*|CC|CXX|AR|CFLAGS|CXXFLAGS|LDFLAGS|LD_LIBRARY_PATH|LIBRARY_PATH|PKG_CONFIG[^=]*|OPENSSL[^=]*)=.+' >/dev/null; then
            echo 'Candidate reuse unavailable for this host/build environment; executing current-source checks.' >&2
            exit 0
        fi
        directory="$root"
        while :; do
            if [[ -e "$directory/.cargo/config" || -e "$directory/.cargo/config.toml" ]]; then
                echo 'Ancestor Cargo configuration requires current-source checks.' >&2; exit 0
            fi
            [[ "$directory" != / ]] || break
            directory="${directory%/*}"; directory="${directory:-/}"
        done
        cargo_home="${CARGO_HOME:-$HOME/.cargo}"
        if [[ -e "$cargo_home/config" || -e "$cargo_home/config.toml" ]]; then
            echo 'Cargo home configuration requires current-source checks.' >&2; exit 0
        fi
        # The only present version-sensitive product surface is CLI version/help.
        # New package-version readers or build scripts require a new contract.
        if rg -l 'CARGO_PKG_' crates/ic-query/src crates/ic-query-topology/src crates/ic-query-sns/src > "$attempt/version-readers" \
            || [[ -n "$(git ls-files '*/build.rs')" ]]; then
            echo 'New version-sensitive library/build inputs require current-source checks.' >&2; exit 0
        fi
        # Admit the supported system compiler before producing any reusable
        # source. Other compiler layouts retain the ordinary product checks.
        case "$(command -v cc)" in /usr/bin/cc|/bin/cc) ;; *) echo 'Non-system compiler requires current-source checks.' >&2; exit 0 ;; esac
        native_programs=()
        for command in cc1 cc1plus collect2 lto1; do
            program="$(cc -print-prog-name="$command")"
            if [[ "$program" != /* || ! -f "$program" ]]; then
                echo 'Unqualified native compiler layout; executing current-source checks.' >&2; exit 0
            fi
            native_programs+=("$program")
        done
        for command in cmake pkg-config as ranlib nm objcopy objdump; do
            if ! command -v "$command" >/dev/null; then
                echo 'Incomplete native tool closure; executing current-source checks.' >&2; exit 0
            fi
        done
        release_timing_begin query-candidate-editor
        editor="$(bash "$root/vendor/toko-tooling/scripts/dev/install-rust-tools.sh" --consumer "$tools_root" \
            --package ic-build --version 0.73.0 --bin ic-build-release-metadata --profile debug --features cli --check)"
        release_timing_end 0
        release_timing_begin query-candidate-projection
        release_receipt_write "$attempt/projection.json" perl "$shared/prepare-cargo-stage-source.pl" \
            --root "$root" --state target/validation/candidates --source "${RELEASE_SOURCE:?}" --editor "$editor"
        jq -e --arg group "${RELEASE_GROUP:?}" --arg version "${RELEASE_VERSION:?}" \
            '.versions[$group].version == $version' "$attempt/projection.json" >/dev/null
        candidate="$(jq -er .directory "$attempt/projection.json")"
        identity="$(jq -er .identity "$attempt/projection.json")"
        release_timing_end 0
        release_timing_begin query-candidate-dependencies
        (cd "$candidate" && cargo fetch --locked)
        release_timing_end 0
        release_timing_begin query-tool-inventory
        mkdir -p "$candidate/target/query-reuse"
        # A recursive Make invocation must not change the stage identity merely
        # by prepending the same tool directories again. Select this bounded
        # native tool path explicitly, then observe every resolved executable.
        cargo_bin="$(dirname "$(command -v cargo)")"
        rust_bin="$(dirname "$(command -v rustc)")"
        export PATH="$tools_root/.tools/host/bin:$tools_root/.tools/rust/common/bin:$cargo_bin:$rust_bin:/usr/bin:/bin"
        # Explicit tool closure supplements the shared Rust/dependency/native
        # inventory. ldd is read-only here; no consumer hashing protocol.
        executables=()
        for command in bash perl make cargo rustc rustdoc rustup cc ld ar as ranlib nm objcopy objdump \
            cmake pkg-config git jq yq sed awk sort sha256sum sha512sum grep cut basename stat find \
            cat cp mv rm mkdir dirname uname date mktemp env tee head tail tr readlink tar cmp; do
            executables+=("$(command -v "$command")")
        done
        executables+=("${native_programs[@]}")
        printf '%s\n' "${executables[@]}" > "$attempt/tools.paths"
        compiler_root="$(dirname "$(cc -print-file-name=libgcc.a)")"
        [[ "$compiler_root" == /* && -d "$compiler_root" ]] || exit 1
        find -L "$compiler_root" -type f >> "$attempt/tools.paths"
        for directory in /usr/share/cmake /usr/share/cmake-*; do
            [[ ! -d "$directory" ]] || find -L "$directory" -type f >> "$attempt/tools.paths"
        done
        ldd "${executables[@]}" > "$attempt/tools.ldd" 2>&1 || : # Static executables have no dynamic dependencies.
        if rg -q '=> not found' "$attempt/tools.ldd"; then cat "$attempt/tools.ldd" >&2; exit 1; fi
        awk '/=> \/|^[[:space:]]*\// {for(i=1;i<=NF;i++) if($i ~ /^\// && $i !~ /:$/) print $i}' "$attempt/tools.ldd" >> "$attempt/tools.paths"
        jq -RscS '{files:(split("\n") | map(select(length > 0)) | unique)}' "$attempt/tools.paths" > "$attempt/tools.json"
        tools_file="$candidate/target/query-reuse/tools.$(git hash-object "$attempt/tools.json").json"
        if [[ -e "$tools_file" ]]; then
            cmp "$attempt/tools.json" "$tools_file"
        else
            release_receipt_write "$tools_file" cat "$attempt/tools.json"
        fi
        jq -cnS --arg path "$PATH" --arg home "$HOME" --arg cargo_home "$cargo_home" \
            --arg rustup_home "${RUSTUP_HOME:-$HOME/.rustup}" --arg tools "$tools_file" --arg identity "$identity" \
            '{PATH:$path,HOME:$home,CARGO_HOME:$cargo_home,RUSTUP_HOME:$rustup_home,
              RUSTUP_AUTO_INSTALL:"0",LC_ALL:"C.UTF-8",TZ:"UTC",
              CARGO_STAGE_TOOLS:$tools,CARGO_STAGE_IDENTITY:$identity}
              + ($ENV | with_entries(select(.key == "CARGO_NET_OFFLINE")))' > "$attempt/reuse-environment.json"
        release_timing_end 0
        ;;
    stage)
        stage="${2:?stage}"
        case "$stage" in
            validate-topology) command=(make --no-print-directory clippy rustdoc test PACKAGE=ic-query-topology) ;;
            validate-sns) command=(make --no-print-directory clippy rustdoc test PACKAGE=ic-query-sns) ;;
            validate-cli) command=(make --no-print-directory clippy rustdoc test PACKAGE=ic-query-cli) ;;
            qualify-umbrella) command=(make --no-print-directory qualify-umbrella-current) ;;
            *) exit 2 ;;
        esac
        exec "${command[@]}"
        ;;
    products)
        attempt="${QUERY_VALIDATION_ATTEMPT:?}"
        candidate="$(jq -er .directory "$attempt/projection.json")"
        identity="$(jq -er .identity "$attempt/projection.json")"
        jq -e --arg source "${RELEASE_SOURCE:?}" '.source == $source' "$attempt/projection.json" >/dev/null
        targets=()
        selected="$(jq -er '.stages[] | select(.required and .target != "validate-common") | .target' "$attempt/obligations.json")"
        while IFS= read -r target; do
            case "$target" in validate-topology|validate-sns|validate-cli|qualify-umbrella) ;; *) exit 1 ;; esac
            targets+=("$target")
        done <<< "$selected"
        if [[ -e "$attempt/products.seal.json" || -L "$attempt/products.seal.json" ]]; then
            release_timing_begin query-product-seal-check
            release_receipt_check "$attempt/products.seal.json" perl "$shared/seal-validation-stage.pl" \
                "$attempt/products.admission.json" "$RELEASE_SOURCE" "$RELEASE_GROUP" "$identity"
        else
            (
                cd "$candidate"
                perl "$shared/release-timing.pl" run query-product-admission executed \
                    perl "$shared/run-validation-stage.pl" --state "$root/target/validation/stage-proofs" \
                    --stage query-products --contract query-final-candidate/2 --source "$RELEASE_SOURCE" --group "${RELEASE_GROUP:?}" \
                    --observe "$shared/observe-cargo-stage.pl" --environment "$attempt/reuse-environment.json" \
                    --max-bytes 536870912 --admission-output "$attempt/products.admission.json" \
                    -- make --no-print-directory "${targets[@]}"
            )
            release_timing_begin query-product-seal-write
            release_receipt_write "$attempt/products.seal.json" perl "$shared/seal-validation-stage.pl" \
                "$attempt/products.admission.json" "$RELEASE_SOURCE" "$RELEASE_GROUP" "$identity"
        fi
        release_timing_end 0
        stage_path="$(jq -er .PATH "$attempt/reuse-environment.json")"
        stage_make="$(env -i PATH="$stage_path" /bin/sh -c 'command -v make')"
        expected_command="$(printf '%s\n' "$stage_make" --no-print-directory "${targets[@]}" | jq -Rsc 'split("\n")[:-1]')"
        jq -e --argjson command "$expected_command" --arg cwd "$candidate" \
            '.command == $command and .cwd == $cwd and .admission.stage == "query-products"' "$attempt/products.seal.json" >/dev/null
        printf 'Selected product obligations covered by admitted checks (%s).\n' "$(jq -r .admission.mode "$attempt/products.seal.json")"
        ;;
    *) exit 2 ;;
esac
