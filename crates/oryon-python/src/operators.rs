use oryon::operators::NegLog as RustNegLog;
use oryon::operators::Subtract as RustSubtract;
use oryon::StreamingTransform;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use crate::{to_python, to_rust};

// --- Subtract ----------------------------------------------------------------

/// Element-wise subtraction: A - B.
#[pyclass(module = "oryon")]
pub(crate) struct Subtract {
    pub(crate) inner: RustSubtract,
}

#[pymethods]
impl Subtract {
    /// Create a new ``Subtract``.
    ///
    /// Args:
    ///     inputs: Names of the two input columns ``[A, B]``.
    ///     outputs: Name of the output column (e.g. ``["spread"]``).
    #[new]
    pub fn new(inputs: Vec<String>, outputs: Vec<String>) -> PyResult<Self> {
        let inner =
            RustSubtract::new(inputs, outputs).map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok(Subtract { inner })
    }

    /// Process one bar. Returns ``[NaN]`` if either input is ``NaN``.
    fn update(&mut self, values: Vec<f64>) -> Vec<f64> {
        to_python(&self.inner.update(&to_rust(&values)))
    }

    /// Reset (no-op for stateless operators).
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
            "Subtract(inputs={:?}, outputs={:?})",
            self.inner.input_names(),
            self.inner.output_names(),
        )
    }
}

// --- NegLog ------------------------------------------------------------------

/// Negative natural logarithm: -ln(x).
#[pyclass(module = "oryon")]
pub(crate) struct NegLog {
    pub(crate) inner: RustNegLog,
}

#[pymethods]
impl NegLog {
    /// Create a new ``NegLog``.
    ///
    /// Args:
    ///     inputs: Name of the input column (e.g. ``["pvalue"]``). Must contain exactly 1 entry.
    ///     outputs: Name of the output column (e.g. ``["neg_log_pvalue"]``).
    #[new]
    pub fn new(inputs: Vec<String>, outputs: Vec<String>) -> PyResult<Self> {
        let inner =
            RustNegLog::new(inputs, outputs).map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok(NegLog { inner })
    }

    /// Process one bar. Returns ``[NaN]`` if input is ``NaN`` or <= 0.
    fn update(&mut self, values: Vec<f64>) -> Vec<f64> {
        to_python(&self.inner.update(&to_rust(&values)))
    }

    /// Reset (no-op for stateless operators).
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
            "NegLog(inputs={:?}, outputs={:?})",
            self.inner.input_names(),
            self.inner.output_names(),
        )
    }
}
