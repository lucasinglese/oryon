use crate::error::OryonError;
use crate::ops::average;
use crate::traits::{Feature, Output};
use smallvec::smallvec;
use std::collections::VecDeque;

/// Simple Moving Average over a rolling window.
///
/// Returns `None` during warm-up (first `window - 1` bars).
/// A `None` input within the window propagates as `None` output
/// until the window is fully refilled with valid values.
#[derive(Debug)]
pub struct Sma {
    inputs: Vec<String>,
    window: usize,
    outputs: Vec<String>,
    buffer: VecDeque<Option<f64>>,
}

impl Sma {
    /// Create a new `Sma`.
    ///
    /// - `inputs` — name of the input column (e.g. `["close"]`).
    /// - `window` — number of bars to average. Must be > 0.
    /// - `outputs` — name of the output column (e.g. `["close_sma_3"]`).
    pub fn new(
        inputs: Vec<String>,
        window: usize,
        outputs: Vec<String>,
    ) -> Result<Self, OryonError> {
        if inputs.is_empty() {
            return Err(OryonError::InvalidInput {
                msg: "inputs must not be empty".into(),
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

        Ok(Sma {
            inputs,
            window,
            outputs,
            buffer: VecDeque::with_capacity(window),
        })
    }
}

impl Feature for Sma {
    fn input_names(&self) -> Vec<String> {
        self.inputs.clone()
    }

    fn output_names(&self) -> Vec<String> {
        self.outputs.clone()
    }

    fn warm_up_period(&self) -> usize {
        self.window - 1
    }

    fn fresh(&self) -> Box<dyn Feature> {
        Box::new(
            Sma::new(self.inputs.clone(), self.window, self.outputs.clone())
                .expect("fresh: config was already validated at construction"),
        )
    }

    fn reset(&mut self) {
        self.buffer.clear();
    }

    fn update(&mut self, state: &[Option<f64>]) -> Output {
        self.buffer.push_back(state[0]);

        if self.buffer.len() > self.window {
            self.buffer.pop_front();
        }

        if self.buffer.len() == self.window {
            let slices = self.buffer.make_contiguous();
            smallvec![average(slices)]
        } else {
            smallvec![None]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature_contract_tests;
    use smallvec::smallvec;

    feature_contract_tests!(
        Sma::new(vec!["close".into()], 3, vec!["close_sma_3".into()]).unwrap(),
        vec!["close".to_string()],
        vec!["close_sma_3".to_string()],
        2,
        &[Some(1.0)],
    );

    fn sma3() -> Sma {
        Sma::new(vec!["close".into()], 3, vec!["close_sma_3".into()]).unwrap()
    }

    fn out(v: Option<f64>) -> Output {
        smallvec![v]
    }

    #[test]
    fn test_update() {
        let mut sma = sma3();
        assert_eq!(sma.update(&[Some(1.0)]), out(None));
        assert_eq!(sma.update(&[Some(2.0)]), out(None));
        assert_eq!(sma.update(&[Some(3.0)]), out(Some(2.0)));
        assert_eq!(sma.update(&[Some(4.0)]), out(Some(3.0)));
        assert_eq!(sma.update(&[Some(5.0)]), out(Some(4.0)));
    }

    #[test]
    fn test_update_none_input() {
        let mut sma = sma3();
        assert_eq!(sma.update(&[Some(1.0)]), out(None));
        assert_eq!(sma.update(&[Some(2.0)]), out(None));
        assert_eq!(sma.update(&[Some(3.0)]), out(Some(2.0)));
        assert_eq!(sma.update(&[None]), out(None));
        assert_eq!(sma.update(&[Some(4.0)]), out(None));
        assert_eq!(sma.update(&[Some(5.0)]), out(None));
        assert_eq!(sma.update(&[Some(6.0)]), out(Some(5.0)));
    }

    #[test]
    fn test_window_size_is_one() {
        let mut sma = Sma::new(vec!["close".into()], 1, vec!["close_sma_1".into()]).unwrap();
        assert_eq!(sma.update(&[Some(1.0)]), out(Some(1.0)));
        assert_eq!(sma.update(&[Some(2.0)]), out(Some(2.0)));
    }

    #[test]
    fn test_reset_preserves_capacity() {
        let mut sma = sma3();
        sma.update(&[Some(1.0)]);
        sma.reset();
        assert_eq!(sma.buffer.len(), 0);
        assert_eq!(sma.buffer.capacity(), sma.window);
    }

    #[test]
    fn test_fresh_instances_are_independent() {
        let mut sma = sma3();
        sma.update(&[Some(1.0)]);

        let mut fresh = sma.fresh();

        // fresh computes correctly from scratch
        assert_eq!(fresh.update(&[Some(1.0)]), out(None));
        assert_eq!(fresh.update(&[Some(2.0)]), out(None));
        assert_eq!(fresh.update(&[Some(3.0)]), out(Some(2.0)));

        // original continues its own state independently
        assert_eq!(sma.update(&[Some(2.0)]), out(None));
        assert_eq!(sma.update(&[Some(3.0)]), out(Some(2.0)));
    }

    #[test]
    fn test_error_raises_when_empty_inputs() {
        let err = Sma::new(vec![], 1, vec!["close_sma_1".into()]).unwrap_err();
        assert!(matches!(err, OryonError::InvalidInput {ref msg} if msg.contains("inputs")));
    }

    #[test]
    fn test_error_raises_when_empty_outputs() {
        let err = Sma::new(vec!["close".into()], 1, vec![]).unwrap_err();
        assert!(matches!(err, OryonError::InvalidInput {ref msg} if msg.contains("outputs")));
    }

    #[test]
    fn test_error_raises_when_window_is_zero() {
        let err = Sma::new(vec!["close".into()], 0, vec!["close_sma_1".into()]).unwrap_err();
        assert!(matches!(err, OryonError::InvalidInput {ref msg} if msg.contains("window")));
    }
}
