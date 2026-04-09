use crate::error::OryonError;
use crate::ops::{kendall_correlation, pearson_correlation, spearman_correlation};
use crate::traits::{Output, StreamingTransform};
use smallvec::smallvec;
use std::collections::VecDeque;

/// Correlation method used by `Correlation`.
#[derive(Debug, Clone, Copy)]
pub enum CorrelationMethod {
    /// Pearson linear correlation. O(n) per bar. Suitable for live trading.
    Pearson,
    /// Spearman rank correlation. O(n log n) per bar. Suitable for live trading.
    Spearman,
    /// Kendall tau-b rank correlation. O(n^2) per bar. Prefer small windows
    /// (window <= 30) for live trading; use `Spearman` for larger windows.
    Kendall,
}

/// Rolling pairwise correlation between two input series over a sliding window.
///
/// Supports three methods: Pearson (linear), Spearman (rank, handles non-linearity),
/// and Kendall tau-b (rank, robust to outliers but O(n^2) per bar).
///
/// Output is `None` during warm-up (first `window - 1` bars), if any value in the
/// window is `None`, or if either series is constant over the window.
#[derive(Debug)]
pub struct Correlation {
    inputs: Vec<String>,
    window: usize,
    outputs: Vec<String>,
    method: CorrelationMethod,
    x_buffer: VecDeque<Option<f64>>,
    y_buffer: VecDeque<Option<f64>>,
}

impl Correlation {
    /// Create a new `Correlation`.
    ///
    /// - `inputs` - names of the two input columns `[x, y]`. Must contain at least 2 entries.
    /// - `window` - number of bars in the rolling window. Must be >= 2.
    /// - `outputs` - name of the single output column. Must contain exactly 1 entry.
    /// - `method` - correlation method: `Pearson`, `Spearman`, or `Kendall`.
    pub fn new(
        inputs: Vec<String>,
        window: usize,
        outputs: Vec<String>,
        method: CorrelationMethod,
    ) -> Result<Self, OryonError> {
        if inputs.len() < 2 {
            return Err(OryonError::InvalidInput {
                msg: "inputs must contain at least 2 column names".into(),
            });
        }
        if window < 2 {
            return Err(OryonError::InvalidInput {
                msg: "window must be >= 2".into(),
            });
        }
        if outputs.len() != 1 {
            return Err(OryonError::InvalidInput {
                msg: "outputs must contain exactly 1 name".into(),
            });
        }
        Ok(Correlation {
            inputs,
            window,
            outputs,
            method,
            x_buffer: VecDeque::with_capacity(window),
            y_buffer: VecDeque::with_capacity(window),
        })
    }
}

impl StreamingTransform for Correlation {
    fn input_names(&self) -> Vec<String> {
        self.inputs.clone()
    }

    fn output_names(&self) -> Vec<String> {
        self.outputs.clone()
    }

    fn warm_up_period(&self) -> usize {
        self.window - 1
    }

    fn fresh(&self) -> Box<dyn StreamingTransform> {
        Box::new(
            Correlation::new(
                self.inputs.clone(),
                self.window,
                self.outputs.clone(),
                self.method,
            )
            .expect("fresh: config was already validated at construction"),
        )
    }

    fn reset(&mut self) {
        self.x_buffer.clear();
        self.y_buffer.clear();
    }

    fn update(&mut self, state: &[Option<f64>]) -> Output {
        self.x_buffer.push_back(state[0]);
        self.y_buffer.push_back(state[1]);

        if self.x_buffer.len() > self.window {
            self.x_buffer.pop_front();
            self.y_buffer.pop_front();
        }

        if self.x_buffer.len() < self.window {
            return smallvec![None];
        }

        let x = self.x_buffer.make_contiguous();
        let y = self.y_buffer.make_contiguous();

        let result = match self.method {
            CorrelationMethod::Pearson => pearson_correlation(x, y),
            CorrelationMethod::Spearman => spearman_correlation(x, y),
            CorrelationMethod::Kendall => kendall_correlation(x, y),
        };

        smallvec![result]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::streaming_transform_contract_tests;

    streaming_transform_contract_tests!(
        Correlation::new(
            vec!["x".into(), "y".into()],
            3,
            vec!["xy_corr_3".into()],
            CorrelationMethod::Pearson,
        )
        .unwrap(),
        vec!["x".to_string(), "y".to_string()],
        vec!["xy_corr_3".to_string()],
        2,
        &[Some(1.0), Some(2.0)],
    );

    fn out(v: Option<f64>) -> Output {
        smallvec![v]
    }

    fn corr_pearson_3() -> Correlation {
        Correlation::new(
            vec!["x".into(), "y".into()],
            3,
            vec!["xy_corr_3".into()],
            CorrelationMethod::Pearson,
        )
        .unwrap()
    }

    fn corr_spearman_4() -> Correlation {
        Correlation::new(
            vec!["x".into(), "y".into()],
            4,
            vec!["xy_corr_4".into()],
            CorrelationMethod::Spearman,
        )
        .unwrap()
    }

    fn corr_kendall_4() -> Correlation {
        Correlation::new(
            vec!["x".into(), "y".into()],
            4,
            vec!["xy_corr_4".into()],
            CorrelationMethod::Kendall,
        )
        .unwrap()
    }

    #[test]
    fn test_update_pearson() {
        let mut c = corr_pearson_3();
        assert_eq!(c.update(&[Some(1.0), Some(1.0)]), out(None));
        assert_eq!(c.update(&[Some(2.0), Some(3.0)]), out(None));

        // x=[1,2,3], y=[1,3,2]: Sxy=1, Sxx=2, Syy=2 → r = 0.5
        let out = c.update(&[Some(3.0), Some(2.0)]);
        assert!((out[0].unwrap() - 0.5).abs() < 1e-10);

        // Perfect positive: y = x
        let mut c2 = corr_pearson_3();
        c2.update(&[Some(1.0), Some(1.0)]);
        c2.update(&[Some(2.0), Some(2.0)]);
        let out2 = c2.update(&[Some(3.0), Some(3.0)]);
        assert!((out2[0].unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_update_spearman() {
        let mut c = corr_spearman_4();
        c.update(&[Some(1.0), Some(1.0)]);
        c.update(&[Some(2.0), Some(3.0)]);
        c.update(&[Some(3.0), Some(2.0)]);

        // x=[1,2,3,4], y=[1,3,2,4]: Spearman = 0.8
        let out = c.update(&[Some(4.0), Some(4.0)]);
        assert!((out[0].unwrap() - 0.8).abs() < 1e-10);
    }

    #[test]
    fn test_update_kendall() {
        let mut c = corr_kendall_4();
        c.update(&[Some(1.0), Some(1.0)]);
        c.update(&[Some(2.0), Some(3.0)]);
        c.update(&[Some(3.0), Some(2.0)]);

        // x=[1,2,3,4], y=[1,3,2,4]: C=5, D=1, n0=6 → τ = 4/6 = 2/3
        let out = c.update(&[Some(4.0), Some(4.0)]);
        assert!((out[0].unwrap() - 2.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_update_none_input() {
        let mut c = corr_pearson_3();
        c.update(&[Some(1.0), Some(2.0)]);
        c.update(&[Some(2.0), Some(4.0)]);
        assert!(c.update(&[Some(3.0), Some(6.0)])[0].is_some());
        // None in x → output None.
        assert_eq!(c.update(&[None, Some(8.0)]), out(None));
        // None still in window for the next 2 bars.
        assert_eq!(c.update(&[Some(5.0), Some(10.0)]), out(None));
        assert_eq!(c.update(&[Some(6.0), Some(12.0)]), out(None));
        // None has been evicted — valid again.
        assert!(c.update(&[Some(7.0), Some(14.0)])[0].is_some());
    }

    #[test]
    fn test_update_constant_series_returns_none() {
        // y constant → sigma_y = 0 → None.
        let mut c = corr_pearson_3();
        c.update(&[Some(1.0), Some(5.0)]);
        c.update(&[Some(2.0), Some(5.0)]);
        assert_eq!(c.update(&[Some(3.0), Some(5.0)]), out(None));
    }

    #[test]
    fn test_reset_clears_buffers() {
        let mut c = corr_pearson_3();
        c.update(&[Some(1.0), Some(2.0)]);
        c.update(&[Some(2.0), Some(4.0)]);
        c.reset();
        assert_eq!(c.x_buffer.len(), 0);
        assert_eq!(c.y_buffer.len(), 0);
        assert_eq!(c.x_buffer.capacity(), c.window);
        assert_eq!(c.y_buffer.capacity(), c.window);
    }

    #[test]
    fn test_fresh_instances_are_independent() {
        let mut original = corr_pearson_3();
        original.update(&[Some(1.0), Some(1.0)]);

        let mut fresh = original.fresh();

        // fresh starts from scratch.
        assert_eq!(fresh.update(&[Some(1.0), Some(1.0)]), out(None));
        assert_eq!(fresh.update(&[Some(2.0), Some(3.0)]), out(None));
        assert!((fresh.update(&[Some(3.0), Some(2.0)])[0].unwrap() - 0.5).abs() < 1e-10);

        // original continues from its own state.
        assert_eq!(original.update(&[Some(2.0), Some(3.0)]), out(None));
        assert!((original.update(&[Some(3.0), Some(2.0)])[0].unwrap() - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_error_raises_when_window_is_zero() {
        assert!(matches!(
            Correlation::new(vec!["x".into(), "y".into()], 0, vec!["c".into()], CorrelationMethod::Pearson).unwrap_err(),
            OryonError::InvalidInput { ref msg } if msg.contains("window")
        ));
    }

    #[test]
    fn test_error_raises_when_window_is_one() {
        assert!(matches!(
            Correlation::new(vec!["x".into(), "y".into()], 1, vec!["c".into()], CorrelationMethod::Pearson).unwrap_err(),
            OryonError::InvalidInput { ref msg } if msg.contains("window")
        ));
    }

    #[test]
    fn test_error_raises_when_inputs_lt_2() {
        assert!(matches!(
            Correlation::new(vec!["x".into()], 3, vec!["c".into()], CorrelationMethod::Pearson).unwrap_err(),
            OryonError::InvalidInput { ref msg } if msg.contains("inputs")
        ));
    }

    #[test]
    fn test_error_raises_when_outputs_not_1() {
        assert!(matches!(
            Correlation::new(vec!["x".into(), "y".into()], 3, vec!["c1".into(), "c2".into()], CorrelationMethod::Pearson).unwrap_err(),
            OryonError::InvalidInput { ref msg } if msg.contains("outputs")
        ));
    }
}
