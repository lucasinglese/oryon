use crate::error::OryonError;
use crate::ops::{adf_pvalue, adf_stat, AdfRegression};
use crate::traits::{Output, StreamingTransform};
use smallvec::smallvec;
use std::collections::VecDeque;

/// Rolling Augmented Dickey-Fuller test.
///
/// Computes the ADF test statistic and its approximate p-value over a rolling window.
/// Two outputs are produced per bar: the ADF statistic (output 0) and its p-value (output 1).
///
/// The regression model is:
/// - `Constant`:      `Δy_t = α + γ y_{t-1} + Σ_j δ_j Δy_{t-j} + ε`
/// - `ConstantTrend`: `Δy_t = α + β t + γ y_{t-1} + Σ_j δ_j Δy_{t-j} + ε`
///
/// A very negative stat (below -3.5 for `Constant`) gives strong evidence of stationarity
/// (rejection of H0: unit root). The p-value is the probability of observing a stat this
/// negative under H0.
///
/// When `lags` is `None`, Schwert's rule `k = floor(12 * (window / 100)^0.25)` is applied
/// once at construction. P-values use the asymptotic MacKinnon (2010) distribution and
/// are most reliable for `window >= 100`. Below that threshold the asymptotic approximation
/// becomes less accurate.
///
/// Returns `None` for both outputs during the first `window - 1` bars (warm-up),
/// when any value in the window is `None`, or when the OLS system is singular.
#[derive(Debug)]
pub struct Adf {
    inputs: Vec<String>,
    window: usize,
    lags: usize,
    regression: AdfRegression,
    outputs: Vec<String>,
    buffer: VecDeque<Option<f64>>,
}

impl Adf {
    /// Create a new `Adf`.
    ///
    /// - `inputs` - name of the input column (e.g. `["close"]`).
    /// - `window` - number of bars in the rolling window. Must be > 0 and satisfy
    ///   `window > 3 + 2 * lags` (enough observations for OLS with at least one
    ///   residual degree of freedom).
    /// - `outputs` - names of the two output columns: `[adf_stat_col, adf_pval_col]`.
    ///   Must contain exactly two names.
    /// - `lags` - number of lagged differences to include. `None` applies Schwert's rule
    ///   `k = floor(12 * (window / 100)^0.25)`.
    /// - `regression` - `Constant` (tests mean-stationarity) or `ConstantTrend`
    ///   (tests trend-stationarity).
    pub fn new(
        inputs: Vec<String>,
        window: usize,
        outputs: Vec<String>,
        lags: Option<usize>,
        regression: AdfRegression,
    ) -> Result<Self, OryonError> {
        if inputs.is_empty() {
            return Err(OryonError::InvalidInput {
                msg: "inputs must not be empty".into(),
            });
        }
        if outputs.len() != 2 {
            return Err(OryonError::InvalidInput {
                msg: "outputs must contain exactly two names: [adf_stat, adf_pvalue]".into(),
            });
        }
        if window == 0 {
            return Err(OryonError::InvalidInput {
                msg: "window must be non-zero".into(),
            });
        }

        let lags =
            lags.unwrap_or_else(|| (12.0 * (window as f64 / 100.0).powf(0.25)).floor() as usize);

        let min_window = 3 + 2 * lags;
        if window <= min_window {
            return Err(OryonError::InvalidInput {
                msg: format!("window ({window}) must be > 3 + 2 * lags ({lags}) = {min_window}"),
            });
        }

        Ok(Adf {
            inputs,
            window,
            lags,
            regression,
            outputs,
            buffer: VecDeque::with_capacity(window),
        })
    }
}

impl StreamingTransform for Adf {
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
            Adf::new(
                self.inputs.clone(),
                self.window,
                self.outputs.clone(),
                Some(self.lags),
                self.regression,
            )
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
            return smallvec![None, None];
        }

        match adf_stat(self.buffer.make_contiguous(), self.lags, self.regression) {
            Some(stat) => {
                let pval = adf_pvalue(stat, self.regression);
                smallvec![Some(stat), pval]
            }
            None => smallvec![None, None],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature_contract_tests;

    // window=20, lags=0 — minimal setup for contract tests.
    fn adf_w20() -> Adf {
        Adf::new(
            vec!["close".into()],
            20,
            vec!["close_adf_stat_20".into(), "close_adf_pval_20".into()],
            Some(0),
            AdfRegression::Constant,
        )
        .unwrap()
    }

    feature_contract_tests!(
        adf_w20(),
        vec!["close".to_string()],
        vec![
            "close_adf_stat_20".to_string(),
            "close_adf_pval_20".to_string()
        ],
        19, // warm_up = window - 1
        &[Some(1.0)],
    );

    // Reference series, same 20 bars used in ops tests, verified against statsmodels.
    const REF_X: [f64; 20] = [
        0.0000, 0.5087, -0.1558, 0.2507, 0.8633, 0.3085, 0.5017, 0.4578, -0.2826, 0.1437, 0.4694,
        -0.0066, 0.2960, 0.6133, -0.1088, 0.3521, 0.3786, 0.1477, 0.5707, 0.1324,
    ];

    fn feed(feature: &mut Adf, series: &[f64]) -> Output {
        let mut out = smallvec![None, None];
        for &v in series {
            out = feature.update(&[Some(v)]);
        }
        out
    }

    #[test]
    fn test_update() {
        // lags=0, regression='c'.
        // statsmodels: adfuller(REF_X, regression='c', maxlag=0, autolag=None)
        //   stat   = -5.656496589965162
        //   pvalue = mackinnonp(-5.656496589965162, regression='c') ≈ 1.0438e-06
        let mut adf = adf_w20();

        // warm-up: first 19 bars return None
        for &v in &REF_X[..19] {
            let out = adf.update(&[Some(v)]);
            assert_eq!(out[0], None);
            assert_eq!(out[1], None);
        }

        // 20th bar: first valid output
        let out = adf.update(&[Some(REF_X[19])]);
        let stat = out[0].unwrap();
        let pval = out[1].unwrap();

        assert!((stat - (-5.656496589965162)).abs() < 1e-10, "stat = {stat}");
        // pvalue verified via adf_pvalue op (MacKinnon 2010)
        assert!(pval > 0.0 && pval < 0.01, "pval = {pval}");
    }

    #[test]
    fn test_update_constant_trend() {
        // lags=0, regression='ct'.
        // statsmodels: adfuller(REF_X, regression='ct', maxlag=0, autolag=None)
        //   stat = -5.465768323608186
        let mut adf = Adf::new(
            vec!["close".into()],
            20,
            vec!["close_adf_stat_20".into(), "close_adf_pval_20".into()],
            Some(0),
            AdfRegression::ConstantTrend,
        )
        .unwrap();

        let out = feed(&mut adf, &REF_X);
        let stat = out[0].unwrap();
        assert!((stat - (-5.465768323608186)).abs() < 1e-10, "stat = {stat}");
        assert!(out[1].unwrap() < 0.01);
    }

    #[test]
    fn test_update_none_input() {
        // A None within the window propagates — both outputs are None.
        let mut adf = adf_w20();
        for &v in &REF_X[..19] {
            adf.update(&[Some(v)]);
        }
        // Replace last bar with None
        let out = adf.update(&[None]);
        assert_eq!(out[0], None);
        assert_eq!(out[1], None);
    }

    #[test]
    fn test_reset_clears_buffer() {
        let mut adf = adf_w20();
        feed(&mut adf, &REF_X);
        adf.reset();
        assert_eq!(adf.buffer.len(), 0);
        // After reset, warm-up restarts
        let out = adf.update(&[Some(1.0)]);
        assert_eq!(out[0], None);
        assert_eq!(out[1], None);
    }

    #[test]
    fn test_fresh_instances_are_independent() {
        let mut original = adf_w20();
        feed(&mut original, &REF_X);

        let mut fresh = original.fresh();

        // fresh starts from scratch — first bar should be None
        let out = fresh.update(&[Some(1.0)]);
        assert_eq!(out[0], None);

        // original still produces valid output on next bar
        let out = original.update(&[Some(REF_X[0])]);
        assert!(out[0].is_some());
    }

    #[test]
    fn test_schwert_rule_applied_when_lags_is_none() {
        // window=100 → k = floor(12 * (100/100)^0.25) = 12
        let adf = Adf::new(
            vec!["close".into()],
            100,
            vec!["adf_stat".into(), "adf_pval".into()],
            None,
            AdfRegression::Constant,
        )
        .unwrap();
        assert_eq!(adf.lags, 12);
        assert_eq!(adf.warm_up_period(), 99);
    }

    #[test]
    fn test_error_raises_when_empty_inputs() {
        let err = Adf::new(
            vec![],
            20,
            vec!["s".into(), "p".into()],
            Some(0),
            AdfRegression::Constant,
        )
        .unwrap_err();
        assert!(matches!(err, OryonError::InvalidInput { ref msg } if msg.contains("inputs")));
    }

    #[test]
    fn test_error_raises_when_wrong_output_count() {
        let err = Adf::new(
            vec!["close".into()],
            20,
            vec!["only_one".into()],
            Some(0),
            AdfRegression::Constant,
        )
        .unwrap_err();
        assert!(matches!(err, OryonError::InvalidInput { ref msg } if msg.contains("outputs")));
    }

    #[test]
    fn test_error_raises_when_window_is_zero() {
        let err = Adf::new(
            vec!["close".into()],
            0,
            vec!["s".into(), "p".into()],
            Some(0),
            AdfRegression::Constant,
        )
        .unwrap_err();
        assert!(matches!(err, OryonError::InvalidInput { ref msg } if msg.contains("window")));
    }

    #[test]
    fn test_error_raises_when_window_too_small_for_lags() {
        // lags=5 → need window > 13. window=13 should fail.
        let err = Adf::new(
            vec!["close".into()],
            13,
            vec!["s".into(), "p".into()],
            Some(5),
            AdfRegression::Constant,
        )
        .unwrap_err();
        assert!(matches!(err, OryonError::InvalidInput { ref msg } if msg.contains("window")));
    }
}
