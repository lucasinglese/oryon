use crate::error::OryonError;
use crate::traits::{Output, StreamingTransform};
use smallvec::smallvec;

/// Element-wise subtraction: `A - B`.
///
/// Returns `None` if either input is `None`.
#[derive(Debug)]
pub struct Subtract {
    inputs: Vec<String>,
    outputs: Vec<String>,
}

impl Subtract {
    /// Create a new `Subtract`.
    ///
    /// - `inputs` - names of the two input columns `[A, B]`. Must contain exactly 2 entries.
    /// - `outputs` - name of the output column (e.g. `["spread"]`).
    pub fn new(inputs: Vec<String>, outputs: Vec<String>) -> Result<Self, OryonError> {
        if inputs.len() != 2 {
            return Err(OryonError::InvalidInput {
                msg: "inputs must contain exactly 2 columns".into(),
            });
        }
        if outputs.is_empty() {
            return Err(OryonError::InvalidInput {
                msg: "outputs must not be empty".into(),
            });
        }
        Ok(Subtract { inputs, outputs })
    }
}

impl StreamingTransform for Subtract {
    fn input_names(&self) -> Vec<String> {
        self.inputs.clone()
    }

    fn output_names(&self) -> Vec<String> {
        self.outputs.clone()
    }

    fn fresh(&self) -> Box<dyn StreamingTransform> {
        Box::new(
            Subtract::new(self.inputs.clone(), self.outputs.clone())
                .expect("fresh: config was already validated at construction"),
        )
    }

    fn reset(&mut self) {}

    fn update(&mut self, state: &[Option<f64>]) -> Output {
        let result = match (state[0], state[1]) {
            (Some(a), Some(b)) => Some(a - b),
            _ => None,
        };
        smallvec![result]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature_contract_tests;
    use smallvec::smallvec;

    feature_contract_tests!(
        Subtract::new(vec!["a".into(), "b".into()], vec!["a_minus_b".into()]).unwrap(),
        vec!["a".to_string(), "b".to_string()],
        vec!["a_minus_b".to_string()],
        0,
        &[Some(1.0), Some(2.0)],
    );

    fn sub() -> Subtract {
        Subtract::new(vec!["a".into(), "b".into()], vec!["a_minus_b".into()]).unwrap()
    }

    fn out(v: Option<f64>) -> Output {
        smallvec![v]
    }

    #[test]
    fn test_update() {
        let mut op = sub();
        assert_eq!(op.update(&[Some(10.0), Some(3.0)]), out(Some(7.0)));
        assert_eq!(op.update(&[Some(5.0), Some(5.0)]), out(Some(0.0)));
        assert_eq!(op.update(&[Some(1.0), Some(4.0)]), out(Some(-3.0)));
    }

    #[test]
    fn test_update_none_input() {
        let mut op = sub();
        assert_eq!(op.update(&[None, Some(1.0)]), out(None));
        assert_eq!(op.update(&[Some(1.0), None]), out(None));
        assert_eq!(op.update(&[None, None]), out(None));
    }

    #[test]
    fn test_fresh_instances_are_independent() {
        let mut op = sub();
        op.update(&[Some(1.0), Some(2.0)]);

        let mut fresh = op.fresh();
        assert_eq!(fresh.update(&[Some(10.0), Some(3.0)]), out(Some(7.0)));
        assert_eq!(op.update(&[Some(5.0), Some(1.0)]), out(Some(4.0)));
    }

    #[test]
    fn test_invalid_params() {
        assert!(matches!(
            Subtract::new(vec!["a".into()], vec!["out".into()]).unwrap_err(),
            OryonError::InvalidInput { ref msg } if msg.contains("2 columns")
        ));
        assert!(matches!(
            Subtract::new(vec!["a".into(), "b".into(), "c".into()], vec!["out".into()]).unwrap_err(),
            OryonError::InvalidInput { ref msg } if msg.contains("2 columns")
        ));
        assert!(matches!(
            Subtract::new(vec!["a".into(), "b".into()], vec![]).unwrap_err(),
            OryonError::InvalidInput { ref msg } if msg.contains("outputs")
        ));
    }
}