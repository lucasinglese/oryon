use oryon::operators::Add as RustAdd;
use oryon::operators::Divide as RustDivide;
use oryon::operators::Log as RustLog;
use oryon::operators::Logit as RustLogit;
use oryon::operators::Multiply as RustMultiply;
use oryon::operators::NegLog as RustNegLog;
use oryon::operators::Reciprocal as RustReciprocal;
use oryon::operators::Subtract as RustSubtract;
use oryon::StreamingTransform;
use pyo3::prelude::*;

use crate::{to_python, to_rust};

// --- Add ---------------------------------------------------------------------

/// Element-wise addition: A + B.
#[pyclass(module = "oryon")]
pub(crate) struct Add {
    pub(crate) inner: RustAdd,
}

#[pymethods]
impl Add {
    /// Create a new ``Add``.
    ///
    /// Args:
    ///     inputs: Names of the two input columns ``[A, B]``.
    ///     outputs: Name of the output column.
    #[new]
    pub fn new(inputs: Vec<String>, outputs: Vec<String>) -> PyResult<Self> {
        let inner = RustAdd::new(inputs, outputs).map_err(crate::oryon_err)?;
        Ok(Add { inner })
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
            "Add(inputs={:?}, outputs={:?})",
            self.inner.input_names(),
            self.inner.output_names(),
        )
    }
}

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
        let inner = RustSubtract::new(inputs, outputs).map_err(crate::oryon_err)?;
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

// --- Multiply ----------------------------------------------------------------

/// Element-wise multiplication: A * B.
#[pyclass(module = "oryon")]
pub(crate) struct Multiply {
    pub(crate) inner: RustMultiply,
}

#[pymethods]
impl Multiply {
    /// Create a new ``Multiply``.
    ///
    /// Args:
    ///     inputs: Names of the two input columns ``[A, B]``.
    ///     outputs: Name of the output column.
    #[new]
    pub fn new(inputs: Vec<String>, outputs: Vec<String>) -> PyResult<Self> {
        let inner = RustMultiply::new(inputs, outputs).map_err(crate::oryon_err)?;
        Ok(Multiply { inner })
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
            "Multiply(inputs={:?}, outputs={:?})",
            self.inner.input_names(),
            self.inner.output_names(),
        )
    }
}

// --- Divide ------------------------------------------------------------------

/// Element-wise division: A / B.
#[pyclass(module = "oryon")]
pub(crate) struct Divide {
    pub(crate) inner: RustDivide,
}

#[pymethods]
impl Divide {
    /// Create a new ``Divide``.
    ///
    /// Args:
    ///     inputs: Names of the two input columns ``[A, B]``.
    ///     outputs: Name of the output column.
    #[new]
    pub fn new(inputs: Vec<String>, outputs: Vec<String>) -> PyResult<Self> {
        let inner = RustDivide::new(inputs, outputs).map_err(crate::oryon_err)?;
        Ok(Divide { inner })
    }

    /// Process one bar. Returns ``[NaN]`` if either input is ``NaN`` or B is 0.
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
            "Divide(inputs={:?}, outputs={:?})",
            self.inner.input_names(),
            self.inner.output_names(),
        )
    }
}

// --- Reciprocal --------------------------------------------------------------

/// Multiplicative inverse: 1 / x.
#[pyclass(module = "oryon")]
pub(crate) struct Reciprocal {
    pub(crate) inner: RustReciprocal,
}

#[pymethods]
impl Reciprocal {
    /// Create a new ``Reciprocal``.
    ///
    /// Args:
    ///     inputs: Name of the input column. Must contain exactly 1 entry.
    ///     outputs: Name of the output column.
    #[new]
    pub fn new(inputs: Vec<String>, outputs: Vec<String>) -> PyResult<Self> {
        let inner = RustReciprocal::new(inputs, outputs).map_err(crate::oryon_err)?;
        Ok(Reciprocal { inner })
    }

    /// Process one bar. Returns ``[NaN]`` if input is ``NaN`` or 0.
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
            "Reciprocal(inputs={:?}, outputs={:?})",
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
        let inner = RustNegLog::new(inputs, outputs).map_err(crate::oryon_err)?;
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

// --- Log ---------------------------------------------------------------------

/// Natural logarithm: ln(x).
#[pyclass(module = "oryon")]
pub(crate) struct Log {
    pub(crate) inner: RustLog,
}

#[pymethods]
impl Log {
    /// Create a new ``Log``.
    ///
    /// Args:
    ///     inputs: Name of the input column. Must contain exactly 1 entry.
    ///     outputs: Name of the output column.
    #[new]
    pub fn new(inputs: Vec<String>, outputs: Vec<String>) -> PyResult<Self> {
        let inner = RustLog::new(inputs, outputs).map_err(crate::oryon_err)?;
        Ok(Log { inner })
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
            "Log(inputs={:?}, outputs={:?})",
            self.inner.input_names(),
            self.inner.output_names(),
        )
    }
}

// --- Logit -------------------------------------------------------------------

/// Logit function: ln(x / (1 - x)).
#[pyclass(module = "oryon")]
pub(crate) struct Logit {
    pub(crate) inner: RustLogit,
}

#[pymethods]
impl Logit {
    /// Create a new ``Logit``.
    ///
    /// Args:
    ///     inputs: Name of the input column. Must contain exactly 1 entry.
    ///     outputs: Name of the output column.
    #[new]
    pub fn new(inputs: Vec<String>, outputs: Vec<String>) -> PyResult<Self> {
        let inner = RustLogit::new(inputs, outputs).map_err(crate::oryon_err)?;
        Ok(Logit { inner })
    }

    /// Process one bar. Returns ``[NaN]`` if input is ``NaN`` or outside the open interval (0, 1).
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
            "Logit(inputs={:?}, outputs={:?})",
            self.inner.input_names(),
            self.inner.output_names(),
        )
    }
}
