use crate::error::OryonError;
use crate::ops::{average, rogers_satchell_sq};
use crate::traits::{Feature, Output};
use smallvec::smallvec;
use std::collections::VecDeque;

/// Rolling Rogers-Satchell volatility estimator.
///
/// Uses all four OHLC prices to estimate realized volatility. Unlike the Parkinson
/// estimator, it is unbiased in the presence of a directional drift, making it more
/// suitable for trending markets (Rogers & Satchell, 1994).
///
/// ```text
/// σ_RS = sqrt( (1/N) · Σ [ ln(H/C)·ln(H/O) + ln(L/C)·ln(L/O) ] )
/// ```
///
/// Returns `None` during warm-up (first `window - 1` bars), if any OHLC value in
/// the window is invalid (None, zero, or high < low), or if the rolling mean of the
/// per-bar terms is non-positive (unusual price action).
#[derive(Debug)]
pub struct RogersSatchellVolatility {
    inputs: Vec<String>,
    window: usize,
    outputs: Vec<String>,
    buffer: VecDeque<Option<f64>>,
}

impl RogersSatchellVolatility {
    /// Create a new `RogersSatchellVolatility`.
    ///
    /// - `inputs`  - names of the high, low, open, and close columns, in that order.
    ///   Must contain at least 4 entries.
    /// - `window`  - number of bars in the rolling window. Must be > 0.
    /// - `outputs` - name of the output column (e.g. `["rs_vol_20"]`).
    pub fn new(
        inputs: Vec<String>,
        window: usize,
        outputs: Vec<String>,
    ) -> Result<Self, OryonError> {
        if inputs.len() < 4 {
            return Err(OryonError::InvalidInput {
                msg: "inputs must contain high, low, open, and close columns".into(),
            });
        }
        if window == 0 {
            return Err(OryonError::InvalidInput {
                msg: "window must be non-zero".into(),
            });
        }
        if outputs.is_empty() {
            return Err(OryonError::InvalidInput {
                msg: "outputs must not be empty".into(),
            });
        }
        Ok(RogersSatchellVolatility {
            inputs,
            window,
            outputs,
            buffer: VecDeque::with_capacity(window),
        })
    }
}

impl Feature for RogersSatchellVolatility {
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
            RogersSatchellVolatility::new(self.inputs.clone(), self.window, self.outputs.clone())
                .expect("fresh: config was already validated at construction"),
        )
    }

    fn reset(&mut self) {
        self.buffer.clear();
    }

    fn update(&mut self, state: &[Option<f64>]) -> Output {
        let sq = rogers_satchell_sq(&[state[0], state[1], state[2], state[3]]);
        self.buffer.push_back(sq);

        if self.buffer.len() > self.window {
            self.buffer.pop_front();
        }

        if self.buffer.len() == self.window {
            let slices = self.buffer.make_contiguous();
            match average(slices) {
                Some(avg) if avg > 0.0 => smallvec![Some(avg.sqrt())],
                _ => smallvec![None],
            }
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
        RogersSatchellVolatility::new(
            vec!["high".into(), "low".into(), "open".into(), "close".into()],
            3,
            vec!["rs_vol_3".into()],
        )
        .unwrap(),
        vec![
            "high".to_string(),
            "low".to_string(),
            "open".to_string(),
            "close".to_string()
        ],
        vec!["rs_vol_3".to_string()],
        2,
        &[Some(108.0), Some(104.0), Some(105.0), Some(107.0)],
    );

    fn rs_3() -> RogersSatchellVolatility {
        RogersSatchellVolatility::new(
            vec!["high".into(), "low".into(), "open".into(), "close".into()],
            3,
            vec!["rs_vol_3".into()],
        )
        .unwrap()
    }

    fn out(v: Option<f64>) -> Output {
        smallvec![v]
    }

    // Reference: rs_sq(H,L,O,C) = ln(H/C)·ln(H/O) + ln(L/C)·ln(L/O)
    // σ_RS(window=3, all same bar) = sqrt(rs_sq)
    fn rs_sq(h: f64, l: f64, o: f64, c: f64) -> f64 {
        (h / c).ln() * (h / o).ln() + (l / c).ln() * (l / o).ln()
    }

    #[test]
    fn test_update() {
        let bar = &[Some(108.0), Some(104.0), Some(105.0), Some(107.0)];
        let expected_sq = rs_sq(108.0, 104.0, 105.0, 107.0);

        let mut rs = rs_3();
        assert_eq!(rs.update(bar), out(None));
        assert_eq!(rs.update(bar), out(None));

        // window full: mean(3 identical bars) = rs_sq → sqrt
        let result = rs.update(bar);
        assert!((result[0].unwrap() - expected_sq.sqrt()).abs() < 1e-12);

        // sliding window: new bar
        let bar2 = &[Some(110.0), Some(106.0), Some(107.0), Some(109.0)];
        let expected_sq2 = rs_sq(110.0, 106.0, 107.0, 109.0);
        let result2 = rs.update(bar2);
        let expected_avg = (2.0 * expected_sq + expected_sq2) / 3.0;
        assert!((result2[0].unwrap() - expected_avg.sqrt()).abs() < 1e-12);
    }

    #[test]
    fn test_update_none_input() {
        let mut rs = rs_3();
        let bar = &[Some(108.0), Some(104.0), Some(105.0), Some(107.0)];
        rs.update(bar);
        rs.update(bar);
        assert!(rs.update(bar)[0].is_some());
        // invalid bar → None
        assert_eq!(
            rs.update(&[Some(99.0), Some(108.0), Some(105.0), Some(107.0)]),
            out(None)
        );
    }

    #[test]
    fn test_reset_clears_buffer() {
        let mut rs = rs_3();
        let bar = &[Some(108.0), Some(104.0), Some(105.0), Some(107.0)];
        rs.update(bar);
        rs.reset();
        assert_eq!(rs.buffer.len(), 0);
        assert_eq!(rs.buffer.capacity(), rs.window);
    }

    #[test]
    fn test_fresh_instances_are_independent() {
        let bar = &[Some(108.0), Some(104.0), Some(105.0), Some(107.0)];
        let expected_sq = rs_sq(108.0, 104.0, 105.0, 107.0);

        let mut original = rs_3();
        original.update(bar);

        let mut fresh = original.fresh();
        assert_eq!(fresh.update(bar), out(None));
        assert_eq!(fresh.update(bar), out(None));
        assert!((fresh.update(bar)[0].unwrap() - expected_sq.sqrt()).abs() < 1e-12);

        assert_eq!(original.update(bar), out(None));
        assert!((original.update(bar)[0].unwrap() - expected_sq.sqrt()).abs() < 1e-12);
    }

    #[test]
    fn test_invalid_params() {
        assert!(matches!(
            RogersSatchellVolatility::new(vec!["h".into(), "l".into(), "o".into()], 3, vec!["out".into()]).unwrap_err(),
            OryonError::InvalidInput { ref msg } if msg.contains("inputs")
        ));
        assert!(matches!(
            RogersSatchellVolatility::new(vec!["h".into(), "l".into(), "o".into(), "c".into()], 0, vec!["out".into()]).unwrap_err(),
            OryonError::InvalidInput { ref msg } if msg.contains("window")
        ));
        assert!(matches!(
            RogersSatchellVolatility::new(vec!["h".into(), "l".into(), "o".into(), "c".into()], 3, vec![]).unwrap_err(),
            OryonError::InvalidInput { ref msg } if msg.contains("outputs")
        ));
    }
}
