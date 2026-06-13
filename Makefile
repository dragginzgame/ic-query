.PHONY: help build check ci clean clippy fmt fmt-check install msrv package publish test

MSRV ?= 1.91.0

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

build:
	cargo build --all-targets --all-features --locked

clean:
	cargo clean
