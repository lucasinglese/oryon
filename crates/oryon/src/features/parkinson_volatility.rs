use crate::error::OryonError;
use crate::ops::{average, parkinson_log_hl_sq};
use crate::traits::{Output, StreamingTransform};
use smallvec::smallvec;
use std::collections::VecDeque;

/// Rolling Parkinson volatility estimator.
///
/// Uses high and low prices to estimate realized volatility -> more efficient
/// than close-to-close for assets with significant intraday movement.
/// Assumes Brownian motion without drift (Parkinson, 1980).
///
/// ```text
/// σ_P = sqrt( (1 / (4N·ln(2))) · Σ ln(H_i / L_i)² )
///     = sqrt( mean(ln(H_i / L_i)²) / (4·ln(2)) )
/// ```
///
/// Returns `None` during warm-up (first `window - 1` bars) or if any
/// high/low pair in the window is invalid (None, zero, or high < low).
#[derive(Debug)]
pub struct ParkinsonVolatility {
    inputs: Vec<String>,
    window: usize,
    outputs: Vec<String>,
    buffer: VecDeque<Option<f64>>,
}

impl ParkinsonVolatility {
    /// Create a new `ParkinsonVolatility`.
    ///
    /// - `inputs`  - names of the high and low columns, in that order
    ///   (e.g. `["high", "low"]`).
    /// - `window`  - number of bars in the rolling window. Must be > 0.
    /// - `outputs` - name of the output column (e.g. `["parkinson_vol_20"]`).
    pub fn new(
        inputs: Vec<String>,
        window: usize,
        outputs: Vec<String>,
    ) -> Result<Self, OryonError> {
        if inputs.len() < 2 {
            return Err(OryonError::InvalidInput {
                msg: "inputs must contain high and low columns".into(),
            });
        }
        if outputs.is_empty() {
            return Err(OryonError::InvalidInput {
                msg: "outputs must not be empty".into(),
            });
        }
        if window == 0 {
            return Err(OryonError::InvalidInput {
                msg: "window must be non-zero".into(),
            });
        }
        Ok(ParkinsonVolatility {
            inputs,
            window,
            outputs,
            buffer: VecDeque::with_capacity(window),
        })
    }
}

impl StreamingTransform for ParkinsonVolatility {
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
            ParkinsonVolatility::new(self.inputs.clone(), self.window, self.outputs.clone())
                .expect("fresh: config was already validated at construction"),
        )
    }

    fn reset(&mut self) {
        self.buffer.clear();
    }

    fn update(&mut self, state: &[Option<f64>]) -> Output {
        let sq = parkinson_log_hl_sq(&[state[0], state[1]]);
        self.buffer.push_back(sq);

        if self.buffer.len() > self.window {
            self.buffer.pop_front();
        }

        if self.buffer.len() == self.window {
            let slices = self.buffer.make_contiguous();
            match average(slices) {
                Some(avg) => smallvec![Some((avg / (4.0 * std::f64::consts::LN_2)).sqrt())],
                None => smallvec![None],
            }
        } else {
            smallvec![None]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::streaming_transform_contract_tests;
    use smallvec::smallvec;

    streaming_transform_contract_tests!(
        ParkinsonVolatility::new(
            vec!["high".into(), "low".into()],
            3,
            vec!["parkinson_vol_3".into()],
        )
        .unwrap(),
        vec!["high".to_string(), "low".to_string()],
        vec!["parkinson_vol_3".to_string()],
        2,
        &[Some(102.0), Some(99.0)],
    );

    fn pv_3() -> ParkinsonVolatility {
        ParkinsonVolatility::new(
            vec!["high".into(), "low".into()],
            3,
            vec!["parkinson_vol_3".into()],
        )
        .unwrap()
    }

    fn out(v: Option<f64>) -> Output {
        smallvec![v]
    }

    #[test]
    fn test_update() {
        // Python reference values (window=3)
        let mut pv = pv_3();
        assert_eq!(pv.update(&[Some(102.0), Some(99.0)]), out(None));
        assert_eq!(pv.update(&[Some(104.0), Some(101.0)]), out(None));
        assert!(
            (pv.update(&[Some(103.0), Some(100.0)])[0].unwrap() - 0.017_753_593_761_606).abs()
                < 1e-12
        );
        assert!(
            (pv.update(&[Some(106.0), Some(103.0)])[0].unwrap() - 0.017_525_511_477_570).abs()
                < 1e-12
        );
        assert!(
            (pv.update(&[Some(108.0), Some(105.0)])[0].unwrap() - 0.017_307_528_183_468).abs()
                < 1e-12
        );
    }

    #[test]
    fn test_update_invalid_bar() {
        let mut pv = pv_3();
        pv.update(&[Some(102.0), Some(99.0)]);
        pv.update(&[Some(104.0), Some(101.0)]);
        assert!(pv.update(&[Some(103.0), Some(100.0)])[0].is_some());
        // high < low → invalid bar → None
        assert_eq!(pv.update(&[Some(99.0), Some(105.0)]), out(None));
    }

    #[test]
    fn test_reset_clears_buffer() {
        let mut pv = pv_3();
        pv.update(&[Some(102.0), Some(99.0)]);
        pv.reset();
        assert_eq!(pv.buffer.len(), 0);
        assert_eq!(pv.buffer.capacity(), pv.window);
    }

    #[test]
    fn test_fresh_instances_are_independent() {
        let mut original = pv_3();
        original.update(&[Some(102.0), Some(99.0)]);

        let mut fresh = original.fresh();

        // fresh starts from scratch
        assert_eq!(fresh.update(&[Some(102.0), Some(99.0)]), out(None));
        assert_eq!(fresh.update(&[Some(104.0), Some(101.0)]), out(None));
        assert!(
            (fresh.update(&[Some(103.0), Some(100.0)])[0].unwrap() - 0.017_753_593_761_606).abs()
                < 1e-12
        );

        // original continues from its own state
        assert_eq!(original.update(&[Some(104.0), Some(101.0)]), out(None));
        assert!(
            (original.update(&[Some(103.0), Some(100.0)])[0].unwrap() - 0.017_753_593_761_606)
                .abs()
                < 1e-12
        );
    }

    #[test]
    fn test_error_raises_when_inputs_too_few() {
        let err = ParkinsonVolatility::new(vec!["high".into()], 3, vec!["pv".into()]).unwrap_err();
        assert!(matches!(err, OryonError::InvalidInput { ref msg } if msg.contains("inputs")));
    }

    #[test]
    fn test_error_raises_when_empty_outputs() {
        let err =
            ParkinsonVolatility::new(vec!["high".into(), "low".into()], 3, vec![]).unwrap_err();
        assert!(matches!(err, OryonError::InvalidInput { ref msg } if msg.contains("outputs")));
    }

    #[test]
    fn test_error_raises_when_window_is_zero() {
        let err = ParkinsonVolatility::new(vec!["high".into(), "low".into()], 0, vec!["pv".into()])
            .unwrap_err();
        assert!(matches!(err, OryonError::InvalidInput { ref msg } if msg.contains("window")));
    }
}
