#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
make_bin="$(command -v make)"
work_dir="$(mktemp -d)"
trap 'rm -rf "${work_dir}"' EXIT

fail() {
  echo "error: $*" >&2
  exit 1
}

changelog_case="${work_dir}/changelog"
mkdir -p "${changelog_case}/bin" "${changelog_case}/docs/changelog"
printf 'version = "0.8.0"\n' > "${changelog_case}/Cargo.toml"
printf -- "- \`0.8.1\` release\n" > "${changelog_case}/CHANGELOG.md"
printf '## 0.8.1\n' > "${changelog_case}/docs/changelog/0.8.md"
printf 'ic-query = { version = "0.8" }\n' > "${changelog_case}/README.md"
printf 'ic-query = { version = "0.8" }\n' > "${changelog_case}/docs/library-usage.md"
cat > "${changelog_case}/bin/git" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
if [[ "${1:-}" != "show" ]]; then
  exit 2
fi
case "${2:-}" in
  HEAD:CHANGELOG.md)
    cat CHANGELOG.md
    ;;
  HEAD:docs/changelog/0.8.md)
    cat docs/changelog/0.8.md
    ;;
  *)
    exit 1
    ;;
esac
EOF
chmod +x "${changelog_case}/bin/git"
(
  cd "${changelog_case}"
  PATH="${changelog_case}/bin:${PATH}" \
    bash "${repo_root}/scripts/ci/check-changelog-version.sh" 0.8.1
) || fail "the changelog check rejected an explicit target version"

printf '## 0.8.1 - Unreleased\n' > "${changelog_case}/docs/changelog/0.8.md"
set +e
(
  cd "${changelog_case}"
  PATH="${changelog_case}/bin:${PATH}" \
    bash "${repo_root}/scripts/ci/check-changelog-version.sh" 0.8.1
) >/dev/null 2>&1
unreleased_status="$?"
set -e
[[ "${unreleased_status}" -ne 0 ]] \
  || fail "the changelog check accepted an Unreleased target version"
printf '## 0.8.1\n' > "${changelog_case}/docs/changelog/0.8.md"

printf 'ic-query = { version = "0.7" }\n' > "${changelog_case}/README.md"
set +e
(
  cd "${changelog_case}"
  PATH="${changelog_case}/bin:${PATH}" \
    bash "${repo_root}/scripts/ci/check-changelog-version.sh" 0.8.1
) >/dev/null 2>&1
stale_usage_status="$?"
set -e
[[ "${stale_usage_status}" -ne 0 ]] \
  || fail "the changelog check accepted a stale dependency example"
printf 'ic-query = { version = "0.8" }\n' > "${changelog_case}/README.md"

cat > "${changelog_case}/bin/bash" <<'EOF'
#!/bin/bash
printf '%s\n' "$*" > "${TRACE_FILE}"
EOF
chmod +x "${changelog_case}/bin/bash"
(
  cd "${changelog_case}"
  PATH="${changelog_case}/bin:${PATH}" TRACE_FILE="${changelog_case}/make-trace" \
    "${make_bin}" --no-print-directory -f "${repo_root}/Makefile" \
      CHANGELOG_VERSION=0.8.1 changelog-check
) || fail "the Make changelog target rejected an explicit target version"
[[ "$(<"${changelog_case}/make-trace")" \
    == "scripts/ci/check-changelog-version.sh 0.8.1" ]] \
  || fail "the Make changelog target did not forward the explicit target version"

bump_case="${work_dir}/bump"
mkdir -p "${bump_case}/bin"
printf 'version = "0.8.0"\n' > "${bump_case}/Cargo.toml"
cat > "${bump_case}/bin/bash" <<'EOF'
#!/bin/bash
printf 'changelog %s\n' "$*" >> "${TRACE_FILE}"
exit "${CHANGELOG_STATUS:-0}"
EOF
cat > "${bump_case}/bin/make" <<'EOF'
#!/bin/bash
printf 'make %s\n' "$*" >> "${TRACE_FILE}"
case "${*: -1}" in
  ensure-clean) exit "${CLEAN_STATUS:-0}" ;;
  ci)
    [[ "${CHANGELOG_VERSION:-}" == "0.8.1" ]] || exit 42
    exit 23
    ;;
  *) exit 2 ;;
esac
EOF
chmod +x "${bump_case}/bin/bash" "${bump_case}/bin/make"
before_bump="$(<"${bump_case}/Cargo.toml")"
set +e
(
  cd "${bump_case}"
  PATH="${bump_case}/bin:${PATH}" TRACE_FILE="${bump_case}/trace" CHANGELOG_STATUS=29 \
    /bin/bash "${repo_root}/scripts/release/bump-version.sh" patch
) >/dev/null 2>&1
changelog_status="$?"
set -e
[[ "${changelog_status}" -eq 29 ]] \
  || fail "the bump script did not propagate a missing target changelog"
[[ "$(<"${bump_case}/Cargo.toml")" == "${before_bump}" ]] \
  || fail "the bump script edited version metadata after a failed changelog gate"
mapfile -t missing_changelog_trace < "${bump_case}/trace"
[[ "${missing_changelog_trace[0]:-}" == "changelog scripts/ci/check-changelog-version.sh 0.8.1" ]] \
  || fail "the bump script did not check the target-version changelog"
[[ "${#missing_changelog_trace[@]}" -eq 1 ]] \
  || fail "the bump script ran CI after a failed target changelog check"

: > "${bump_case}/trace"
set +e
(
  cd "${bump_case}"
  PATH="${bump_case}/bin:${PATH}" TRACE_FILE="${bump_case}/trace" \
    /bin/bash "${repo_root}/scripts/release/bump-version.sh" patch
) >/dev/null 2>&1
bump_status="$?"
set -e
[[ "${bump_status}" -eq 23 ]] || fail "the bump script did not propagate a failing CI gate"
[[ "$(<"${bump_case}/Cargo.toml")" == "${before_bump}" ]] \
  || fail "the bump script edited version metadata before CI passed"
mapfile -t bump_trace < "${bump_case}/trace"
[[ "${bump_trace[0]:-}" == "changelog scripts/ci/check-changelog-version.sh 0.8.1" ]] \
  || fail "the bump script did not check the target-version changelog first"
[[ "${bump_trace[1]:-}" == "make --no-print-directory ensure-clean" ]] \
  || fail "the bump script did not check cleanliness before CI"
[[ "${bump_trace[2]:-}" == "make --no-print-directory ci" ]] \
  || fail "the bump script did not run the complete CI gate after the cleanliness check"

: > "${bump_case}/trace"
if (
  cd "${bump_case}"
  PATH="${bump_case}/bin:${PATH}" TRACE_FILE="${bump_case}/trace" CLEAN_STATUS=31 \
    /bin/bash "${repo_root}/scripts/release/bump-version.sh" patch
) >/dev/null 2>&1; then
  dirty_bump_status=0
else
  dirty_bump_status="$?"
fi
[[ "${dirty_bump_status}" -eq 31 ]] \
  || fail "the bump script did not preserve a failed cleanliness check"
[[ "$(<"${bump_case}/Cargo.toml")" == "${before_bump}" ]] \
  || fail "the bump script edited version metadata after a failed cleanliness check"
mapfile -t dirty_bump_trace < "${bump_case}/trace"
[[ "${dirty_bump_trace[0]:-}" == "changelog scripts/ci/check-changelog-version.sh 0.8.1" ]] \
  || fail "the bump script did not check the target-version changelog before cleanliness"
[[ "${dirty_bump_trace[1]:-}" == "make --no-print-directory ensure-clean" ]] \
  || fail "the bump script did not run the cleanliness check"
[[ "${#dirty_bump_trace[@]}" -eq 2 ]] \
  || fail "the bump script ran CI after a failed cleanliness check"

clean_case="${work_dir}/clean"
mkdir -p "${clean_case}/bin"
cat > "${clean_case}/bin/git" <<'EOF'
#!/usr/bin/env bash
case "${1:-}" in
  diff-index)
    exit 0
    ;;
  ls-files)
    printf 'untracked-release-note.md\n'
    exit 0
    ;;
  *)
    exit 2
    ;;
esac
EOF
chmod +x "${clean_case}/bin/git"
set +e
(
  cd "${clean_case}"
  PATH="${clean_case}/bin:${PATH}" \
    make --no-print-directory -f "${repo_root}/Makefile" ensure-clean
) >/dev/null 2>&1
clean_status="$?"
set -e
[[ "${clean_status}" -ne 0 ]] || fail "ensure-clean accepted an untracked file"

commit_case="${work_dir}/commit"
mkdir -p "${commit_case}/bin"
printf 'version = "0.8.1"\n' > "${commit_case}/Cargo.toml"
cat > "${commit_case}/bin/git" <<'EOF'
#!/usr/bin/env bash
case "${1:-}" in
  rev-parse)
    exit 1
    ;;
  commit)
    exit 37
    ;;
  tag)
    : > "${TAG_MARKER}"
    exit 0
    ;;
  *)
    exit 2
    ;;
esac
EOF
chmod +x "${commit_case}/bin/git"
set +e
(
  cd "${commit_case}"
  PATH="${commit_case}/bin:${PATH}" TAG_MARKER="${commit_case}/tagged" \
    make --no-print-directory -f "${repo_root}/Makefile" release-commit
) >/dev/null 2>&1
commit_status="$?"
set -e
[[ "${commit_status}" -ne 0 ]] || fail "release-commit hid a failed commit"
[[ ! -e "${commit_case}/tagged" ]] || fail "release-commit tagged after a failed commit"

push_case="${work_dir}/push"
mkdir -p "${push_case}/bin"
printf 'version = "0.8.1"\n' > "${push_case}/Cargo.toml"
cat > "${push_case}/bin/git" <<'EOF'
#!/usr/bin/env bash
case "${1:-}" in
  diff-index)
    exit 0
    ;;
  ls-files)
    exit 0
    ;;
  rev-parse)
    case "${2:-}" in
      'v0.8.1^{}') printf '%s\n' "${TAG_COMMIT}" ;;
      HEAD) printf '%s\n' "${HEAD_COMMIT}" ;;
      *) exit 2 ;;
    esac
    ;;
  push)
    : > "${PUSH_MARKER}"
    exit "${PUSH_STATUS:-0}"
    ;;
  *)
    exit 2
    ;;
esac
EOF
chmod +x "${push_case}/bin/git"
set +e
(
  cd "${push_case}"
  PATH="${push_case}/bin:${PATH}" PUSH_MARKER="${push_case}/pushed" \
    PUSH_STATUS=41 TAG_COMMIT=release HEAD_COMMIT=release \
    "${make_bin}" --no-print-directory -f "${repo_root}/Makefile" release-push
) >/dev/null 2>&1
push_status="$?"
set -e
[[ "${push_status}" -ne 0 ]] || fail "release-push hid a failed push"
[[ -e "${push_case}/pushed" ]] || fail "release-push did not push a validated release"

rm -f "${push_case}/pushed"
set +e
(
  cd "${push_case}"
  PATH="${push_case}/bin:${PATH}" PUSH_MARKER="${push_case}/pushed" \
    TAG_COMMIT=stale HEAD_COMMIT=current \
    "${make_bin}" --no-print-directory -f "${repo_root}/Makefile" release-push
) >/dev/null 2>&1
stale_tag_status="$?"
set -e
[[ "${stale_tag_status}" -ne 0 ]] || fail "release-push accepted a stale release tag"
[[ ! -e "${push_case}/pushed" ]] || fail "release-push pushed a stale release tag"

release_push_block="$(awk '
  $0 ~ /^release-push:/ { found = 1; next }
  found && /^[^[:space:]].*:/ { exit }
  found { print }
' "${repo_root}/Makefile")"
[[ "${release_push_block}" == $'\tgit push --follow-tags' ]] \
  || fail "release-push must not rerun CI after the version bump"

for release_kind in patch minor major; do
  release_block="$(awk -v target="release-${release_kind}:" '
    $0 == target { found = 1; next }
    found && /^[^[:space:]].*:/ { exit }
    found { print }
  ' "${repo_root}/Makefile")"
  expected_block="$(printf "\t+\$(MAKE) --no-print-directory %s\n\t+\$(MAKE) --no-print-directory release-stage\n\t+\$(MAKE) --no-print-directory release-commit\n\t+\$(MAKE) --no-print-directory release-push" "${release_kind}")"
  [[ "${release_block}" == "${expected_block}" ]] \
    || fail "release-${release_kind} is not a sequential fail-closed recipe"
done
