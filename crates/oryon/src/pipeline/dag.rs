use std::collections::{HashMap, VecDeque};

use crate::error::OryonError;
use crate::traits::Feature;

/// Resolves execution order of features as a DAG.
///
/// Dependencies are inferred by matching each feature's `input_names()`
/// against the `names()` (output keys) of other features.
/// Columns not produced by any feature must come from the input data.
///
/// Execution order is determined via Kahn's algorithm (level-based topological sort).
pub struct FeatureDag {
    /// Features grouped by execution level.
    /// Within a level, features are independent and could run in parallel.
    execution_order: Vec<Vec<Box<dyn Feature>>>,
    /// All output column names produced by the DAG, in execution order.
    output_names: Vec<String>,
    /// Columns that must come from the input data (not produced by any feature).
    input_names: Vec<String>,
}

impl FeatureDag {
    /// Create a new `FeatureDag` from a list of features.
    ///
    /// Infers dependency edges by matching each feature's `input_names()` against
    /// the `output_names()` of other features. Features with no shared dependency
    /// are placed in the same execution level.
    ///
    /// Returns `Err` if output keys are duplicated or if a cyclic dependency is detected.
    pub fn new(features: Vec<Box<dyn Feature>>) -> Result<Self, OryonError> {
        let n = features.len();

        // 1. Build output_key → feature index mapping
        let mut output_key_to_idx: HashMap<String, usize> = HashMap::new();
        for (i, feature) in features.iter().enumerate() {
            for key in feature.output_names() {
                if let Some(existing_idx) = output_key_to_idx.get(&key) {
                    return Err(OryonError::DuplicateOutputKey {
                        key,
                        idx_a: *existing_idx,
                        idx_b: i,
                    });
                }
                output_key_to_idx.insert(key, i);
            }
        }

        // 2. Build adjacency list + in-degree
        let mut adj: Vec<Vec<usize>> = vec![vec![]; n];
        let mut in_degree: Vec<usize> = vec![0; n];
        let mut input_names_set: Vec<String> = Vec::new();
        let mut seen_required: HashMap<String, ()> = HashMap::new();

        for (i, feature) in features.iter().enumerate() {
            for col in feature.input_names() {
                if let Some(&producer_idx) = output_key_to_idx.get(&col) {
                    if producer_idx != i {
                        adj[producer_idx].push(i);
                        in_degree[i] += 1;
                    }
                } else if !seen_required.contains_key(&col) {
                    seen_required.insert(col.clone(), ());
                    input_names_set.push(col);
                }
            }
        }

        // 3. Kahn's algorithm — level-based topological sort
        let mut queue: VecDeque<usize> = VecDeque::new();
        for (i, &deg) in in_degree.iter().enumerate() {
            if deg == 0 {
                queue.push_back(i);
            }
        }

        let mut level_indices: Vec<Vec<usize>> = Vec::new();
        let mut processed: usize = 0;

        while !queue.is_empty() {
            let level_size = queue.len();
            let mut current_level: Vec<usize> = Vec::new();

            for _ in 0..level_size {
                let idx = queue.pop_front().unwrap();
                current_level.push(idx);
                processed += 1;

                for &neighbor in &adj[idx] {
                    in_degree[neighbor] -= 1;
                    if in_degree[neighbor] == 0 {
                        queue.push_back(neighbor);
                    }
                }
            }
            level_indices.push(current_level);
        }

        if processed != n {
            return Err(OryonError::CyclicDependency);
        }

        // 4. Build output_names in execution order + move features into levels
        let mut output_names: Vec<String> = Vec::new();
        let mut slots: Vec<Option<Box<dyn Feature>>> = features.into_iter().map(Some).collect();

        let mut execution_order: Vec<Vec<Box<dyn Feature>>> = Vec::new();
        for level in &level_indices {
            let mut level_features: Vec<Box<dyn Feature>> = Vec::new();
            for &idx in level {
                let feature = slots[idx].take().unwrap();
                for name in feature.output_names() {
                    output_names.push(name);
                }
                level_features.push(feature);
            }
            execution_order.push(level_features);
        }

        Ok(FeatureDag {
            execution_order,
            output_names,
            input_names: input_names_set,
        })
    }

    /// Features grouped by execution level.
    pub fn execution_order(&self) -> &[Vec<Box<dyn Feature>>] {
        &self.execution_order
    }

    /// Mutable access to execution levels (needed by pipeline for calling update).
    pub fn execution_order_mut(&mut self) -> &mut [Vec<Box<dyn Feature>>] {
        &mut self.execution_order
    }

    /// All output column names, in execution order.
    pub fn output_names(&self) -> &[String] {
        &self.output_names
    }

    /// Columns required from input data (not produced by any feature).
    pub fn input_names(&self) -> &[String] {
        &self.input_names
    }

    /// Total number of features.
    pub fn len(&self) -> usize {
        self.execution_order.iter().map(|level| level.len()).sum()
    }

    /// Returns true if the DAG contains no features.
    pub fn is_empty(&self) -> bool {
        self.execution_order.is_empty()
    }

    /// Maximum warm_up_period across all features.
    pub fn warm_up_period(&self) -> usize {
        self.execution_order
            .iter()
            .flat_map(|level| level.iter())
            .map(|f| f.warm_up_period())
            .max()
            .unwrap_or(0)
    }

    /// Reset all features.
    pub fn reset(&mut self) {
        for level in &mut self.execution_order {
            for feature in level {
                feature.reset();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{AddOneStub, WarmUpOneStub};

    fn a(inputs: &[&str], outputs: &[&str]) -> Box<dyn Feature> {
        Box::new(AddOneStub::new(
            inputs.iter().map(|s| s.to_string()).collect(),
            outputs.iter().map(|s| s.to_string()).collect(),
        ))
    }

    fn w(inputs: &[&str], outputs: &[&str]) -> Box<dyn Feature> {
        Box::new(WarmUpOneStub::new(
            inputs.iter().map(|s| s.to_string()).collect(),
            outputs.iter().map(|s| s.to_string()).collect(),
        ))
    }

    #[test]
    fn test_single_feature() {
        let dag = FeatureDag::new(vec![a(&["close"], &["out"])]).unwrap();

        assert_eq!(dag.len(), 1);
        assert_eq!(dag.output_names(), &["out".to_string()]);
        assert_eq!(dag.input_names(), &["close".to_string()]);
        assert_eq!(dag.execution_order().len(), 1);
    }

    #[test]
    fn test_independent_features_same_level() {
        let dag = FeatureDag::new(vec![a(&["close"], &["a"]), a(&["close"], &["b"])]).unwrap();

        assert_eq!(dag.len(), 2);
        assert_eq!(dag.execution_order().len(), 1);
        assert_eq!(dag.input_names(), &["close".to_string()]);
    }

    #[test]
    fn test_chained_features_two_levels() {
        let dag = FeatureDag::new(vec![a(&["a"], &["b"]), a(&["close"], &["a"])]).unwrap();

        assert_eq!(dag.execution_order().len(), 2);
        assert_eq!(dag.input_names(), &["close".to_string()]);
        assert_eq!(
            dag.execution_order()[0][0].output_names(),
            vec!["a".to_string()]
        );
        assert_eq!(
            dag.execution_order()[1][0].output_names(),
            vec!["b".to_string()]
        );
    }

    #[test]
    fn test_duplicate_output_key_error() {
        let result = FeatureDag::new(vec![a(&["close"], &["out"]), a(&["close"], &["out"])]);
        assert!(result.is_err());
    }

    #[test]
    fn test_warm_up_period() {
        let dag = FeatureDag::new(vec![a(&["close"], &["a"]), w(&["close"], &["b"])]).unwrap();

        assert_eq!(dag.warm_up_period(), 1);
    }

    #[test]
    fn test_reset() {
        let mut dag = FeatureDag::new(vec![w(&["close"], &["out"])]).unwrap();

        dag.execution_order_mut()[0][0].update(&[Some(1.0)]);
        dag.reset();
        let out = dag.execution_order_mut()[0][0].update(&[Some(1.0)]);
        assert_eq!(out[0], None);
    }
}
