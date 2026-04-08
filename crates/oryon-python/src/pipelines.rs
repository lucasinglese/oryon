use oryon::pipeline::FeaturePipeline as RustFeaturePipeline;
use oryon::pipeline::TargetPipeline as RustTargetPipeline;
use pyo3::prelude::*;

use crate::{extract_feature, extract_target, to_python, to_rust};

// --- FeaturePipeline ---------------------------------------------------------

/// Orchestrates features in DAG-resolved order.
///
/// Features are resolved automatically — pass them in any order.
/// Use ``run_research()`` for batch mode or ``update()`` for live mode.
#[pyclass(module = "oryon")]
pub(crate) struct FeaturePipeline {
    pub(crate) inner: RustFeaturePipeline,
}

#[pymethods]
impl FeaturePipeline {
    /// Create a new ``FeaturePipeline``.
    ///
    /// Args:
    ///     features: List of feature objects. Dependencies resolved automatically.
    ///     input_columns: Column names in the order passed to ``update()``.
    #[new]
    pub fn new(features: Vec<Bound<'_, PyAny>>, input_columns: Vec<String>) -> PyResult<Self> {
        let rust_features = features
            .iter()
            .map(extract_feature)
            .collect::<PyResult<Vec<_>>>()?;
        let inner =
            RustFeaturePipeline::new(rust_features, input_columns).map_err(crate::oryon_err)?;
        Ok(FeaturePipeline { inner })
    }

    /// Process one bar (live mode).
    ///
    /// Args:
    ///     values: One float per input column. Use ``float('nan')`` for missing.
    ///
    /// Returns:
    ///     Flat list of output values matching ``output_names()``.
    fn update(&mut self, values: Vec<f64>) -> PyResult<Vec<f64>> {
        self.inner
            .update(&to_rust(&values))
            .map(|v| to_python(&v))
            .map_err(crate::oryon_err)
    }

    /// Process a full dataset bar by bar (research mode).
    ///
    /// Args:
    ///     data: List of bars, each bar is a list of input values.
    ///
    /// Returns:
    ///     List of bars, each bar is a list of output values.
    fn run_research(&mut self, data: Vec<Vec<f64>>) -> PyResult<Vec<Vec<f64>>> {
        let rust_data: Vec<Vec<Option<f64>>> = data.iter().map(|row| to_rust(row)).collect();
        self.inner
            .run_research(&rust_data)
            .map(|rows| rows.iter().map(|row| to_python(row)).collect())
            .map_err(crate::oryon_err)
    }

    /// Reset all features (e.g. between CPCV splits).
    fn reset(&mut self) {
        self.inner.reset();
    }

    /// Output column names in execution order.
    fn output_names(&self) -> Vec<String> {
        self.inner.output_names().to_vec()
    }

    /// Input columns in the order expected by ``update()``.
    fn input_names(&self) -> Vec<String> {
        self.inner.input_names().to_vec()
    }

    /// Maximum warm-up period across all features.
    fn warm_up_period(&self) -> usize {
        self.inner.warm_up_period()
    }

    /// Number of features in the pipeline.
    fn __len__(&self) -> usize {
        self.inner.len()
    }
}

// --- TargetPipeline ----------------------------------------------------------

/// Orchestrates multiple targets over a full dataset.
///
/// Targets are stateless and independent — no DAG needed.
/// Use ``run_research()`` to label an entire dataset at once.
#[pyclass(module = "oryon")]
pub(crate) struct TargetPipeline {
    inner: RustTargetPipeline,
}

#[pymethods]
impl TargetPipeline {
    /// Create a new ``TargetPipeline``.
    ///
    /// Args:
    ///     targets: List of target objects.
    ///     input_columns: Column names in the order passed to ``run_research()``.
    #[new]
    pub fn new(targets: Vec<Bound<'_, PyAny>>, input_columns: Vec<String>) -> PyResult<Self> {
        let rust_targets = targets
            .iter()
            .map(extract_target)
            .collect::<PyResult<Vec<_>>>()?;
        let inner =
            RustTargetPipeline::new(rust_targets, input_columns).map_err(crate::oryon_err)?;
        Ok(TargetPipeline { inner })
    }

    /// Run all targets over the full dataset (research mode).
    ///
    /// Args:
    ///     data: One list per input column, each containing one float per bar.
    ///           Use ``float('nan')`` for missing values.
    ///
    /// Returns:
    ///     One list per output column, in ``output_names()`` order.
    fn run_research(&self, data: Vec<Vec<f64>>) -> PyResult<Vec<Vec<f64>>> {
        let rust_cols: Vec<Vec<Option<f64>>> = data.iter().map(|col| to_rust(col)).collect();
        let refs: Vec<&[Option<f64>]> = rust_cols.iter().map(|c| c.as_slice()).collect();
        self.inner
            .run_research(&refs)
            .map(|cols| cols.iter().map(|col| to_python(col)).collect())
            .map_err(crate::oryon_err)
    }

    /// Output column names, in order.
    fn output_names(&self) -> Vec<String> {
        self.inner.output_names().to_vec()
    }

    /// Input columns in the order expected by ``run_research()``.
    fn input_names(&self) -> Vec<String> {
        self.inner.input_names().to_vec()
    }

    /// Maximum forward period across all targets.
    fn forward_period(&self) -> usize {
        self.inner.forward_period()
    }

    /// Number of targets in the pipeline.
    fn __len__(&self) -> usize {
        self.inner.len()
    }
}
