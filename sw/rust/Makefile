.PHONY: build test lint fmt check clean doc

build:
	cargo build --release

test:
	cargo test --release

lint:
	cargo clippy --all-targets -- -D warnings

fmt:
	cargo fmt --all

check:
	cargo fmt --check
	cargo clippy --all-targets -- -D warnings
	cargo build --release
	cargo test --release

doc:
	cargo doc --no-deps

clean:
	cargo clean
