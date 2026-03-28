use std::collections::HashMap;

use crate::error::OryonError;
use crate::traits::Target;

/// Orchestrates multiple targets over a full dataset.
///
/// Targets are independent and don't chain, so no DAG is needed.
/// `compute(data)` runs all targets and concatenates their outputs.
pub struct TargetPipeline {
    targets: Vec<Box<dyn Target>>,
    /// Maps input column name → position in the data slices passed to compute().
    input_col_mapping: HashMap<String, usize>,
    /// Input columns in the order expected by compute().
    input_columns: Vec<String>,
    /// All output column names, in order.
    output_names: Vec<String>,
}

impl TargetPipeline {
    /// Create a new `TargetPipeline`.
    ///
    /// - `targets` — the targets to compute.
    /// - `input_columns` — column names in the order they appear in `compute()`'s data.
    pub fn new(
        targets: Vec<Box<dyn Target>>,
        input_columns: Vec<String>,
    ) -> Result<Self, OryonError> {
        // Check for duplicate output keys across targets.
        let mut seen_keys: HashMap<String, usize> = HashMap::new();
        for (i, target) in targets.iter().enumerate() {
            for key in target.output_names() {
                if let Some(&existing_idx) = seen_keys.get(&key) {
                    return Err(OryonError::DuplicateOutputKey {
                        key,
                        idx_a: existing_idx,
                        idx_b: i,
                    });
                }
                seen_keys.insert(key, i);
            }
        }

        // Check all required columns are present.
        let mut missing_keys: Vec<String> = Vec::new();
        let mut seen_missing: HashMap<String, ()> = HashMap::new();
        for target in &targets {
            for col in target.input_names() {
                if !input_columns.contains(&col) && !seen_missing.contains_key(&col) {
                    seen_missing.insert(col.clone(), ());
                    missing_keys.push(col);
                }
            }
        }

        if !missing_keys.is_empty() {
            return Err(OryonError::MissingInputColumn {
                missing: missing_keys,
            });
        }

        let mut input_col_mapping: HashMap<String, usize> = HashMap::new();
        for (i, col) in input_columns.iter().enumerate() {
            input_col_mapping.insert(col.clone(), i);
        }

        let output_names: Vec<String> = targets.iter().flat_map(|t| t.output_names()).collect();

        Ok(TargetPipeline {
            targets,
            input_col_mapping,
            input_columns,
            output_names,
        })
    }

    /// Compute all targets over the full dataset.
    ///
    /// `data` contains one slice per entry in `input_columns`, in the same order.
    /// Returns one `Vec<Option<f64>>` per output key (see `output_names()`).
    pub fn compute(&self, data: &[&[Option<f64>]]) -> Vec<Vec<Option<f64>>> {
        let mut result: Vec<Vec<Option<f64>>> = Vec::new();

        for target in &self.targets {
            let target_columns: Vec<&[Option<f64>]> = target
                .input_names()
                .iter()
                .map(|col| {
                    let idx = self.input_col_mapping[col];
                    data[idx]
                })
                .collect();

            let outputs = target.compute(&target_columns);
            result.extend(outputs);
        }

        result
    }

    /// Output column names, in order.
    pub fn output_names(&self) -> &[String] {
        &self.output_names
    }

    /// Input columns in the order expected by compute().
    pub fn input_names(&self) -> &[String] {
        &self.input_columns
    }

    /// Maximum forward period across all targets.
    pub fn forward_period(&self) -> usize {
        self.targets
            .iter()
            .map(|t| t.forward_period())
            .max()
            .unwrap_or(0)
    }

    /// Total number of targets in the pipeline.
    pub fn len(&self) -> usize {
        self.targets.len()
    }

    /// Returns true if the pipeline contains no targets.
    pub fn is_empty(&self) -> bool {
        self.targets.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::targets::FutureCTCVolatility;

    fn sample_prices() -> Vec<Option<f64>> {
        vec![
            Some(100.0),
            Some(101.0),
            Some(103.0),
            Some(102.0),
            Some(105.0),
            Some(107.0),
            Some(106.0),
            Some(108.0),
            Some(110.0),
            Some(109.0),
        ]
    }

    #[test]
    fn test_single_target() {
        let target = FutureCTCVolatility::new("close", 3).unwrap();
        let pipeline =
            TargetPipeline::new(vec![Box::new(target)], vec!["close".into()]).unwrap();

        assert_eq!(pipeline.len(), 1);
        assert_eq!(pipeline.output_names(), &["close_future_ctc_vol_3".to_string()]);
        assert_eq!(pipeline.forward_period(), 3);
    }

    #[test]
    fn test_multiple_targets() {
        let t1 = FutureCTCVolatility::new("close", 3).unwrap();
        let t2 = FutureCTCVolatility::new("close", 5).unwrap();
        let pipeline = TargetPipeline::new(
            vec![Box::new(t1), Box::new(t2)],
            vec!["close".into()],
        )
        .unwrap();

        assert_eq!(pipeline.len(), 2);
        assert_eq!(pipeline.forward_period(), 5);
        assert_eq!(
            pipeline.output_names(),
            &[
                "close_future_ctc_vol_3".to_string(),
                "close_future_ctc_vol_5".to_string(),
            ]
        );
    }

    #[test]
    fn test_compute() {
        let target = FutureCTCVolatility::new("close", 3).unwrap();
        let pipeline =
            TargetPipeline::new(vec![Box::new(target)], vec!["close".into()]).unwrap();

        let prices = sample_prices();
        let result = pipeline.compute(&[&prices]);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].len(), prices.len());

        // Last 3 values must be None.
        assert_eq!(result[0][7], None);
        assert_eq!(result[0][8], None);
        assert_eq!(result[0][9], None);
    }

    #[test]
    fn test_compute_multiple() {
        let t1 = FutureCTCVolatility::new("close", 3).unwrap();
        let t2 = FutureCTCVolatility::new("close", 5).unwrap();
        let pipeline = TargetPipeline::new(
            vec![Box::new(t1), Box::new(t2)],
            vec!["close".into()],
        )
        .unwrap();

        let prices = sample_prices();
        let result = pipeline.compute(&[&prices]);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].len(), prices.len());
        assert_eq!(result[1].len(), prices.len());
    }

    #[test]
    fn test_missing_column() {
        let target = FutureCTCVolatility::new("close", 3).unwrap();
        let result = TargetPipeline::new(vec![Box::new(target)], vec!["volume".into()]);
        assert!(result.is_err());
    }

    #[test]
    fn test_duplicate_output_key() {
        let t1 = FutureCTCVolatility::new("close", 3).unwrap();
        let t2 = FutureCTCVolatility::new("close", 3).unwrap(); // same output key
        let result = TargetPipeline::new(
            vec![Box::new(t1), Box::new(t2)],
            vec!["close".into()],
        );
        assert!(result.is_err());
    }
}