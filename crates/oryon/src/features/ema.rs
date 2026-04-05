use crate::error::OryonError;
use crate::ops::average;
use crate::traits::{Output, StreamingTransform};
use smallvec::smallvec;
use std::collections::VecDeque;

/// Exponential Moving Average with SMA seeding.
///
/// Uses `α = 2 / (window + 1)`. The first output is the SMA of the first `window`
/// bars (seed). Subsequent outputs apply `EMA_t = α * P_t + (1 - α) * EMA_{t-1}`.
/// Returns `None` during warm-up (first `window - 1` bars). A `None` input resets
/// the state and restarts the warm-up.
#[derive(Debug)]
pub struct Ema {
    inputs: Vec<String>,
    window: usize,
    outputs: Vec<String>,
    alpha: f64,
    prev_ema: Option<f64>,
    buffer: VecDeque<Option<f64>>,
}

impl Ema {
    /// Create a new `Ema`.
    ///
    /// - `inputs` — name of the input column (e.g. `["close"]`).
    /// - `window` — number of bars for seeding and smoothing factor. Must be > 0.
    /// - `outputs` — name of the output column (e.g. `["close_ema_20"]`).
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
        let alpha = 2.0 / (window as f64 + 1.0);
        Ok(Ema {
            inputs,
            window,
            outputs,
            alpha,
            prev_ema: None,
            buffer: VecDeque::with_capacity(window),
        })
    }
}

impl StreamingTransform for Ema {
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
            Ema::new(self.inputs.clone(), self.window, self.outputs.clone())
                .expect("fresh: config was already validated at construction"),
        )
    }

    fn reset(&mut self) {
        self.prev_ema = None;
        self.buffer.clear();
    }

    fn update(&mut self, state: &[Option<f64>]) -> Output {
        if self.prev_ema.is_none() {
            // Seeding phase: accumulate until window is full.
            self.buffer.push_back(state[0]);
            if self.buffer.len() == self.window {
                let slices = self.buffer.make_contiguous();
                self.prev_ema = average(slices);
                self.buffer.clear();
                return smallvec![self.prev_ema];
            }
            return smallvec![None];
        }

        // Recursive phase.
        match state[0] {
            Some(price) => {
                let ema = self.alpha * price + (1.0 - self.alpha) * self.prev_ema.unwrap();
                self.prev_ema = Some(ema);
                smallvec![Some(ema)]
            }
            None => {
                self.prev_ema = None;
                self.buffer.clear();
                smallvec![None]
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature_contract_tests;
    use smallvec::smallvec;

    feature_contract_tests!(
        Ema::new(vec!["close".into()], 3, vec!["close_ema_3".into()]).unwrap(),
        vec!["close".to_string()],
        vec!["close_ema_3".to_string()],
        2,
        &[Some(1.0)],
    );

    fn ema_3() -> Ema {
        Ema::new(vec!["close".into()], 3, vec!["close_ema_3".into()]).unwrap()
    }

    fn out(v: Option<f64>) -> Output {
        smallvec![v]
    }

    #[test]
    fn test_update() {
        // alpha = 2/(3+1) = 0.5
        let mut e = ema_3();
        assert_eq!(e.update(&[Some(100.0)]), out(None));
        assert_eq!(e.update(&[Some(200.0)]), out(None));
        // seed = SMA([100,200,300]) = 200
        assert_eq!(e.update(&[Some(300.0)]), out(Some(200.0)));
        // 0.5*400 + 0.5*200 = 300
        assert_eq!(e.update(&[Some(400.0)]), out(Some(300.0)));
        // 0.5*500 + 0.5*300 = 400
        assert_eq!(e.update(&[Some(500.0)]), out(Some(400.0)));
    }

    #[test]
    fn test_update_none_input_resets_state() {
        let mut e = ema_3();
        e.update(&[Some(100.0)]);
        e.update(&[Some(200.0)]);
        e.update(&[Some(300.0)]); // seeded
        assert_eq!(e.update(&[None]), out(None)); // reset

        // re-seeds from scratch
        assert_eq!(e.update(&[Some(100.0)]), out(None));
        assert_eq!(e.update(&[Some(200.0)]), out(None));
        assert_eq!(e.update(&[Some(300.0)]), out(Some(200.0)));
    }

    #[test]
    fn test_window_size_is_one() {
        // alpha = 1.0 → EMA = current price
        let mut e = Ema::new(vec!["close".into()], 1, vec!["close_ema_1".into()]).unwrap();
        assert_eq!(e.update(&[Some(100.0)]), out(Some(100.0)));
        assert_eq!(e.update(&[Some(200.0)]), out(Some(200.0)));
    }

    #[test]
    fn test_reset_clears_state() {
        let mut e = ema_3();
        e.update(&[Some(100.0)]);
        e.reset();
        assert_eq!(e.buffer.len(), 0);
        assert_eq!(e.prev_ema, None);
    }

    #[test]
    fn test_fresh_instances_are_independent() {
        let mut original = ema_3();
        original.update(&[Some(100.0)]);

        let mut fresh = original.fresh();

        // fresh starts from scratch
        assert_eq!(fresh.update(&[Some(100.0)]), out(None));
        assert_eq!(fresh.update(&[Some(200.0)]), out(None));
        assert_eq!(fresh.update(&[Some(300.0)]), out(Some(200.0)));

        // original continues from its own state
        assert_eq!(original.update(&[Some(200.0)]), out(None));
        assert_eq!(original.update(&[Some(300.0)]), out(Some(200.0)));
    }

    #[test]
    fn test_error_raises_when_empty_inputs() {
        let err = Ema::new(vec![], 3, vec!["ema".into()]).unwrap_err();
        assert!(matches!(err, OryonError::InvalidInput { ref msg } if msg.contains("inputs")));
    }

    #[test]
    fn test_error_raises_when_empty_outputs() {
        let err = Ema::new(vec!["close".into()], 3, vec![]).unwrap_err();
        assert!(matches!(err, OryonError::InvalidInput { ref msg } if msg.contains("outputs")));
    }

    #[test]
    fn test_error_raises_when_window_is_zero() {
        let err = Ema::new(vec!["close".into()], 0, vec!["ema".into()]).unwrap_err();
        assert!(matches!(err, OryonError::InvalidInput { ref msg } if msg.contains("window")));
    }
}
