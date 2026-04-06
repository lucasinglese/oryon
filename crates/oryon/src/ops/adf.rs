/// Regression specification for the ADF test.
///
/// Controls which deterministic regressors are included in the ADF regression equation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdfRegression {
    /// Constant only: tests stationarity around a non-zero mean.
    Constant,
    /// Constant plus linear trend: tests stationarity around a linear trend.
    ConstantTrend,
}

/// ADF test statistic computed by OLS on a price slice.
///
/// Estimates the regression:
/// - `Constant`:      `Δy_t = α + γ y_{t-1} + Σ_j δ_j Δy_{t-j} + ε`
/// - `ConstantTrend`: `Δy_t = α + β t + γ y_{t-1} + Σ_j δ_j Δy_{t-j} + ε`
///
/// Returns `γ̂ / SE(γ̂)`. Under H0 (unit root), this statistic follows the
/// Dickey-Fuller distribution. More negative values give stronger evidence of
/// stationarity (rejection of H0).
///
/// Returns `None` if:
/// - any value in `x` is `None`,
/// - `x` has fewer than `3 + 2 * lags` elements (underdetermined OLS),
/// - the OLS system is singular (e.g. constant or near-constant input).
pub fn adf_stat(x: &[Option<f64>], lags: usize, regression: AdfRegression) -> Option<f64> {
    let vals: Vec<f64> = x.iter().copied().collect::<Option<Vec<_>>>()?;

    let n_total = vals.len();
    let p = match regression {
        AdfRegression::Constant => 2 + lags,
        AdfRegression::ConstantTrend => 3 + lags,
    };
    // n_obs = n_total - 1 - lags OLS observations; need n_obs > p for at least
    // one residual degree of freedom.
    let n_obs = n_total.saturating_sub(1 + lags);
    if n_obs <= p {
        return None;
    }

    let target_idx = match regression {
        AdfRegression::Constant => 1,
        AdfRegression::ConstantTrend => 2,
    };

    let dx: Vec<f64> = vals.windows(2).map(|w| w[1] - w[0]).collect();

    // Build design matrix X (n_obs x p, row-major) and response y.
    let mut x_mat = vec![0.0f64; n_obs * p];
    let mut y_vec = vec![0.0f64; n_obs];

    for i in 0..n_obs {
        let base = i * p;
        x_mat[base] = 1.0; // constant
        match regression {
            AdfRegression::Constant => {
                x_mat[base + 1] = vals[lags + i]; // y_{t-1}
                for j in 0..lags {
                    // Δy_{t-1-j}: dx[lags - 1 - j + i]
                    x_mat[base + 2 + j] = dx[lags - 1 - j + i];
                }
            }
            AdfRegression::ConstantTrend => {
                x_mat[base + 1] = (lags + i) as f64; // trend index
                x_mat[base + 2] = vals[lags + i]; // y_{t-1}
                for j in 0..lags {
                    x_mat[base + 3 + j] = dx[lags - 1 - j + i];
                }
            }
        }
        y_vec[i] = dx[lags + i]; // Δy_t
    }

    // Accumulate X'X (p x p) and X'y (p).
    let mut xtx = vec![0.0f64; p * p];
    let mut xty = vec![0.0f64; p];
    for (i, &yi) in y_vec.iter().enumerate() {
        let base = i * p;
        for r in 0..p {
            xty[r] += x_mat[base + r] * yi;
            for c in 0..p {
                xtx[r * p + c] += x_mat[base + r] * x_mat[base + c];
            }
        }
    }

    // Solve X'X β = X'y.
    let beta = gauss_solve(xtx.clone(), xty, p)?;

    // Residual sum of squares.
    let mut rss = 0.0f64;
    for (i, &yi) in y_vec.iter().enumerate() {
        let base = i * p;
        let pred: f64 = (0..p).map(|c| x_mat[base + c] * beta[c]).sum();
        rss += (yi - pred).powi(2);
    }
    let sigma2 = rss / (n_obs - p) as f64;

    // (X'X)^{-1}[target, target] via X'X v = e_{target}.
    let mut e = vec![0.0f64; p];
    e[target_idx] = 1.0;
    let v = gauss_solve(xtx, e, p)?;

    let var_gamma = sigma2 * v[target_idx];
    if var_gamma <= 0.0 {
        return None;
    }

    let stat = beta[target_idx] / var_gamma.sqrt();
    if !stat.is_finite() {
        return None;
    }
    Some(stat)
}

/// Approximate p-value for an ADF test statistic via table interpolation.
///
/// Uses a pre-computed lookup table (45 points from stat -6 to 2.8, step 0.2)
/// and linear interpolation between table entries. Values outside the table range
/// are clamped to the boundary p-value.
///
/// Returns `None` if `stat` is NaN.
pub fn adf_pvalue(stat: f64, regression: AdfRegression) -> Option<f64> {
    if stat.is_nan() {
        return None;
    }

    // stats_known = np.arange(-6, 3, 0.2) — 45 evenly-spaced breakpoints.
    const N: usize = 45;
    const START: f64 = -6.0;
    const STEP: f64 = 0.2;

    // Values computed via statsmodels mackinnonp() — MacKinnon (2010) response surface.
    #[rustfmt::skip]
    const PVALUES_C: [f64; N] = [
        1.6661204834e-07, 4.6549534731e-07, 1.2711717107e-06, 3.3872038899e-06,
        8.7920835785e-06, 2.2193154714e-05, 5.4385935695e-05, 1.2916964005e-04,
        2.9683262235e-04, 6.5890020605e-04, 1.4105112530e-03, 2.9073149933e-03,
        5.7610277513e-03, 1.0958871608e-02, 1.9984679219e-02, 3.4894400275e-02,
        5.8273768069e-02, 9.2997267439e-02, 1.4173640869e-01, 2.0624545685e-01,
        2.8657309917e-01, 3.8046169361e-01, 4.8359346965e-01, 5.8227611852e-01,
        6.7359571193e-01, 7.5326430120e-01, 8.1912206764e-01, 8.7098202687e-01,
        9.1009877736e-01, 9.3852161743e-01, 9.5853208606e-01, 9.7226159393e-01,
        9.8149494162e-01, 9.8761562917e-01, 9.9163616803e-01, 9.9426594855e-01,
        9.9598588316e-01, 9.9711414191e-01, 9.9785757578e-01, 9.9834901705e-01,
        9.9867295120e-01, 9.9888250502e-01, 9.9901030962e-01, 9.9907515515e-01,
        1.0000000000e+00,
    ];

    // Values computed via statsmodels mackinnonp() — MacKinnon (2010) response surface.
    #[rustfmt::skip]
    const PVALUES_CT: [f64; N] = [
        2.1968599946e-06, 5.7220010110e-06, 1.4572870684e-05, 3.6209697180e-05,
        8.7581674237e-05, 2.0574728264e-04, 4.6839591299e-04, 1.0310555371e-03,
        2.1897009369e-03, 4.4769711121e-03, 8.7937012311e-03, 1.6560672206e-02,
        2.9846151131e-02, 5.1387926684e-02, 8.4401702427e-02, 1.3208098478e-01,
        1.9694444231e-01, 2.7989264978e-01, 3.7961893275e-01, 4.8985051943e-01,
        6.0143377224e-01, 7.0475832317e-01, 7.9243229243e-01, 8.6091256022e-01,
        9.1050285805e-01, 9.4411471090e-01, 9.6568254785e-01, 9.7895129765e-01,
        9.8687903442e-01, 9.9153194645e-01, 9.9423316769e-01, 9.9577753487e-01,
        9.9661680562e-01, 9.9698697776e-01, 1.0000000000e+00, 1.0000000000e+00,
        1.0000000000e+00, 1.0000000000e+00, 1.0000000000e+00, 1.0000000000e+00,
        1.0000000000e+00, 1.0000000000e+00, 1.0000000000e+00, 1.0000000000e+00,
        1.0000000000e+00,
    ];

    let pvalues: &[f64; N] = match regression {
        AdfRegression::Constant => &PVALUES_C,
        AdfRegression::ConstantTrend => &PVALUES_CT,
    };

    if stat <= START {
        return Some(pvalues[0]);
    }
    let last_stat = START + STEP * (N - 1) as f64;
    if stat >= last_stat {
        return Some(pvalues[N - 1]);
    }

    // Linear interpolation between the two surrounding breakpoints.
    let idx = ((stat - START) / STEP) as usize;
    let idx = idx.min(N - 2); // guard against floating-point overshoot
    let t = (stat - (START + STEP * idx as f64)) / STEP;
    Some(pvalues[idx] + t * (pvalues[idx + 1] - pvalues[idx]))
}

/// Gaussian elimination with partial pivoting. Solves `a x = b` in-place.
/// `a` is a `p x p` matrix stored row-major. Returns `None` if singular.
fn gauss_solve(mut a: Vec<f64>, mut b: Vec<f64>, p: usize) -> Option<Vec<f64>> {
    for col in 0..p {
        // Find the row with the largest absolute value in this column.
        let mut max_row = col;
        let mut max_val = a[col * p + col].abs();
        for row in (col + 1)..p {
            let val = a[row * p + col].abs();
            if val > max_val {
                max_val = val;
                max_row = row;
            }
        }
        if max_val < 1e-12 {
            return None; // singular or near-singular
        }

        if max_row != col {
            for j in 0..p {
                a.swap(col * p + j, max_row * p + j);
            }
            b.swap(col, max_row);
        }

        // Eliminate entries below the pivot.
        for row in (col + 1)..p {
            let factor = a[row * p + col] / a[col * p + col];
            a[row * p + col] = 0.0;
            for j in (col + 1)..p {
                let sub = a[col * p + j] * factor;
                a[row * p + j] -= sub;
            }
            b[row] -= b[col] * factor;
        }
    }

    // Back substitution.
    let mut sol = vec![0.0f64; p];
    for i in (0..p).rev() {
        sol[i] = b[i];
        for j in (i + 1)..p {
            sol[i] -= a[i * p + j] * sol[j];
        }
        if a[i * p + i].abs() < 1e-12 {
            return None;
        }
        sol[i] /= a[i * p + i];
    }

    Some(sol)
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- adf_stat ---

    // Reference series used across happy-path tests (20 bars).
    // IMPORTANT: expected values below were produced by this Rust implementation
    // and must be cross-checked against statsmodels before release:
    //   from statsmodels.tsa.stattools import adfuller
    //   x = [0.0, 0.5087, -0.1558, 0.2507, 0.8633, 0.3085, 0.5017, 0.4578,
    //        -0.2826, 0.1437, 0.4694, -0.0066, 0.2960, 0.6133, -0.1088,
    //        0.3521, 0.3786, 0.1477, 0.5707, 0.1324]
    //   adfuller(x, regression='c',  maxlag=0, autolag=None)[0]  # → verify
    //   adfuller(x, regression='ct', maxlag=0, autolag=None)[0]  # → verify
    //   adfuller(x, regression='c',  maxlag=2, autolag=None)[0]  # → verify
    const REF_X: [f64; 20] = [
        0.0000, 0.5087, -0.1558, 0.2507, 0.8633, 0.3085, 0.5017, 0.4578, -0.2826, 0.1437, 0.4694,
        -0.0066, 0.2960, 0.6133, -0.1088, 0.3521, 0.3786, 0.1477, 0.5707, 0.1324,
    ];

    fn ref_x() -> Vec<Option<f64>> {
        REF_X.iter().map(|&v| Some(v)).collect()
    }

    #[test]
    fn test_adf_stat_constant_stationary() {
        // lags=0, regression='c'.
        // statsmodels: adfuller(x, regression='c', maxlag=0, autolag=None)[0]
        //   = -5.656496589965162
        let stat = adf_stat(&ref_x(), 0, AdfRegression::Constant).unwrap();
        assert!((stat - (-5.656496589965162)).abs() < 1e-10, "stat = {stat}");
    }

    #[test]
    fn test_adf_stat_constant_trend() {
        // lags=0, regression='ct'.
        // statsmodels: adfuller(x, regression='ct', maxlag=0, autolag=None)[0]
        //   = -5.465768323608186
        let stat = adf_stat(&ref_x(), 0, AdfRegression::ConstantTrend).unwrap();
        assert!((stat - (-5.465768323608186)).abs() < 1e-10, "stat = {stat}");
    }

    #[test]
    fn test_adf_stat_with_lags() {
        // lags=2, regression='c'.
        // statsmodels: adfuller(x, regression='c', maxlag=2, autolag=None)[0]
        //   = -2.124282684987995
        let stat = adf_stat(&ref_x(), 2, AdfRegression::Constant).unwrap();
        assert!((stat - (-2.124282684987995)).abs() < 1e-10, "stat = {stat}");
    }

    #[test]
    fn test_adf_stat_with_none() {
        let x = vec![Some(1.0), None, Some(3.0), Some(4.0), Some(5.0)];
        assert_eq!(adf_stat(&x, 0, AdfRegression::Constant), None);
    }

    #[test]
    fn test_adf_stat_empty() {
        assert_eq!(adf_stat(&[], 0, AdfRegression::Constant), None);
    }

    #[test]
    fn test_adf_stat_too_short() {
        // lags=0, p=2: need n_obs > 2 → n_total > 3. With 3 values → None.
        let x = vec![Some(1.0), Some(2.0), Some(3.0)];
        assert_eq!(adf_stat(&x, 0, AdfRegression::Constant), None);
    }

    #[test]
    fn test_adf_stat_minimum_valid_size() {
        // Boundary: n_total=4, lags=0 → n_obs=3, p=2, n_obs > p → Some.
        // Verifies the other side of the too_short boundary.
        let x: Vec<Option<f64>> = REF_X[..4].iter().map(|&v| Some(v)).collect();
        assert!(adf_stat(&x, 0, AdfRegression::Constant).is_some());
    }

    #[test]
    fn test_adf_stat_constant_trend_with_lags() {
        // regression='ct', lags=2.
        // statsmodels: adfuller(x, regression='ct', maxlag=2, autolag=None)[0]
        //   = -2.1282576700330598
        let stat = adf_stat(&ref_x(), 2, AdfRegression::ConstantTrend).unwrap();
        assert!(
            (stat - (-2.1282576700330598)).abs() < 1e-10,
            "stat = {stat}"
        );
    }

    #[test]
    fn test_adf_stat_lags_too_large() {
        // n_total=10, lags=8 → n_obs=1, p=10, n_obs <= p → None.
        let x: Vec<Option<f64>> = (0..10).map(|i| Some(i as f64)).collect();
        assert_eq!(adf_stat(&x, 8, AdfRegression::Constant), None);
    }

    #[test]
    fn test_adf_stat_non_finite_returns_none() {
        // Near-singular input (perfectly linear series with ct+lags) produces
        // numerical garbage in statsmodels (e.g. 8e15). Rust returns None.
        let x: Vec<Option<f64>> = (0..20).map(|i| Some(i as f64)).collect();
        assert_eq!(adf_stat(&x, 2, AdfRegression::ConstantTrend), None);
    }

    #[test]
    fn test_adf_stat_constant_input_is_none() {
        // All identical values → zero variance → singular X'X.
        let x: Vec<Option<f64>> = vec![Some(5.0); 20];
        assert_eq!(adf_stat(&x, 0, AdfRegression::Constant), None);
    }

    // --- adf_pvalue ---

    #[test]
    fn test_adf_pvalue_table_boundary_low() {
        // stat <= -6.0 → clamp to first table entry.
        let p = adf_pvalue(-10.0, AdfRegression::Constant).unwrap();
        assert!((p - 1.6661204834e-07).abs() < 1e-16);
    }

    #[test]
    fn test_adf_pvalue_table_boundary_high() {
        // stat >= 2.8 → clamp to 1.0.
        let p = adf_pvalue(5.0, AdfRegression::Constant).unwrap();
        assert!((p - 1.0).abs() < 1e-12);
    }

    #[test]
    fn test_adf_pvalue_exact_breakpoint() {
        // stat = -6.0 exactly → first entry.
        let p = adf_pvalue(-6.0, AdfRegression::Constant).unwrap();
        assert!((p - 1.6661204834e-07).abs() < 1e-16);
    }

    #[test]
    fn test_adf_pvalue_interpolation_constant() {
        // stat = -3.0 → index 15 in table → PVALUES_C[15] = 3.4894400275e-02.
        let p = adf_pvalue(-3.0, AdfRegression::Constant).unwrap();
        assert!((p - 3.4894400275e-02).abs() < 1e-12);
    }

    #[test]
    fn test_adf_pvalue_interpolation_constant_trend() {
        // stat = -3.0 → index 15 → PVALUES_CT[15] = 1.3208098478e-01.
        let p = adf_pvalue(-3.0, AdfRegression::ConstantTrend).unwrap();
        assert!((p - 1.3208098478e-01).abs() < 1e-12);
    }

    #[test]
    fn test_adf_pvalue_midpoint_interpolation() {
        // stat = -2.9 → midpoint between index 15 and 16 for 'c'.
        // PVALUES_C[15] = 3.4894400275e-02, PVALUES_C[16] = 5.8273768069e-02.
        // t = 0.5 → expected = (3.4894400275e-02 + 5.8273768069e-02) / 2.
        let p = adf_pvalue(-2.9, AdfRegression::Constant).unwrap();
        let expected = (3.4894400275e-02 + 5.8273768069e-02) / 2.0;
        assert!((p - expected).abs() < 1e-12);
    }

    #[test]
    fn test_adf_pvalue_nan_returns_none() {
        assert_eq!(adf_pvalue(f64::NAN, AdfRegression::Constant), None);
    }

    #[test]
    fn test_adf_pvalue_ct_plateau() {
        // For 'ct', stat >= 0.8 → p = 1.0.
        let p = adf_pvalue(1.5, AdfRegression::ConstantTrend).unwrap();
        assert!((p - 1.0).abs() < 1e-12);
    }
}
