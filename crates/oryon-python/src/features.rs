use oryon::features::Adf as RustAdf;
use oryon::features::BinMethod as RustBinMethod;
use oryon::features::Ema as RustEma;
use oryon::features::Kama as RustKama;
use oryon::features::Kurtosis as RustKurtosis;
use oryon::features::LinearSlope as RustLinearSlope;
use oryon::features::LogReturn as RustLogReturn;
use oryon::features::Mma as RustMma;
use oryon::features::ParkinsonVolatility as RustParkinsonVolatility;
use oryon::features::RogersSatchellVolatility as RustRogersSatchellVolatility;
use oryon::features::ShannonEntropy as RustShannonEntropy;
use oryon::features::SimpleReturn as RustSimpleReturn;
use oryon::features::Skewness as RustSkewness;
use oryon::features::Sma as RustSma;
use oryon::ops::AdfRegression;
use oryon::StreamingTransform;
use pyo3::prelude::*;

use crate::{to_python, to_rust, InvalidConfigError};

// --- Adf ---------------------------------------------------------------------

fn parse_regression(regression: &str) -> PyResult<AdfRegression> {
    match regression {
        "c" => Ok(AdfRegression::Constant),
        "ct" => Ok(AdfRegression::ConstantTrend),
        other => Err(InvalidConfigError::new_err(format!(
            "regression must be 'c' or 'ct', got '{other}'"
        ))),
    }
}

/// Rolling Augmented Dickey-Fuller test.
///
/// Produces two outputs per bar: the ADF statistic and its approximate p-value.
/// A very negative stat (e.g. below -3.5 for ``regression='c'``) and a small
/// p-value indicate stationarity (rejection of the unit-root null hypothesis).
///
/// P-values use the asymptotic MacKinnon (2010) distribution. Results are most
/// reliable for ``window >= 100``.
#[pyclass(module = "oryon")]
pub(crate) struct Adf {
    pub(crate) inner: RustAdf,
    window: usize,
    regression: String,
}

#[pymethods]
impl Adf {
    /// Create a new ``Adf``.
    ///
    /// Args:
    ///     inputs: Name of the input column (e.g. ``["close"]``).
    ///     window: Number of bars in the rolling window. Must satisfy
    ///         ``window > 3 + 2 * lags``.
    ///     outputs: Names of the two output columns ``[adf_stat_col, adf_pval_col]``.
    ///     lags: Number of lagged differences. ``None`` applies Schwert's rule
    ///         ``k = floor(12 * (window / 100) ** 0.25)``.
    ///     regression: ``'c'`` (constant only) or ``'ct'`` (constant + trend).
    #[new]
    #[pyo3(signature = (inputs, window, outputs, lags=None, regression="c"))]
    pub fn new(
        inputs: Vec<String>,
        window: usize,
        outputs: Vec<String>,
        lags: Option<usize>,
        regression: &str,
    ) -> PyResult<Self> {
        let reg = parse_regression(regression)?;
        let inner = RustAdf::new(inputs, window, outputs, lags, reg).map_err(crate::oryon_err)?;
        Ok(Adf {
            inner,
            window,
            regression: regression.to_string(),
        })
    }

    /// Process one bar. Returns ``[adf_stat, adf_pval]``, both ``NaN`` during
    /// warm-up, on ``NaN`` input, or when OLS is singular.
    fn update(&mut self, values: Vec<f64>) -> Vec<f64> {
        to_python(&self.inner.update(&to_rust(&values)))
    }

    /// Reset internal state (e.g. between CPCV splits).
    fn reset(&mut self) {
        self.inner.reset();
    }

    /// Input column names.
    fn input_names(&self) -> Vec<String> {
        self.inner.input_names()
    }

    /// Output column names.
    fn output_names(&self) -> Vec<String> {
        self.inner.output_names()
    }

    /// Number of bars before the first valid output.
    fn warm_up_period(&self) -> usize {
        self.inner.warm_up_period()
    }

    fn __repr__(&self) -> String {
        format!(
            "Adf(inputs={:?}, window={}, outputs={:?}, regression={:?})",
            self.inner.input_names(),
            self.window,
            self.inner.output_names(),
            self.regression,
        )
    }
}

// --- Mma ---------------------------------------------------------------------

/// Median Moving Average over a rolling window.
#[pyclass(module = "oryon")]
pub(crate) struct Mma {
    pub(crate) inner: RustMma,
}

#[pymethods]
impl Mma {
    /// Create a new ``Mma``.
    ///
    /// Args:
    ///     inputs: Name of the input column (e.g. ``["close"]``).
    ///     window: Number of bars in the rolling window. Must be >= 1.
    ///     outputs: Name of the output column (e.g. ``["close_mma_20"]``).
    #[new]
    pub fn new(inputs: Vec<String>, window: usize, outputs: Vec<String>) -> PyResult<Self> {
        let inner = RustMma::new(inputs, window, outputs).map_err(crate::oryon_err)?;
        Ok(Mma { inner })
    }

    /// Process one bar. Returns ``[NaN]`` during warm-up or on ``NaN`` input.
    fn update(&mut self, values: Vec<f64>) -> Vec<f64> {
        to_python(&self.inner.update(&to_rust(&values)))
    }

    /// Reset internal state (e.g. between CPCV splits).
    fn reset(&mut self) {
        self.inner.reset();
    }

    /// Input column names.
    fn input_names(&self) -> Vec<String> {
        self.inner.input_names()
    }

    /// Output column names.
    fn output_names(&self) -> Vec<String> {
        self.inner.output_names()
    }

    /// Number of bars before the first valid output.
    fn warm_up_period(&self) -> usize {
        self.inner.warm_up_period()
    }

    fn __repr__(&self) -> String {
        format!(
            "Mma(inputs={:?}, window={}, outputs={:?})",
            self.inner.input_names(),
            self.inner.warm_up_period() + 1,
            self.inner.output_names(),
        )
    }
}

// --- Sma ---------------------------------------------------------------------

/// Simple Moving Average over a rolling window.
#[pyclass(module = "oryon")]
pub(crate) struct Sma {
    pub(crate) inner: RustSma,
}

#[pymethods]
impl Sma {
    /// Create a new ``Sma``.
    ///
    /// Args:
    ///     inputs: Name of the input column (e.g. ``["close"]``).
    ///     window: Number of bars in the rolling window. Must be >= 1.
    ///     outputs: Name of the output column (e.g. ``["close_sma_20"]``).
    #[new]
    pub fn new(inputs: Vec<String>, window: usize, outputs: Vec<String>) -> PyResult<Self> {
        let inner = RustSma::new(inputs, window, outputs).map_err(crate::oryon_err)?;
        Ok(Sma { inner })
    }

    /// Process one bar. Returns ``[NaN]`` during warm-up or on ``NaN`` input.
    fn update(&mut self, values: Vec<f64>) -> Vec<f64> {
        to_python(&self.inner.update(&to_rust(&values)))
    }

    /// Reset internal state (e.g. between CPCV splits).
    fn reset(&mut self) {
        self.inner.reset();
    }

    /// Input column names.
    fn input_names(&self) -> Vec<String> {
        self.inner.input_names()
    }

    /// Output column names.
    fn output_names(&self) -> Vec<String> {
        self.inner.output_names()
    }

    /// Number of bars before the first valid output.
    fn warm_up_period(&self) -> usize {
        self.inner.warm_up_period()
    }

    fn __repr__(&self) -> String {
        format!(
            "Sma(inputs={:?}, window={}, outputs={:?})",
            self.inner.input_names(),
            self.inner.warm_up_period() + 1,
            self.inner.output_names(),
        )
    }
}

// --- Ema ---------------------------------------------------------------------

/// Exponential Moving Average with SMA seeding.
#[pyclass(module = "oryon")]
pub(crate) struct Ema {
    pub(crate) inner: RustEma,
}

#[pymethods]
impl Ema {
    /// Create a new ``Ema``.
    ///
    /// Args:
    ///     inputs: Name of the input column (e.g. ``["close"]``).
    ///     window: Number of bars for seeding and smoothing factor. Must be >= 1.
    ///     outputs: Name of the output column (e.g. ``["close_ema_20"]``).
    #[new]
    pub fn new(inputs: Vec<String>, window: usize, outputs: Vec<String>) -> PyResult<Self> {
        let inner = RustEma::new(inputs, window, outputs).map_err(crate::oryon_err)?;
        Ok(Ema { inner })
    }

    /// Process one bar. Returns ``[NaN]`` during warm-up or on ``NaN`` input.
    fn update(&mut self, values: Vec<f64>) -> Vec<f64> {
        to_python(&self.inner.update(&to_rust(&values)))
    }

    /// Reset internal state (e.g. between CPCV splits).
    fn reset(&mut self) {
        self.inner.reset();
    }

    /// Input column names.
    fn input_names(&self) -> Vec<String> {
        self.inner.input_names()
    }

    /// Output column names.
    fn output_names(&self) -> Vec<String> {
        self.inner.output_names()
    }

    /// Number of bars before the first valid output.
    fn warm_up_period(&self) -> usize {
        self.inner.warm_up_period()
    }

    fn __repr__(&self) -> String {
        format!(
            "Ema(inputs={:?}, window={}, outputs={:?})",
            self.inner.input_names(),
            self.inner.warm_up_period() + 1,
            self.inner.output_names(),
        )
    }
}

// --- Kama --------------------------------------------------------------------

/// Kaufman's Adaptive Moving Average.
#[pyclass(module = "oryon")]
pub(crate) struct Kama {
    pub(crate) inner: RustKama,
}

#[pymethods]
impl Kama {
    /// Create a new ``Kama``.
    ///
    /// Args:
    ///     inputs: Name of the input column (e.g. ``["close"]``).
    ///     window: Lookback for the Efficiency Ratio. Must be >= 1. Kaufman default: 10.
    ///     outputs: Name of the output column (e.g. ``["close_kama_10"]``).
    ///     fast: Period for the fast smoothing constant. Must be >= 1. Default: 2.
    ///     slow: Period for the slow smoothing constant. Must be > fast. Default: 30.
    #[new]
    #[pyo3(signature = (inputs, window, outputs, fast=2, slow=30))]
    pub fn new(
        inputs: Vec<String>,
        window: usize,
        outputs: Vec<String>,
        fast: usize,
        slow: usize,
    ) -> PyResult<Self> {
        let inner = RustKama::new(inputs, window, outputs, fast, slow).map_err(crate::oryon_err)?;
        Ok(Kama { inner })
    }

    /// Process one bar. Returns ``[NaN]`` during warm-up or on ``NaN`` input.
    fn update(&mut self, values: Vec<f64>) -> Vec<f64> {
        to_python(&self.inner.update(&to_rust(&values)))
    }

    /// Reset internal state (e.g. between CPCV splits).
    fn reset(&mut self) {
        self.inner.reset();
    }

    /// Input column names.
    fn input_names(&self) -> Vec<String> {
        self.inner.input_names()
    }

    /// Output column names.
    fn output_names(&self) -> Vec<String> {
        self.inner.output_names()
    }

    /// Number of bars before the first valid output.
    fn warm_up_period(&self) -> usize {
        self.inner.warm_up_period()
    }

    fn __repr__(&self) -> String {
        format!(
            "Kama(inputs={:?}, window={}, outputs={:?})",
            self.inner.input_names(),
            self.inner.warm_up_period(),
            self.inner.output_names(),
        )
    }
}

// --- SimpleReturn ------------------------------------------------------------

/// Simple (arithmetic) return over a configurable lookback window.
#[pyclass(module = "oryon")]
pub(crate) struct SimpleReturn {
    pub(crate) inner: RustSimpleReturn,
}

#[pymethods]
impl SimpleReturn {
    /// Create a new ``SimpleReturn``.
    ///
    /// Args:
    ///     inputs: Name of the input column (e.g. ``["close"]``).
    ///     window: Lookback in bars. Must be >= 1.
    ///     outputs: Name of the output column (e.g. ``["close_simple_return_5"]``).
    #[new]
    pub fn new(inputs: Vec<String>, window: usize, outputs: Vec<String>) -> PyResult<Self> {
        let inner = RustSimpleReturn::new(inputs, window, outputs).map_err(crate::oryon_err)?;
        Ok(SimpleReturn { inner })
    }

    /// Process one bar. Returns ``[NaN]`` during warm-up or on ``NaN`` input.
    fn update(&mut self, values: Vec<f64>) -> Vec<f64> {
        to_python(&self.inner.update(&to_rust(&values)))
    }

    /// Reset internal state (e.g. between CPCV splits).
    fn reset(&mut self) {
        self.inner.reset();
    }

    /// Input column names.
    fn input_names(&self) -> Vec<String> {
        self.inner.input_names()
    }

    /// Output column names.
    fn output_names(&self) -> Vec<String> {
        self.inner.output_names()
    }

    /// Number of bars before the first valid output.
    fn warm_up_period(&self) -> usize {
        self.inner.warm_up_period()
    }

    fn __repr__(&self) -> String {
        format!(
            "SimpleReturn(inputs={:?}, window={}, outputs={:?})",
            self.inner.input_names(),
            self.inner.warm_up_period(),
            self.inner.output_names(),
        )
    }
}

// --- LogReturn ---------------------------------------------------------------

/// Log return over a configurable lookback window.
#[pyclass(module = "oryon")]
pub(crate) struct LogReturn {
    pub(crate) inner: RustLogReturn,
}

#[pymethods]
impl LogReturn {
    /// Create a new ``LogReturn``.
    ///
    /// Args:
    ///     inputs: Name of the input column (e.g. ``["close"]``).
    ///     window: Lookback in bars. Must be >= 1.
    ///     outputs: Name of the output column (e.g. ``["close_log_return_5"]``).
    #[new]
    pub fn new(inputs: Vec<String>, window: usize, outputs: Vec<String>) -> PyResult<Self> {
        let inner = RustLogReturn::new(inputs, window, outputs).map_err(crate::oryon_err)?;
        Ok(LogReturn { inner })
    }

    /// Process one bar. Returns ``[NaN]`` during warm-up or on ``NaN`` input.
    fn update(&mut self, values: Vec<f64>) -> Vec<f64> {
        to_python(&self.inner.update(&to_rust(&values)))
    }

    /// Reset internal state (e.g. between CPCV splits).
    fn reset(&mut self) {
        self.inner.reset();
    }

    /// Input column names.
    fn input_names(&self) -> Vec<String> {
        self.inner.input_names()
    }

    /// Output column names.
    fn output_names(&self) -> Vec<String> {
        self.inner.output_names()
    }

    /// Number of bars before the first valid output.
    fn warm_up_period(&self) -> usize {
        self.inner.warm_up_period()
    }

    fn __repr__(&self) -> String {
        format!(
            "LogReturn(inputs={:?}, window={}, outputs={:?})",
            self.inner.input_names(),
            self.inner.warm_up_period(),
            self.inner.output_names(),
        )
    }
}

// --- Skewness ----------------------------------------------------------------

/// Rolling sample skewness (Fisher-Pearson corrected).
#[pyclass(module = "oryon")]
pub(crate) struct Skewness {
    pub(crate) inner: RustSkewness,
}

#[pymethods]
impl Skewness {
    /// Create a new ``Skewness``.
    ///
    /// Args:
    ///     inputs: Name of the input column (e.g. ``["close"]``).
    ///     window: Number of bars. Must be >= 3.
    ///     outputs: Name of the output column (e.g. ``["close_skewness_20"]``).
    #[new]
    pub fn new(inputs: Vec<String>, window: usize, outputs: Vec<String>) -> PyResult<Self> {
        let inner = RustSkewness::new(inputs, window, outputs).map_err(crate::oryon_err)?;
        Ok(Skewness { inner })
    }

    /// Process one bar. Returns ``[NaN]`` during warm-up or on ``NaN`` input.
    fn update(&mut self, values: Vec<f64>) -> Vec<f64> {
        to_python(&self.inner.update(&to_rust(&values)))
    }

    /// Reset internal state (e.g. between CPCV splits).
    fn reset(&mut self) {
        self.inner.reset();
    }

    /// Input column names.
    fn input_names(&self) -> Vec<String> {
        self.inner.input_names()
    }

    /// Output column names.
    fn output_names(&self) -> Vec<String> {
        self.inner.output_names()
    }

    /// Number of bars before the first valid output.
    fn warm_up_period(&self) -> usize {
        self.inner.warm_up_period()
    }

    fn __repr__(&self) -> String {
        format!(
            "Skewness(inputs={:?}, window={}, outputs={:?})",
            self.inner.input_names(),
            self.inner.warm_up_period() + 1,
            self.inner.output_names(),
        )
    }
}

// --- Kurtosis ----------------------------------------------------------------

/// Rolling excess kurtosis (Fisher).
#[pyclass(module = "oryon")]
pub(crate) struct Kurtosis {
    pub(crate) inner: RustKurtosis,
}

#[pymethods]
impl Kurtosis {
    /// Create a new ``Kurtosis``.
    ///
    /// Args:
    ///     inputs: Name of the input column (e.g. ``["close"]``).
    ///     window: Number of bars. Must be >= 4.
    ///     outputs: Name of the output column (e.g. ``["close_kurtosis_20"]``).
    #[new]
    pub fn new(inputs: Vec<String>, window: usize, outputs: Vec<String>) -> PyResult<Self> {
        let inner = RustKurtosis::new(inputs, window, outputs).map_err(crate::oryon_err)?;
        Ok(Kurtosis { inner })
    }

    /// Process one bar. Returns ``[NaN]`` during warm-up or on ``NaN`` input.
    fn update(&mut self, values: Vec<f64>) -> Vec<f64> {
        to_python(&self.inner.update(&to_rust(&values)))
    }

    /// Reset internal state (e.g. between CPCV splits).
    fn reset(&mut self) {
        self.inner.reset();
    }

    /// Input column names.
    fn input_names(&self) -> Vec<String> {
        self.inner.input_names()
    }

    /// Output column names.
    fn output_names(&self) -> Vec<String> {
        self.inner.output_names()
    }

    /// Number of bars before the first valid output.
    fn warm_up_period(&self) -> usize {
        self.inner.warm_up_period()
    }

    fn __repr__(&self) -> String {
        format!(
            "Kurtosis(inputs={:?}, window={}, outputs={:?})",
            self.inner.input_names(),
            self.inner.warm_up_period() + 1,
            self.inner.output_names(),
        )
    }
}

// --- LinearSlope -------------------------------------------------------------

/// Rolling OLS regression: slope and R² of y on x over a sliding window.
#[pyclass(module = "oryon")]
pub(crate) struct LinearSlope {
    pub(crate) inner: RustLinearSlope,
}

#[pymethods]
impl LinearSlope {
    /// Create a new ``LinearSlope``.
    ///
    /// Args:
    ///     inputs: Names of the x and y columns in that order (e.g. ``["time_idx", "close"]``).
    ///     window: Number of bars in the rolling window. Must be >= 2.
    ///     outputs: Names of the two output columns ``[slope_name, r2_name]``.
    #[new]
    pub fn new(inputs: Vec<String>, window: usize, outputs: Vec<String>) -> PyResult<Self> {
        let inner = RustLinearSlope::new(inputs, window, outputs).map_err(crate::oryon_err)?;
        Ok(LinearSlope { inner })
    }

    /// Process one bar. Returns ``[slope, r2]``, ``NaN`` during warm-up.
    fn update(&mut self, values: Vec<f64>) -> Vec<f64> {
        to_python(&self.inner.update(&to_rust(&values)))
    }

    /// Reset internal state (e.g. between CPCV splits).
    fn reset(&mut self) {
        self.inner.reset();
    }

    /// Input column names.
    fn input_names(&self) -> Vec<String> {
        self.inner.input_names()
    }

    /// Output column names.
    fn output_names(&self) -> Vec<String> {
        self.inner.output_names()
    }

    /// Number of bars before the first valid output.
    fn warm_up_period(&self) -> usize {
        self.inner.warm_up_period()
    }

    fn __repr__(&self) -> String {
        format!(
            "LinearSlope(inputs={:?}, window={}, outputs={:?})",
            self.inner.input_names(),
            self.inner.warm_up_period() + 1,
            self.inner.output_names(),
        )
    }
}

// --- ParkinsonVolatility -----------------------------------------------------

/// Rolling Parkinson volatility estimator (high/low).
#[pyclass(module = "oryon")]
pub(crate) struct ParkinsonVolatility {
    pub(crate) inner: RustParkinsonVolatility,
}

#[pymethods]
impl ParkinsonVolatility {
    /// Create a new ``ParkinsonVolatility``.
    ///
    /// Args:
    ///     inputs: Names of the high and low columns in that order (e.g. ``["high", "low"]``).
    ///     window: Number of bars in the rolling window. Must be >= 1.
    ///     outputs: Name of the output column (e.g. ``["parkinson_vol_20"]``).
    #[new]
    pub fn new(inputs: Vec<String>, window: usize, outputs: Vec<String>) -> PyResult<Self> {
        let inner =
            RustParkinsonVolatility::new(inputs, window, outputs).map_err(crate::oryon_err)?;
        Ok(ParkinsonVolatility { inner })
    }

    /// Process one bar. Returns ``[NaN]`` during warm-up or on invalid input.
    fn update(&mut self, values: Vec<f64>) -> Vec<f64> {
        to_python(&self.inner.update(&to_rust(&values)))
    }

    /// Reset internal state (e.g. between CPCV splits).
    fn reset(&mut self) {
        self.inner.reset();
    }

    /// Input column names.
    fn input_names(&self) -> Vec<String> {
        self.inner.input_names()
    }

    /// Output column names.
    fn output_names(&self) -> Vec<String> {
        self.inner.output_names()
    }

    /// Number of bars before the first valid output.
    fn warm_up_period(&self) -> usize {
        self.inner.warm_up_period()
    }

    fn __repr__(&self) -> String {
        format!(
            "ParkinsonVolatility(inputs={:?}, window={}, outputs={:?})",
            self.inner.input_names(),
            self.inner.warm_up_period() + 1,
            self.inner.output_names(),
        )
    }
}

// --- ShannonEntropy ----------------------------------------------------------

/// Rolling Shannon entropy over a fixed window.
///
/// Discretizes the last ``window`` values into equal-width bins and computes
/// ``H = -sum(p_i * ln(p_i))`` in nats. When ``normalize`` is ``True``,
/// outputs ``H / ln(n_bins)`` in [0, 1].
/// Returns ``NaN`` during warm-up or while any ``NaN`` is in the window.
#[pyclass(module = "oryon")]
pub(crate) struct ShannonEntropy {
    pub(crate) inner: RustShannonEntropy,
    window: usize,
    bins: Option<usize>,
    normalize: bool,
}

#[pymethods]
impl ShannonEntropy {
    /// Create a new ``ShannonEntropy``.
    ///
    /// Args:
    ///     inputs: Name of the input column (e.g. ``["returns"]``).
    ///     window: Number of bars in the rolling window. Must be >= 2.
    ///     outputs: Name of the output column (e.g. ``["returns_entropy_20"]``).
    ///     bins: Number of equal-width bins. Must be >= 2. ``None`` applies
    ///         Sturges' rule ``k = ceil(1 + log2(window))``. Default: ``None``.
    ///     normalize: If ``True``, output is ``H / ln(bins)`` in [0, 1]. Default: ``True``.
    #[new]
    #[pyo3(signature = (inputs, window, outputs, bins=None, normalize=true))]
    pub fn new(
        inputs: Vec<String>,
        window: usize,
        outputs: Vec<String>,
        bins: Option<usize>,
        normalize: bool,
    ) -> PyResult<Self> {
        let method = match bins {
            None => RustBinMethod::Sturges,
            Some(n) => RustBinMethod::FixedCount(n),
        };
        let inner = RustShannonEntropy::new(inputs, window, outputs, method, normalize)
            .map_err(crate::oryon_err)?;
        Ok(ShannonEntropy {
            inner,
            window,
            bins,
            normalize,
        })
    }

    /// Process one bar. Returns ``[NaN]`` during warm-up or while any ``NaN`` is in the window.
    fn update(&mut self, values: Vec<f64>) -> Vec<f64> {
        to_python(&self.inner.update(&to_rust(&values)))
    }

    /// Reset internal state (e.g. between CPCV splits).
    fn reset(&mut self) {
        self.inner.reset();
    }

    /// Input column names.
    fn input_names(&self) -> Vec<String> {
        self.inner.input_names()
    }

    /// Output column names.
    fn output_names(&self) -> Vec<String> {
        self.inner.output_names()
    }

    /// Number of bars before the first valid output.
    fn warm_up_period(&self) -> usize {
        self.inner.warm_up_period()
    }

    fn __repr__(&self) -> String {
        format!(
            "ShannonEntropy(inputs={:?}, window={}, outputs={:?}, bins={}, normalize={})",
            self.inner.input_names(),
            self.window,
            self.inner.output_names(),
            self.bins.map_or("None".to_string(), |n| n.to_string()),
            self.normalize,
        )
    }
}

// --- RogersSatchellVolatility ------------------------------------------------

/// Rolling Rogers-Satchell volatility estimator (OHLC).
#[pyclass(module = "oryon")]
pub(crate) struct RogersSatchellVolatility {
    pub(crate) inner: RustRogersSatchellVolatility,
}

#[pymethods]
impl RogersSatchellVolatility {
    /// Create a new ``RogersSatchellVolatility``.
    ///
    /// Args:
    ///     inputs: Names of the high, low, open, and close columns in that order.
    ///     window: Number of bars in the rolling window. Must be >= 1.
    ///     outputs: Name of the output column (e.g. ``["rs_vol_20"]``).
    #[new]
    pub fn new(inputs: Vec<String>, window: usize, outputs: Vec<String>) -> PyResult<Self> {
        let inner =
            RustRogersSatchellVolatility::new(inputs, window, outputs).map_err(crate::oryon_err)?;
        Ok(RogersSatchellVolatility { inner })
    }

    /// Process one bar. Returns ``[NaN]`` during warm-up or on invalid input.
    fn update(&mut self, values: Vec<f64>) -> Vec<f64> {
        to_python(&self.inner.update(&to_rust(&values)))
    }

    /// Reset internal state (e.g. between CPCV splits).
    fn reset(&mut self) {
        self.inner.reset();
    }

    /// Input column names.
    fn input_names(&self) -> Vec<String> {
        self.inner.input_names()
    }

    /// Output column names.
    fn output_names(&self) -> Vec<String> {
        self.inner.output_names()
    }

    /// Number of bars before the first valid output.
    fn warm_up_period(&self) -> usize {
        self.inner.warm_up_period()
    }

    fn __repr__(&self) -> String {
        format!(
            "RogersSatchellVolatility(inputs={:?}, window={}, outputs={:?})",
            self.inner.input_names(),
            self.inner.warm_up_period() + 1,
            self.inner.output_names(),
        )
    }
}
