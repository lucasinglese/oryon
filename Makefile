.PHONY: all lint test test-rust test-python build bench docs docs-serve

# Mirrors CI exactly. Run before pushing

all: lint test-rust test-python

lint:
	cargo fmt --all -- --check
	cargo clippy --all-targets -- -D warnings
	cargo doc --no-deps
	ruff check python/

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

docs-serve:
	cd docs && mkdocs serve

docs:
	cd docs && mkdocs build