use crate::error::OryonError;
use crate::features::correlation::CorrelationMethod;
use crate::ops::{kendall_correlation, pearson_correlation, spearman_correlation};
use crate::traits::{Output, StreamingTransform};
use smallvec::smallvec;
use std::collections::VecDeque;

/// Rolling autocorrelation of a single series at a fixed lag.
///
/// Computes the Pearson, Spearman, or Kendall correlation between the current
/// window `x[t-window+1..t]` and the lagged window `x[t-window+1-lag..t-lag]`.
/// This is equivalent to correlating a series with a time-shifted copy of itself.
///
/// Output is `None` during warm-up (first `window + lag - 1` bars), if any value
/// in the combined buffer is `None`, or if either sub-window is constant.
#[derive(Debug)]
pub struct AutoCorrelation {
    inputs: Vec<String>,
    window: usize,
    outputs: Vec<String>,
    lag: usize,
    method: CorrelationMethod,
    buffer: VecDeque<Option<f64>>,
}

impl AutoCorrelation {
    /// Create a new `AutoCorrelation`.
    ///
    /// - `inputs` - name of the single input column. Must contain exactly 1 entry.
    /// - `window` - number of bars in each correlation sub-window. Must be >= 2.
    /// - `outputs` - name of the single output column. Must contain exactly 1 entry.
    /// - `lag` - number of bars to shift the lagged window. Must be >= 1.
    /// - `method` - correlation method: `Pearson`, `Spearman`, or `Kendall`.
    pub fn new(
        inputs: Vec<String>,
        window: usize,
        outputs: Vec<String>,
        lag: usize,
        method: CorrelationMethod,
    ) -> Result<Self, OryonError> {
        if inputs.len() != 1 {
            return Err(OryonError::InvalidInput {
                msg: "inputs must contain exactly 1 column name".into(),
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
        if lag < 1 {
            return Err(OryonError::InvalidInput {
                msg: "lag must be >= 1".into(),
            });
        }
        Ok(AutoCorrelation {
            inputs,
            window,
            outputs,
            lag,
            method,
            buffer: VecDeque::with_capacity(window + lag),
        })
    }
}

impl StreamingTransform for AutoCorrelation {
    fn input_names(&self) -> Vec<String> {
        self.inputs.clone()
    }

    fn output_names(&self) -> Vec<String> {
        self.outputs.clone()
    }

    fn warm_up_period(&self) -> usize {
        self.window + self.lag - 1
    }

    fn fresh(&self) -> Box<dyn StreamingTransform> {
        Box::new(
            AutoCorrelation::new(
                self.inputs.clone(),
                self.window,
                self.outputs.clone(),
                self.lag,
                self.method,
            )
            .expect("fresh: config was already validated at construction"),
        )
    }

    fn reset(&mut self) {
        self.buffer.clear();
    }

    fn update(&mut self, state: &[Option<f64>]) -> Output {
        let capacity = self.window + self.lag;
        self.buffer.push_back(state[0]);

        if self.buffer.len() > capacity {
            self.buffer.pop_front();
        }

        if self.buffer.len() < capacity {
            return smallvec![None];
        }

        let buf = self.buffer.make_contiguous();
        // buf[0..window]           = lagged sub-window (older values)
        // buf[lag..window+lag]     = current sub-window (recent values)
        let lagged = &buf[..self.window];
        let recent = &buf[self.lag..];

        let result = match self.method {
            CorrelationMethod::Pearson => pearson_correlation(recent, lagged),
            CorrelationMethod::Spearman => spearman_correlation(recent, lagged),
            CorrelationMethod::Kendall => kendall_correlation(recent, lagged),
        };

        smallvec![result]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::streaming_transform_contract_tests;

    streaming_transform_contract_tests!(
        AutoCorrelation::new(
            vec!["x".into()],
            3,
            vec!["x_autocorr_3_1".into()],
            1,
            CorrelationMethod::Pearson,
        )
        .unwrap(),
        vec!["x".to_string()],
        vec!["x_autocorr_3_1".to_string()],
        3,
        &[Some(1.0)],
    );

    fn out(v: Option<f64>) -> Output {
        smallvec![v]
    }

    fn autocorr_pearson_w3_l1() -> AutoCorrelation {
        AutoCorrelation::new(
            vec!["x".into()],
            3,
            vec!["x_autocorr_3_1".into()],
            1,
            CorrelationMethod::Pearson,
        )
        .unwrap()
    }

    #[test]
    fn test_update_pearson_perfect_positive() {
        let mut c = autocorr_pearson_w3_l1();
        assert_eq!(c.update(&[Some(1.0)]), out(None));
        assert_eq!(c.update(&[Some(2.0)]), out(None));
        assert_eq!(c.update(&[Some(3.0)]), out(None));

        // buf=[1,2,3,4]: recent=[2,3,4], lagged=[1,2,3] — both monotone -> r=1.0
        let result = c.update(&[Some(4.0)]);
        assert!((result[0].unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_update_pearson_non_trivial() {
        let mut c = autocorr_pearson_w3_l1();
        c.update(&[Some(1.0)]);
        c.update(&[Some(3.0)]);
        c.update(&[Some(2.0)]);

        // buf=[1,3,2,4]: recent=[3,2,4], lagged=[1,3,2]
        // mean_recent=3, mean_lagged=2
        // dx=[0,-1,1], dy=[-1,1,0]  Sxy=-1, Sxx=2, Syy=2  r=-0.5
        let result = c.update(&[Some(4.0)]);
        assert!((result[0].unwrap() + 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_update_pearson_perfect_negative() {
        let mut c = autocorr_pearson_w3_l1();
        c.update(&[Some(1.0)]);
        c.update(&[Some(-1.0)]);
        c.update(&[Some(1.0)]);

        // buf=[1,-1,1,-1]: recent=[-1,1,-1], lagged=[1,-1,1]
        // Each pair is negated -> r=-1.0
        let result = c.update(&[Some(-1.0)]);
        assert!((result[0].unwrap() + 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_update_lag2() {
        let mut c = AutoCorrelation::new(
            vec!["x".into()],
            3,
            vec!["x_autocorr_3_2".into()],
            2,
            CorrelationMethod::Pearson,
        )
        .unwrap();
        // warm_up_period = 3+2-1 = 4
        assert_eq!(c.update(&[Some(1.0)]), out(None));
        assert_eq!(c.update(&[Some(2.0)]), out(None));
        assert_eq!(c.update(&[Some(3.0)]), out(None));
        assert_eq!(c.update(&[Some(4.0)]), out(None));

        // buf=[1,2,3,4,5]: recent=[3,4,5], lagged=[1,2,3] — both monotone -> r=1.0
        let result = c.update(&[Some(5.0)]);
        assert!((result[0].unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_update_none_input() {
        let mut c = autocorr_pearson_w3_l1();
        c.update(&[Some(1.0)]);
        c.update(&[Some(2.0)]);
        c.update(&[Some(3.0)]);
        assert!(c.update(&[Some(4.0)])[0].is_some());

        // None enters the buffer (capacity=4)
        assert_eq!(c.update(&[None]), out(None));
        // None still present in buffer for 3 more bars
        assert_eq!(c.update(&[Some(6.0)]), out(None));
        assert_eq!(c.update(&[Some(7.0)]), out(None));
        assert_eq!(c.update(&[Some(8.0)]), out(None));
        // None evicted after 4 consecutive valid bars
        assert!(c.update(&[Some(9.0)])[0].is_some());
    }

    #[test]
    fn test_update_constant_series_returns_none() {
        let mut c = autocorr_pearson_w3_l1();
        c.update(&[Some(5.0)]);
        c.update(&[Some(5.0)]);
        c.update(&[Some(5.0)]);
        // Both sub-windows are constant -> sigma=0 -> None
        assert_eq!(c.update(&[Some(5.0)]), out(None));
    }

    #[test]
    fn test_reset_clears_buffer() {
        let mut c = autocorr_pearson_w3_l1();
        c.update(&[Some(1.0)]);
        c.update(&[Some(2.0)]);
        c.reset();
        assert_eq!(c.buffer.len(), 0);
        assert_eq!(c.buffer.capacity(), c.window + c.lag);
    }

    #[test]
    fn test_fresh_instances_are_independent() {
        let mut original = autocorr_pearson_w3_l1();
        original.update(&[Some(1.0)]);
        original.update(&[Some(2.0)]);

        let mut fresh = original.fresh();

        // fresh starts from scratch
        assert_eq!(fresh.update(&[Some(1.0)]), out(None));
        assert_eq!(fresh.update(&[Some(2.0)]), out(None));
        assert_eq!(fresh.update(&[Some(3.0)]), out(None));
        assert!((fresh.update(&[Some(4.0)])[0].unwrap() - 1.0).abs() < 1e-10);

        // original continues from its own state
        assert_eq!(original.update(&[Some(3.0)]), out(None));
        assert!((original.update(&[Some(4.0)])[0].unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_error_raises_when_window_is_zero() {
        assert!(matches!(
            AutoCorrelation::new(vec!["x".into()], 0, vec!["c".into()], 1, CorrelationMethod::Pearson).unwrap_err(),
            OryonError::InvalidInput { ref msg } if msg.contains("window")
        ));
    }

    #[test]
    fn test_error_raises_when_window_is_one() {
        assert!(matches!(
            AutoCorrelation::new(vec!["x".into()], 1, vec!["c".into()], 1, CorrelationMethod::Pearson).unwrap_err(),
            OryonError::InvalidInput { ref msg } if msg.contains("window")
        ));
    }

    #[test]
    fn test_error_raises_when_lag_is_zero() {
        assert!(matches!(
            AutoCorrelation::new(vec!["x".into()], 3, vec!["c".into()], 0, CorrelationMethod::Pearson).unwrap_err(),
            OryonError::InvalidInput { ref msg } if msg.contains("lag")
        ));
    }

    #[test]
    fn test_error_raises_when_inputs_not_1() {
        assert!(matches!(
            AutoCorrelation::new(vec!["x".into(), "y".into()], 3, vec!["c".into()], 1, CorrelationMethod::Pearson).unwrap_err(),
            OryonError::InvalidInput { ref msg } if msg.contains("inputs")
        ));
    }

    #[test]
    fn test_error_raises_when_outputs_not_1() {
        assert!(matches!(
            AutoCorrelation::new(vec!["x".into()], 3, vec!["c1".into(), "c2".into()], 1, CorrelationMethod::Pearson).unwrap_err(),
            OryonError::InvalidInput { ref msg } if msg.contains("outputs")
        ));
    }
}
