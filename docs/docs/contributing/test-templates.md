# Test Templates

Tests live in the same file as the implementation, inside `#[cfg(test)]`.

---

## Feature tests

### Mandatory order

```
1. streaming_transform_contract_tests!(...): always first
2. test_update: correct values on the happy path
3. test_update_none_input: None propagation behavior
4. test_reset_*: internal invariants after reset
5. test_fresh_instances_are_independent
```

Recommended if applicable:

```
6. test_error_raises_when_<condition>: one test per invalid param (e.g. test_error_raises_when_window_is_zero)
```

### The contract macro

`streaming_transform_contract_tests!` generates 6 tests automatically:

- `test_contract_input_names` - `input_names()` matches expected
- `test_contract_output_names` - `output_names()` matches expected
- `test_contract_warm_up_period` - `warm_up_period()` matches expected
- `test_contract_update_output_shape` - `update()` output length matches `output_names()` length
- `test_contract_reset` - after reset, first `update()` returns `None` (skipped if `warm_up == 0`)
- `test_contract_fresh` - fresh instance starts from clean state (skipped if `warm_up == 0`)

### Full template

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::streaming_transform_contract_tests;
    use smallvec::smallvec;

    streaming_transform_contract_tests!(
        YourFeature::new(vec!["close".into()], 3, vec!["close_your_feature_3".into()]).unwrap(),
        vec!["close".to_string()],            // expected input_names
        vec!["close_your_feature_3".to_string()], // expected output_names
        2,                                    // expected warm_up_period (window - 1)
        &[Some(1.0)],                         // a minimal valid state slice
    );

    fn make() -> YourFeature {
        YourFeature::new(vec!["close".into()], 3, vec!["close_your_feature_3".into()]).unwrap()
    }

    fn out(v: Option<f64>) -> Output {
        smallvec![v]
    }

    #[test]
    fn test_update() {
        let mut f = make();
        assert_eq!(f.update(&[Some(1.0)]), out(None));
        assert_eq!(f.update(&[Some(2.0)]), out(None));
        assert_eq!(f.update(&[Some(3.0)]), out(Some(/* expected value */)));
        assert_eq!(f.update(&[Some(4.0)]), out(Some(/* expected value */)));
    }

    #[test]
    fn test_update_none_input() {
        let mut f = make();
        // warm up
        f.update(&[Some(1.0)]);
        f.update(&[Some(2.0)]);
        assert_eq!(f.update(&[Some(3.0)]), out(Some(/* expected */)));
        // None input resets validity
        assert_eq!(f.update(&[None]), out(None));
    }

    #[test]
    fn test_reset_clears_state() {
        let mut f = make();
        f.update(&[Some(1.0)]);
        f.update(&[Some(2.0)]);
        f.reset();
        assert_eq!(f.update(&[Some(1.0)]), out(None)); // back to warm-up
    }

    #[test]
    fn test_fresh_instances_are_independent() {
        let mut f = make();
        f.update(&[Some(1.0)]);

        let mut fresh = f.fresh();

        // fresh starts from scratch
        assert_eq!(fresh.update(&[Some(1.0)]), out(None));
        assert_eq!(fresh.update(&[Some(2.0)]), out(None));
        assert_eq!(fresh.update(&[Some(3.0)]), out(Some(/* expected */)));

        // original continues its own state
        assert_eq!(f.update(&[Some(2.0)]), out(None));
        assert_eq!(f.update(&[Some(3.0)]), out(Some(/* expected */)));
    }

    #[test]
    fn test_error_raises_when_empty_inputs() {
        assert!(YourFeature::new(vec![], 3, vec!["out".into()]).is_err());
    }

    // One test per invalid condition. Name each after the specific case:
    // test_error_raises_when_window_is_zero
    // test_error_raises_when_fast_exceeds_slow
    // test_error_raises_when_empty_outputs
    // etc.
}
```

---

## Target tests

### Mandatory order

```
1. target_contract_tests!(...)
2. test_compute_forward_none: last horizon values are None
3. test_compute_valid_values: correct values on known input
4. test_compute_stateless: same input always returns same output
```

Recommended if applicable:

```
5. test_error_raises_when_<condition>: one test per invalid param — not one monolithic block
   e.g. test_error_raises_when_empty_inputs, test_error_raises_when_horizon_is_zero
```

### The contract macro

`target_contract_tests!` generates 5 tests automatically:

- `test_contract_input_names`
- `test_contract_output_names`
- `test_contract_forward_period`
- `test_contract_warm_up_period`
- `test_contract_compute_shape` - output has the right number of columns, each with the correct length

### Full template

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::target_contract_tests;

    let prices: Vec<Option<f64>> = vec![
        Some(100.0), Some(101.0), Some(103.0), Some(102.0), Some(105.0),
    ];

    target_contract_tests!(
        YourTarget::new(vec!["close".into()], 3, vec!["close_your_target_3".into()]).unwrap(),
        vec!["close".to_string()],
        vec!["close_your_target_3".to_string()],
        3,   // forward_period = horizon
        0,   // warm_up_period
        &prices,
    );

    fn make(horizon: usize) -> YourTarget {
        YourTarget::new(
            vec!["close".into()],
            horizon,
            vec![format!("close_your_target_{horizon}")],
        ).unwrap()
    }

    #[test]
    fn test_compute_forward_none() {
        let t = make(3);
        let prices: Vec<Option<f64>> = vec![
            Some(100.0), Some(101.0), Some(103.0), Some(102.0), Some(105.0),
        ];
        let result = &t.run_research(&[&prices])[0];
        assert_eq!(result[result.len() - 3], None);
        assert_eq!(result[result.len() - 2], None);
        assert_eq!(result[result.len() - 1], None);
    }

    #[test]
    fn test_compute_valid_values() {
        let t = make(2);
        let prices: Vec<Option<f64>> = vec![
            Some(100.0), Some(102.0), Some(101.0), Some(105.0),
        ];
        let result = &t.run_research(&[&prices])[0];
        // verify index 0 and 1 against hand-calculated values
        let val0 = result[0].unwrap();
        let val1 = result[1].unwrap();
        assert!((val0 - /* expected */).abs() < 1e-9);
        assert!((val1 - /* expected */).abs() < 1e-9);
    }

    #[test]
    fn test_compute_stateless() {
        let t = make(2);
        let prices: Vec<Option<f64>> = vec![
            Some(100.0), Some(102.0), Some(101.0), Some(105.0),
        ];
        let r1 = t.run_research(&[&prices]);
        let r2 = t.run_research(&[&prices]);
        assert_eq!(r1, r2);
    }

    #[test]
    fn test_error_raises_when_empty_inputs() {
        assert!(YourTarget::new(vec![], 3, vec!["out".into()]).is_err());
    }

    // One test per invalid condition. Name each after the specific case:
    // test_error_raises_when_horizon_is_zero
    // test_error_raises_when_empty_outputs
    // etc.
}
```

---

## Scaler tests

### Mandatory order

```
1. streaming_transform_contract_tests!(...): always first
2. test_update: correct values on the happy path
3. test_update_none_input: None propagation behavior
4. test_reset_*: internal invariants after reset (for scalers with a rolling window)
5. test_fresh_instances_are_independent
```

Recommended if applicable:

```
6. test_update_all_equal: zero std edge case (for z-score style scalers)
7. test_error_raises_when_<condition>: one test per invalid param
```

Note: `test_contract_reset` and `test_contract_fresh` are skipped automatically by the macro when `warm_up_period == 0` (e.g. `FixedZScore`).

### Full template

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::streaming_transform_contract_tests;
    use smallvec::smallvec;

    streaming_transform_contract_tests!(
        YourScaler::new(vec!["x".into()], 3, vec!["x_scaled".into()]).unwrap(),
        vec!["x".to_string()],
        vec!["x_scaled".to_string()],
        2,             // expected warm_up_period (window - 1)
        &[Some(1.0)],  // a minimal valid state slice
    );

    fn make() -> YourScaler {
        YourScaler::new(vec!["x".into()], 3, vec!["x_scaled".into()]).unwrap()
    }

    fn out(v: Option<f64>) -> Output {
        smallvec![v]
    }

    #[test]
    fn test_update() {
        let mut s = make();
        assert_eq!(s.update(&[Some(1.0)]), out(None));
        assert_eq!(s.update(&[Some(2.0)]), out(None));
        assert_eq!(s.update(&[Some(3.0)]), out(Some(/* expected value */)));
        assert_eq!(s.update(&[Some(4.0)]), out(Some(/* expected value */)));
    }

    #[test]
    fn test_update_none_input() {
        let mut s = make();
        s.update(&[Some(1.0)]);
        s.update(&[Some(2.0)]);
        assert_eq!(s.update(&[Some(3.0)]), out(Some(/* expected */)));
        assert_eq!(s.update(&[None]), out(None));
    }

    #[test]
    fn test_reset_clears_buffer() {
        let mut s = make();
        s.update(&[Some(1.0)]);
        s.update(&[Some(2.0)]);
        s.update(&[Some(3.0)]);
        s.reset();
        assert_eq!(s.update(&[Some(1.0)]), out(None)); // back to warm-up
    }

    #[test]
    fn test_fresh_instances_are_independent() {
        let mut s = make();
        s.update(&[Some(1.0)]);
        s.update(&[Some(2.0)]);

        let mut fresh = s.fresh();
        assert_eq!(fresh.update(&[Some(1.0)]), out(None));
        // original continues
        assert_eq!(s.update(&[Some(3.0)]), out(Some(/* expected */)));
    }

    #[test]
    fn test_error_raises_when_empty_inputs() {
        assert!(YourScaler::new(vec![], 3, vec!["out".into()]).is_err());
    }
}
```

---

## Operator tests

### Mandatory order

```
1. streaming_transform_contract_tests!(...): always first (warm_up = 0, contract_reset and contract_fresh are skipped)
2. test_update: correct values on the happy path
3. test_update_none_input: None propagation for each input
4. test_fresh_instances_are_independent
```

Recommended if applicable:

```
5. test_error_raises_when_<condition>: one test per invalid param
```

Note: no `test_reset_*` - operators are stateless, `reset()` is a no-op. The `test_contract_reset` and `test_contract_fresh` tests are automatically skipped because `warm_up_period == 0`.

### Full template

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::streaming_transform_contract_tests;
    use smallvec::smallvec;

    streaming_transform_contract_tests!(
        YourOperator::new(vec!["a".into(), "b".into()], vec!["result".into()]).unwrap(),
        vec!["a".to_string(), "b".to_string()],
        vec!["result".to_string()],
        0,                          // warm_up_period is always 0 for operators
        &[Some(1.0), Some(2.0)],    // one value per input
    );

    fn op() -> YourOperator {
        YourOperator::new(vec!["a".into(), "b".into()], vec!["result".into()]).unwrap()
    }

    fn out(v: Option<f64>) -> Output {
        smallvec![v]
    }

    #[test]
    fn test_update() {
        let mut op = op();
        assert_eq!(op.update(&[Some(10.0), Some(3.0)]), out(Some(/* expected */)));
    }

    #[test]
    fn test_update_none_input() {
        let mut op = op();
        // None on each input independently
        assert_eq!(op.update(&[None, Some(3.0)]), out(None));
        assert_eq!(op.update(&[Some(10.0), None]), out(None));
        assert_eq!(op.update(&[None, None]), out(None));
    }

    #[test]
    fn test_fresh_instances_are_independent() {
        let mut op = op();
        op.update(&[Some(1.0), Some(2.0)]);

        let mut fresh = op.fresh();
        assert_eq!(fresh.update(&[Some(10.0), Some(3.0)]), out(Some(/* expected */)));
        assert_eq!(op.update(&[Some(5.0), Some(1.0)]), out(Some(/* expected */)));
    }

    #[test]
    fn test_error_raises_when_empty_inputs() {
        assert!(YourOperator::new(vec![], vec!["out".into()]).is_err());
    }
}
```

---

## Python binding tests

One file per feature/scaler/operator/target in `tests/features/`, `tests/scalers/`, `tests/operators/`, or `tests/targets/`.

=== "Feature / Scaler"

    ```python
    import math
    import pytest
    import oryon
    from oryon import YourFeature  # or YourScaler

    def test_warm_up():
        f = YourFeature(inputs=["close"], window=3, outputs=["out"])
        assert math.isnan(f.update([1.0])[0])
        assert math.isnan(f.update([2.0])[0])

    def test_valid_value():
        f = YourFeature(inputs=["close"], window=3, outputs=["out"])
        f.update([1.0])
        f.update([2.0])
        assert abs(f.update([3.0])[0] - /* expected */) < 1e-10

    def test_nan_input_propagates():
        f = YourFeature(inputs=["close"], window=3, outputs=["out"])
        f.update([1.0])
        f.update([2.0])
        assert math.isnan(f.update([float("nan")])[0])

    def test_reset():
        f = YourFeature(inputs=["close"], window=3, outputs=["out"])
        f.update([1.0])
        f.update([2.0])
        f.reset()
        assert math.isnan(f.update([1.0])[0])

    def test_warm_up_period():
        f = YourFeature(inputs=["close"], window=3, outputs=["out"])
        assert f.warm_up_period() == 2

    def test_input_names():
        f = YourFeature(inputs=["close"], window=3, outputs=["out"])
        assert f.input_names() == ["close"]

    def test_output_names():
        f = YourFeature(inputs=["close"], window=3, outputs=["out"])
        assert f.output_names() == ["out"]

    def test_invalid_window():
        with pytest.raises(oryon.InvalidInputError):
            YourFeature(inputs=["close"], window=0, outputs=["out"])

    def test_invalid_inputs():
        with pytest.raises(oryon.InvalidInputError):
            YourFeature(inputs=[], window=3, outputs=["out"])
    ```

=== "Operator"

    ```python
    import math
    import pytest
    import oryon
    from oryon import YourOperator

    def test_valid_value():
        op = YourOperator(inputs=["a", "b"], outputs=["result"])
        assert abs(op.update([10.0, 3.0])[0] - /* expected */) < 1e-10

    def test_nan_input_a():
        op = YourOperator(inputs=["a", "b"], outputs=["result"])
        assert math.isnan(op.update([float("nan"), 3.0])[0])

    def test_nan_input_b():
        op = YourOperator(inputs=["a", "b"], outputs=["result"])
        assert math.isnan(op.update([10.0, float("nan")])[0])

    def test_warm_up_period():
        op = YourOperator(inputs=["a", "b"], outputs=["result"])
        assert op.warm_up_period() == 0

    def test_input_names():
        op = YourOperator(inputs=["a", "b"], outputs=["result"])
        assert op.input_names() == ["a", "b"]

    def test_output_names():
        op = YourOperator(inputs=["a", "b"], outputs=["result"])
        assert op.output_names() == ["result"]

    def test_reset_is_noop():
        op = YourOperator(inputs=["a", "b"], outputs=["result"])
        op.update([10.0, 3.0])
        op.reset()
        assert abs(op.update([5.0, 2.0])[0] - /* expected */) < 1e-10

    def test_invalid_inputs():
        with pytest.raises(oryon.InvalidInputError):
            YourOperator(inputs=[], outputs=["result"])

    def test_invalid_outputs():
        with pytest.raises(oryon.InvalidInputError):
            YourOperator(inputs=["a", "b"], outputs=[])
    ```

---

## Real examples in the codebase

| Type | File |
|---|---|
| Feature | `crates/oryon/src/features/sma.rs` |
| Feature | `crates/oryon/src/features/ema.rs` |
| Scaler | `crates/oryon/src/scalers/rolling_zscore.rs` |
| Operator | `crates/oryon/src/operators/subtract.rs` |
| Target | `crates/oryon/src/targets/future_return.rs` |
| Target | `crates/oryon/src/targets/future_ctc_volatility.rs` |
| Python feature | `tests/features/test_sma.py` |
| Python scaler | `tests/scalers/test_rolling_z_score.py` |
| Python operator | `tests/operators/test_subtract.py` |
