.PHONY: lint test test-rust test-python build bench

# Mirrors CI exactly — run before pushing

lint:
	cargo fmt --all -- --check
	cargo clippy --all-targets -- -D warnings
	cargo doc --no-deps

test-rust:
	cargo test --all

test-python:
	maturin develop --release
	pytest

test: test-rust test-python

build:
	maturin develop --release

bench-rust:
	cargo bench --bench features -- --output-format bencher
	cargo bench --bench ops     -- --output-format bencher
	cargo bench --bench targets -- --output-format bencher

bench-python:
	pytest tests/benchmarks/ --benchmark-only