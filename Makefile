.PHONY: help version tags patch minor major release-patch release-minor release-major release-stage release-commit release-push test-bump actions-check build changelog-check check ci clean clippy ensure-clean fmt fmt-check install msrv package package-contents-check publish test

MSRV ?= 1.91.0

help:
	@echo "Available commands:"
	@echo ""
	@echo "  fmt        Format Rust code"
	@echo "  fmt-check  Check Rust formatting"
	@echo "  actions-check  Check GitHub Actions are pinned to commit SHAs"
	@echo "  changelog-check  Check changelog entries for the package version"
	@echo "  package-contents-check  Check crate package excludes internal files"
	@echo "  check      Run cargo check with locked dependencies"
	@echo "  clippy     Run clippy with warnings denied"
	@echo "  test       Run all tests with locked dependencies"
	@echo "  msrv       Check the crate with the declared MSRV"
	@echo "  package    Build a publishable crate tarball"
	@echo "  ci         Run the local push gate"
	@echo "  install    Install the local icq binary"
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
	@echo "  release-push   Push the release commit and tags"
	@echo "  clean      Remove build artifacts"

ensure-clean:
	@if ! git diff-index --quiet HEAD --; then \
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
	cargo check --all-targets --all-features --locked

actions-check:
	bash scripts/ci/check-github-actions-pinned.sh

changelog-check:
	bash scripts/ci/check-changelog-version.sh

package-contents-check:
	bash scripts/ci/check-package-contents.sh

clippy:
	cargo clippy --all-targets --all-features --locked -- -D warnings

test:
	cargo test --all-targets --all-features --locked

msrv:
	cargo +$(MSRV) check --all-targets --all-features --locked

package: ensure-clean
	cargo package --locked

ci: changelog-check actions-check package-contents-check fmt-check check clippy test package

test-bump: clippy test

install:
	cargo install --locked --path . --bin icq

publish: ensure-clean
	cargo publish --locked

patch: ensure-clean fmt test-bump
	bash scripts/release/bump-version.sh patch

minor: ensure-clean fmt test-bump
	bash scripts/release/bump-version.sh minor

major: ensure-clean fmt test-bump
	bash scripts/release/bump-version.sh major

release-patch: patch release-stage release-commit release-push

release-minor: minor release-stage release-commit release-push

release-major: major release-stage release-commit release-push

release-stage:
	git add Cargo.toml Cargo.lock

release-commit:
	@version="$$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)"; \
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

release-push:
	git push --follow-tags

build:
	cargo build --all-targets --all-features --locked

clean:
	cargo clean
