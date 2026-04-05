use crate::error::OryonError;
use crate::ops::simple_return;
use crate::{Output, StreamingTransform};
use smallvec::smallvec;
use std::collections::VecDeque;

/// Simple (arithmetic) return over a configurable lookback window.
///
/// Computes `(P_t - P_{t-n}) / P_{t-n}` where `n` is the window. Returns `None`
/// during warm-up (first `window` bars) or if the previous price is <= 0.
/// Unlike log return, a negative current price yields a valid negative return.
#[derive(Debug)]
pub struct SimpleReturn {
    inputs: Vec<String>,
    window: usize,
    outputs: Vec<String>,
    buffer: VecDeque<Option<f64>>,
}

impl SimpleReturn {
    /// Create a new `SimpleReturn`.
    ///
    /// - `inputs` — name of the input column (e.g. `["close"]`).
    /// - `window` — lookback in bars. Must be > 0.
    /// - `outputs` — name of the output column (e.g. `["close_simple_return_5"]`).
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
        Ok(SimpleReturn {
            inputs,
            window,
            outputs,
            buffer: VecDeque::with_capacity(window + 1),
        })
    }
}

impl StreamingTransform for SimpleReturn {
    fn input_names(&self) -> Vec<String> {
        self.inputs.clone()
    }

    fn output_names(&self) -> Vec<String> {
        self.outputs.clone()
    }

    fn warm_up_period(&self) -> usize {
        self.window
    }

    fn fresh(&self) -> Box<dyn StreamingTransform> {
        Box::new(
            SimpleReturn::new(self.inputs.clone(), self.window, self.outputs.clone())
                .expect("fresh: config was already validated at construction"),
        )
    }

    fn reset(&mut self) {
        self.buffer.clear();
    }

    fn update(&mut self, state: &[Option<f64>]) -> Output {
        self.buffer.push_back(state[0]);

        if self.buffer.len() > self.window + 1 {
            self.buffer.pop_front();
        }

        if self.buffer.len() == self.window + 1 {
            let prev = self.buffer[0];
            let next = self.buffer[self.window];
            smallvec![simple_return(&[prev, next])]
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
        SimpleReturn::new(
            vec!["close".to_string()],
            2,
            vec!["close_simple_return_2".to_string()]
        )
        .unwrap(),
        vec!["close".to_string()],
        vec!["close_simple_return_2".to_string()],
        2,
        &[Some(0.1)],
    );

    fn simple_return_2() -> SimpleReturn {
        SimpleReturn::new(
            vec!["close".to_string()],
            2,
            vec!["close_simple_return_2".to_string()],
        )
        .unwrap()
    }

    fn out(v: Option<f64>) -> Output {
        smallvec![v]
    }

    #[test]
    fn test_update() {
        let mut sr = simple_return_2();
        assert_eq!(sr.update(&[Some(100.0)]), out(None));
        assert_eq!(sr.update(&[Some(100.0)]), out(None));
        assert!((sr.update(&[Some(110.0)])[0].unwrap() - 0.1).abs() < 1e-10);
        assert!((sr.update(&[Some(120.0)])[0].unwrap() - 0.2).abs() < 1e-10);
    }

    #[test]
    fn test_update_negative_current() {
        let mut sr = SimpleReturn::new(
            vec!["close".to_string()],
            1,
            vec!["close_simple_return_1".to_string()],
        )
        .unwrap();
        assert_eq!(sr.update(&[Some(100.0)]), out(None));
        assert!((sr.update(&[Some(90.0)])[0].unwrap() - (-0.1)).abs() < 1e-10);
    }

    #[test]
    fn test_window_size_is_one() {
        let mut sr = SimpleReturn::new(
            vec!["close".to_string()],
            1,
            vec!["close_simple_return_1".to_string()],
        )
        .unwrap();
        assert_eq!(sr.update(&[Some(100.0)]), out(None));
        assert!((sr.update(&[Some(110.0)])[0].unwrap() - 0.1).abs() < 1e-10);
    }

    #[test]
    fn test_reset_clears_buffer() {
        let mut sr = simple_return_2();
        sr.update(&[Some(100.0)]);
        sr.reset();
        assert_eq!(sr.buffer.len(), 0);
    }

    #[test]
    fn test_buffer_capacity() {
        let sr = simple_return_2();
        assert_eq!(sr.buffer.capacity(), sr.window + 1);
    }

    #[test]
    fn test_fresh_instances_are_independent() {
        let mut original = simple_return_2();
        original.update(&[Some(100.0)]);

        let mut fresh = original.fresh();

        // fresh starts from scratch
        assert_eq!(fresh.update(&[Some(100.0)]), out(None));
        assert_eq!(fresh.update(&[Some(100.0)]), out(None));
        assert!((fresh.update(&[Some(110.0)])[0].unwrap() - 0.1).abs() < 1e-10);

        // original continues from its own state
        assert_eq!(original.update(&[Some(100.0)]), out(None));
        assert!((original.update(&[Some(110.0)])[0].unwrap() - 0.1).abs() < 1e-10);
    }

    #[test]
    fn test_error_raises_when_empty_inputs() {
        let err = SimpleReturn::new(vec![], 1, vec!["sr".into()]).unwrap_err();
        assert!(matches!(err, OryonError::InvalidInput { ref msg } if msg.contains("inputs")));
    }

    #[test]
    fn test_error_raises_when_empty_outputs() {
        let err = SimpleReturn::new(vec!["close".into()], 1, vec![]).unwrap_err();
        assert!(matches!(err, OryonError::InvalidInput { ref msg } if msg.contains("outputs")));
    }

    #[test]
    fn test_error_raises_when_window_is_zero() {
        let err = SimpleReturn::new(vec!["close".into()], 0, vec!["sr".into()]).unwrap_err();
        assert!(matches!(err, OryonError::InvalidInput { ref msg } if msg.contains("window")));
    }
}
