.PHONY: \
	canister-build canister-bundle canister-smoke \
	build changelog-check check ci ci-scripts-check clean clippy \
	dependency-check dependency-pins-check doc-links-check ensure-clean feature-boundary-check fmt fmt-check format-tools-check help \
	install install-dev library-process-boundary-check msrv package \
	package-contents-check public-docs-check publish publish-guards-check \
	release-guards-check release-major release-minor release-patch release-resume \
	release-version release-preflight release-verify release-prepare-version \
	release-prepared-check release-files release-commit-check release-committed-check \
	release-tagged-check release-push-check release-tag-check shared-tooling-check \
	schema-version-check tags test type-docs-check version

.DEFAULT_GOAL := help
REPO_ROOT := $(dir $(abspath $(lastword $(MAKEFILE_LIST))))
RELEASE_REMOTE ?= origin
RELEASE_BRANCH ?= main

# GNU Make 3.81 can put short flags after long options in MFLAGS. MFLAGS also
# preserves invocation options when a caller overrides MAKEFLAGS explicitly.
override icq_make_execution_flags := $(filter-out --% %=%,$(firstword $(MAKEFLAGS)) $(MFLAGS))
ifneq ($(strip $(foreach mode,i n t q,$(findstring $(mode),$(icq_make_execution_flags)))),)
$(error IC Query requires execution with errors enforced; remove ignore-errors, dry-run, touch and question modes)
endif

ifneq ($(word 2,$(filter release-patch release-minor release-major release-resume,$(MAKECMDGOALS))),)
$(error Select exactly one release target)
endif

MSRV ?= 1.91.0
CARGO_AUDIT_VERSION ?= 0.22.2
CARGO_MACHETE_VERSION ?= 0.9.2
CARGO_HTTP_MULTIPLEXING ?= false
CARGO_NET_RETRY ?= 10
CARGO_PUBLISH_INDEX_ATTEMPTS ?= 12
CARGO_PUBLISH_INDEX_DELAY_SECONDS ?= 10
CHANGELOG_VERSION ?=
YQ ?= $(REPO_ROOT).tools/host/bin/yq
SHARED_TOOLING_ROOT := $(REPO_ROOT)
include $(REPO_ROOT)make/tools.mk
export IC_TOOL_PINS

CI_TARGETS := changelog-check shared-tooling-check host-tools-check dependency-pins-check package-contents-check \
	feature-boundary-check library-process-boundary-check ci-scripts-check \
	publish-guards-check release-guards-check type-docs-check doc-links-check public-docs-check dependency-check \
	schema-version-check fmt-check check clippy test package

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
	@echo "  install-dev  Install pinned tools required by the local CI gate"
	@echo "  install-tools  Install the repository-local host and IC toolsets"
	@echo "  tools-check  Verify both toolsets offline"
	@echo "  install-host-tools  Install repository-local jq, Mike Farah yq, ripgrep with PCRE2 and cloc"
	@echo "  host-tools-check  Verify the host toolset offline"
	@echo "  install-ic-tools  Install repository-local Quill, ICP, didc, ic-wasm and wasm-opt"
	@echo "  ic-tools-check  Verify the IC toolset offline"
	@echo "  install-rust-tools  Install the optional shared Cargo-tool set locally"
	@echo "  rust-tools-check  Verify that optional Cargo-tool set offline"
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
	@untracked="$$(git ls-files --others --exclude-standard)" || { \
		echo "error: cannot inventory untracked source" >&2; exit 1; \
	}; \
	if ! git diff-index --quiet HEAD -- || test -n "$$untracked"; then \
		echo "error: working directory is not clean; commit or stash changes first" >&2; \
		exit 1; \
	fi

version release-version:
	@YQ="$(YQ)" bash "$(REPO_ROOT)scripts/ci/read-cargo-workspace-version.sh" --stable Cargo.toml

tags:
	@git tag --sort=-version:refname | head -10

format-tools-check:
	@. "$(HOST_TOOL_VERSIONS)" && bash "$(REPO_ROOT)scripts/ci/check-format-tools.sh" "$$SHARED_TOOLING_CARGO_SORT_VERSION"

fmt fmt-check: format-tools-check

fmt:
	CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0 cargo sort --workspace
	CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0 cargo fmt --all

fmt-check:
	CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0 cargo sort --workspace --check
	CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check

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

shared-tooling-check:
	bash scripts/ci/verify-shared-tooling-snapshot.sh

ci-scripts-check:
	bash scripts/ci/check-ci-scripts.sh
	bash scripts/ci/test-rust-tools.sh
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
	bash scripts/ci/check-dependencies.sh

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
	+@bash "$(REPO_ROOT)scripts/ci/run-validation-targets.sh" --fail-fast $(CI_TARGETS)

install:
	cargo install --locked --force --path crates/ic-query-cli --bin icq

install-dev: install-host-tools
	@. "$(HOST_TOOL_VERSIONS)" && cargo install --locked cargo-sort --version "$$SHARED_TOOLING_CARGO_SORT_VERSION"
	cargo install --locked cargo-audit --version $(CARGO_AUDIT_VERSION)
	cargo install --locked cargo-machete --version $(CARGO_MACHETE_VERSION)

publish: ensure-clean release-tag-check
	bash scripts/release/publish-workspace.sh

release-patch release-minor release-major:
	+@IC_QUERY_RELEASE_PREPARE_CACHE=1 bash "$(REPO_ROOT)scripts/ci/run-release.sh" "$(@:release-%=%)" "$(RELEASE_REMOTE)" "$(RELEASE_BRANCH)"

release-resume:
	+@IC_QUERY_RELEASE_PREPARE_CACHE=1 bash "$(REPO_ROOT)scripts/ci/run-release.sh" resume "$(VERSION)" "$(RELEASE_REMOTE)" "$(RELEASE_BRANCH)"

export RELEASE_KIND RELEASE_PREVIOUS RELEASE_VERSION RELEASE_DATE RELEASE_SOURCE RELEASE_COMMIT RELEASE_REMOTE RELEASE_BRANCH

release-files:
	@bash "$(REPO_ROOT)scripts/release/adapter.sh" files

release-preflight release-verify release-prepared-check release-commit-check release-committed-check release-tagged-check release-push-check:
	@bash "$(REPO_ROOT)scripts/release/adapter.sh" "$(@:release-%=%)"

release-prepare-version:
	@bash "$(REPO_ROOT)scripts/release/adapter.sh" prepare

release-tag-check:
	bash "$(REPO_ROOT)scripts/release/check-tag-at-head.sh"

build:
	cargo build --workspace --all-targets --all-features --locked --offline

clean:
	cargo clean
