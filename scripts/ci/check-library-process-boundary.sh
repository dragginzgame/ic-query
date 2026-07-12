#!/usr/bin/env bash
set -euo pipefail

pattern='(eprintln!|println!|print!|dbg!|std::io::stderr|std::io::stdout|io::stderr|io::stdout|std::env::args|std::env::args_os|std::env::current_dir|std::process::exit)'

if ! command -v rg >/dev/null 2>&1; then
  echo "error: ripgrep is required; run 'make install-dev'" >&2
  exit 1
fi

set +e
matches="$(rg -n "${pattern}" crates/ic-query/src --glob '*.rs')"
status=$?
set -e

case "${status}" in
  0)
    echo "error: ic-query library code must not own process arguments or process output" >&2
    echo "${matches}" >&2
    exit 1
    ;;
  1)
    exit 0
    ;;
  *)
    echo "error: failed to scan the ic-query process boundary" >&2
    exit "${status}"
    ;;
esac
