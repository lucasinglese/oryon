use crate::error::OryonError;
use crate::ops::median;
use crate::traits::{Output, StreamingTransform};
use smallvec::smallvec;
use std::collections::VecDeque;

/// Median Moving Average over a rolling window.
///
/// Returns `None` during warm-up (first `window - 1` bars).
/// A `None` input within the window propagates as `None` output
/// until the window is fully refilled with valid values.
#[derive(Debug)]
pub struct Mma {
    inputs: Vec<String>,
    window: usize,
    outputs: Vec<String>,
    buffer: VecDeque<Option<f64>>,
}

impl Mma {
    /// Create a new `Mma`.
    ///
    /// - `inputs` - name of the input column (e.g. `["close"]`).
    /// - `window` - number of bars for the rolling median. Must be > 0.
    /// - `outputs` - name of the output column (e.g. `["close_mma_20"]`).
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

        Ok(Mma {
            inputs,
            window,
            outputs,
            buffer: VecDeque::with_capacity(window),
        })
    }
}

impl StreamingTransform for Mma {
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
            Mma::new(self.inputs.clone(), self.window, self.outputs.clone())
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
            smallvec![median(slices)]
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
        Mma::new(vec!["close".into()], 3, vec!["close_mma_3".into()]).unwrap(),
        vec!["close".to_string()],
        vec!["close_mma_3".to_string()],
        2,
        &[Some(1.0)],
    );

    fn mma3() -> Mma {
        Mma::new(vec!["close".into()], 3, vec!["close_mma_3".into()]).unwrap()
    }

    fn out(v: Option<f64>) -> Output {
        smallvec![v]
    }

    #[test]
    fn test_update() {
        let mut mma = mma3();
        assert_eq!(mma.update(&[Some(1.0)]), out(None));
        assert_eq!(mma.update(&[Some(3.0)]), out(None));
        assert_eq!(mma.update(&[Some(2.0)]), out(Some(2.0)));
        assert_eq!(mma.update(&[Some(2.0)]), out(Some(2.0)));
        assert_eq!(mma.update(&[Some(5.0)]), out(Some(2.0)));
        assert_eq!(mma.update(&[Some(3.0)]), out(Some(3.0)));
    }

    #[test]
    fn test_update_none_input() {
        let mut mma = mma3();
        assert_eq!(mma.update(&[Some(1.0)]), out(None));
        assert_eq!(mma.update(&[Some(3.0)]), out(None));
        assert_eq!(mma.update(&[Some(2.0)]), out(Some(2.0)));
        assert_eq!(mma.update(&[None]), out(None));
        assert_eq!(mma.update(&[Some(5.0)]), out(None));
        assert_eq!(mma.update(&[Some(3.0)]), out(None));
        assert_eq!(mma.update(&[Some(2.0)]), out(Some(3.0)));
    }

    #[test]
    fn test_window_size_is_one() {
        let mut mma = Mma::new(vec!["close".into()], 1, vec!["close_mma_1".into()]).unwrap();
        assert_eq!(mma.update(&[Some(1.0)]), out(Some(1.0)));
        assert_eq!(mma.update(&[Some(2.0)]), out(Some(2.0)));
    }

    #[test]
    fn test_reset_preserves_capacity() {
        let mut mma = mma3();
        mma.update(&[Some(1.0)]);
        mma.reset();
        assert_eq!(mma.buffer.len(), 0);
        assert_eq!(mma.buffer.capacity(), mma.window);
    }

    #[test]
    fn test_fresh_instances_are_independent() {
        let mut mma = mma3();
        mma.update(&[Some(1.0)]);

        let mut fresh = mma.fresh();

        // fresh computes correctly from scratch
        assert_eq!(fresh.update(&[Some(1.0)]), out(None));
        assert_eq!(fresh.update(&[Some(2.0)]), out(None));
        assert_eq!(fresh.update(&[Some(3.0)]), out(Some(2.0)));

        // original continues its own state independently
        assert_eq!(mma.update(&[Some(2.0)]), out(None));
        assert_eq!(mma.update(&[Some(3.0)]), out(Some(2.0)));
    }

    #[test]
    fn test_error_raises_when_empty_inputs() {
        let err = Mma::new(vec![], 1, vec!["close_mma_1".into()]).unwrap_err();
        assert!(matches!(err, OryonError::InvalidInput {ref msg} if msg.contains("inputs")));
    }

    #[test]
    fn test_error_raises_when_empty_outputs() {
        let err = Mma::new(vec!["close".into()], 1, vec![]).unwrap_err();
        assert!(matches!(err, OryonError::InvalidInput {ref msg} if msg.contains("outputs")));
    }

    #[test]
    fn test_error_raises_when_window_is_zero() {
        let err = Mma::new(vec!["close".into()], 0, vec!["close_mma_1".into()]).unwrap_err();
        assert!(matches!(err, OryonError::InvalidInput {ref msg} if msg.contains("window")));
    }
}
