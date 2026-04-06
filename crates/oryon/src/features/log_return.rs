use crate::error::OryonError;
use crate::ops::log_return;
use crate::{Output, StreamingTransform};
use smallvec::smallvec;
use std::collections::VecDeque;

/// Log return over a configurable lookback window.
///
/// Computes `ln(P_t / P_{t-n})` where `n` is the window. Returns `None`
/// during warm-up (first `window` bars). Only the two endpoints of the window
/// matter: a `None` at the current bar or at the lookback bar returns `None`,
/// but a `None` at an intermediate position does not affect the output.
#[derive(Debug)]
pub struct LogReturn {
    inputs: Vec<String>,
    window: usize,
    outputs: Vec<String>,
    buffer: VecDeque<Option<f64>>,
}

impl LogReturn {
    /// Create a new `LogReturn`.
    ///
    /// - `inputs` — name of the input column (e.g. `["close"]`).
    /// - `window` — lookback in bars. Must be > 0.
    /// - `outputs` — name of the output column (e.g. `["close_log_return_5"]`).
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

        Ok(LogReturn {
            inputs,
            window,
            outputs,
            buffer: VecDeque::with_capacity(window + 1),
        })
    }
}

impl StreamingTransform for LogReturn {
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
            LogReturn::new(self.inputs.clone(), self.window, self.outputs.clone())
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
            smallvec![log_return(&[prev, next])]
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
        LogReturn::new(
            vec!["close".to_string()],
            2,
            vec!["close_log_return_2".to_string()]
        )
        .unwrap(),
        vec!["close".to_string()],
        vec!["close_log_return_2".to_string()],
        2,
        &[Some(1.0)],
    );

    fn log_return_2() -> LogReturn {
        LogReturn::new(
            vec!["close".to_string()],
            2,
            vec!["close_log_return_2".to_string()],
        )
        .unwrap()
    }

    fn out(v: Option<f64>) -> Output {
        smallvec![v]
    }

    #[test]
    fn test_update() {
        let mut log_return = log_return_2();
        assert_eq!(log_return.update(&[Some(1.0_f64.exp())]), out(None));
        assert_eq!(log_return.update(&[Some(1.0_f64.exp())]), out(None));
        assert!((log_return.update(&[Some(1.0_f64.exp())])[0].unwrap() - 0.0).abs() < 1e-10);
        assert!((log_return.update(&[Some(1.1_f64.exp())])[0].unwrap() - 0.1).abs() < 1e-10);
        assert!((log_return.update(&[Some(1.2_f64.exp())])[0].unwrap() - 0.2).abs() < 1e-10);
    }

    #[test]
    fn test_update_none_input() {
        let mut lr = log_return_2(); // window = 2, buffer size = 3
        assert_eq!(lr.update(&[Some(1.0_f64.exp())]), out(None)); // bar 1
        assert_eq!(lr.update(&[Some(1.0_f64.exp())]), out(None)); // bar 2
                                                                  // None at current (end position) → None
        assert_eq!(lr.update(&[None]), out(None)); // bar 3: buffer=[e, e, None]
                                                   // None now in middle position → valid (only endpoints matter)
        assert!((lr.update(&[Some(1.2_f64.exp())])[0].unwrap() - 0.2).abs() < 1e-10); // bar 4: buffer=[e, None, e^1.2]
                                                                                      // None at lookback (start position) → None
        assert_eq!(lr.update(&[Some(1.3_f64.exp())]), out(None)); // bar 5: buffer=[None, e^1.2, e^1.3]
                                                                  // None flushed — both endpoints valid again
        assert!((lr.update(&[Some(1.4_f64.exp())])[0].unwrap() - 0.2).abs() < 1e-10);
        // bar 6: buffer=[e^1.2, e^1.3, e^1.4]
    }

    #[test]
    fn test_window_size_is_one() {
        let mut log_return = LogReturn::new(
            vec!["close".to_string()],
            1,
            vec!["close_log_return_1".to_string()],
        )
        .unwrap();
        assert_eq!(log_return.update(&[Some(1.0_f64.exp())]), out(None));
        assert!((log_return.update(&[Some(1.1_f64.exp())])[0].unwrap() - 0.1).abs() < 1e-10);
    }

    #[test]
    fn test_reset_preserves_capacity() {
        let mut log_return = log_return_2();
        log_return.update(&[Some(1.0)]);
        log_return.reset();
        assert_eq!(log_return.buffer.len(), 0);
    }

    #[test]
    fn test_buffer_capacity() {
        let log_return = log_return_2();
        assert_eq!(log_return.buffer.capacity(), log_return.window + 1);
    }

    #[test]
    fn test_fresh_instances_are_independent() {
        let mut log_return = log_return_2();
        log_return.update(&[Some(1.0_f64.exp())]);

        let mut fresh = log_return.fresh();

        // fresh computes correctly from scratch
        assert_eq!(fresh.update(&[Some(1.0_f64.exp())]), out(None));
        assert_eq!(fresh.update(&[Some(1.0_f64.exp())]), out(None));
        assert!((fresh.update(&[Some(1.2_f64.exp())])[0].unwrap() - 0.2).abs() < 1e-10);

        // original continues its own state independently
        assert_eq!(log_return.update(&[Some(1.0_f64.exp())]), out(None));
        assert!((log_return.update(&[Some(1.2_f64.exp())])[0].unwrap() - 0.2).abs() < 1e-10);
    }

    #[test]
    fn test_error_raises_when_empty_inputs() {
        let err = LogReturn::new(vec![], 1, vec!["log_return_1".into()]).unwrap_err();
        assert!(matches!(err, OryonError::InvalidInput {ref msg} if msg.contains("inputs")));
    }

    #[test]
    fn test_error_raises_when_empty_outputs() {
        let err = LogReturn::new(vec!["close".into()], 1, vec![]).unwrap_err();
        assert!(matches!(err, OryonError::InvalidInput {ref msg} if msg.contains("outputs")));
    }

    #[test]
    fn test_error_raises_when_window_is_zero() {
        let err = LogReturn::new(vec!["close".into()], 0, vec!["log_return_1".into()]).unwrap_err();
        assert!(matches!(err, OryonError::InvalidInput {ref msg} if msg.contains("window")));
    }
}
