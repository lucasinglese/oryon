# Test Templates

Tests live in the same file as the implementation, inside `#[cfg(test)]`.

---

## Feature tests

### Mandatory order

```
1. feature_contract_tests!(...): always first
2. test_update: correct values on the happy path
3. test_update_none_input: None propagation behavior
4. test_reset_*: internal invariants after reset
5. test_fresh_instances_are_independent
```

Recommended if applicable:

```
6. test_invalid_params: constructor rejects bad inputs
```

### The contract macro

`feature_contract_tests!` generates 6 tests automatically:

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
    use crate::feature_contract_tests;
    use smallvec::smallvec;

    feature_contract_tests!(
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
    fn test_invalid_params() {
        assert!(YourFeature::new(vec![], 3, vec!["out".into()]).is_err());
        assert!(YourFeature::new(vec!["close".into()], 0, vec!["out".into()]).is_err());
        assert!(YourFeature::new(vec!["close".into()], 3, vec![]).is_err());
    }
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
5. test_invalid_params
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
    fn test_invalid_params() {
        assert!(YourTarget::new(vec![], 3, vec!["out".into()]).is_err());
        assert!(YourTarget::new(vec!["close".into()], 0, vec!["out".into()]).is_err());
    }
}
```

---

## Real examples in the codebase

| Type | File |
|---|---|
| Feature | `crates/oryon/src/features/sma.rs` |
| Feature | `crates/oryon/src/features/ema.rs` |
| Target | `crates/oryon/src/targets/future_return.rs` |
| Target | `crates/oryon/src/targets/future_ctc_volatility.rs` |
