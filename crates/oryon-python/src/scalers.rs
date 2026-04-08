use oryon::fitting::{fit_standard_scaler as rust_fit, StandardScalerParams};
use oryon::scalers::FixedZScore as RustFixedZScore;
use oryon::scalers::RollingZScore as RustRollingZScore;
use oryon::StreamingTransform;
use pyo3::prelude::*;

use crate::{to_python, to_rust};

// --- fit_standard_scaler (function) ------------------------------------------

/// Compute mean and std from a column of data.
///
/// Args:
///     data: List of floats. Use ``float('nan')`` for missing values.
///
/// Returns:
///     Tuple ``(mean, std)``.
#[pyfunction]
pub(crate) fn fit_standard_scaler(data: Vec<f64>) -> PyResult<(f64, f64)> {
    let rust_data = to_rust(&data);
    let params = rust_fit(&rust_data).map_err(crate::oryon_err)?;
    Ok((params.mean, params.std))
}

// --- RollingZScore -----------------------------------------------------------

/// Rolling z-score normalization over a sliding window.
#[pyclass(module = "oryon")]
pub(crate) struct RollingZScore {
    pub(crate) inner: RustRollingZScore,
}

#[pymethods]
impl RollingZScore {
    /// Create a new ``RollingZScore``.
    ///
    /// Args:
    ///     inputs: Name of the input column (e.g. ``["close_sma_20"]``). Must contain exactly 1 entry.
    ///     window: Number of bars for rolling statistics. Must be >= 2.
    ///     outputs: Name of the output column (e.g. ``["close_sma_20_z"]``).
    #[new]
    pub fn new(inputs: Vec<String>, window: usize, outputs: Vec<String>) -> PyResult<Self> {
        let inner = RustRollingZScore::new(inputs, window, outputs).map_err(crate::oryon_err)?;
        Ok(RollingZScore { inner })
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
            "RollingZScore(inputs={:?}, window={}, outputs={:?})",
            self.inner.input_names(),
            self.inner.warm_up_period() + 1,
            self.inner.output_names(),
        )
    }
}

// --- FixedZScore -------------------------------------------------------------

/// Z-score normalization with pre-fitted mean and std.
#[pyclass(module = "oryon")]
pub(crate) struct FixedZScore {
    pub(crate) inner: RustFixedZScore,
}

#[pymethods]
impl FixedZScore {
    /// Create a new ``FixedZScore``.
    ///
    /// Args:
    ///     inputs: Name of the input column. Must contain exactly 1 entry.
    ///     outputs: Name of the output column.
    ///     mean: Pre-fitted mean (from ``fit_standard_scaler``).
    ///     std: Pre-fitted std (from ``fit_standard_scaler``). Must be > 0.
    #[new]
    pub fn new(inputs: Vec<String>, outputs: Vec<String>, mean: f64, std: f64) -> PyResult<Self> {
        let params = StandardScalerParams { mean, std };
        let inner = RustFixedZScore::new(inputs, outputs, params).map_err(crate::oryon_err)?;
        Ok(FixedZScore { inner })
    }

    /// Process one bar. Returns ``[NaN]`` if input is ``NaN``.
    fn update(&mut self, values: Vec<f64>) -> Vec<f64> {
        to_python(&self.inner.update(&to_rust(&values)))
    }

    /// Reset (no-op for stateless scalers).
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
            "FixedZScore(inputs={:?}, outputs={:?})",
            self.inner.input_names(),
            self.inner.output_names(),
        )
    }
}
