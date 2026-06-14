.PHONY: help build check ci clean clippy fmt fmt-check install major minor msrv package patch publish release-major release-minor release-patch release-push test

MSRV ?= 1.91.0
RELEASE_REMOTE ?= origin

help:
	@echo "Available commands:"
	@echo ""
	@echo "  fmt        Format Rust code"
	@echo "  fmt-check  Check Rust formatting"
	@echo "  check      Run cargo check with locked dependencies"
	@echo "  clippy     Run clippy with warnings denied"
	@echo "  test       Run all tests with locked dependencies"
	@echo "  msrv       Check the crate with the declared MSRV"
	@echo "  package    Build a publishable crate tarball"
	@echo "  ci         Run the local push gate"
	@echo "  install    Install the local icq binary"
	@echo "  patch      Bump patch version, commit, tag, and push"
	@echo "  minor      Bump minor version, commit, tag, and push"
	@echo "  major      Bump major version, commit, tag, and push"
	@echo "  release-patch  Alias for patch"
	@echo "  release-minor  Alias for minor"
	@echo "  release-major  Alias for major"
	@echo "  release-push   Push the current release commit and tag"
	@echo "  clean      Remove build artifacts"

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all -- --check

check:
	cargo check --all-targets --all-features --locked

clippy:
	cargo clippy --all-targets --all-features --locked -- -D warnings

test:
	cargo test --all-targets --all-features --locked

msrv:
	cargo +$(MSRV) check --all-targets --all-features --locked

package:
	cargo package --locked

ci: fmt-check check clippy test package

install:
	cargo install --locked --path . --bin icq

publish:
	cargo publish --locked

patch:
	bash scripts/release/bump-version.sh patch
	$(MAKE) release-push

minor:
	bash scripts/release/bump-version.sh minor
	$(MAKE) release-push

major:
	bash scripts/release/bump-version.sh major
	$(MAKE) release-push

release-patch: patch

release-minor: minor

release-major: major

release-push:
	@version="$$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)"; \
	if [ -z "$$version" ]; then \
		echo "error: failed to read package version from Cargo.toml" >&2; \
		exit 1; \
	fi; \
	tag="v$$version"; \
	if ! git rev-parse --verify --quiet "refs/tags/$$tag" >/dev/null; then \
		echo "error: expected release tag $$tag to exist before push" >&2; \
		exit 1; \
	fi; \
	git push "$(RELEASE_REMOTE)" HEAD "$$tag"

build:
	cargo build --all-targets --all-features --locked

clean:
	cargo clean
