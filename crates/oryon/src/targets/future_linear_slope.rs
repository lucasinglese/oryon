use crate::error::OryonError;
use crate::traits::Target;

/// Future OLS slope and R² of `y` regressed on `x` over the next `horizon` bars.
///
/// For each bar *t*, regresses `y[t..t+horizon]` on `x[t..t+horizon]` and returns
/// the slope `a = Sxy / Sxx` and the coefficient of determination `R² = Sxy² / (Sxx · Syy)`.
/// Both `x` and `y` are real input columns - `x` can be time indices, cumulative volume,
/// or any other series.
///
/// The last `horizon` values of both outputs are `None` because the future is not yet known.
/// Any `None` within a window also produces `None` for that bar. `R²` is additionally `None`
/// when `y` is constant over the window (`Syy == 0`). Both outputs are `None` when `x` is
/// constant over the window (`Sxx == 0`).
#[derive(Debug)]
pub struct FutureLinearSlope {
    inputs: Vec<String>,
    horizon: usize,
    outputs: Vec<String>,
}

impl FutureLinearSlope {
    /// Create a new `FutureLinearSlope` target.
    ///
    /// - `inputs`  - names of the x and y columns, in that order (e.g. `["time_idx", "close"]`).
    ///   Must contain at least 2 non-empty entries. `inputs[0]` is x, `inputs[1]` is y.
    /// - `horizon` - number of bars to look ahead. Must be >= 2.
    /// - `outputs` - names of the two output columns: `[slope_name, r2_name]`. Must contain
    ///   exactly 2 entries.
    pub fn new(
        inputs: Vec<String>,
        horizon: usize,
        outputs: Vec<String>,
    ) -> Result<Self, OryonError> {
        if inputs.len() < 2 {
            return Err(OryonError::InvalidInput {
                msg: "inputs must contain x and y columns".into(),
            });
        }
        if inputs.iter().any(|s| s.is_empty()) {
            return Err(OryonError::InvalidInput {
                msg: "input names must not be empty".into(),
            });
        }
        if horizon < 2 {
            return Err(OryonError::InvalidInput {
                msg: "horizon must be >= 2".into(),
            });
        }
        if outputs.len() != 2 {
            return Err(OryonError::InvalidInput {
                msg: "outputs must contain exactly 2 names: [slope, r2]".into(),
            });
        }
        Ok(FutureLinearSlope {
            inputs,
            horizon,
            outputs,
        })
    }
}

impl Target for FutureLinearSlope {
    fn input_names(&self) -> Vec<String> {
        self.inputs.clone()
    }

    fn output_names(&self) -> Vec<String> {
        self.outputs.clone()
    }

    fn forward_period(&self) -> usize {
        self.horizon
    }

    fn run_research(&self, columns: &[&[Option<f64>]]) -> Vec<Vec<Option<f64>>> {
        let x = columns[0];
        let y = columns[1];
        let n = x.len();
        let h = self.horizon;

        let mut slopes = vec![None; n];
        let mut r2s = vec![None; n];

        for t in 0..n {
            if t + h > n {
                break;
            }
            let x_win = &x[t..t + h];
            let y_win = &y[t..t + h];

            // First pass: compute means, bail on any None.
            let mut x_sum = 0.0f64;
            let mut y_sum = 0.0f64;
            let mut valid = true;
            for i in 0..h {
                match (x_win[i], y_win[i]) {
                    (Some(xi), Some(yi)) => {
                        x_sum += xi;
                        y_sum += yi;
                    }
                    _ => {
                        valid = false;
                        break;
                    }
                }
            }
            if !valid {
                continue;
            }
            let x_mean = x_sum / h as f64;
            let y_mean = y_sum / h as f64;

            // Second pass: compute Sxx, Sxy, Syy.
            let mut sxx = 0.0f64;
            let mut sxy = 0.0f64;
            let mut syy = 0.0f64;
            for i in 0..h {
                let dx = x_win[i].unwrap() - x_mean;
                let dy = y_win[i].unwrap() - y_mean;
                sxx += dx * dx;
                sxy += dx * dy;
                syy += dy * dy;
            }

            if sxx == 0.0 {
                continue;
            }

            slopes[t] = Some(sxy / sxx);
            r2s[t] = if syy == 0.0 {
                None
            } else {
                Some(sxy * sxy / (sxx * syy))
            };
        }

        vec![slopes, r2s]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn time_idx(n: usize) -> Vec<Option<f64>> {
        (0..n).map(|i| Some(i as f64)).collect()
    }

    fn prices() -> Vec<Option<f64>> {
        vec![
            Some(100.0),
            Some(101.0),
            Some(103.0),
            Some(102.0),
            Some(105.0),
            Some(107.0),
            Some(106.0),
        ]
    }

    fn fls3() -> FutureLinearSlope {
        FutureLinearSlope::new(
            vec!["t".into(), "close".into()],
            3,
            vec!["close_slope_3".into(), "close_r2_3".into()],
        )
        .unwrap()
    }

    // Contract tests (manual — target_contract_tests! supports single-input only).
    #[test]
    fn test_contract_input_names() {
        assert_eq!(
            fls3().input_names(),
            vec!["t".to_string(), "close".to_string()]
        );
    }

    #[test]
    fn test_contract_output_names() {
        assert_eq!(
            fls3().output_names(),
            vec!["close_slope_3".to_string(), "close_r2_3".to_string()],
        );
    }

    #[test]
    fn test_contract_forward_period() {
        assert_eq!(fls3().forward_period(), 3);
    }

    #[test]
    fn test_contract_warm_up_period() {
        assert_eq!(fls3().warm_up_period(), 0);
    }

    #[test]
    fn test_contract_compute_shape() {
        let x = time_idx(7);
        let p = prices();
        let result = fls3().run_research(&[&x, &p]);
        assert_eq!(result.len(), 2);
        assert!(result.iter().all(|col| col.len() == 7));
    }

    #[test]
    fn test_compute_forward_none() {
        let x = time_idx(7);
        let result = fls3().run_research(&[&x, &prices()]);
        for col in &result {
            assert_eq!(col[5], None);
            assert_eq!(col[6], None);
        }
    }

    #[test]
    fn test_compute_valid_values() {
        let x = time_idx(7);
        let result = fls3().run_research(&[&x, &prices()]);
        let slopes = &result[0];
        let r2s = &result[1];

        // bar 0: x=[0,1,2], y=[100,101,103] → slope=1.5, R²=27/28
        assert!((slopes[0].unwrap() - 1.5).abs() < 1e-10);
        assert!((r2s[0].unwrap() - 27.0_f64 / 28.0_f64).abs() < 1e-10);

        // bar 1: x=[1,2,3], y=[101,103,102] → slope=0.5, R²=0.25
        assert!((slopes[1].unwrap() - 0.5).abs() < 1e-10);
        assert!((r2s[1].unwrap() - 0.25).abs() < 1e-10);
    }

    #[test]
    fn test_compute_stateless() {
        let target = fls3();
        let x = time_idx(7);
        let p = prices();
        assert_eq!(
            target.run_research(&[&x, &p]),
            target.run_research(&[&x, &p])
        );
    }

    #[test]
    fn test_compute_constant_x_slope_none() {
        // constant x → Sxx=0 → both None
        let flat_x: Vec<Option<f64>> = vec![Some(1.0); 5];
        let p: Vec<Option<f64>> = vec![
            Some(100.0),
            Some(101.0),
            Some(102.0),
            Some(103.0),
            Some(104.0),
        ];
        let result = fls3().run_research(&[&flat_x, &p]);
        assert_eq!(result[0][0], None);
        assert_eq!(result[1][0], None);
    }

    #[test]
    fn test_compute_constant_y_r2_none() {
        // constant y → slope=0, R²=None
        let x = time_idx(5);
        let flat_y: Vec<Option<f64>> = vec![Some(100.0); 5];
        let result = fls3().run_research(&[&x, &flat_y]);
        assert!((result[0][0].unwrap() - 0.0).abs() < 1e-10);
        assert_eq!(result[1][0], None);
    }

    #[test]
    fn test_invalid_params() {
        assert!(matches!(
            FutureLinearSlope::new(vec!["t".into()], 3, vec!["s".into(), "r".into()]).unwrap_err(),
            OryonError::InvalidInput { ref msg } if msg.contains("inputs")
        ));
        assert!(matches!(
            FutureLinearSlope::new(vec!["t".into(), "".into()], 3, vec!["s".into(), "r".into()]).unwrap_err(),
            OryonError::InvalidInput { ref msg } if msg.contains("empty")
        ));
        assert!(matches!(
            FutureLinearSlope::new(vec!["t".into(), "close".into()], 1, vec!["s".into(), "r".into()]).unwrap_err(),
            OryonError::InvalidInput { ref msg } if msg.contains("horizon")
        ));
        assert!(matches!(
            FutureLinearSlope::new(vec!["t".into(), "close".into()], 3, vec!["slope".into()]).unwrap_err(),
            OryonError::InvalidInput { ref msg } if msg.contains("outputs")
        ));
    }
}
