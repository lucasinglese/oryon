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
/// Streaming transformations (Feature trait and implementations).
pub mod features;
/// Pure math functions shared by features and targets.
pub mod ops;
/// DAG-based feature pipeline and target pipeline orchestrators.
pub mod pipeline;
/// Batch label computations for research (Target trait and implementations).
pub mod targets;
/// Stateless data manipulation utilities: `rolling`, `shift`, `pairwise`.
pub mod tools;
/// Core traits: `Feature`, `Target`, and the `Output` type alias.
pub mod traits;

pub use error::OryonError;
pub use traits::{Feature, Output, Target};
