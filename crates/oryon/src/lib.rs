//! Institutional-grade feature and target engineering for quantitative research.
#![deny(missing_docs)]

#[cfg(test)]
#[allow(missing_docs)]
pub mod testing;

/// Bar-by-bar value checks: `is_none`, `is_inf`, `is_valid`.
pub mod checks;
/// Column-level data quality diagnostics: `null_rate`, `valid_rate`, `has_inf`, `has_nan`.
pub mod diagnostics;
/// Typed error variants for the Oryon library.
pub mod error;
/// Streaming feature transformations (SMA, EMA, KAMA, etc.).
pub mod features;
/// Batch fitting functions for pre-computed scaler parameters.
pub mod fitting;
/// Stateless streaming operators (Subtract, NegLog, etc.).
pub mod operators;
/// Pure math functions shared by features and targets.
pub mod ops;
/// DAG-based feature pipeline and target pipeline orchestrators.
pub mod pipeline;
/// Streaming scalers (RollingZScore, FixedZScore, etc.).
pub mod scalers;
/// Batch label computations for research (Target trait and implementations).
pub mod targets;
/// Stateless data manipulation utilities: `rolling`, `shift`, `pairwise`.
pub mod tools;
/// Core traits: `StreamingTransform`, `Target`, and the `Output` type alias.
pub mod traits;

pub use error::OryonError;
pub use traits::{Output, StreamingTransform, Target};
