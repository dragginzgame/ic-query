.PHONY: \
	canister-build canister-bundle canister-smoke \
	build changelog-check check check-cargo-tool-versions ci ci-scripts-check clean clippy \
	dependency-check dependency-pins-check doc-links-check ensure-clean feature-boundary-check help \
	install install-dependency-tools dependency-tools-check format-installation-check library-process-boundary-check msrv package \
	package-contents-check public-docs-check publish publish-guards-check \
	release-guards-check \
	release-version release-preflight release-verify release-prepare-version \
	release-prepared-check release-files release-commit-check release-committed-check \
	release-tagged-check release-push-check release-tag-check \
	schema-version-check tags test type-docs-check version

.DEFAULT_GOAL := help
REPO_ROOT := $(dir $(abspath $(lastword $(MAKEFILE_LIST))))

# GNU Make 3.81 cannot combine target-specific export and override. Preserve
# caller defaults here so the exported target policies can supersede them.
ifneq ($(origin IC_QUERY_RELEASE_PREPARE_CACHE),undefined)
override IC_QUERY_RELEASE_PREPARE_CACHE := $(IC_QUERY_RELEASE_PREPARE_CACHE)
export IC_QUERY_RELEASE_PREPARE_CACHE
endif
ifneq ($(origin CARGO_NET_OFFLINE),undefined)
override CARGO_NET_OFFLINE := $(CARGO_NET_OFFLINE)
export CARGO_NET_OFFLINE
endif
ifneq ($(origin RUSTUP_AUTO_INSTALL),undefined)
override RUSTUP_AUTO_INSTALL := $(RUSTUP_AUTO_INSTALL)
export RUSTUP_AUTO_INSTALL
endif

MSRV ?= 1.91.0
CARGO_AUDIT_VERSION ?= 0.22.2
CARGO_MACHETE_VERSION ?= 0.9.2
AUDIT_TOOL = bash "$(SHARED_TOOLING_ROOT)/scripts/dev/install-rust-tools.sh" --consumer "$(CURDIR)" --package cargo-audit --version "$(CARGO_AUDIT_VERSION)" --bin cargo-audit --profile release
MACHETE_TOOL = bash "$(SHARED_TOOLING_ROOT)/scripts/dev/install-rust-tools.sh" --consumer "$(CURDIR)" --package cargo-machete --version "$(CARGO_MACHETE_VERSION)" --bin cargo-machete --profile release
CARGO_HTTP_MULTIPLEXING ?= false
CARGO_NET_RETRY ?= 10
CARGO_PUBLISH_INDEX_ATTEMPTS ?= 12
CARGO_PUBLISH_INDEX_DELAY_SECONDS ?= 10
CHANGELOG_VERSION ?=
YQ ?= $(REPO_ROOT).tools/host/bin/yq
override SHARED_TOOLING_ROOT := $(patsubst %/,%,$(REPO_ROOT))
LOCAL_TOOL_INSTALL_TARGETS += install-dependency-tools
LOCAL_TOOL_CHECK_TARGETS += format-tools-check dependency-tools-check
include $(REPO_ROOT)make/consumer.mk
export IC_TOOL_PINS

format-tools-check: format-installation-check

release-patch release-minor release-major release-resume: export IC_QUERY_RELEASE_PREPARE_CACHE := 1

CI_TARGETS := shared-tooling-check tools-check changelog-check dependency-pins-check package-contents-check \
	feature-boundary-check library-process-boundary-check ci-scripts-check \
	publish-guards-check release-guards-check type-docs-check doc-links-check public-docs-check dependency-check \
	schema-version-check fmt-check check clippy test package
CI_INPUTS := Cargo.toml Cargo.lock rust-toolchain.toml crates/ic-query/Cargo.toml \
	crates/ic-query-cli/Cargo.toml ci/tool-versions.env ci/ic-tools.tsv .shared-tooling.snapshot

export CARGO_HTTP_MULTIPLEXING
export CARGO_NET_RETRY
export CARGO_PUBLISH_INDEX_ATTEMPTS
export CARGO_PUBLISH_INDEX_DELAY_SECONDS

help:
	@echo "Available commands:"
	@echo ""
	@echo "  canister-build   Build the isolated Governance probe with ICP CLI 1.6.0"
	@echo "  canister-bundle  Bundle the probe for explicit mainnet-smoke deployment"
	@echo "  canister-smoke   Deploy on a local NNS network and retain an execution receipt"
	@echo "  fmt        Sort Cargo manifests and format Rust code"
	@echo "  fmt-check  Check Cargo dependency order and Rust formatting"
	@echo "  format-tools-check  Check prepared cargo-sort and rustfmt offline"
	@echo "  changelog-check  Check changelog entries for the package version"
	@echo "  package-contents-check  Check crate package excludes internal files"
	@echo "  feature-boundary-check  Check library default/no-default feature boundaries"
	@echo "  library-process-boundary-check  Check process IO remains in the CLI crate"
	@echo "  ci-scripts-check  Check CI helper-script failure and environment handling"
	@echo "  publish-guards-check  Check workspace publication fails closed and resumes safely"
	@echo "  release-guards-check  Check release automation fails closed"
	@echo "  shared-tooling-check  Verify the pinned release-tooling snapshot"
	@echo "  type-docs-check  Check cross-module type documentation blocks"
	@echo "  doc-links-check  Check local links in current guides and contracts"
	@echo "  public-docs-check  Prevent growth in the public rustdoc backlog"
	@echo "  dependency-check  Check advisories and unused direct dependencies"
	@echo "  dependency-pins-check  Check dependency declarations and tracked lockfiles"
	@echo "  schema-version-check  Keep every active pre-1.0 schema identifier at 1"
	@echo "  check      Run cargo check with locked/offline dependencies"
	@echo "  clippy     Run clippy with warnings denied"
	@echo "  test       Run all tests with locked dependencies"
	@echo "  msrv       Check the crate with the declared MSRV"
	@echo "  package    Build a publishable crate tarball"
	@echo "  ci         Run the local push gate"
	@echo "  install    Install the local icq binary"
	@echo "  install-dependency-tools  Install pinned Cargo Audit and Cargo Machete"
	@echo "  dependency-tools-check  Check the selected audit tools offline"
	@echo "  install-tools  Prepare the complete common and Query toolsets in order"
	@echo "  install-hooks  Activate the reviewed formatting hook in this clone"
	@echo "  tools-check  Verify the complete common and Query toolsets offline"
	@echo "  install-host-tools  Install repository-local jq, Mike Farah yq, ripgrep with PCRE2 and cloc"
	@echo "  host-tools-check  Verify the host toolset offline"
	@echo "  install-ic-tools  Install repository-local Quill, ICP, didc, ic-wasm and wasm-opt"
	@echo "  ic-tools-check  Verify the IC toolset offline"
	@echo "  install-rust-tools  Install the common Cargo-tool set locally"
	@echo "  rust-tools-check  Verify the common Cargo-tool set offline"
	@echo "  check-cargo-tool-versions  Check common Cargo pins and user receipts against crates.io (online, read-only)"
	@echo "  cloc       Report Rust runtime/test LOC for this workspace"
	@echo "  publish    Publish the library, then the CLI, to crates.io"
	@echo "  version    Show current version"
	@echo "  tags       List recent git tags"
	@echo "  release-patch  Bump, stage, commit, tag, and push a patch release"
	@echo "  release-minor  Bump, stage, commit, tag, and push a minor release"
	@echo "  release-major  Bump, stage, commit, tag, and push a major release"
	@echo "  release-resume  Reconcile a retained release with VERSION=X.Y.Z"
	@echo "  clean      Remove build artifacts"

ensure-clean:
	@bash "$(REPO_ROOT)scripts/ci/check-release-source.sh"

version release-version:
	@YQ="$(YQ)" bash "$(REPO_ROOT)scripts/ci/read-cargo-workspace-version.sh" --stable Cargo.toml

tags:
	@git tag --sort=-version:refname | head -10

check:
	cargo check --workspace --all-targets --all-features --locked --offline

dependency-pins-check:
	YQ="$(YQ)" bash scripts/ci/check-dependency-pins.sh --cargo-inheritance

changelog-check:
	bash scripts/ci/check-changelog-version.sh $(CHANGELOG_VERSION)

schema-version-check:
	bash scripts/ci/check-schema-versions.sh

package-contents-check:
	bash scripts/ci/check-package-contents.sh

feature-boundary-check:
	bash scripts/ci/check-library-feature-boundaries.sh

library-process-boundary-check:
	bash scripts/ci/check-library-process-boundary.sh

release-guards-check:
	bash scripts/ci/check-release-guards.sh

ci-scripts-check:
	bash scripts/ci/check-ci-scripts.sh
	bash scripts/ci/test-rust-tools.sh
	bash scripts/ci/test-cargo-tool-versions.sh
	bash scripts/ci/check-formatting-hook.sh
	cargo test -p ic-query-cli --example governance_smoke --locked --offline

canister-build canister-bundle canister-smoke:
	cargo fetch --locked
	cargo run -p ic-query-cli --example governance_smoke --locked --offline -- $(if $(filter canister-smoke,$@),local,$(@:canister-%=%))

publish-guards-check:
	bash scripts/ci/check-publish-guards.sh

type-docs-check:
	perl scripts/ci/check-type-docs.pl

doc-links-check:
	perl "$(REPO_ROOT)scripts/ci/check-documentation-links.pl" --root "$(REPO_ROOT)" \
		"$(REPO_ROOT)README.md" "$(REPO_ROOT)AGENTS.md" "$(REPO_ROOT)CHANGELOG.md" \
		"$(REPO_ROOT)docs/"*.md "$(REPO_ROOT)docs/design/"*.md "$(REPO_ROOT)docs/roadmap/"*.md \
		"$(REPO_ROOT)tasks/"*.md

public-docs-check:
	bash scripts/ci/check-public-docs.sh

dependency-check:
	+@audit="$$( $(AUDIT_TOOL) --check )" && \
		machete="$$( $(MACHETE_TOOL) --check )" && \
		bash scripts/ci/check-dependencies.sh "$$audit" "$$machete"

clippy:
	cargo clippy -p ic-query --all-targets --no-default-features --locked --offline -- -D warnings
	cargo clippy --workspace --all-targets --all-features --locked --offline -- -D warnings

test:
	cargo test --workspace --all-targets --all-features --locked --offline

msrv:
	cargo +$(MSRV) check --workspace --all-targets --all-features --locked --offline

package: ensure-clean
	bash scripts/ci/package-workspace.sh --offline

ci:
	+@bash "$(REPO_ROOT)scripts/ci/run-validation-targets.sh" --fail-fast \
		$(foreach input,$(CI_INPUTS),--input "$(input)") $(CI_TARGETS)

install:
	cargo install --locked --force --path crates/ic-query-cli --bin icq

format-installation-check:
	+@bash "$(REPO_ROOT)scripts/dev/install-rust-tools.sh" --consumer "$(CURDIR)" \
		--versions "$(RUST_TOOL_VERSIONS)" --check > /dev/null

check-cargo-tool-versions:
	@perl "$(SHARED_TOOLING_ROOT)/scripts/dev/check-cargo-tool-versions.pl" \
		--consumer "$(CURDIR)" --versions "$(RUST_TOOL_VERSIONS)"

install-dependency-tools:
	+@$(AUDIT_TOOL) > /dev/null
	+@$(MACHETE_TOOL) > /dev/null

dependency-tools-check: export CARGO_NET_OFFLINE := true
dependency-tools-check: export RUSTUP_AUTO_INSTALL := 0
dependency-tools-check:
	+@$(AUDIT_TOOL) --check > /dev/null
	+@$(MACHETE_TOOL) --check > /dev/null

publish: ensure-clean release-tag-check
	bash scripts/release/publish-workspace.sh

export RELEASE_KIND RELEASE_PREVIOUS RELEASE_VERSION RELEASE_DATE RELEASE_SOURCE RELEASE_COMMIT RELEASE_REMOTE RELEASE_BRANCH

release-files:
	@bash "$(REPO_ROOT)scripts/release/adapter.sh" files

release-preflight release-verify release-prepared-check release-commit-check release-committed-check release-tagged-check release-push-check:
	+@bash "$(REPO_ROOT)scripts/release/adapter.sh" "$(@:release-%=%)"

release-prepare-version:
	@bash "$(REPO_ROOT)scripts/release/adapter.sh" prepare

release-tag-check:
	bash "$(REPO_ROOT)scripts/release/check-tag-at-head.sh"

build:
	cargo build --workspace --all-targets --all-features --locked --offline

clean:
	cargo clean
