.DEFAULT_GOAL := help
REPO_ROOT := $(dir $(abspath $(lastword $(MAKEFILE_LIST))))
TOKO_TOOLING_ROOT := $(REPO_ROOT)vendor/toko-tooling
include $(TOKO_TOOLING_ROOT)/make/consumer.mk
include $(TOKO_TOOLING_ROOT)/make/cargo-dependencies.mk

PACKAGE ?=
CARGO_SELECTION = $(if $(strip $(PACKAGE)),-p "$(PACKAGE)",--workspace)
export RUSTUP_AUTO_INSTALL := 0

.PHONY: help icq check doc-links-check fetch-dependencies clippy rustdoc test qualify-cli
help:
	@printf '%s\n' \
	  'icq                 Install this checkout as icq on PATH (locked, offline)' \
	  'check               Aggregate offline checks (all packages by default)' \
	  'check PACKAGE=name  Select one package for Rust checks' \
	  'qualify-cli         Test icq help, version and error process contracts' \
	  'fetch-dependencies  Explicit locked Cargo cache preparation (network)' \
	  'deps-list           List local workspace package versions' \
	  'deps-update         Explicit Cargo dependency update (network)' \
	  'toko-cloc -d 0      Run the installed LOC command in your shell' \
	  'doc-links-check     Check local documentation file links' \
	  'toko-tooling-check  Admit Cargo pin, members, lockfile and snapshot' \
	  'install-tools       Prepare shared host and Rust developer tools'

check: toko-tooling-check doc-links-check fmt-check clippy rustdoc test
ci: check

icq: toko-tooling-check | toko-execution-check
	cargo install --path crates/ic-query-cli --bin icq --locked --offline --debug --force
	@command -v icq >/dev/null || { printf '%s\n' 'Add your Cargo installation bin directory to PATH, then run icq --help.' >&2; exit 1; }
	@icq --version
	@printf '%s\n' 'Run: icq --help'

fetch-dependencies: | toko-execution-check
	cargo fetch --locked

clippy: toko-tooling-check
	cargo clippy $(CARGO_SELECTION) --all-targets --locked --offline -- -D warnings

rustdoc: toko-tooling-check
	RUSTDOCFLAGS="-D warnings" cargo doc $(CARGO_SELECTION) --no-deps --locked --offline

test: toko-tooling-check
	RUSTFLAGS="-D warnings" cargo test $(CARGO_SELECTION) --locked --offline

qualify-cli: toko-tooling-check
	RUSTFLAGS="-D warnings" cargo test -p ic-query-cli --test cli --locked --offline

doc-links-check: toko-tooling-check
	@perl "$(TOKO_TOOLING_ROOT)/scripts/ci/check-documentation-links.pl" \
	  --root "$(REPO_ROOT)" README.md AGENTS.md CHANGELOG.md docs releases
