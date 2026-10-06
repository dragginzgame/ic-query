.PHONY: \
	canister-build canister-bundle canister-smoke \
	build changelog-check check ci ci-scripts-check clean clippy \
	dependency-check dependency-pins-check ensure-clean feature-boundary-check fmt fmt-check help \
	install install-dev install-tools tools-check install-host-tools host-tools-check \
	install-ic-tools ic-tools-check library-process-boundary-check msrv package \
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

ifneq ($(word 2,$(filter release-patch release-minor release-major release-resume,$(MAKECMDGOALS))),)
$(error Select exactly one release target)
endif

MSRV ?= 1.91.0
CARGO_AUDIT_VERSION ?= 0.22.2
CARGO_MACHETE_VERSION ?= 0.9.2
RIPGREP_VERSION ?= 15.1.0
CARGO_HTTP_MULTIPLEXING ?= false
CARGO_NET_RETRY ?= 10
CARGO_PUBLISH_INDEX_ATTEMPTS ?= 12
CARGO_PUBLISH_INDEX_DELAY_SECONDS ?= 10
CHANGELOG_VERSION ?=
HOST_TOOL_VERSIONS ?= $(REPO_ROOT)ci/tool-versions.env
IC_TOOL_PINS ?= $(REPO_ROOT)ci/ic-tools.tsv
YQ ?= $(REPO_ROOT).tools/host/bin/yq
export IC_TOOL_PINS
export PATH := $(REPO_ROOT).tools/host/bin:$(REPO_ROOT).tools/ic/bin:$(PATH)

CI_TARGETS := changelog-check shared-tooling-check host-tools-check dependency-pins-check package-contents-check \
	feature-boundary-check library-process-boundary-check ci-scripts-check \
	publish-guards-check release-guards-check type-docs-check public-docs-check dependency-check \
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
	@echo "  fmt        Format Rust code"
	@echo "  fmt-check  Check Rust formatting"
	@echo "  changelog-check  Check changelog entries for the package version"
	@echo "  package-contents-check  Check crate package excludes internal files"
	@echo "  feature-boundary-check  Check library default/no-default feature boundaries"
	@echo "  library-process-boundary-check  Check process IO remains in the CLI crate"
	@echo "  ci-scripts-check  Check CI helper-script failure and environment handling"
	@echo "  publish-guards-check  Check workspace publication fails closed and resumes safely"
	@echo "  release-guards-check  Check release automation fails closed"
	@echo "  shared-tooling-check  Verify the pinned release-tooling snapshot"
	@echo "  type-docs-check  Check cross-module type documentation blocks"
	@echo "  public-docs-check  Prevent growth in the public rustdoc backlog"
	@echo "  dependency-check  Check advisories and unused direct dependencies"
	@echo "  dependency-pins-check  Check dependency declarations and tracked lockfiles"
	@echo "  schema-version-check  Keep every active pre-1.0 schema identifier at 1"
	@echo "  check      Run cargo check with locked dependencies"
	@echo "  clippy     Run clippy with warnings denied"
	@echo "  test       Run all tests with locked dependencies"
	@echo "  msrv       Check the crate with the declared MSRV"
	@echo "  package    Build a publishable crate tarball"
	@echo "  ci         Run the local push gate"
	@echo "  install    Install the local icq binary"
	@echo "  install-dev  Install pinned tools required by the local CI gate"
	@echo "  install-tools  Install the repository-local host and IC toolsets"
	@echo "  tools-check  Verify both toolsets offline"
	@echo "  install-host-tools  Install repository-local jq and Mike Farah yq"
	@echo "  host-tools-check  Verify the host toolset offline"
	@echo "  install-ic-tools  Install repository-local Quill, ICP, didc, ic-wasm, PocketIC and wasm-opt"
	@echo "  ic-tools-check  Verify the IC toolset offline"
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
	@perl "$(REPO_ROOT)scripts/release/metadata.pl" version

tags:
	@git tag --sort=-version:refname | head -10

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all -- --check

check:
	cargo check --workspace --all-targets --all-features --locked

dependency-pins-check:
	YQ="$(YQ)" bash scripts/ci/check-dependency-pins.sh

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
	python3 -m unittest discover -s scripts/canister -p 'test_*.py'

canister-build:
	python3 scripts/canister/smoke.py build

canister-bundle:
	python3 scripts/canister/smoke.py bundle

canister-smoke:
	python3 scripts/canister/smoke.py local

publish-guards-check:
	bash scripts/ci/check-publish-guards.sh

type-docs-check:
	perl scripts/ci/check-type-docs.pl

public-docs-check:
	bash scripts/ci/check-public-docs.sh

dependency-check:
	bash scripts/ci/check-dependencies.sh

clippy:
	cargo clippy -p ic-query --all-targets --no-default-features --locked -- -D warnings
	cargo clippy --workspace --all-targets --all-features --locked -- -D warnings

test:
	cargo test --workspace --all-targets --all-features --locked

msrv:
	cargo +$(MSRV) check --workspace --all-targets --all-features --locked

package: ensure-clean
	bash scripts/ci/package-workspace.sh

ci:
	+@bash "$(REPO_ROOT)scripts/ci/run-validation-targets.sh" --fail-fast $(CI_TARGETS)

install:
	cargo install --locked --force --path crates/ic-query-cli --bin icq

install-tools:
	+$(MAKE) --no-print-directory install-host-tools
	+$(MAKE) --no-print-directory install-ic-tools

tools-check:
	+$(MAKE) --no-print-directory host-tools-check
	+$(MAKE) --no-print-directory ic-tools-check

install-host-tools:
	bash scripts/dev/install-host-tools.sh --versions "$(HOST_TOOL_VERSIONS)"

host-tools-check:
	bash scripts/dev/install-host-tools.sh --versions "$(HOST_TOOL_VERSIONS)" --check

install-ic-tools:
	bash scripts/dev/install-ic-tools.sh --pins "$(IC_TOOL_PINS)"

ic-tools-check:
	bash scripts/dev/install-ic-tools.sh --pins "$(IC_TOOL_PINS)" --check

install-dev: install-host-tools
	cargo install --locked ripgrep --version $(RIPGREP_VERSION)
	cargo install --locked cargo-audit --version $(CARGO_AUDIT_VERSION)
	cargo install --locked cargo-machete --version $(CARGO_MACHETE_VERSION)

publish: ensure-clean release-tag-check
	bash scripts/release/publish-workspace.sh

release-patch release-minor release-major:
	+@bash "$(REPO_ROOT)scripts/ci/run-release.sh" "$(@:release-%=%)" "$(RELEASE_REMOTE)" "$(RELEASE_BRANCH)"

release-resume:
	+@bash "$(REPO_ROOT)scripts/ci/run-release.sh" resume "$(VERSION)" "$(RELEASE_REMOTE)" "$(RELEASE_BRANCH)"

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
	cargo build --workspace --all-targets --all-features --locked

clean:
	cargo clean
