use std::collections::VecDeque;
use smallvec::smallvec;
use crate::error::OryonError;
use crate::{Feature, Output};
use crate::ops::log_return;

/// Log return over a configurable lookback window.
///
/// Computes `ln(P_t / P_{t-n})` where `n` is the window. Returns `None`
/// during warm-up (first `window` bars). A `None` input propagates as
/// `None` output until the window is fully refilled with valid values.
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
    pub fn new(inputs: Vec<String>, window: usize, outputs: Vec<String>) -> Result<Self, OryonError> {
        if inputs.is_empty() {
            return Err(OryonError::InvalidInput { msg: "inputs must not be empty".into() });
        }

        if outputs.is_empty() {
            return Err(OryonError::InvalidInput { msg: "outputs must not be empty".into() });
        }

        if window == 0 {
            return Err(OryonError::InvalidInput { msg: "window must be non-zero".into() });
        }

        Ok(LogReturn {
            inputs,
            window,
            outputs,
            buffer: VecDeque::with_capacity(window+1),
        })
    }
}

impl Feature for LogReturn {
    fn input_names(&self) -> Vec<String> {
        self.inputs.clone()
    }

    fn output_names(&self) -> Vec<String> {
        self.outputs.clone()
    }

    fn warm_up_period(&self) -> usize { self.window }

    fn fresh(&self) -> Box<dyn Feature> {
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

        if self.buffer.len() == self.window + 1{
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
    use crate::feature_contract_tests;
    use smallvec::smallvec;


    feature_contract_tests!(LogReturn::new(vec!["close".to_string()], 2, vec!["close_log_return_2".to_string()]).unwrap(),
        vec!["close".to_string()],
        vec!["close_log_return_2".to_string()],
        2,
        &[Some(1.0)],
    );

    fn log_return_2() -> LogReturn {
        LogReturn::new(vec!["close".to_string()], 2, vec!["close_log_return_2".to_string()]).unwrap()
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
    fn test_window_size_is_one() {
        let mut log_return = LogReturn::new(vec!["close".to_string()], 1, vec!["close_log_return_1".to_string()]).unwrap();
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
        assert_eq!(log_return.buffer.capacity(), log_return.window+1);
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