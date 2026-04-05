use crate::error::OryonError;
use crate::ops::skewness;
use crate::traits::{Output, StreamingTransform};
use smallvec::smallvec;
use std::collections::VecDeque;

/// Rolling sample skewness (Fisher-Pearson corrected, same as pandas `.skew()`).
///
/// Computes `(n / ((n-1)(n-2))) * sum(((x - mean) / s)^3)` over the last `window` bars.
/// Returns `None` during warm-up (first `window - 1` bars) or if all values in the
/// window are equal (standard deviation = 0).
#[derive(Debug)]
pub struct Skewness {
    inputs: Vec<String>,
    window: usize,
    outputs: Vec<String>,
    buffer: VecDeque<Option<f64>>,
}

impl Skewness {
    /// Create a new `Skewness`.
    ///
    /// - `inputs` — name of the input column (e.g. `["close"]`).
    /// - `window` — number of bars. Must be >= 3 (skewness requires at least 3 values).
    /// - `outputs` — name of the output column (e.g. `["close_skewness_20"]`).
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
        if window < 3 {
            return Err(OryonError::InvalidInput {
                msg: "window must be >= 3".into(),
            });
        }
        Ok(Skewness {
            inputs,
            window,
            outputs,
            buffer: VecDeque::with_capacity(window),
        })
    }
}

impl StreamingTransform for Skewness {
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
            Skewness::new(self.inputs.clone(), self.window, self.outputs.clone())
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
            smallvec![skewness(slices)]
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
        Skewness::new(vec!["close".into()], 3, vec!["close_skewness_3".into()]).unwrap(),
        vec!["close".to_string()],
        vec!["close_skewness_3".to_string()],
        2,
        &[Some(1.0)],
    );

    fn skewness_3() -> Skewness {
        Skewness::new(vec!["close".into()], 3, vec!["close_skewness_3".into()]).unwrap()
    }

    fn out(v: Option<f64>) -> Output {
        smallvec![v]
    }

    #[test]
    fn test_update() {
        let mut sk = skewness_3();
        assert_eq!(sk.update(&[Some(1.0)]), out(None));
        assert_eq!(sk.update(&[Some(2.0)]), out(None));
        // Python: skewness([1,2,4]) = 0.935219529582824
        assert!((sk.update(&[Some(4.0)])[0].unwrap() - 0.935219529582824).abs() < 1e-10);
        // Python: skewness([2,4,6]) = 0.0 (symmetric)
        assert!(sk.update(&[Some(6.0)])[0].unwrap().abs() < 1e-10);
    }

    #[test]
    fn test_update_none_input() {
        let mut sk = skewness_3();
        assert_eq!(sk.update(&[Some(1.0)]), out(None));
        assert_eq!(sk.update(&[Some(2.0)]), out(None));
        assert!(sk.update(&[Some(4.0)])[0].is_some());
        assert_eq!(sk.update(&[None]), out(None));
        assert_eq!(sk.update(&[Some(5.0)]), out(None));
        assert_eq!(sk.update(&[Some(6.0)]), out(None));
        // window=[5,6,7] — all valid again
        assert!(sk.update(&[Some(7.0)])[0].is_some());
    }

    #[test]
    fn test_update_all_equal() {
        let mut sk = skewness_3();
        sk.update(&[Some(5.0)]);
        sk.update(&[Some(5.0)]);
        // std_dev = 0 → None
        assert_eq!(sk.update(&[Some(5.0)]), out(None));
    }

    #[test]
    fn test_reset_clears_buffer() {
        let mut sk = skewness_3();
        sk.update(&[Some(1.0)]);
        sk.reset();
        assert_eq!(sk.buffer.len(), 0);
        assert_eq!(sk.buffer.capacity(), sk.window);
    }

    #[test]
    fn test_fresh_instances_are_independent() {
        let mut original = skewness_3();
        original.update(&[Some(1.0)]);

        let mut fresh = original.fresh();

        // fresh starts from scratch
        assert_eq!(fresh.update(&[Some(1.0)]), out(None));
        assert_eq!(fresh.update(&[Some(2.0)]), out(None));
        assert!((fresh.update(&[Some(4.0)])[0].unwrap() - 0.935219529582824).abs() < 1e-10);

        // original continues from its own state
        assert_eq!(original.update(&[Some(2.0)]), out(None));
        assert!((original.update(&[Some(4.0)])[0].unwrap() - 0.935219529582824).abs() < 1e-10);
    }

    #[test]
    fn test_error_raises_when_empty_inputs() {
        let err = Skewness::new(vec![], 3, vec!["sk".into()]).unwrap_err();
        assert!(matches!(err, OryonError::InvalidInput { ref msg } if msg.contains("inputs")));
    }

    #[test]
    fn test_error_raises_when_empty_outputs() {
        let err = Skewness::new(vec!["close".into()], 3, vec![]).unwrap_err();
        assert!(matches!(err, OryonError::InvalidInput { ref msg } if msg.contains("outputs")));
    }

    #[test]
    fn test_error_raises_when_window_too_small() {
        let err = Skewness::new(vec!["close".into()], 2, vec!["sk".into()]).unwrap_err();
        assert!(matches!(err, OryonError::InvalidInput { ref msg } if msg.contains("window")));
    }
}
