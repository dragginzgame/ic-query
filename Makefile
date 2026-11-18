.DEFAULT_GOAL := help
REPO_ROOT := $(dir $(abspath $(lastword $(MAKEFILE_LIST))))
TOKO_TOOLING_ROOT := $(REPO_ROOT)vendor/toko-tooling
TOKO_TOOLING_RELEASE := 1
export RELEASE_DELIVERY := direct
export PATH := $(CURDIR)/.tools/host/bin:$(CURDIR)/.tools/rust/common/bin:$(PATH)
RELEASE_DELIVERY_POLICY := ci/release-delivery.txt
RELEASE_SETUP_TARGET := release-setup
RELEASE_PUBLISH_TARGET := publish
CARGO_PUBLISH_PACKAGES = $(if $(filter topology,$(RELEASE_GROUP)),ic-query-topology,$(if $(filter sns-migration,$(RELEASE_GROUP)),ic-query-sns,$(if $(filter query,$(RELEASE_GROUP)),ic-query,$(if $(filter cli,$(RELEASE_GROUP)),ic-query-cli,))))
CARGO_PUBLISH_REGISTRY := crates-io
CARGO_PUBLISH_VERIFY_TARGETS := publication-admission
include $(TOKO_TOOLING_ROOT)/make/consumer.mk
include $(TOKO_TOOLING_ROOT)/make/cargo-dependencies.mk

PACKAGE ?=
CARGO_SELECTION = $(if $(strip $(PACKAGE)),-p "$(PACKAGE)",--workspace)
export RUSTUP_AUTO_INSTALL := 0

.PHONY: help icq check doc-links-check fetch-dependencies clippy rustdoc test qualify-cli
help:
	@printf '%s\n' \
	  'icq                 Install this checkout as icq on PATH (locked)' \
	  'check               Aggregate checks (all packages by default)' \
	  'check PACKAGE=name  Select one package for Rust checks' \
	  'qualify-cli         Test icq help, version and error process contracts' \
	  'qualify-umbrella    Check empty, topology, SNS and combined features' \
	  'qualify-release-validation  Check release dispatch and recovery fixtures' \
	  'qualify-release-reuse       Explicit Linux native multi-group qualification' \
	  'fetch-dependencies  Explicit locked Cargo cache preparation (network)' \
	  'deps-list           List local workspace package versions' \
	  'deps-update         Explicit Cargo dependency update (network)' \
	  'toko-cloc -d 0      Run the installed LOC command in your shell' \
	  'doc-links-check     Check local documentation file links' \
	  'toko-tooling-check  Admit Cargo pin, members, lockfile and snapshot' \
	  'install-tools       Prepare shared host and Rust developer tools'
	@printf '%s\n' 'release-minor RELEASE_GROUP=topology  Release and publish topology' 'release-minor RELEASE_GROUP=query     Release and publish the umbrella' 'release-patch RELEASE_GROUP=sns-migration  Release and publish migration evidence' 'release-patch RELEASE_GROUP=cli       Release and publish the CLI'

check:
	+@$(if $(strip $(PACKAGE)),$(MAKE) --no-print-directory validate-common clippy rustdoc test,bash scripts/ci/validation.sh full)
ci: check

.PHONY: validate-common validate-native validate-topology validate-sns validate-cli validate-products validate-current-cli qualify-release-validation qualify-release-reuse
validate-common: toko-tooling-check tools-check doc-links-check fmt-check

validate-native:
	+@bash scripts/ci/validation.sh native

validate-topology:
	+@bash scripts/ci/reuse-validation.sh stage $@

validate-sns:
	+@bash scripts/ci/reuse-validation.sh stage $@

validate-cli:
	+@bash scripts/ci/reuse-validation.sh stage $@

validate-products:
	+@bash scripts/ci/reuse-validation.sh products

validate-current-cli:
	+@perl "$(TOKO_TOOLING_ROOT)/scripts/ci/release-timing.pl" run query-current-cli executed $(MAKE) --no-print-directory qualify-cli

qualify-release-validation:
	@bash scripts/ci/test-release-validation.sh

qualify-release-reuse:
	@bash scripts/ci/test-release-reuse.sh

icq: toko-tooling-check | toko-execution-check
	cargo install --path crates/ic-query-cli --bin icq --locked --debug --force
	@command -v icq >/dev/null || { printf '%s\n' 'Add your Cargo installation bin directory to PATH, then run icq --help.' >&2; exit 1; }
	@icq --version
	@printf '%s\n' 'Run: icq --help'

fetch-dependencies: | toko-execution-check
	cargo fetch --locked

clippy: toko-tooling-check
	cargo clippy $(CARGO_SELECTION) --all-targets --locked -- -D warnings

rustdoc: toko-tooling-check
	RUSTDOCFLAGS="-D warnings" cargo doc $(CARGO_SELECTION) --no-deps --locked

test: toko-tooling-check
	RUSTFLAGS="-D warnings" cargo test $(CARGO_SELECTION) --locked

qualify-cli: toko-tooling-check
	RUSTFLAGS="-D warnings" cargo test -p ic-query-cli --test cli --locked

.PHONY: qualify-umbrella qualify-umbrella-current
qualify-umbrella:
	+@bash scripts/ci/reuse-validation.sh stage $@

qualify-umbrella-current: toko-tooling-check
	cargo check -p ic-query --no-default-features --locked
	cargo check -p ic-query --no-default-features --features topology --locked
	cargo check -p ic-query --no-default-features --features sns --locked
	cargo clippy -p ic-query --all-features --all-targets --locked -- -D warnings
	RUSTDOCFLAGS="-D warnings" cargo doc -p ic-query --all-features --no-deps --locked
	RUSTFLAGS="-D warnings" cargo test -p ic-query --all-features --locked

doc-links-check: toko-tooling-check
	@perl "$(TOKO_TOOLING_ROOT)/scripts/ci/check-documentation-links.pl" \
	  --root "$(REPO_ROOT)" README.md AGENTS.md CHANGELOG.md docs releases

.PHONY: release-setup release-version release-preflight release-verify release-prepare-version release-prepared-check release-files release-commit-check release-committed-check release-tagged-check release-push-check publication-admission
release-setup:
	+@bash scripts/release.sh setup
release-version:
	@bash scripts/release.sh version
release-preflight:
	+@bash scripts/release.sh preflight
release-verify:
	+@bash scripts/release.sh verify
release-prepare-version:
	@bash scripts/release.sh prepare
release-prepared-check:
	@bash scripts/release.sh prepared
release-files:
	@bash scripts/release.sh files
release-commit-check:
	@bash scripts/release.sh commit
release-committed-check release-tagged-check release-push-check:
	@bash scripts/release.sh committed
publication-admission:
	@bash scripts/release.sh publication
