# How to Contribute

Oryon accepts contributions of **features** and **targets**. The core traits, pipeline, and PyO3 layer follow a fixed pattern - new implementations slot in without touching the architecture.

Every contribution touches **both** layers: the Rust core and the Python binding. The steps below are mandatory and in order.

---

## Adding a Feature

### 1. Write the Rust struct

Create `crates/oryon/src/features/<your_feature>.rs`.

Implement the `Feature` trait. Use `Sma` as your reference implementation.

```rust
use crate::error::OryonError;
use crate::traits::{Feature, Output};
use smallvec::smallvec;

pub struct YourFeature {
    inputs: Vec<String>,
    window: usize,
    outputs: Vec<String>,
    // internal state fields last
}

impl YourFeature {
    pub fn new(
        inputs: Vec<String>,
        window: usize,
        outputs: Vec<String>,
    ) -> Result<Self, OryonError> {
        if inputs.is_empty() {
            return Err(OryonError::InvalidInput { msg: "inputs must not be empty".into() });
        }
        if outputs.is_empty() {
            return Err(OryonError::InvalidInput { msg: "outputs must not be empty".into() });
        }
        if window == 0 {
            return Err(OryonError::InvalidInput { msg: "window must be non-zero".into() });
        }
        Ok(YourFeature { inputs, window, outputs })
    }
}

impl Feature for YourFeature {
    fn input_names(&self) -> Vec<String> { self.inputs.clone() }
    fn output_names(&self) -> Vec<String> { self.outputs.clone() }
    fn warm_up_period(&self) -> usize { self.window - 1 }

    fn fresh(&self) -> Box<dyn Feature> {
        Box::new(YourFeature::new(self.inputs.clone(), self.window, self.outputs.clone())
            .expect("fresh: config was already validated at construction"))
    }

    fn reset(&mut self) {
        // clear internal state
    }

    fn update(&mut self, state: &[Option<f64>]) -> Output {
        // compute and return
        smallvec![None]
    }
}
```

Rules:

- `state[i]` maps to `input_names()[i]` in order.

- Return `smallvec![None]` during warm-up or on `None` input propagation.

- `fresh()` must return a clean-state instance with the same config.

- No `.unwrap()` on `Result` - use `?` or return `OryonError`. Unwrapping an `Option` is fine when the surrounding code logically guarantees `Some`.

### 2. Register in `mod.rs`

In `crates/oryon/src/features/mod.rs`, add:

```rust
pub mod your_feature;
pub use your_feature::YourFeature;
```

### 3. Write tests

Tests go in the same file, inside `#[cfg(test)]`. See [Test Templates](test-templates.md) for the mandatory structure and order.

### 4. Write benchmarks

In `benches/features.rs`, add two benchmark groups:

```rust
c.bench_function("your_feature_update/w20", |b| { ... });
c.bench_function("your_feature_update/w200", |b| { ... });
```

Run with:

```bash
make bench-rust
```

Target: `update` under **1µs** at `w200`. Note it in the PR if exceeded.

### 5. Add the PyO3 wrapper

In `crates/oryon-python/src/features.rs`, add a wrapper following the `Sma` pattern exactly:

```rust
use oryon::features::YourFeature as RustYourFeature;

#[pyclass(module = "oryon")]
pub(crate) struct YourFeature {
    pub(crate) inner: RustYourFeature,
}

#[pymethods]
impl YourFeature {
    #[new]
    pub fn new(inputs: Vec<String>, window: usize, outputs: Vec<String>) -> PyResult<Self> {
        let inner = RustYourFeature::new(inputs, window, outputs)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok(YourFeature { inner })
    }

    fn update(&mut self, values: Vec<f64>) -> Vec<f64> {
        to_python(&self.inner.update(&to_rust(&values)))
    }

    fn reset(&mut self) { self.inner.reset(); }
    fn input_names(&self) -> Vec<String> { self.inner.input_names() }
    fn output_names(&self) -> Vec<String> { self.inner.output_names() }
    fn warm_up_period(&self) -> usize { self.inner.warm_up_period() }

    fn __repr__(&self) -> String {
        format!("YourFeature(inputs={:?}, window={}, outputs={:?})",
            self.inner.input_names(), /* window */ 0, self.inner.output_names())
    }
}
```

### 6. Register in `lib.rs`

In `crates/oryon-python/src/lib.rs`, add your type in three places:

```rust
// 1. use statement at the top
use features::YourFeature;

// 2. branch in extract_feature()
if let Ok(f) = obj.extract::<PyRef<YourFeature>>() {
    return Ok(f.inner.fresh());
}

// 3. module registration
m.add_class::<YourFeature>()?;
```

### 7. Re-export in Python

Two files to update:

In `python/oryon/features.py`, add `YourFeature` to the import from `._oryon` and to `__all__`.

In `python/oryon/__init__.py`, add `YourFeature` to the import from `.features` and to `__all__`.

### 8. Write documentation

Add an entry to the correct API Reference page. See [Doc Templates](doc-templates.md).

---

## Adding a Target

The process mirrors adding a feature, with three differences:

1. Files go in `crates/oryon/src/targets/` and `crates/oryon-python/src/targets.rs`.
2. Implement the `Target` trait instead of `Feature` (no `reset()`, no `fresh()`, stateless `run_research()`).
3. `extract_target()` in `lib.rs` reconstructs the target from stored params (see `FutureReturn` as reference).

See [Architecture](architecture.md) for the full `Target` trait interface.

---

## Before opening a PR

Run the full check suite locally:

```bash
make lint      # cargo fmt + clippy + cargo doc
make test      # cargo test + pytest (requires maturin develop)
make bench-rust
```

**CI must be green before merge.**

Checklist:

- [ ] `make lint` passes (no fmt diff, no clippy warnings, docs compile)
- [ ] `make test` passes (Rust + Python)
- [ ] Benchmarks added with the correct naming convention (the benchmark page is updated by the maintainer before each release)
- [ ] PyO3 wrapper added and registered in all three places in `lib.rs`
- [ ] Python re-export updated in `python/oryon/features.py` (or `targets.py`) and `python/oryon/__init__.py`
- [ ] Documentation entry added using the template
- [ ] No `.unwrap()` on `Result` in library code - use `?` or return `OryonError::InvalidInput`
- [ ] Constructor validates all parameters and returns `OryonError::InvalidInput`
- [ ] `output_names()` pattern follows `{input}_{name}_{param}`