use crate::error::OryonError;
use crate::traits::{Feature, Output};
use smallvec::smallvec;
use std::collections::VecDeque;

/// Rolling OLS regression of `y` on `x` over a sliding window.
///
/// Fits a straight line `y = a·x + b` at each bar using the last `window` observations
/// and returns two outputs: the slope `a = Sxy / Sxx` and the coefficient of
/// determination `R² = Sxy² / (Sxx · Syy)`.
///
/// Both outputs are `None` during warm-up (first `window - 1` bars), if any value in the
/// window is `None`, or if `x` is constant over the window (`Sxx == 0`). `R²` is also
/// `None` if `y` is constant (`Syy == 0`), while slope remains valid in that case.
#[derive(Debug)]
pub struct LinearSlope {
    inputs: Vec<String>,
    window: usize,
    outputs: Vec<String>,
    x_buffer: VecDeque<Option<f64>>,
    y_buffer: VecDeque<Option<f64>>,
}

impl LinearSlope {
    /// Create a new `LinearSlope`.
    ///
    /// - `inputs`  - names of the x and y columns, in that order (e.g. `["time_idx", "price"]`).
    ///   Must contain at least 2 entries.
    /// - `window`  - number of bars in the rolling window. Must be >= 2.
    /// - `outputs` - names of the two output columns: `[slope_name, r2_name]`. Must contain
    ///   exactly 2 entries.
    pub fn new(
        inputs: Vec<String>,
        window: usize,
        outputs: Vec<String>,
    ) -> Result<Self, OryonError> {
        if inputs.len() < 2 {
            return Err(OryonError::InvalidInput {
                msg: "inputs must contain x and y columns".into(),
            });
        }
        if window < 2 {
            return Err(OryonError::InvalidInput {
                msg: "window must be >= 2".into(),
            });
        }
        if outputs.len() != 2 {
            return Err(OryonError::InvalidInput {
                msg: "outputs must contain exactly 2 names: [slope, r2]".into(),
            });
        }
        Ok(LinearSlope {
            inputs,
            window,
            outputs,
            x_buffer: VecDeque::with_capacity(window),
            y_buffer: VecDeque::with_capacity(window),
        })
    }
}

impl Feature for LinearSlope {
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
            LinearSlope::new(self.inputs.clone(), self.window, self.outputs.clone())
                .expect("fresh: config was already validated at construction"),
        )
    }

    fn reset(&mut self) {
        self.x_buffer.clear();
        self.y_buffer.clear();
    }

    fn update(&mut self, state: &[Option<f64>]) -> Output {
        self.x_buffer.push_back(state[0]);
        self.y_buffer.push_back(state[1]);

        if self.x_buffer.len() > self.window {
            self.x_buffer.pop_front();
            self.y_buffer.pop_front();
        }

        if self.x_buffer.len() < self.window {
            return smallvec![None, None];
        }

        let x = self.x_buffer.make_contiguous();
        let y = self.y_buffer.make_contiguous();
        let n = x.len() as f64;

        // First pass: compute means, bail out on any None.
        let mut x_sum = 0.0f64;
        let mut y_sum = 0.0f64;
        for i in 0..x.len() {
            match (x[i], y[i]) {
                (Some(xi), Some(yi)) => {
                    x_sum += xi;
                    y_sum += yi;
                }
                _ => return smallvec![None, None],
            }
        }
        let x_mean = x_sum / n;
        let y_mean = y_sum / n;

        // Second pass: compute Sxx, Sxy, Syy.
        let mut sxx = 0.0f64;
        let mut sxy = 0.0f64;
        let mut syy = 0.0f64;
        for i in 0..x.len() {
            let dx = x[i].unwrap() - x_mean;
            let dy = y[i].unwrap() - y_mean;
            sxx += dx * dx;
            sxy += dx * dy;
            syy += dy * dy;
        }

        if sxx == 0.0 {
            return smallvec![None, None];
        }

        let slope = sxy / sxx;
        let r2 = if syy == 0.0 {
            None
        } else {
            Some(sxy * sxy / (sxx * syy))
        };

        smallvec![Some(slope), r2]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature_contract_tests;
    use smallvec::smallvec;

    feature_contract_tests!(
        LinearSlope::new(
            vec!["x".into(), "y".into()],
            3,
            vec!["xy_slope_3".into(), "xy_r2_3".into()],
        )
        .unwrap(),
        vec!["x".to_string(), "y".to_string()],
        vec!["xy_slope_3".to_string(), "xy_r2_3".to_string()],
        2,
        &[Some(1.0), Some(2.0)],
    );

    fn ls_3() -> LinearSlope {
        LinearSlope::new(
            vec!["x".into(), "y".into()],
            3,
            vec!["xy_slope_3".into(), "xy_r2_3".into()],
        )
        .unwrap()
    }

    fn none_out() -> Output {
        smallvec![None, None]
    }

    #[test]
    fn test_update() {
        // x=[1,2,3], y=[2.1,3.9,6.2] → slope=2.05, R²=151.29/152.04≈0.99507
        // x=[1,2,3], y=[2,4,6]       → slope=2.0,  R²=1.0 (perfect fit)
        let mut ls = ls_3();
        assert_eq!(ls.update(&[Some(1.0), Some(2.1)]), none_out());
        assert_eq!(ls.update(&[Some(2.0), Some(3.9)]), none_out());

        let out = ls.update(&[Some(3.0), Some(6.2)]);
        assert!((out[0].unwrap() - 2.05).abs() < 1e-10);
        assert!((out[1].unwrap() - 151.29_f64 / 152.04_f64).abs() < 1e-6);

        // Perfect linear fit: y = 2x
        let mut ls2 = ls_3();
        ls2.update(&[Some(1.0), Some(2.0)]);
        ls2.update(&[Some(2.0), Some(4.0)]);
        let out2 = ls2.update(&[Some(3.0), Some(6.0)]);
        assert!((out2[0].unwrap() - 2.0).abs() < 1e-10);
        assert!((out2[1].unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_update_constant_y_slope_valid_r2_none() {
        // y constant → slope=0, R²=None (Syy=0)
        let mut ls = ls_3();
        ls.update(&[Some(1.0), Some(5.0)]);
        ls.update(&[Some(2.0), Some(5.0)]);
        let out = ls.update(&[Some(3.0), Some(5.0)]);
        assert!((out[0].unwrap() - 0.0).abs() < 1e-10);
        assert_eq!(out[1], None);
    }

    #[test]
    fn test_update_none_input() {
        let mut ls = ls_3();
        ls.update(&[Some(1.0), Some(2.0)]);
        ls.update(&[Some(2.0), Some(4.0)]);
        assert!(ls.update(&[Some(3.0), Some(6.0)])[0].is_some());
        // None in x → both outputs None
        assert_eq!(ls.update(&[None, Some(8.0)]), none_out());
        // window still contains None for the next 2 bars (window=3)
        assert_eq!(ls.update(&[Some(5.0), Some(10.0)]), none_out());
        assert_eq!(ls.update(&[Some(6.0), Some(12.0)]), none_out());
        // None has now been popped out — valid again
        assert!(ls.update(&[Some(7.0), Some(14.0)])[0].is_some());
    }

    #[test]
    fn test_reset_clears_buffers() {
        let mut ls = ls_3();
        ls.update(&[Some(1.0), Some(2.0)]);
        ls.update(&[Some(2.0), Some(4.0)]);
        ls.reset();
        assert_eq!(ls.x_buffer.len(), 0);
        assert_eq!(ls.y_buffer.len(), 0);
        assert_eq!(ls.x_buffer.capacity(), ls.window);
        assert_eq!(ls.y_buffer.capacity(), ls.window);
    }

    #[test]
    fn test_fresh_instances_are_independent() {
        let mut original = ls_3();
        original.update(&[Some(1.0), Some(2.1)]);

        let mut fresh = original.fresh();

        // fresh starts from scratch
        assert_eq!(fresh.update(&[Some(1.0), Some(2.1)]), none_out());
        assert_eq!(fresh.update(&[Some(2.0), Some(3.9)]), none_out());
        assert!((fresh.update(&[Some(3.0), Some(6.2)])[0].unwrap() - 2.05).abs() < 1e-10);

        // original continues from its own state
        assert_eq!(original.update(&[Some(2.0), Some(3.9)]), none_out());
        assert!((original.update(&[Some(3.0), Some(6.2)])[0].unwrap() - 2.05).abs() < 1e-10);
    }

    #[test]
    fn test_invalid_params() {
        assert!(matches!(
            LinearSlope::new(vec!["x".into()], 3, vec!["s".into(), "r".into()]).unwrap_err(),
            OryonError::InvalidInput { ref msg } if msg.contains("inputs")
        ));
        assert!(matches!(
            LinearSlope::new(vec!["x".into(), "y".into()], 1, vec!["s".into(), "r".into()]).unwrap_err(),
            OryonError::InvalidInput { ref msg } if msg.contains("window")
        ));
        assert!(matches!(
            LinearSlope::new(vec!["x".into(), "y".into()], 3, vec!["slope".into()]).unwrap_err(),
            OryonError::InvalidInput { ref msg } if msg.contains("outputs")
        ));
    }
}
