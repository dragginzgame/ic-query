#!/usr/bin/env bash
set -euo pipefail
[[ $# -le 1 ]] || { echo "usage: $0 [VERSION]" >&2; exit 2; }
requested="${1:-}"
if [[ -n "$requested" ]]; then
  [[ "$requested" =~ ^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$ ]] || exit 2
  [[ "${RELEASE_GROUP:-}" == query || "${RELEASE_GROUP:-}" == cli ]] || {
    echo 'select RELEASE_GROUP with an explicit changelog version' >&2; exit 2;
  }
  groups=("$RELEASE_GROUP")
else
  groups=(query cli)
fi
for group in "${groups[@]}"; do
  current="$(CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0 bash scripts/ci/cargo-release-group.sh version "$group")"
  version="${requested:-$current}"
  ledger="$(jq -er --arg group "$group" '.groups[] | select(.name == $group) | .changelog' ci/cargo-release-groups.json)"
  escaped="${version//./\\.}"
  if grep -Eq "^## \\[$escaped\\]( - [0-9]{4}-[0-9]{2}-[0-9]{2})?$" "$ledger"; then
    detail="${ledger%/*}/${version%.*}.md"
    grep -Eq "^## $escaped( - [0-9]{4}-[0-9]{2}-[0-9]{2})?$" "$detail" || {
      echo "missing $group detailed changelog for $version" >&2; exit 1;
    }
  else
    # The first grouped release inherits the common finalized history once.
    [[ -z "$requested" ]] && grep -Fq "Baseline version: \`$version\`." "$ledger" \
      && grep -Fq "## [$version] - " releases/history.md || {
      echo "missing $group changelog for $version" >&2; exit 1;
    }
  fi
done
