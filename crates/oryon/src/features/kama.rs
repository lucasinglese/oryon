use crate::error::OryonError;
use crate::traits::{Output, StreamingTransform};
use smallvec::smallvec;
use std::collections::VecDeque;

/// Kaufman's Adaptive Moving Average (KAMA).
///
/// KAMA adapts its smoothing speed based on market efficiency. When price moves
/// directionally (trending), it reacts fast. When price oscillates (noisy), it
/// slows down. The adaptation is driven by the **Efficiency Ratio (ER)**:
///
/// ```text
/// ER         = |P_t - P_{t-n}| / sum(|P_i - P_{i-1}|, i=t-n+1..t)
/// fast_sc    = 2 / (fast + 1)
/// slow_sc    = 2 / (slow + 1)
/// SC         = (ER * (fast_sc - slow_sc) + slow_sc)^2
/// KAMA_t     = KAMA_{t-1} + SC * (P_t - KAMA_{t-1})
/// ```
///
/// - ER → 1 (strong trend): SC ≈ fast_sc → KAMA tracks price closely.
/// - ER → 0 (choppy market): SC ≈ slow_sc^2 → KAMA barely moves.
///
/// Returns `None` during warm-up (first `window` bars). A `None` input within
/// the window propagates as `None` and resets the seed for the next valid window.
///
/// **Kaufman defaults**: `window = 10`, `fast = 2`, `slow = 30`.
#[derive(Debug)]
pub struct Kama {
    inputs: Vec<String>,
    window: usize,
    outputs: Vec<String>,
    fast_sc: f64,
    slow_sc: f64,
    prev_kama: Option<f64>,
    buffer: VecDeque<Option<f64>>,
}

impl Kama {
    /// Create a new `Kama`.
    ///
    /// - `inputs`  - name of the input column (e.g. `["close"]`).
    /// - `window`  - lookback for the Efficiency Ratio. Must be >= 1.
    ///   Kaufman default: 10.
    /// - `outputs` - name of the output column (e.g. `["close_kama_10"]`).
    /// - `fast`    - period for the fast smoothing constant. Must be >= 1.
    ///   Kaufman default: 2.
    /// - `slow`    - period for the slow smoothing constant. Must be > `fast`.
    ///   Kaufman default: 30.
    pub fn new(
        inputs: Vec<String>,
        window: usize,
        outputs: Vec<String>,
        fast: usize,
        slow: usize,
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
                msg: "window must be >= 1".into(),
            });
        }
        if fast == 0 {
            return Err(OryonError::InvalidInput {
                msg: "fast must be >= 1".into(),
            });
        }
        if slow <= fast {
            return Err(OryonError::InvalidInput {
                msg: "slow must be > fast".into(),
            });
        }
        Ok(Kama {
            inputs,
            window,
            outputs,
            fast_sc: 2.0 / (fast as f64 + 1.0),
            slow_sc: 2.0 / (slow as f64 + 1.0),
            prev_kama: None,
            buffer: VecDeque::with_capacity(window + 1),
        })
    }
}

impl StreamingTransform for Kama {
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
        Box::new(Kama {
            inputs: self.inputs.clone(),
            window: self.window,
            outputs: self.outputs.clone(),
            fast_sc: self.fast_sc,
            slow_sc: self.slow_sc,
            prev_kama: None,
            buffer: VecDeque::with_capacity(self.window + 1),
        })
    }

    fn reset(&mut self) {
        self.prev_kama = None;
        self.buffer.clear();
    }

    fn update(&mut self, state: &[Option<f64>]) -> Output {
        self.buffer.push_back(state[0]);

        if self.buffer.len() > self.window + 1 {
            self.buffer.pop_front();
        }

        if self.buffer.len() < self.window + 1 {
            return smallvec![None];
        }

        // Collect values — any None in window resets seed and returns None.
        let prices: Vec<f64> = match self.buffer.iter().copied().collect::<Option<Vec<_>>>() {
            Some(v) => v,
            None => {
                self.prev_kama = None;
                return smallvec![None];
            }
        };

        let direction = (prices[self.window] - prices[0]).abs();
        let volatility: f64 = prices.windows(2).map(|w| (w[1] - w[0]).abs()).sum();

        let er = if volatility > 0.0 {
            direction / volatility
        } else {
            0.0
        };
        let sc = (er * (self.fast_sc - self.slow_sc) + self.slow_sc).powi(2);

        let seed = self.prev_kama.unwrap_or(prices[self.window - 1]);
        let kama = seed + sc * (prices[self.window] - seed);
        self.prev_kama = Some(kama);
        smallvec![Some(kama)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::streaming_transform_contract_tests;
    use smallvec::smallvec;

    streaming_transform_contract_tests!(
        Kama::new(vec!["close".into()], 3, vec!["close_kama_3".into()], 2, 5).unwrap(),
        vec!["close".to_string()],
        vec!["close_kama_3".to_string()],
        3,
        &[Some(1.0)],
    );

    fn kama_3() -> Kama {
        Kama::new(vec!["close".into()], 3, vec!["close_kama_3".into()], 2, 5).unwrap()
    }

    fn out(v: Option<f64>) -> Output {
        smallvec![v]
    }

    #[test]
    fn test_update() {
        // window=3, fast=2, slow=5 — Python reference values
        let mut k = kama_3();
        assert_eq!(k.update(&[Some(100.0)]), out(None));
        assert_eq!(k.update(&[Some(101.0)]), out(None));
        assert_eq!(k.update(&[Some(103.0)]), out(None));
        assert!((k.update(&[Some(102.0)])[0].unwrap() - 102.75).abs() < 1e-10);
        assert!((k.update(&[Some(105.0)])[0].unwrap() - 103.444_444_444_444_44).abs() < 1e-10);
        assert!((k.update(&[Some(107.0)])[0].unwrap() - 104.541_838_134_430_72).abs() < 1e-10);
        assert!((k.update(&[Some(106.0)])[0].unwrap() - 104.991_888_092_939_76).abs() < 1e-10);
    }

    #[test]
    fn test_update_none_input() {
        let mut k = kama_3();
        k.update(&[Some(100.0)]);
        k.update(&[Some(101.0)]);
        k.update(&[Some(103.0)]);
        assert!(k.update(&[Some(102.0)])[0].is_some());
        // None mid-window → None output, seed reset
        assert_eq!(k.update(&[None]), out(None));
        assert_eq!(k.update(&[Some(105.0)]), out(None));
        assert_eq!(k.update(&[Some(107.0)]), out(None));
        assert_eq!(k.update(&[Some(106.0)]), out(None));
        // None flushed — window all valid again, re-seeded
        assert!(k.update(&[Some(108.0)])[0].is_some());
    }

    #[test]
    fn test_reset_clears_state() {
        let mut k = kama_3();
        k.update(&[Some(100.0)]);
        k.reset();
        assert_eq!(k.buffer.len(), 0);
        assert_eq!(k.prev_kama, None);
    }

    #[test]
    fn test_fresh_instances_are_independent() {
        let mut original = kama_3();
        original.update(&[Some(100.0)]);

        let mut fresh = original.fresh();

        // fresh starts from scratch
        assert_eq!(fresh.update(&[Some(100.0)]), out(None));
        assert_eq!(fresh.update(&[Some(101.0)]), out(None));
        assert_eq!(fresh.update(&[Some(103.0)]), out(None));
        assert!((fresh.update(&[Some(102.0)])[0].unwrap() - 102.75).abs() < 1e-10);

        // original continues from its own state
        assert_eq!(original.update(&[Some(101.0)]), out(None));
        assert_eq!(original.update(&[Some(103.0)]), out(None));
        assert!((original.update(&[Some(102.0)])[0].unwrap() - 102.75).abs() < 1e-10);
    }

    #[test]
    fn test_error_raises_when_empty_inputs() {
        let err = Kama::new(vec![], 3, vec!["k".into()], 2, 30).unwrap_err();
        assert!(matches!(err, OryonError::InvalidInput { ref msg } if msg.contains("inputs")));
    }

    #[test]
    fn test_error_raises_when_empty_outputs() {
        let err = Kama::new(vec!["close".into()], 3, vec![], 2, 30).unwrap_err();
        assert!(matches!(err, OryonError::InvalidInput { ref msg } if msg.contains("outputs")));
    }

    #[test]
    fn test_error_raises_when_window_is_zero() {
        let err = Kama::new(vec!["close".into()], 0, vec!["k".into()], 2, 30).unwrap_err();
        assert!(matches!(err, OryonError::InvalidInput { ref msg } if msg.contains("window")));
    }

    #[test]
    fn test_error_raises_when_slow_not_greater_than_fast() {
        let err = Kama::new(vec!["close".into()], 3, vec!["k".into()], 5, 5).unwrap_err();
        assert!(matches!(err, OryonError::InvalidInput { ref msg } if msg.contains("slow")));
    }
}
