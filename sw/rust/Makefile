.PHONY: build test lint fmt check clean doc bench bench-fast

build:
	cargo build --release

test:
	cargo test --release

bench:
	cargo bench -p tagma-benchmarks --release

bench-fast:
	cargo bench -p tagma-benchmarks --release -- stress  # 500k mixed ops only

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
