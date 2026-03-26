use std::collections::HashMap;

use crate::error::OryonError;
use crate::pipeline::dag::FeatureDag;
use crate::traits::{Feature, Output};

/// Orchestrates features in DAG-resolved order.
///
/// - `update(state)` — process one bar (live mode)
/// - `run_research(data)` — process a full dataset bar by bar
/// - `reset()` — reset all features (between CPCV splits)
pub struct FeaturePipeline {
    dag: FeatureDag,
    /// Maps input column name → position in the state slice passed to update().
    input_col_mapping: HashMap<String, usize>,
    /// Input columns in the order expected by update().
    input_columns: Vec<String>,
}

impl FeaturePipeline {
    pub fn new(
        features: Vec<Box<dyn Feature>>,
        input_columns: Vec<String>,
    ) -> Result<Self, OryonError> {
        let dag = FeatureDag::new(features)?;

        let required_columns = dag.required_columns();
        let missing_keys: Vec<String> = required_columns
            .iter()
            .filter(|col| !input_columns.contains(col))
            .cloned()
            .collect();

        if !missing_keys.is_empty() {
            return Err(OryonError::MissingInputColumn {
                missing: missing_keys,
            });
        }

        let mut input_col_mapping: HashMap<String, usize> = HashMap::new();
        for (i, col) in input_columns.iter().enumerate() {
            input_col_mapping.insert(col.clone(), i);
        }

        Ok(FeaturePipeline {
            dag,
            input_col_mapping,
            input_columns,
        })
    }

    /// Process one bar. `state` contains the raw input values
    /// in the same order as `input_columns` passed to `new()`.
    ///
    /// Returns a flat `Vec<Option<f64>>` with all feature outputs,
    /// in the same order as `output_keys()`.
    pub fn update(&mut self, state: &[Option<f64>]) -> Vec<Option<f64>> {
        let mut all_values: HashMap<String, Option<f64>> = HashMap::new();

        for (col_name, &col_idx) in &self.input_col_mapping {
            all_values.insert(col_name.clone(), state[col_idx]);
        }

        let mut result: Vec<Option<f64>> = Vec::new();

        for level in self.dag.execution_order_mut() {
            for feature in level.iter_mut() {
                let feature_state: Vec<Option<f64>> = feature
                    .required_columns()
                    .iter()
                    .map(|col| all_values.get(col).copied().flatten())
                    .collect();

                let output: Output = feature.update(&feature_state);

                let names = feature.names();
                for (i, name) in names.iter().enumerate() {
                    if i < output.len() {
                        all_values.insert(name.clone(), output[i]);
                    }
                }

                result.extend(output.iter().copied());
            }
        }

        result
    }

    /// Process a full dataset bar by bar (research mode).
    /// Each inner slice is one bar's raw input values.
    ///
    /// Returns a matrix: one row per bar, columns matching `output_keys()`.
    pub fn run_research(&mut self, data: &[Vec<Option<f64>>]) -> Vec<Vec<Option<f64>>> {
        self.dag.reset();
        let mut results: Vec<Vec<Option<f64>>> = Vec::with_capacity(data.len());
        for bar in data {
            results.push(self.update(bar));
        }
        results
    }

    /// Reset all features (between CPCV splits).
    pub fn reset(&mut self) {
        self.dag.reset();
    }

    /// Output column names in execution order.
    pub fn output_keys(&self) -> &[String] {
        self.dag.output_keys()
    }

    /// Input columns in the order expected by update().
    pub fn required_columns(&self) -> &[String] {
        &self.input_columns
    }

    /// Maximum warm-up period across all features.
    pub fn warm_up_period(&self) -> usize {
        self.dag.warm_up_period()
    }

    /// Total number of features in the pipeline.
    pub fn len(&self) -> usize {
        self.dag.len()
    }

    /// Returns true if the pipeline contains no features.
    pub fn is_empty(&self) -> bool {
        self.dag.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use smallvec::smallvec;

    use super::*;

    struct AddOneStub {
        col: Vec<String>,
        name: Vec<String>,
    }

    impl AddOneStub {
        fn new(col: Vec<String>, name: Vec<String>) -> Self {
            AddOneStub { col, name }
        }
    }

    impl Feature for AddOneStub {
        fn names(&self) -> Vec<String> {
            self.name.clone()
        }
        fn required_columns(&self) -> Vec<String> {
            self.col.clone()
        }
        fn update(&mut self, state: &[Option<f64>]) -> Output {
            smallvec![state[0].map(|x| x + 1.0)]
        }
        fn reset(&mut self) {}
        fn fresh(&self) -> Box<dyn Feature> {
            Box::new(AddOneStub::new(self.col.clone(), self.name.clone()))
        }
    }

    struct WarmUpOneStub {
        col: Vec<String>,
        name: Vec<String>,
        seen_one: bool,
    }

    impl WarmUpOneStub {
        fn new(col: Vec<String>, name: Vec<String>) -> Self {
            WarmUpOneStub {
                col,
                name,
                seen_one: false,
            }
        }
    }

    impl Feature for WarmUpOneStub {
        fn names(&self) -> Vec<String> {
            self.name.clone()
        }
        fn required_columns(&self) -> Vec<String> {
            self.col.clone()
        }
        fn update(&mut self, state: &[Option<f64>]) -> Output {
            if !self.seen_one {
                self.seen_one = true;
                smallvec![None]
            } else {
                smallvec![state[0]]
            }
        }
        fn reset(&mut self) {
            self.seen_one = false;
        }
        fn fresh(&self) -> Box<dyn Feature> {
            Box::new(WarmUpOneStub::new(self.col.clone(), self.name.clone()))
        }
    }

    #[test]
    fn test_update_single() {
        let f = AddOneStub::new(vec!["close".into()], vec!["out".into()]);
        let mut pipeline =
            FeaturePipeline::new(vec![Box::new(f)], vec!["close".into()]).unwrap();

        assert_eq!(pipeline.update(&[Some(1.0)]), vec![Some(2.0)]);
        assert_eq!(pipeline.update(&[Some(5.0)]), vec![Some(6.0)]);
    }

    #[test]
    fn test_update_two_independent() {
        let a = AddOneStub::new(vec!["close".into()], vec!["a".into()]);
        let b = AddOneStub::new(vec!["close".into()], vec!["b".into()]);
        let mut pipeline = FeaturePipeline::new(
            vec![Box::new(a), Box::new(b)],
            vec!["close".into()],
        )
        .unwrap();

        assert_eq!(pipeline.update(&[Some(1.0)]), vec![Some(2.0), Some(2.0)]);
        assert_eq!(pipeline.update(&[Some(4.0)]), vec![Some(5.0), Some(5.0)]);
    }

    #[test]
    fn test_update_chained() {
        let a = AddOneStub::new(vec!["close".into()], vec!["close_plus_one".into()]);
        let b = AddOneStub::new(vec!["close_plus_one".into()], vec!["close_plus_two".into()]);
        let mut pipeline = FeaturePipeline::new(
            vec![Box::new(b), Box::new(a)],
            vec!["close".into()],
        )
        .unwrap();

        assert_eq!(pipeline.update(&[Some(1.0)]), vec![Some(2.0), Some(3.0)]);
        assert_eq!(
            pipeline.update(&[Some(10.0)]),
            vec![Some(11.0), Some(12.0)]
        );
    }

    #[test]
    fn test_run_research() {
        let f = AddOneStub::new(vec!["close".into()], vec!["out".into()]);
        let mut pipeline =
            FeaturePipeline::new(vec![Box::new(f)], vec!["close".into()]).unwrap();

        let data = vec![
            vec![Some(1.0)],
            vec![Some(2.0)],
            vec![Some(3.0)],
        ];

        let results = pipeline.run_research(&data);
        assert_eq!(results.len(), 3);
        assert_eq!(results[0], vec![Some(2.0)]);
        assert_eq!(results[1], vec![Some(3.0)]);
        assert_eq!(results[2], vec![Some(4.0)]);
    }

    #[test]
    fn test_reset_between_splits() {
        let f = WarmUpOneStub::new(vec!["close".into()], vec!["out".into()]);
        let mut pipeline =
            FeaturePipeline::new(vec![Box::new(f)], vec!["close".into()]).unwrap();

        assert_eq!(pipeline.update(&[Some(1.0)]), vec![None]);
        assert_eq!(pipeline.update(&[Some(2.0)]), vec![Some(2.0)]);

        pipeline.reset();
        assert_eq!(pipeline.update(&[Some(10.0)]), vec![None]);
        assert_eq!(pipeline.update(&[Some(20.0)]), vec![Some(20.0)]);
    }

    #[test]
    fn test_output_keys() {
        let a = AddOneStub::new(vec!["close".into()], vec!["a".into()]);
        let b = AddOneStub::new(vec!["close".into()], vec!["b".into()]);
        let pipeline = FeaturePipeline::new(
            vec![Box::new(a), Box::new(b)],
            vec!["close".into()],
        )
        .unwrap();

        assert_eq!(pipeline.output_keys(), &["a".to_string(), "b".to_string()]);
    }

    #[test]
    fn test_chained_output_order() {
        let a = AddOneStub::new(vec!["close".into()], vec!["close_plus_one".into()]);
        let b = AddOneStub::new(vec!["close_plus_one".into()], vec!["close_plus_two".into()]);

        let pipeline = FeaturePipeline::new(
            vec![Box::new(b), Box::new(a)],
            vec!["close".into()],
        )
        .unwrap();

        assert_eq!(
            pipeline.output_keys(),
            &["close_plus_one".to_string(), "close_plus_two".to_string()]
        );
    }

    #[test]
    fn test_missing_input_column() {
        let f = AddOneStub::new(vec!["close".into()], vec!["out".into()]);
        let result = FeaturePipeline::new(vec![Box::new(f)], vec!["volume".into()]);
        assert!(result.is_err());
    }
}
