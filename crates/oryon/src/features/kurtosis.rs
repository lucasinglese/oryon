use crate::error::OryonError;
use crate::ops::kurtosis;
use crate::traits::{Feature, Output};
use smallvec::smallvec;
use std::collections::VecDeque;

/// Rolling excess kurtosis (Fisher, same as pandas `.kurt()`).
///
/// Computes `(n(n+1)/((n-1)(n-2)(n-3))) * sum(((x-mean)/s)^4) - 3(n-1)^2/((n-2)(n-3))`
/// over the last `window` bars. Returns `None` during warm-up (first `window - 1` bars)
/// or if all values in the window are equal (standard deviation = 0).
#[derive(Debug)]
pub struct Kurtosis {
    inputs: Vec<String>,
    window: usize,
    outputs: Vec<String>,
    buffer: VecDeque<Option<f64>>,
}

impl Kurtosis {
    /// Create a new `Kurtosis`.
    ///
    /// - `inputs` — name of the input column (e.g. `["close"]`).
    /// - `window` — number of bars. Must be >= 4 (kurtosis requires at least 4 values).
    /// - `outputs` — name of the output column (e.g. `["close_kurtosis_20"]`).
    pub fn new(inputs: Vec<String>, window: usize, outputs: Vec<String>) -> Result<Self, OryonError> {
        if inputs.is_empty() {
            return Err(OryonError::InvalidInput { msg: "inputs must not be empty".into() });
        }
        if outputs.is_empty() {
            return Err(OryonError::InvalidInput { msg: "outputs must not be empty".into() });
        }
        if window < 4 {
            return Err(OryonError::InvalidInput { msg: "window must be >= 4".into() });
        }
        Ok(Kurtosis {
            inputs,
            window,
            outputs,
            buffer: VecDeque::with_capacity(window),
        })
    }
}

impl Feature for Kurtosis {
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
            Kurtosis::new(self.inputs.clone(), self.window, self.outputs.clone())
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
            smallvec![kurtosis(slices)]
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
        Kurtosis::new(vec!["close".into()], 4, vec!["close_kurtosis_4".into()]).unwrap(),
        vec!["close".to_string()],
        vec!["close_kurtosis_4".to_string()],
        3,
        &[Some(1.0)],
    );

    fn kurtosis_4() -> Kurtosis {
        Kurtosis::new(vec!["close".into()], 4, vec!["close_kurtosis_4".into()]).unwrap()
    }

    fn out(v: Option<f64>) -> Output {
        smallvec![v]
    }

    #[test]
    fn test_update() {
        let mut k = kurtosis_4();
        assert_eq!(k.update(&[Some(1.0)]), out(None));
        assert_eq!(k.update(&[Some(2.0)]), out(None));
        assert_eq!(k.update(&[Some(4.0)]), out(None));
        // Python: kurtosis([1,2,4,8]) = 0.757655954631380
        assert!((k.update(&[Some(8.0)])[0].unwrap() - 0.757655954631380).abs() < 1e-10);
    }

    #[test]
    fn test_update_none_input() {
        let mut k = kurtosis_4();
        k.update(&[Some(1.0)]);
        k.update(&[Some(2.0)]);
        k.update(&[Some(4.0)]);
        assert!(k.update(&[Some(8.0)])[0].is_some());
        assert_eq!(k.update(&[None]), out(None));
        assert_eq!(k.update(&[Some(5.0)]), out(None));
        assert_eq!(k.update(&[Some(6.0)]), out(None));
        assert_eq!(k.update(&[Some(7.0)]), out(None));
        // None flushed out — window=[5,6,7,8] all valid
        assert!(k.update(&[Some(8.0)])[0].is_some());
    }

    #[test]
    fn test_update_all_equal() {
        let mut k = kurtosis_4();
        k.update(&[Some(5.0)]);
        k.update(&[Some(5.0)]);
        k.update(&[Some(5.0)]);
        assert_eq!(k.update(&[Some(5.0)]), out(None));
    }

    #[test]
    fn test_reset_clears_buffer() {
        let mut k = kurtosis_4();
        k.update(&[Some(1.0)]);
        k.reset();
        assert_eq!(k.buffer.len(), 0);
        assert_eq!(k.buffer.capacity(), k.window);
    }

    #[test]
    fn test_fresh_instances_are_independent() {
        let mut original = kurtosis_4();
        original.update(&[Some(1.0)]);

        let mut fresh = original.fresh();

        // fresh starts from scratch
        assert_eq!(fresh.update(&[Some(1.0)]), out(None));
        assert_eq!(fresh.update(&[Some(2.0)]), out(None));
        assert_eq!(fresh.update(&[Some(4.0)]), out(None));
        assert!((fresh.update(&[Some(8.0)])[0].unwrap() - 0.757655954631380).abs() < 1e-10);

        // original continues from its own state
        assert_eq!(original.update(&[Some(2.0)]), out(None));
        assert_eq!(original.update(&[Some(4.0)]), out(None));
        assert!((original.update(&[Some(8.0)])[0].unwrap() - 0.757655954631380).abs() < 1e-10);
    }

    #[test]
    fn test_error_raises_when_empty_inputs() {
        let err = Kurtosis::new(vec![], 4, vec!["k".into()]).unwrap_err();
        assert!(matches!(err, OryonError::InvalidInput { ref msg } if msg.contains("inputs")));
    }

    #[test]
    fn test_error_raises_when_empty_outputs() {
        let err = Kurtosis::new(vec!["close".into()], 4, vec![]).unwrap_err();
        assert!(matches!(err, OryonError::InvalidInput { ref msg } if msg.contains("outputs")));
    }

    #[test]
    fn test_error_raises_when_window_too_small() {
        let err = Kurtosis::new(vec!["close".into()], 3, vec!["k".into()]).unwrap_err();
        assert!(matches!(err, OryonError::InvalidInput { ref msg } if msg.contains("window")));
    }
}