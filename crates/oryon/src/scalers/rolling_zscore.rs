use crate::error::OryonError;
use crate::ops::{average, std_dev};
use crate::traits::{Output, StreamingTransform};
use smallvec::smallvec;
use std::collections::VecDeque;

/// Rolling z-score normalization over a sliding window.
///
/// Computes `(x - rolling_mean) / rolling_std` using the last `window` values.
/// Returns `None` during warm-up (first `window - 1` bars), if std is zero
/// (all values in window are equal), or if any value in the window is `None`.
#[derive(Debug)]
pub struct RollingZScore {
    inputs: Vec<String>,
    window: usize,
    outputs: Vec<String>,
    buffer: VecDeque<Option<f64>>,
}

impl RollingZScore {
    /// Create a new `RollingZScore`.
    ///
    /// - `inputs` - name of the input column. Must contain exactly 1 entry.
    /// - `window` - number of bars for rolling statistics. Must be >= 2.
    /// - `outputs` - name of the output column.
    pub fn new(
        inputs: Vec<String>,
        window: usize,
        outputs: Vec<String>,
    ) -> Result<Self, OryonError> {
        if inputs.len() != 1 {
            return Err(OryonError::InvalidInput {
                msg: "inputs must contain exactly 1 column".into(),
            });
        }
        if window < 2 {
            return Err(OryonError::InvalidInput {
                msg: "window must be >= 2 (need at least 2 values for std)".into(),
            });
        }
        if outputs.is_empty() {
            return Err(OryonError::InvalidInput {
                msg: "outputs must not be empty".into(),
            });
        }
        Ok(RollingZScore {
            inputs,
            window,
            outputs,
            buffer: VecDeque::with_capacity(window),
        })
    }
}

impl StreamingTransform for RollingZScore {
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
            RollingZScore::new(self.inputs.clone(), self.window, self.outputs.clone())
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

        if self.buffer.len() < self.window {
            return smallvec![None];
        }

        let slice = self.buffer.make_contiguous();
        let mean = match average(slice) {
            Some(m) => m,
            None => return smallvec![None],
        };
        let std = match std_dev(slice) {
            Some(s) if s > 0.0 => s,
            _ => return smallvec![None],
        };

        let current = match state[0] {
            Some(x) => x,
            None => return smallvec![None],
        };

        smallvec![Some((current - mean) / std)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature_contract_tests;
    use smallvec::smallvec;

    feature_contract_tests!(
        RollingZScore::new(vec!["x".into()], 3, vec!["x_z".into()]).unwrap(),
        vec!["x".to_string()],
        vec!["x_z".to_string()],
        2,
        &[Some(1.0)],
    );

    fn rz3() -> RollingZScore {
        RollingZScore::new(vec!["x".into()], 3, vec!["x_z".into()]).unwrap()
    }

    fn out(v: Option<f64>) -> Output {
        smallvec![v]
    }

    #[test]
    fn test_update() {
        let mut s = rz3();
        // warm-up
        assert_eq!(s.update(&[Some(1.0)]), out(None));
        assert_eq!(s.update(&[Some(2.0)]), out(None));
        // window = [1, 2, 3], mean = 2, std = 1, z = (3-2)/1 = 1
        assert_eq!(s.update(&[Some(3.0)]), out(Some(1.0)));
        // window = [2, 3, 4], mean = 3, std = 1, z = (4-3)/1 = 1
        assert_eq!(s.update(&[Some(4.0)]), out(Some(1.0)));
    }

    #[test]
    fn test_update_none_input() {
        let mut s = rz3();
        assert_eq!(s.update(&[Some(1.0)]), out(None));
        assert_eq!(s.update(&[Some(2.0)]), out(None));
        assert_eq!(s.update(&[Some(3.0)]), out(Some(1.0)));
        // None enters the window
        assert_eq!(s.update(&[None]), out(None));
        assert_eq!(s.update(&[Some(5.0)]), out(None));
        assert_eq!(s.update(&[Some(6.0)]), out(None));
        // window fully refilled: [5, 6, 7]
        assert_eq!(s.update(&[Some(7.0)]), out(Some(1.0)));
    }

    #[test]
    fn test_update_all_equal() {
        let mut s = rz3();
        assert_eq!(s.update(&[Some(5.0)]), out(None));
        assert_eq!(s.update(&[Some(5.0)]), out(None));
        // std = 0 → None
        assert_eq!(s.update(&[Some(5.0)]), out(None));
    }

    #[test]
    fn test_reset_clears_buffer() {
        let mut s = rz3();
        s.update(&[Some(1.0)]);
        s.update(&[Some(2.0)]);
        s.update(&[Some(3.0)]);
        s.reset();
        assert_eq!(s.buffer.len(), 0);
        assert_eq!(s.buffer.capacity(), s.window);
        // back to warm-up
        assert_eq!(s.update(&[Some(10.0)]), out(None));
    }

    #[test]
    fn test_fresh_instances_are_independent() {
        let mut s = rz3();
        s.update(&[Some(1.0)]);
        s.update(&[Some(2.0)]);

        let mut fresh = s.fresh();
        assert_eq!(fresh.update(&[Some(10.0)]), out(None));
        // original continues
        assert_eq!(s.update(&[Some(3.0)]), out(Some(1.0)));
    }

    #[test]
    fn test_invalid_params() {
        assert!(matches!(
            RollingZScore::new(vec![], 3, vec!["out".into()]).unwrap_err(),
            OryonError::InvalidInput { ref msg } if msg.contains("1 column")
        ));
        assert!(matches!(
            RollingZScore::new(vec!["x".into()], 1, vec!["out".into()]).unwrap_err(),
            OryonError::InvalidInput { ref msg } if msg.contains("window")
        ));
        assert!(matches!(
            RollingZScore::new(vec!["x".into()], 3, vec![]).unwrap_err(),
            OryonError::InvalidInput { ref msg } if msg.contains("outputs")
        ));
    }
}
