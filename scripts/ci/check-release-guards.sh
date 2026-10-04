#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
make_bin="$(command -v make)"
work_dir="$(mktemp -d "${TMPDIR:-/tmp}/ic-query-release-guards.XXXXXX")"
trap 'rm -rf -- "${work_dir}"' EXIT

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
(
  cd "${changelog_case}"
  PATH="${changelog_case}/bin:${PATH}" \
    bash "${repo_root}/scripts/ci/check-changelog-version.sh" 0.8.1
) || fail "the changelog check rejected an Unreleased target-version heading"
printf '## 0.8.1\n' > "${changelog_case}/docs/changelog/0.8.md"

printf 'ic-query = { version = "0.7" }\n' > "${changelog_case}/README.md"
(
  cd "${changelog_case}"
  PATH="${changelog_case}/bin:${PATH}" \
    bash "${repo_root}/scripts/ci/check-changelog-version.sh" 0.8.1
) || fail "the changelog check still requires pre-bumped dependency examples"
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
mkdir -p "${bump_case}/bin" "${bump_case}/docs"
printf 'version = "0.8.0"\n' > "${bump_case}/Cargo.toml"
printf 'ic-query = { version = "0.8", default-features = false }\n' \
  > "${bump_case}/README.md"
printf 'ic-query = { version = "0.8", features = ["host"] }\n' \
  > "${bump_case}/docs/library-usage.md"
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
    [[ "${CHANGELOG_VERSION:-}" == "${EXPECTED_CHANGELOG_VERSION:-0.8.1}" ]] || exit 42
    bash scripts/ci/check-changelog-version.sh "${CHANGELOG_VERSION}" || exit "$?"
    exit "${CI_STATUS:-23}"
    ;;
  *) exit 2 ;;
esac
EOF
cat > "${bump_case}/bin/cargo" <<'EOF'
#!/bin/bash
version="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)"
printf 'cargo %s version=%s\n' "$*" "${version}" >> "${TRACE_FILE}"
exit 2
EOF
chmod +x "${bump_case}/bin/bash" "${bump_case}/bin/make" "${bump_case}/bin/cargo"
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
[[ "${missing_changelog_trace[0]:-}" == "make --no-print-directory ensure-clean" \
  && "${missing_changelog_trace[1]:-}" == "make --no-print-directory ci" \
  && "${missing_changelog_trace[2]:-}" == "changelog scripts/ci/check-changelog-version.sh 0.8.1" ]] \
  || fail "the bump script did not check the target-version changelog"
[[ "${#missing_changelog_trace[@]}" -eq 3 ]] \
  || fail "the bump script continued after a failed target changelog check"

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
[[ "${bump_trace[0]:-}" == "make --no-print-directory ensure-clean" ]] \
  || fail "the bump script did not check cleanliness before CI"
[[ "${bump_trace[1]:-}" == "make --no-print-directory ci" ]] \
  || fail "the bump script did not run the complete CI gate after the cleanliness check"
[[ "${bump_trace[2]:-}" == "changelog scripts/ci/check-changelog-version.sh 0.8.1" ]] \
  || fail "the CI gate did not check the target-version changelog"
[[ "${#bump_trace[@]}" -eq 3 ]] \
  || fail "the bump script ran unexpected commands after a failed CI gate"

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
[[ "${dirty_bump_trace[0]:-}" == "make --no-print-directory ensure-clean" ]] \
  || fail "the bump script did not run the cleanliness check"
[[ "${#dirty_bump_trace[@]}" -eq 1 ]] \
  || fail "the bump script ran CI after a failed cleanliness check"

: > "${bump_case}/trace"
(
  cd "${bump_case}"
  PATH="${bump_case}/bin:${PATH}" TRACE_FILE="${bump_case}/trace" CI_STATUS=0 \
    /bin/bash "${repo_root}/scripts/release/bump-version.sh" patch
) >/dev/null 2>&1 \
  || fail "the bump script rejected a successful CI gate"
[[ "$(<"${bump_case}/Cargo.toml")" == 'version = "0.8.1"' ]] \
  || fail "the bump script did not update version metadata after CI passed"
mapfile -t successful_bump_trace < "${bump_case}/trace"
[[ "${successful_bump_trace[0]:-}" == "make --no-print-directory ensure-clean" ]] \
  || fail "the successful bump did not check cleanliness before CI"
[[ "${successful_bump_trace[1]:-}" == "make --no-print-directory ci" ]] \
  || fail "the successful bump did not run the complete CI gate"
[[ "${successful_bump_trace[2]:-}" == "changelog scripts/ci/check-changelog-version.sh 0.8.1" ]] \
  || fail "the successful bump did not check the target-version changelog through CI"
[[ "${#successful_bump_trace[@]}" -eq 3 ]] \
  || fail "the successful bump ran unexpected commands"

: > "${bump_case}/trace"
(
  cd "${bump_case}"
  PATH="${bump_case}/bin:${PATH}" TRACE_FILE="${bump_case}/trace" CI_STATUS=0 \
    EXPECTED_CHANGELOG_VERSION=0.9.0 \
    /bin/bash "${repo_root}/scripts/release/bump-version.sh" minor
) >/dev/null 2>&1 \
  || fail "the bump script failed to automate dependency example versions"
[[ "$(<"${bump_case}/Cargo.toml")" == 'version = "0.9.0"' ]] \
  || fail "the minor bump did not update version metadata"
grep -Fq 'ic-query = { version = "0.9", default-features = false }' \
  "${bump_case}/README.md" \
  || fail "the minor bump did not update the README dependency example"
grep -Fq 'ic-query = { version = "0.9", features = ["host"] }' \
  "${bump_case}/docs/library-usage.md" \
  || fail "the minor bump did not update the library dependency example"

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

stage_case="${work_dir}/stage"
mkdir -p "${stage_case}/bin"
cat > "${stage_case}/bin/git" <<'EOF'
#!/usr/bin/env bash
printf 'git %s\n' "$*" > "${TRACE_FILE}"
EOF
chmod +x "${stage_case}/bin/git"
(
  cd "${stage_case}"
  PATH="${stage_case}/bin:${PATH}" TRACE_FILE="${stage_case}/trace" \
    "${make_bin}" --no-print-directory -f "${repo_root}/Makefile" release-stage
) >/dev/null
expected_stage='git add Cargo.toml Cargo.lock README.md docs/library-usage.md'
[[ "$(<"${stage_case}/trace")" == "${expected_stage}" ]] \
  || fail "release-stage omitted a file generated by the version bump"

commit_case="${work_dir}/commit"
mkdir -p "${commit_case}/bin"
printf 'version = "0.8.1"\n' > "${commit_case}/Cargo.toml"
cat > "${commit_case}/bin/git" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail

case "${1:-}" in
  rev-parse)
    exit 1
    ;;
  diff)
    case "$*" in
      'diff --quiet --')
        exit "${UNSTAGED_STATUS:-0}"
        ;;
      'diff --cached --quiet --')
        exit 1
        ;;
      'diff --cached --name-only --diff-filter=ACDMRTUXB --')
        printf '%s\n' "${STAGED_PATH:-Cargo.toml}"
        ;;
      *)
        exit 2
        ;;
    esac
    ;;
  diff-index)
    exit "${POST_COMMIT_DIRTY_STATUS:-0}"
    ;;
  ls-files)
    [[ -z "${UNTRACKED_PATH:-}" ]] || printf '%s\n' "${UNTRACKED_PATH}"
    ;;
  commit)
    : > "${COMMIT_MARKER}"
    exit "${COMMIT_STATUS:-0}"
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

if (
  cd "${commit_case}"
  PATH="${commit_case}/bin:${PATH}" COMMIT_MARKER="${commit_case}/committed" \
    TAG_MARKER="${commit_case}/tagged" COMMIT_STATUS=37 \
    "${make_bin}" --no-print-directory -f "${repo_root}/Makefile" release-commit
) >/dev/null 2>&1; then
  commit_status=0
else
  commit_status="$?"
fi
[[ "${commit_status}" -ne 0 ]] || fail "release-commit hid a failed commit"
[[ -e "${commit_case}/committed" ]] || fail "release-commit did not reach the failing commit"
[[ ! -e "${commit_case}/tagged" ]] || fail "release-commit tagged after a failed commit"

rm -f "${commit_case}/committed" "${commit_case}/tagged"
if (
  cd "${commit_case}"
  PATH="${commit_case}/bin:${PATH}" COMMIT_MARKER="${commit_case}/committed" \
    TAG_MARKER="${commit_case}/tagged" UNSTAGED_STATUS=1 \
    "${make_bin}" --no-print-directory -f "${repo_root}/Makefile" release-commit
) >/dev/null 2>&1; then
  fail "release-commit accepted unstaged changes"
fi
[[ ! -e "${commit_case}/committed" ]] \
  || fail "release-commit committed while unstaged changes remained"

if (
  cd "${commit_case}"
  PATH="${commit_case}/bin:${PATH}" COMMIT_MARKER="${commit_case}/committed" \
    TAG_MARKER="${commit_case}/tagged" UNTRACKED_PATH=release-notes.tmp \
    "${make_bin}" --no-print-directory -f "${repo_root}/Makefile" release-commit
) >/dev/null 2>&1; then
  fail "release-commit accepted an untracked file"
fi
[[ ! -e "${commit_case}/committed" ]] \
  || fail "release-commit committed while an untracked file remained"

if (
  cd "${commit_case}"
  PATH="${commit_case}/bin:${PATH}" COMMIT_MARKER="${commit_case}/committed" \
    TAG_MARKER="${commit_case}/tagged" STAGED_PATH=unrelated.txt \
    "${make_bin}" --no-print-directory -f "${repo_root}/Makefile" release-commit
) >/dev/null 2>&1; then
  fail "release-commit accepted an unexpected staged path"
fi
[[ ! -e "${commit_case}/committed" ]] \
  || fail "release-commit committed an unexpected staged path"

if (
  cd "${commit_case}"
  PATH="${commit_case}/bin:${PATH}" COMMIT_MARKER="${commit_case}/committed" \
    TAG_MARKER="${commit_case}/tagged" POST_COMMIT_DIRTY_STATUS=1 \
    "${make_bin}" --no-print-directory -f "${repo_root}/Makefile" release-commit
) >/dev/null 2>&1; then
  fail "release-commit accepted post-commit working-tree changes"
fi
[[ -e "${commit_case}/committed" ]] \
  || fail "the post-commit cleanliness guard ran before the commit"
[[ ! -e "${commit_case}/tagged" ]] \
  || fail "release-commit tagged a commit that left working-tree changes"

rm -f "${commit_case}/committed" "${commit_case}/tagged"
(
  cd "${commit_case}"
  PATH="${commit_case}/bin:${PATH}" COMMIT_MARKER="${commit_case}/committed" \
    TAG_MARKER="${commit_case}/tagged" \
    STAGED_PATH=$'Cargo.toml\nCargo.lock\nREADME.md\ndocs/library-usage.md' \
    "${make_bin}" --no-print-directory -f "${repo_root}/Makefile" release-commit
) >/dev/null \
  || fail "release-commit rejected a complete clean staged release"
[[ -e "${commit_case}/committed" && -e "${commit_case}/tagged" ]] \
  || fail "release-commit did not commit and tag the complete release"

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
cat > "${push_case}/bin/make" <<'EOF'
#!/usr/bin/env bash
exit 83
EOF
chmod +x "${push_case}/bin/make"
set +e
(
  cd "${push_case}"
  PATH="${push_case}/bin:${PATH}" PUSH_MARKER="${push_case}/pushed" \
    PUSH_STATUS=41 TAG_COMMIT=release HEAD_COMMIT=release \
    "${make_bin}" --no-print-directory -f "${repo_root}/Makefile" \
      MAKE="${push_case}/bin/make" release-push
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
    "${make_bin}" --no-print-directory -f "${repo_root}/Makefile" \
      MAKE="${push_case}/bin/make" release-push
) >/dev/null 2>&1
stale_tag_status="$?"
set -e
[[ "${stale_tag_status}" -ne 0 ]] || fail "release-push accepted a stale release tag"
[[ ! -e "${push_case}/pushed" ]] || fail "release-push pushed a stale release tag"

(
  cd "${push_case}"
  PATH="${push_case}/bin:${PATH}" PUSH_MARKER="${push_case}/pushed" \
    TAG_COMMIT=release HEAD_COMMIT=release \
    "${make_bin}" --no-print-directory -f "${repo_root}/Makefile" \
      MAKE="${push_case}/bin/make" release-push
) >/dev/null \
  || fail "release-push rejected a validated release or ran an extra gate"
[[ -e "${push_case}/pushed" ]] \
  || fail "release-push did not push the validated release"

sequence_case="${work_dir}/sequence"
mkdir -p "${sequence_case}/bin"
cat > "${sequence_case}/bin/make" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
step="${*: -1}"
printf '%s\n' "${step}" >> "${TRACE_FILE}"
[[ "${step}" != "${FAIL_STEP:-}" ]] || exit 43
EOF
chmod +x "${sequence_case}/bin/make"
for release_kind in patch minor major; do
  expected_steps=("${release_kind}" release-stage release-commit release-push)
  : > "${sequence_case}/trace"
  TRACE_FILE="${sequence_case}/trace" \
    "${make_bin}" --no-print-directory -j4 -f "${repo_root}/Makefile" \
      MAKE="${sequence_case}/bin/make" "release-${release_kind}" >/dev/null
  mapfile -t actual_steps < "${sequence_case}/trace"
  [[ "${actual_steps[*]}" == "${expected_steps[*]}" ]] \
    || fail "release-${release_kind} did not execute its steps in order"

  for failed_index in "${!expected_steps[@]}"; do
    : > "${sequence_case}/trace"
    if TRACE_FILE="${sequence_case}/trace" FAIL_STEP="${expected_steps[failed_index]}" \
      "${make_bin}" --no-print-directory -j4 -f "${repo_root}/Makefile" \
        MAKE="${sequence_case}/bin/make" "release-${release_kind}" >/dev/null 2>&1; then
      fail "release-${release_kind} hid a failure in ${expected_steps[failed_index]}"
    fi
    mapfile -t actual_steps < "${sequence_case}/trace"
    [[ "${actual_steps[*]}" == "${expected_steps[*]:0:failed_index+1}" ]] \
      || fail "release-${release_kind} continued after ${expected_steps[failed_index]} failed"
  done
done
