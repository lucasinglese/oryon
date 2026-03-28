use oryon::targets::FutureCTCVolatility as RustFutureCTCVolatility;
use oryon::targets::FutureLinearSlope as RustFutureLinearSlope;
use oryon::targets::FutureReturn as RustFutureReturn;
use oryon::Target;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

// --- FutureReturn ------------------------------------------------------------

/// Future simple return over ``horizon`` bars.
#[pyclass(module = "oryon")]
pub(crate) struct FutureReturn {
    pub(crate) inputs: Vec<String>,
    pub(crate) horizon: usize,
    pub(crate) outputs: Vec<String>,
}

#[pymethods]
impl FutureReturn {
    /// Create a new ``FutureReturn`` target.
    ///
    /// Args:
    ///     inputs: Price series name (e.g. ``["close"]``).
    ///     horizon: Number of bars to look ahead. Must be >= 1.
    ///     outputs: Name of the output column (e.g. ``["close_future_return_5"]``).
    #[new]
    pub fn new(inputs: Vec<String>, horizon: usize, outputs: Vec<String>) -> PyResult<Self> {
        RustFutureReturn::new(inputs.clone(), horizon, outputs.clone())
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok(FutureReturn { inputs, horizon, outputs })
    }

    /// Input column names.
    fn input_names(&self) -> Vec<String> {
        self.inputs.clone()
    }

    /// Output column names.
    fn output_names(&self) -> Vec<String> {
        self.outputs.clone()
    }

    /// Number of bars at the end that will be ``NaN``.
    fn forward_period(&self) -> usize {
        self.horizon
    }

    fn __repr__(&self) -> String {
        format!(
            "FutureReturn(inputs={:?}, horizon={}, outputs={:?})",
            self.inputs, self.horizon, self.outputs,
        )
    }
}

// --- FutureCTCVolatility -----------------------------------------------------

/// Future close-to-close realized volatility.
#[pyclass(module = "oryon")]
pub(crate) struct FutureCTCVolatility {
    pub(crate) input: String,
    pub(crate) horizon: usize,
    pub(crate) output: String,
}

#[pymethods]
impl FutureCTCVolatility {
    /// Create a new ``FutureCTCVolatility`` target.
    ///
    /// Args:
    ///     input: Price series name (e.g. ``"close"``).
    ///     horizon: Number of bars to look ahead. Must be >= 1.
    ///
    /// The output column name is auto-generated as ``"{input}_future_ctc_vol_{horizon}"``.
    #[new]
    pub fn new(input: String, horizon: usize) -> PyResult<Self> {
        let rust = RustFutureCTCVolatility::new(&input, horizon)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        let output = rust.output_names()[0].clone();
        Ok(FutureCTCVolatility { input, horizon, output })
    }

    /// Input column names.
    fn input_names(&self) -> Vec<String> {
        vec![self.input.clone()]
    }

    /// Output column names.
    fn output_names(&self) -> Vec<String> {
        vec![self.output.clone()]
    }

    /// Number of bars at the end that will be ``NaN``.
    fn forward_period(&self) -> usize {
        self.horizon
    }

    fn __repr__(&self) -> String {
        format!(
            "FutureCTCVolatility(input={:?}, horizon={})",
            self.input, self.horizon,
        )
    }
}

// --- FutureLinearSlope -------------------------------------------------------

/// Future OLS slope and R² of y on x over the next ``horizon`` bars.
#[pyclass(module = "oryon")]
pub(crate) struct FutureLinearSlope {
    pub(crate) inputs: Vec<String>,
    pub(crate) horizon: usize,
    pub(crate) outputs: Vec<String>,
}

#[pymethods]
impl FutureLinearSlope {
    /// Create a new ``FutureLinearSlope`` target.
    ///
    /// Args:
    ///     inputs: Names of the x and y columns in that order (e.g. ``["time_idx", "close"]``).
    ///     horizon: Number of bars to look ahead. Must be >= 2.
    ///     outputs: Names of the two output columns ``[slope_name, r2_name]``.
    #[new]
    pub fn new(inputs: Vec<String>, horizon: usize, outputs: Vec<String>) -> PyResult<Self> {
        RustFutureLinearSlope::new(inputs.clone(), horizon, outputs.clone())
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok(FutureLinearSlope { inputs, horizon, outputs })
    }

    /// Input column names.
    fn input_names(&self) -> Vec<String> {
        self.inputs.clone()
    }

    /// Output column names (slope, r2).
    fn output_names(&self) -> Vec<String> {
        self.outputs.clone()
    }

    /// Number of bars at the end that will be ``NaN``.
    fn forward_period(&self) -> usize {
        self.horizon
    }

    fn __repr__(&self) -> String {
        format!(
            "FutureLinearSlope(inputs={:?}, horizon={}, outputs={:?})",
            self.inputs, self.horizon, self.outputs,
        )
    }
}
