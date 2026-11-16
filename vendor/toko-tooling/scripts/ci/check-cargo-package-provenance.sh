#!/usr/bin/env bash
# Shared companions:
set -euo pipefail

# Check prepared archives and retrieve their exact commit into a fresh store.
# No Cargo execution, publication, setup, retries or recovery decisions.
fail() { printf 'package provenance refused: %s\n' "$*" >&2; exit 2; }
[[ $# -ge 7 ]] || fail 'usage: check-cargo-package-provenance.sh EVIDENCE ARCHIVES REPOSITORY SOURCE --package NAME@X.Y.Z PATH [...]'
evidence="$1" archives="$2" repository="$3" source="$4"
shift 4
path_ok() { case "$1" in *$'\r'*|*$'\n'*) fail 'paths must not contain CR/LF' ;; esac; }
path_ok "$PWD"
for path in "$evidence" "$archives" "$repository"; do path_ok "$path"; done
[[ -n "$repository" && "$repository" != -* ]] || fail 'select an explicit source repository'
[[ "$source" =~ ^([0-9a-f]{40}|[0-9a-f]{64})$ ]] || fail 'invalid source commit'
[[ "$evidence" == /* ]] || evidence="$PWD/$evidence"
parent="${evidence%/*}" name="${evidence##*/}"
[[ -n "$name" && "$name" != . && "$name" != .. ]] || fail 'select a fresh evidence directory'
parent="$(CDPATH='' cd -P -- "$parent" && printf '%s/.' "$PWD")"
parent="${parent%/.}"
path_ok "$parent"
evidence="$parent/$name"
[[ ! -e "$evidence" && ! -L "$evidence" ]] || fail 'evidence already exists; retain and reconcile it'
archives="$(CDPATH='' cd -P -- "$archives" && printf '%s/.' "$PWD")"
archives="${archives%/.}"
path_ok "$archives"
packages=() paths=()
seen='|'
while [[ $# -gt 0 ]]; do
    [[ $# -ge 3 && "$1" == --package ]] || fail 'expected --package NAME@X.Y.Z PATH'
    [[ "$2" =~ ^[A-Za-z][A-Za-z0-9_-]*@(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$ ]] || fail 'select an exact package version'
    package="${2%@*}" version="${2##*@}" path="$3"
    [[ "$seen" != *"|$package|"* ]] || fail 'duplicate package selection'
    seen="$seen$package|"
    path_ok "$path"
    if [[ "$path" == . ]]; then path=''
    else
        case "/$path/" in *'//'*|*'/./'*|*'/../'*) fail 'select a normal repository-relative package path or .' ;; esac
    fi
    archive="$archives/$package-$version.crate"
    [[ -f "$archive" && ! -L "$archive" ]] || fail "missing regular package archive for $package"
    packages+=("$2") paths+=("$path")
    shift 3
done
umask 077
mkdir -- "$evidence"
printf '%s\n' "$repository" "$source" > "$evidence/source-request"
printf '%s\0' "${packages[@]}" > "$evidence/packages"
printf '%s\0' "${paths[@]}" > "$evidence/paths"
for index in "${!packages[@]}"; do
    package="${packages[$index]%@*}" version="${packages[$index]##*@}"
    tar -xOf "$archives/$package-$version.crate" "$package-$version/.cargo_vcs_info.json" \
        > "$evidence/$package-vcs.json" 2> "$evidence/$package-vcs.log" || fail "missing package provenance for $package"
    # Exactly one JSON value; duplicate archive members must not hide bad evidence.
    jq -se --arg source "$source" --arg path "${paths[$index]}" '
        length == 1 and (.[0] | .git.sha1 == $source
            and (.git.dirty // false) == false and .path_in_vcs == $path)
    ' "$evidence/$package-vcs.json" > /dev/null || fail "package source identity differs for $package"
done
# Do not let inherited Git object locations turn local cache presence into proof
# of retrieval. Caller credentials/configuration and offline policy stay intact.
unset GIT_DIR GIT_WORK_TREE GIT_COMMON_DIR GIT_INDEX_FILE GIT_OBJECT_DIRECTORY GIT_ALTERNATE_OBJECT_DIRECTORIES
object_format=sha1
[[ ${#source} == 40 ]] || object_format=sha256
git init --bare --object-format="$object_format" "$evidence/source.git" > "$evidence/source-init.log" 2>&1 || fail 'cannot prepare source evidence'
GIT_TERMINAL_PROMPT=0 git -C "$evidence/source.git" fetch --no-tags --depth=1 -- \
    "$repository" "$source" > "$evidence/source-fetch.log" 2>&1 || fail 'packaged source is not retrievable from the declared repository'
retrieved="$(git -C "$evidence/source.git" rev-parse --verify 'FETCH_HEAD^{commit}')" || fail 'retrieved source is not a commit'
[[ "$retrieved" == "$source" ]] || fail 'retrieved source differs from package provenance'
printf '%s\n' "$retrieved" > "$evidence/source-verified"
