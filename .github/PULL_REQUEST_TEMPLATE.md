## What does this PR do?

<!-- One sentence. "Adds X feature / Fixes Y bug / ..." -->

## Checklist

### Rust core
- [ ] `make lint` passes (no fmt diff, no clippy warnings, docs compile)
- [ ] `make test` passes (Rust + Python)
- [ ] `make bench-rust` runs without error
- [ ] Struct and `new()` added in `crates/oryon/src/features/` (or `targets/`)
- [ ] Registered in `crates/oryon/src/features/mod.rs` (or `targets/mod.rs`)
- [ ] No `.unwrap()` on `Result` in library code
- [ ] Constructor validates all parameters and returns `OryonError::InvalidInput`
- [ ] `output_names()` pattern follows `{input}_{name}_{param}`

### Tests
- [ ] `feature_contract_tests!` or `target_contract_tests!` macro present (always first)
- [ ] `test_update` / `test_compute_valid_values` with known expected values
- [ ] `test_update_none_input` / `test_compute_forward_none`
- [ ] `test_reset_*` and `test_fresh_instances_are_independent` (features only)
- [ ] `test_compute_stateless` (targets only)

### Benchmarks
- [ ] Features: `{name}_update/w20` and `{name}_update/w200` added in `benches/features.rs`
- [ ] Targets: `{name}_compute/h20/1000_bars` and `{name}_compute/h200/1000_bars` added in `benches/targets.rs`
- [ ] `update` stays under 1µs at `w200`. Note here if exceeded: 

### Python binding
- [ ] PyO3 wrapper added in `crates/oryon-python/src/features.rs` (or `targets.rs`)
- [ ] Registered in `lib.rs` in all three places: use statement, `extract_feature()` / `extract_target()`, `m.add_class`
- [ ] Re-exported in `python/oryon/features.py` (or `targets.py`) and `python/oryon/__init__.py`

### Documentation
- [ ] Entry added to the API Reference page using the doc template