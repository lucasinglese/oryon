# Architecture

Oryon is split into two crates and a thin Python layer.

---

## Layer diagram

```
crates/oryon/src/
  ops/              Pure stateless functions (no trait, no state)
  features/         Streaming transforms — implement Feature trait
  targets/          Batch labels — implement Target trait
  pipeline/         FeaturePipeline + TargetPipeline orchestration
  checks/           Bar-level guards (single value)
  diagnostics/      Dataset-level guards (full column slice)
  testing/          feature_contract_tests! and target_contract_tests! macros
  error.rs          OryonError enum
  traits.rs         Feature and Target trait definitions

crates/oryon-python/src/
  features.rs       PyO3 wrappers for every Feature
  targets.rs        PyO3 wrappers for every Target
  pipelines.rs      PyO3 wrappers for pipelines
  lib.rs            Module registration + extract_feature / extract_target dispatch

python/oryon/
  features.py       Re-exports from the compiled .so
  targets.py        Re-exports from the compiled .so
  __init__.py       Top-level exports
```

---

## The `Feature` trait

Defined in `crates/oryon/src/traits.rs`. Features are streaming and stateful.

| Method | Signature | Notes |
|---|---|---|
| `input_names` | `&self -> Vec<String>` | Ordered list of input column names |
| `output_names` | `&self -> Vec<String>` | Ordered list of output column names |
| `warm_up_period` | `&self -> usize` | Bars before first valid output. Default: `0` |
| `fresh` | `&self -> Box<dyn Feature>` | New instance, same config, clean state |
| `reset` | `&mut self` | Clear internal state in place |
| `update` | `&mut self, state: &[Option<f64>] -> Output` | Process one bar |

`Output` is `SmallVec<[Option<f64>; 4]>` - stack-allocated for up to 4 outputs.

`state[i]` maps to `input_names()[i]`. During warm-up or on `None` input, return `smallvec![None]`.

Features must be `Send + Sync`.

---

## The `Target` trait

Targets are stateless. `run_research()` takes `&self` - no `reset()`, no `fresh()`.

| Method | Signature | Notes |
|---|---|---|
| `input_names` | `&self -> Vec<String>` | Ordered list of input column names |
| `output_names` | `&self -> Vec<String>` | Ordered list of output column names |
| `forward_period` | `&self -> usize` | Bars at the end that will be `None` |
| `warm_up_period` | `&self -> usize` | Bars at the start that will be `None`. Default: `0` |
| `run_research` | `&self, columns: &[&[Option<f64>]] -> Vec<Vec<Option<f64>>>` | Full-series computation |

`columns[i]` maps to `input_names()[i]`. Returns one `Vec<Option<f64>>` per output name, each the same length as the input.

---

## `ops/`

Pure functions with no state and no trait. Two signature conventions:

- Mono-column: `fn op(data: &[Option<f64>]) -> Option<f64>`
- Multi-column: `fn op(x: &[Option<f64>], y: &[Option<f64>]) -> Option<f64>`

When you need a new computation, check `ops/` first - it may already exist. If not, add it there so it is reusable by future features and targets.

---

## `checks/` vs `diagnostics/`

- `checks/` - functions on a **single value** (`Option<f64>`). Used bar-by-bar in streaming.
- `diagnostics/` - functions on a **full column slice** (`&[Option<f64>]`). Used in pre-flight research.

---

## PyO3 binding layer

`crates/oryon-python/` wraps each Rust type in a thin `#[pyclass]`. The wrapper stores either an `inner: RustType` (for features) or the raw constructor params (for targets, which are reconstructed on demand).

`lib.rs` has two dispatch functions:

- `extract_feature(obj)` - matches a Python object to a Rust `Box<dyn Feature>` via `fresh()`
- `extract_target(obj)` - matches a Python object and reconstructs a `Box<dyn Target>`

Every new type must be added to both the relevant dispatch function and the `#[pymodule]` registration block.

---

## Naming conventions

| What | Pattern | Example |
|---|---|---|
| Feature file | `crates/oryon/src/features/<snake>.rs` | `simple_moving_average.rs` |
| Target file | `crates/oryon/src/targets/<snake>.rs` | `future_return.rs` |
| Output column | `{input}_{name}_{param}` | `close_sma_20` |
| Bench group | `{name}_update/w{n}` (feature) | `sma_update/w200` |
| Bench group | `{name}_compute/h{n}/{k}_bars` (target) | `future_return_compute/h20/1000_bars` |
