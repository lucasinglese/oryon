use crate::ops::{log_return, std_dev};
use crate::tools::{pairwise, rolling, shift};
use crate::traits::Target;

/// Future close-to-close realized volatility.
///
/// For each bar *t*, computes the rolling standard deviation of log returns
/// over the next `horizon` bars. The last `horizon` values are `None`
/// because the future is not yet known.
///
/// Output key: `{col}_future_ctc_vol_{horizon}`.
pub struct FutureCTCVolatility {
    col: String,
    horizon: usize,
    name: String,
}

impl FutureCTCVolatility {
    /// Create a new `FutureCTCVolatility` target.
    ///
    /// - `col` — price column name (e.g. `"close"`).
    /// - `horizon` — number of bars to look ahead.
    pub fn new(col: &str, horizon: usize) -> Self {
        let name = format!("{col}_future_ctc_vol_{horizon}");
        FutureCTCVolatility {
            col: col.to_string(),
            horizon,
            name,
        }
    }
}

impl Target for FutureCTCVolatility {
    fn compute(&self, columns: &[&[Option<f64>]]) -> Vec<Vec<Option<f64>>> {
        let prices = columns[0];
        let shifted_prices = shift(prices, 1);
        let lr = pairwise(prices, &shifted_prices, log_return);
        let vol = rolling(&lr, self.horizon, std_dev);
        let result = shift(&vol, -(self.horizon as isize));
        vec![result]
    }

    fn forward_period(&self) -> usize {
        self.horizon
    }

    fn names(&self) -> Vec<String> {
        vec![self.name.clone()]
    }

    fn required_columns(&self) -> Vec<String> {
        vec![self.col.clone()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_names() {
        let target = FutureCTCVolatility::new("close", 3);
        assert_eq!(target.names(), vec!["close_future_ctc_vol_3".to_string()]);
    }

    #[test]
    fn test_required_columns() {
        let target = FutureCTCVolatility::new("close", 3);
        assert_eq!(target.required_columns(), vec!["close".to_string()]);
    }

    #[test]
    fn test_forward_period() {
        let target = FutureCTCVolatility::new("close", 5);
        assert_eq!(target.forward_period(), 5);
    }

    #[test]
    fn test_compute_shape() {
        let target = FutureCTCVolatility::new("close", 3);
        let prices: Vec<Option<f64>> = vec![
            Some(100.0),
            Some(101.0),
            Some(103.0),
            Some(102.0),
            Some(105.0),
            Some(107.0),
            Some(106.0),
        ];

        let result = target.compute(&[&prices]);
        assert_eq!(result.len(), 1); // one output column
        assert_eq!(result[0].len(), prices.len()); // same length as input
    }

    #[test]
    fn test_compute_forward_none() {
        let target = FutureCTCVolatility::new("close", 3);
        let prices: Vec<Option<f64>> = vec![
            Some(100.0),
            Some(101.0),
            Some(103.0),
            Some(102.0),
            Some(105.0),
            Some(107.0),
            Some(106.0),
        ];

        let result = target.compute(&[&prices]);
        let col = &result[0];

        // Last `horizon` values must be None (future unknown).
        assert_eq!(col[4], None);
        assert_eq!(col[5], None);
        assert_eq!(col[6], None);
    }

    #[test]
    fn test_compute_valid_values() {
        let target = FutureCTCVolatility::new("close", 3);
        let prices: Vec<Option<f64>> = vec![
            Some(100.0),
            Some(101.0),
            Some(103.0),
            Some(102.0),
            Some(105.0),
            Some(107.0),
            Some(106.0),
        ];

        let result = target.compute(&[&prices]);
        let col = &result[0];

        // First valid value should be at index 0 (warm-up absorbed by shift).
        assert!(col[0].is_some());
        assert!(col[0].unwrap() > 0.0);
    }

    #[test]
    fn test_compute_stateless() {
        let target = FutureCTCVolatility::new("close", 3);
        let prices: Vec<Option<f64>> = vec![
            Some(100.0),
            Some(101.0),
            Some(103.0),
            Some(102.0),
            Some(105.0),
            Some(107.0),
            Some(106.0),
        ];

        let r1 = target.compute(&[&prices]);
        let r2 = target.compute(&[&prices]);
        assert_eq!(r1, r2); // same input → same output, no state
    }
}