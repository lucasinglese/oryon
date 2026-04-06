mod features;
mod operators;
mod pipelines;
mod scalers;
mod targets;

use features::{
    Ema, Kama, Kurtosis, LinearSlope, LogReturn, Mma, ParkinsonVolatility,
    RogersSatchellVolatility, SimpleReturn, Skewness, Sma,
};
use operators::{NegLog, Subtract};
use oryon::targets::FutureCTCVolatility as RustFutureCTCVolatility;
use oryon::targets::FutureLinearSlope as RustFutureLinearSlope;
use oryon::targets::FutureReturn as RustFutureReturn;
use oryon::traits::{StreamingTransform, Target};
use pipelines::{FeaturePipeline, TargetPipeline};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use scalers::{fit_standard_scaler, FixedZScore, RollingZScore};
use targets::{FutureCTCVolatility, FutureLinearSlope, FutureReturn};

// --- conversion helpers ------------------------------------------------------

pub(crate) fn to_rust(values: &[f64]) -> Vec<Option<f64>> {
    values
        .iter()
        .map(|&v| if v.is_nan() { None } else { Some(v) })
        .collect()
}

pub(crate) fn to_python(values: &[Option<f64>]) -> Vec<f64> {
    values.iter().map(|v| v.unwrap_or(f64::NAN)).collect()
}

// --- dispatch helpers --------------------------------------------------------

/// Extract a `Box<dyn StreamingTransform>` from a Python feature object.
/// Uses `fresh()` so the pipeline always starts with a clean state.
/// Add a branch here for each new feature type.
pub(crate) fn extract_feature(obj: &Bound<'_, PyAny>) -> PyResult<Box<dyn StreamingTransform>> {
    if let Ok(f) = obj.extract::<PyRef<Sma>>() {
        return Ok(f.inner.fresh());
    }
    if let Ok(f) = obj.extract::<PyRef<Ema>>() {
        return Ok(f.inner.fresh());
    }
    if let Ok(f) = obj.extract::<PyRef<Kama>>() {
        return Ok(f.inner.fresh());
    }
    if let Ok(f) = obj.extract::<PyRef<SimpleReturn>>() {
        return Ok(f.inner.fresh());
    }
    if let Ok(f) = obj.extract::<PyRef<LogReturn>>() {
        return Ok(f.inner.fresh());
    }
    if let Ok(f) = obj.extract::<PyRef<Mma>>() {
        return Ok(f.inner.fresh());
    }
    if let Ok(f) = obj.extract::<PyRef<Skewness>>() {
        return Ok(f.inner.fresh());
    }
    if let Ok(f) = obj.extract::<PyRef<Kurtosis>>() {
        return Ok(f.inner.fresh());
    }
    if let Ok(f) = obj.extract::<PyRef<LinearSlope>>() {
        return Ok(f.inner.fresh());
    }
    if let Ok(f) = obj.extract::<PyRef<ParkinsonVolatility>>() {
        return Ok(f.inner.fresh());
    }
    if let Ok(f) = obj.extract::<PyRef<RogersSatchellVolatility>>() {
        return Ok(f.inner.fresh());
    }
    // operators
    if let Ok(f) = obj.extract::<PyRef<Subtract>>() {
        return Ok(f.inner.fresh());
    }
    if let Ok(f) = obj.extract::<PyRef<NegLog>>() {
        return Ok(f.inner.fresh());
    }
    // scalers
    if let Ok(f) = obj.extract::<PyRef<RollingZScore>>() {
        return Ok(f.inner.fresh());
    }
    if let Ok(f) = obj.extract::<PyRef<FixedZScore>>() {
        return Ok(f.inner.fresh());
    }
    Err(PyValueError::new_err(format!(
        "unsupported feature type: {}",
        obj.get_type().name()?
    )))
}

/// Extract a `Box<dyn Target>` from a Python target object.
/// Add a branch here for each new target type.
pub(crate) fn extract_target(obj: &Bound<'_, PyAny>) -> PyResult<Box<dyn Target>> {
    if let Ok(t) = obj.extract::<PyRef<FutureReturn>>() {
        return Ok(Box::new(
            RustFutureReturn::new(t.inputs.clone(), t.horizon, t.outputs.clone())
                .map_err(|e| PyValueError::new_err(e.to_string()))?,
        ));
    }
    if let Ok(t) = obj.extract::<PyRef<FutureCTCVolatility>>() {
        return Ok(Box::new(
            RustFutureCTCVolatility::new(&t.input, t.horizon)
                .map_err(|e| PyValueError::new_err(e.to_string()))?,
        ));
    }
    if let Ok(t) = obj.extract::<PyRef<FutureLinearSlope>>() {
        return Ok(Box::new(
            RustFutureLinearSlope::new(t.inputs.clone(), t.horizon, t.outputs.clone())
                .map_err(|e| PyValueError::new_err(e.to_string()))?,
        ));
    }
    Err(PyValueError::new_err(format!(
        "unsupported target type: {}",
        obj.get_type().name()?
    )))
}

// --- module ------------------------------------------------------------------

#[pymodule]
fn _oryon(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // features
    m.add_class::<Sma>()?;
    m.add_class::<Ema>()?;
    m.add_class::<Kama>()?;
    m.add_class::<SimpleReturn>()?;
    m.add_class::<LogReturn>()?;
    m.add_class::<Mma>()?;
    m.add_class::<Skewness>()?;
    m.add_class::<Kurtosis>()?;
    m.add_class::<LinearSlope>()?;
    m.add_class::<ParkinsonVolatility>()?;
    m.add_class::<RogersSatchellVolatility>()?;
    // operators
    m.add_class::<Subtract>()?;
    m.add_class::<NegLog>()?;
    // scalers
    m.add_class::<RollingZScore>()?;
    m.add_class::<FixedZScore>()?;
    m.add_function(wrap_pyfunction!(fit_standard_scaler, m)?)?;
    // targets
    m.add_class::<FutureReturn>()?;
    m.add_class::<FutureCTCVolatility>()?;
    m.add_class::<FutureLinearSlope>()?;
    // pipelines
    m.add_class::<FeaturePipeline>()?;
    m.add_class::<TargetPipeline>()?;
    Ok(())
}
