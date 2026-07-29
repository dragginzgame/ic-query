.PHONY: \
	actions-check build changelog-check check ci ci-scripts-check clean clippy \
	dependency-check ensure-clean feature-boundary-check fmt fmt-check help \
	install install-dev library-process-boundary-check major minor msrv package \
	package-contents-check patch public-docs-check publish publish-guards-check \
	release-commit \
	release-guards-check release-major release-minor release-patch release-push \
	release-stage release-tag-check tags test type-docs-check version

REPO_ROOT := $(dir $(abspath $(lastword $(MAKEFILE_LIST))))

MSRV ?= 1.91.0
CARGO_AUDIT_VERSION ?= 0.22.2
CARGO_MACHETE_VERSION ?= 0.9.2
RIPGREP_VERSION ?= 15.1.0
CARGO_HTTP_MULTIPLEXING ?= false
CARGO_NET_RETRY ?= 10
CARGO_PACKAGE_RETRIES ?= 3
CARGO_PUBLISH_INDEX_ATTEMPTS ?= 12
CARGO_PUBLISH_INDEX_DELAY_SECONDS ?= 10
CHANGELOG_VERSION ?=

CI_TARGETS := changelog-check actions-check package-contents-check \
	feature-boundary-check library-process-boundary-check ci-scripts-check \
	publish-guards-check release-guards-check type-docs-check public-docs-check dependency-check \
	fmt-check check clippy test package

export CARGO_HTTP_MULTIPLEXING
export CARGO_NET_RETRY
export CARGO_PACKAGE_RETRIES
export CARGO_PUBLISH_INDEX_ATTEMPTS
export CARGO_PUBLISH_INDEX_DELAY_SECONDS

help:
	@echo "Available commands:"
	@echo ""
	@echo "  fmt        Format Rust code"
	@echo "  fmt-check  Check Rust formatting"
	@echo "  actions-check  Check GitHub Actions are pinned to commit SHAs"
	@echo "  changelog-check  Check changelog entries for the package version"
	@echo "  package-contents-check  Check crate package excludes internal files"
	@echo "  feature-boundary-check  Check library default/no-default feature boundaries"
	@echo "  library-process-boundary-check  Check process IO remains in the CLI crate"
	@echo "  ci-scripts-check  Check CI helper-script failure and environment handling"
	@echo "  publish-guards-check  Check workspace publication fails closed and resumes safely"
	@echo "  release-guards-check  Check release automation fails closed"
	@echo "  type-docs-check  Check cross-module type documentation blocks"
	@echo "  public-docs-check  Prevent growth in the public rustdoc backlog"
	@echo "  dependency-check  Check advisories and unused direct dependencies"
	@echo "  check      Run cargo check with locked dependencies"
	@echo "  clippy     Run clippy with warnings denied"
	@echo "  test       Run all tests with locked dependencies"
	@echo "  msrv       Check the crate with the declared MSRV"
	@echo "  package    Build a publishable crate tarball"
	@echo "  ci         Run the local push gate"
	@echo "  install    Install the local icq binary"
	@echo "  install-dev  Install pinned tools required by the local CI gate"
	@echo "  publish    Publish the library, then the CLI, to crates.io"
	@echo "  version    Show current version"
	@echo "  tags       List recent git tags"
	@echo "  patch      Run release gate, then bump patch version files"
	@echo "  minor      Run release gate, then bump minor version files"
	@echo "  major      Run release gate, then bump major version files"
	@echo "  release-patch  Bump, stage, commit, tag, and push a patch release"
	@echo "  release-minor  Bump, stage, commit, tag, and push a minor release"
	@echo "  release-major  Bump, stage, commit, tag, and push a major release"
	@echo "  release-stage  Stage release version files after review"
	@echo "  release-commit Commit and tag the staged release"
	@echo "  release-push   Verify and push the release commit and tags"
	@echo "  clean      Remove build artifacts"

ensure-clean:
	@if ! git diff-index --quiet HEAD -- || test -n "$$(git ls-files --others --exclude-standard)"; then \
		echo "error: working directory is not clean; commit or stash changes first" >&2; \
		exit 1; \
	fi

version:
	@sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1

tags:
	@git tag --sort=-version:refname | head -10

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all -- --check

check:
	cargo check --workspace --all-targets --all-features --locked

actions-check:
	bash scripts/ci/check-github-actions-pinned.sh

changelog-check:
	bash scripts/ci/check-changelog-version.sh $(CHANGELOG_VERSION)

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

publish-guards-check:
	bash scripts/ci/check-publish-guards.sh

type-docs-check:
	perl scripts/ci/check-type-docs.pl

public-docs-check:
	bash scripts/ci/check-public-docs.sh

dependency-check:
	# These maintenance advisories are transitive through ic-agent/candid.
	# Deny every warning that is not part of this reviewed baseline.
	cargo audit --deny warnings \
		--ignore RUSTSEC-2021-0127 \
		--ignore RUSTSEC-2024-0384 \
		--ignore RUSTSEC-2024-0436 \
		--ignore RUSTSEC-2025-0012
	cargo machete --with-metadata

clippy:
	cargo clippy --workspace --all-targets --all-features --locked -- -D warnings

test:
	cargo test --workspace --all-targets --all-features --locked

msrv:
	cargo +$(MSRV) check --workspace --all-targets --all-features --locked

package: ensure-clean
	bash scripts/ci/package-workspace.sh

ci:
	+@set -e; for target in $(CI_TARGETS); do \
		$(MAKE) --no-print-directory "$$target"; \
	done

install:
	cargo install --locked --path crates/ic-query-cli --bin icq

install-dev:
	cargo install --locked ripgrep --version $(RIPGREP_VERSION)
	cargo install --locked cargo-audit --version $(CARGO_AUDIT_VERSION)
	cargo install --locked cargo-machete --version $(CARGO_MACHETE_VERSION)

publish: ensure-clean release-tag-check
	bash scripts/release/publish-workspace.sh

patch:
	bash scripts/release/bump-version.sh patch

minor:
	bash scripts/release/bump-version.sh minor

major:
	bash scripts/release/bump-version.sh major

release-patch:
	+$(MAKE) --no-print-directory patch
	+$(MAKE) --no-print-directory release-stage
	+$(MAKE) --no-print-directory release-commit
	+$(MAKE) --no-print-directory release-push

release-minor:
	+$(MAKE) --no-print-directory minor
	+$(MAKE) --no-print-directory release-stage
	+$(MAKE) --no-print-directory release-commit
	+$(MAKE) --no-print-directory release-push

release-major:
	+$(MAKE) --no-print-directory major
	+$(MAKE) --no-print-directory release-stage
	+$(MAKE) --no-print-directory release-commit
	+$(MAKE) --no-print-directory release-push

release-stage:
	git add Cargo.toml Cargo.lock crates/ic-query/Cargo.toml crates/ic-query-cli/Cargo.toml

release-commit:
	@set -eu; \
	version="$$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)"; \
	if [ -z "$$version" ]; then \
		echo "error: failed to read package version from Cargo.toml" >&2; \
		exit 1; \
	fi; \
	if git rev-parse "v$$version" >/dev/null 2>&1; then \
		echo "error: tag v$$version already exists; aborting" >&2; \
		exit 1; \
	fi; \
	git commit -m "Release $$version"; \
	git tag -a "v$$version" -m "Release $$version"

release-tag-check:
	bash "$(REPO_ROOT)scripts/release/check-tag-at-head.sh"

release-push: ensure-clean release-tag-check
	+$(MAKE) --no-print-directory ci
	git push --follow-tags

build:
	cargo build --workspace --all-targets --all-features --locked

clean:
	cargo clean
