use crate::error::OryonError;
use crate::fitting::StandardScalerParams;
use crate::traits::{Output, StreamingTransform};
use smallvec::smallvec;

/// Z-score normalization with pre-fitted mean and std.
///
/// Computes `(x - mean) / std` using fixed parameters from [`fit_standard_scaler`].
/// Returns `None` if the input is `None`.
///
/// [`fit_standard_scaler`]: crate::fitting::fit_standard_scaler
#[derive(Debug)]
pub struct FixedZScore {
    inputs: Vec<String>,
    outputs: Vec<String>,
    params: StandardScalerParams,
}

impl FixedZScore {
    /// Create a new `FixedZScore`.
    ///
    /// - `inputs` - name of the input column. Must contain exactly 1 entry.
    /// - `outputs` - name of the output column.
    /// - `params` - pre-fitted mean and std from `fit_standard_scaler`.
    pub fn new(
        inputs: Vec<String>,
        outputs: Vec<String>,
        params: StandardScalerParams,
    ) -> Result<Self, OryonError> {
        if inputs.len() != 1 {
            return Err(OryonError::InvalidInput {
                msg: "inputs must contain exactly 1 column".into(),
            });
        }
        if outputs.is_empty() {
            return Err(OryonError::InvalidInput {
                msg: "outputs must not be empty".into(),
            });
        }
        if params.std <= 0.0 {
            return Err(OryonError::InvalidInput {
                msg: "std must be > 0".into(),
            });
        }
        Ok(FixedZScore {
            inputs,
            outputs,
            params,
        })
    }
}

impl StreamingTransform for FixedZScore {
    fn input_names(&self) -> Vec<String> {
        self.inputs.clone()
    }

    fn output_names(&self) -> Vec<String> {
        self.outputs.clone()
    }

    fn fresh(&self) -> Box<dyn StreamingTransform> {
        Box::new(
            FixedZScore::new(self.inputs.clone(), self.outputs.clone(), self.params)
                .expect("fresh: config was already validated at construction"),
        )
    }

    fn reset(&mut self) {}

    fn update(&mut self, state: &[Option<f64>]) -> Output {
        let result = state[0].map(|x| (x - self.params.mean) / self.params.std);
        smallvec![result]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::streaming_transform_contract_tests;
    use smallvec::smallvec;

    fn params() -> StandardScalerParams {
        StandardScalerParams {
            mean: 3.0,
            std: 2.0,
        }
    }

    streaming_transform_contract_tests!(
        FixedZScore::new(vec!["x".into()], vec!["x_z".into()], params()).unwrap(),
        vec!["x".to_string()],
        vec!["x_z".to_string()],
        0,
        &[Some(5.0)],
    );

    fn fz() -> FixedZScore {
        FixedZScore::new(vec!["x".into()], vec!["x_z".into()], params()).unwrap()
    }

    fn out(v: Option<f64>) -> Output {
        smallvec![v]
    }

    #[test]
    fn test_update() {
        let mut s = fz();
        // (5 - 3) / 2 = 1.0
        assert_eq!(s.update(&[Some(5.0)]), out(Some(1.0)));
        // (3 - 3) / 2 = 0.0
        assert_eq!(s.update(&[Some(3.0)]), out(Some(0.0)));
        // (1 - 3) / 2 = -1.0
        assert_eq!(s.update(&[Some(1.0)]), out(Some(-1.0)));
    }

    #[test]
    fn test_update_none_input() {
        let mut s = fz();
        assert_eq!(s.update(&[None]), out(None));
    }

    #[test]
    fn test_fresh_instances_are_independent() {
        let mut s = fz();
        s.update(&[Some(5.0)]);

        let mut fresh = s.fresh();
        assert_eq!(fresh.update(&[Some(5.0)]), out(Some(1.0)));
        assert_eq!(s.update(&[Some(3.0)]), out(Some(0.0)));
    }

    #[test]
    fn test_invalid_params() {
        let p = params();
        assert!(matches!(
            FixedZScore::new(vec![], vec!["out".into()], p).unwrap_err(),
            OryonError::InvalidInput { ref msg } if msg.contains("1 column")
        ));
        assert!(matches!(
            FixedZScore::new(vec!["x".into()], vec![], p).unwrap_err(),
            OryonError::InvalidInput { ref msg } if msg.contains("outputs")
        ));
        let bad = StandardScalerParams {
            mean: 0.0,
            std: 0.0,
        };
        assert!(matches!(
            FixedZScore::new(vec!["x".into()], vec!["out".into()], bad).unwrap_err(),
            OryonError::InvalidInput { ref msg } if msg.contains("std")
        ));
    }
}
