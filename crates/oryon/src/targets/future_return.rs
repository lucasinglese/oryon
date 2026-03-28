use crate::error::OryonError;
use crate::ops::simple_return;
use crate::tools::{pairwise, shift};
use crate::traits::Target;

/// Future simple return over `horizon` bars.
///
/// For each bar *t*, computes `(price[t + horizon] - price[t]) / price[t]`.
/// The last `horizon` values are `None` because the future is not yet known.
#[derive(Debug)]
pub struct FutureReturn {
    inputs: Vec<String>,
    horizon: usize,
    outputs: Vec<String>,
}

impl FutureReturn {
    /// Create a new `FutureReturn` target.
    ///
    /// - `inputs`  - price series name (e.g. `["close"]`). Must contain at least 1 entry.
    /// - `horizon` - number of bars to look ahead. Must be > 0.
    /// - `outputs` - name of the output column (e.g. `["close_future_return_5"]`).
    pub fn new(inputs: Vec<String>, horizon: usize, outputs: Vec<String>) -> Result<Self, OryonError> {
        if inputs.is_empty() {
            return Err(OryonError::InvalidInput {
                msg: "inputs must not be empty".into(),
            });
        }
        if horizon == 0 {
            return Err(OryonError::InvalidInput {
                msg: "horizon must be > 0".into(),
            });
        }
        if outputs.is_empty() {
            return Err(OryonError::InvalidInput {
                msg: "outputs must not be empty".into(),
            });
        }
        Ok(FutureReturn { inputs, horizon, outputs })
    }
}

impl Target for FutureReturn {
    fn input_names(&self) -> Vec<String> {
        self.inputs.clone()
    }

    fn output_names(&self) -> Vec<String> {
        self.outputs.clone()
    }

    fn forward_period(&self) -> usize {
        self.horizon
    }

    fn compute(&self, columns: &[&[Option<f64>]]) -> Vec<Vec<Option<f64>>> {
        let prices = columns[0];
        let future_prices = shift(prices, -(self.horizon as isize));
        // simple_return(data): data[0]=previous=price[t], data[1]=current=price[t+horizon]
        vec![pairwise(prices, &future_prices, simple_return)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::target_contract_tests;

    fn prices() -> Vec<Option<f64>> {
        vec![
            Some(100.0), Some(102.0), Some(105.0), Some(103.0),
            Some(108.0), Some(107.0), Some(110.0),
        ]
    }

    fn fr2() -> FutureReturn {
        FutureReturn::new(vec!["close".into()], 2, vec!["close_future_return_2".into()]).unwrap()
    }

    target_contract_tests!(
        FutureReturn::new(vec!["close".into()], 2, vec!["close_future_return_2".into()]).unwrap(),
        vec!["close".to_string()],
        vec!["close_future_return_2".to_string()],
        2,
        0,
        &prices(),
    );

    #[test]
    fn test_compute_forward_none() {
        let col = &fr2().compute(&[&prices()])[0];
        assert_eq!(col[5], None);
        assert_eq!(col[6], None);
    }

    #[test]
    fn test_compute_valid_values() {
        let col = &fr2().compute(&[&prices()])[0];
        // bar 0: (105 - 100) / 100 = 0.05
        assert!((col[0].unwrap() - 0.05).abs() < 1e-10);
        // bar 1: (103 - 102) / 102 = 0.009803921...
        assert!((col[1].unwrap() - 1.0_f64 / 102.0).abs() < 1e-10);
        // bar 2: (108 - 105) / 105 = 0.028571428...
        assert!((col[2].unwrap() - 3.0_f64 / 105.0).abs() < 1e-10);
        // bar 4: (110 - 108) / 108 = 0.018518518...
        assert!((col[4].unwrap() - 2.0_f64 / 108.0).abs() < 1e-10);
    }

    #[test]
    fn test_compute_stateless() {
        let target = fr2();
        let p = prices();
        assert_eq!(target.compute(&[&p]), target.compute(&[&p]));
    }

    #[test]
    fn test_invalid_params() {
        assert!(matches!(
            FutureReturn::new(vec![], 2, vec!["out".into()]).unwrap_err(),
            OryonError::InvalidInput { ref msg } if msg.contains("inputs")
        ));
        assert!(matches!(
            FutureReturn::new(vec!["close".into()], 0, vec!["out".into()]).unwrap_err(),
            OryonError::InvalidInput { ref msg } if msg.contains("horizon")
        ));
        assert!(matches!(
            FutureReturn::new(vec!["close".into()], 2, vec![]).unwrap_err(),
            OryonError::InvalidInput { ref msg } if msg.contains("outputs")
        ));
    }
}
