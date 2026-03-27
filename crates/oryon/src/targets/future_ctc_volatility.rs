use crate::error::OryonError;
use crate::ops::{log_return, std_dev};
use crate::tools::{pairwise, rolling, shift};
use crate::traits::Target;

/// Future close-to-close realized volatility.
///
/// For each bar *t*, computes the rolling standard deviation of log returns
/// over the next `horizon` bars. The last `horizon` values are `None`
/// because the future is not yet known.
///
/// Output name: `{input}_future_ctc_vol_{horizon}`.
#[derive(Debug)]
pub struct FutureCTCVolatility {
    input: String,
    horizon: usize,
    output: String,
}

impl FutureCTCVolatility {
    /// Create a new `FutureCTCVolatility` target.
    ///
    /// - `input` — price series name (e.g. `"close"`).
    /// - `horizon` — number of bars to look ahead.
    pub fn new(input: &str, horizon: usize) -> Result<Self, OryonError> {
        if input.is_empty() {
            return Err(OryonError::InvalidInput { msg: "input must not be empty".into() });
        }
        if horizon == 0 {
            return Err(OryonError::InvalidInput { msg: "horizon must be > 0".into() });
        }
        let output = format!("{input}_future_ctc_vol_{horizon}");
        Ok(FutureCTCVolatility {
            input: input.to_string(),
            horizon,
            output,
        })
    }
}

impl Target for FutureCTCVolatility {
    fn input_names(&self) -> Vec<String> {
        vec![self.input.clone()]
    }

    fn output_names(&self) -> Vec<String> {
        vec![self.output.clone()]
    }

    fn forward_period(&self) -> usize {
        self.horizon
    }

    fn compute(&self, columns: &[&[Option<f64>]]) -> Vec<Vec<Option<f64>>> {
        let prices = columns[0];
        let shifted_prices = shift(prices, 1);
        let lr = pairwise(prices, &shifted_prices, log_return);
        let vol = rolling(&lr, self.horizon, std_dev);
        let result = shift(&vol, -(self.horizon as isize));
        vec![result]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::target_contract_tests;

    fn prices() -> Vec<Option<f64>> {
        vec![
            Some(100.0), Some(101.0), Some(103.0), Some(102.0),
            Some(105.0), Some(107.0), Some(106.0),
        ]
    }

    fn vol3() -> FutureCTCVolatility {
        FutureCTCVolatility::new("close", 3).unwrap()
    }

    target_contract_tests!(
        FutureCTCVolatility::new("close", 3).unwrap(),
        vec!["close".to_string()],
        vec!["close_future_ctc_vol_3".to_string()],
        3,
        0,
        &prices(),
    );

    #[test]
    fn test_compute_forward_none() {
        let col = &vol3().compute(&[&prices()])[0];
        // last horizon values must be None
        assert_eq!(col[4], None);
        assert_eq!(col[5], None);
        assert_eq!(col[6], None);
    }

    #[test]
    fn test_compute_valid_values() {
        let col = &vol3().compute(&[&prices()])[0];
        assert!((col[0].unwrap() - 0.014966120092234598).abs() < 1e-10);
        assert!((col[1].unwrap() - 0.020212720949768705).abs() < 1e-10);
        assert!((col[2].unwrap() - 0.020094947737925430).abs() < 1e-10);
        assert!((col[3].unwrap() - 0.019890273555006756).abs() < 1e-10);
    }

    #[test]
    fn test_compute_stateless() {
        let target = vol3();
        let p = prices();
        assert_eq!(target.compute(&[&p]), target.compute(&[&p]));
    }

    #[test]
    fn test_invalid_input_empty() {
        let err = FutureCTCVolatility::new("", 3).unwrap_err();
        assert!(matches!(err, OryonError::InvalidInput { ref msg } if msg.contains("input")));
    }

    #[test]
    fn test_invalid_horizon_zero() {
        let err = FutureCTCVolatility::new("close", 0).unwrap_err();
        assert!(matches!(err, OryonError::InvalidInput { ref msg } if msg.contains("horizon")));
    }
}
