use crate::error::OryonError;
use crate::traits::{Output, StreamingTransform};
use smallvec::smallvec;

/// Negative natural logarithm: `-ln(x)`.
///
/// Returns `None` if the input is `None` or `<= 0`.
#[derive(Debug)]
pub struct NegLog {
    inputs: Vec<String>,
    outputs: Vec<String>,
}

impl NegLog {
    /// Create a new `NegLog`.
    ///
    /// - `inputs` - name of the input column (e.g. `["pvalue"]`). Must contain exactly 1 entry.
    /// - `outputs` - name of the output column (e.g. `["neg_log_pvalue"]`).
    pub fn new(inputs: Vec<String>, outputs: Vec<String>) -> Result<Self, OryonError> {
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
        Ok(NegLog { inputs, outputs })
    }
}

impl StreamingTransform for NegLog {
    fn input_names(&self) -> Vec<String> {
        self.inputs.clone()
    }

    fn output_names(&self) -> Vec<String> {
        self.outputs.clone()
    }

    fn fresh(&self) -> Box<dyn StreamingTransform> {
        Box::new(
            NegLog::new(self.inputs.clone(), self.outputs.clone())
                .expect("fresh: config was already validated at construction"),
        )
    }

    fn reset(&mut self) {}

    fn update(&mut self, state: &[Option<f64>]) -> Output {
        let result = match state[0] {
            Some(x) if x > 0.0 => Some(-x.ln()),
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
        NegLog::new(vec!["x".into()], vec!["neg_log_x".into()]).unwrap(),
        vec!["x".to_string()],
        vec!["neg_log_x".to_string()],
        0,
        &[Some(1.0)],
    );

    fn nl() -> NegLog {
        NegLog::new(vec!["x".into()], vec!["neg_log_x".into()]).unwrap()
    }

    fn out(v: Option<f64>) -> Output {
        smallvec![v]
    }

    #[test]
    fn test_update() {
        let mut op = nl();
        // -ln(1) = 0
        assert_eq!(op.update(&[Some(1.0)]), out(Some(0.0)));
        // -ln(e) = -1
        assert!((op.update(&[Some(std::f64::consts::E)])[0].unwrap() - (-1.0)).abs() < 1e-10);
        // -ln(0.5) = ln(2)
        assert!((op.update(&[Some(0.5)])[0].unwrap() - 2.0_f64.ln()).abs() < 1e-10);
    }

    #[test]
    fn test_update_none_input() {
        let mut op = nl();
        assert_eq!(op.update(&[None]), out(None));
    }

    #[test]
    fn test_update_zero_and_negative() {
        let mut op = nl();
        assert_eq!(op.update(&[Some(0.0)]), out(None));
        assert_eq!(op.update(&[Some(-1.0)]), out(None));
    }

    #[test]
    fn test_fresh_instances_are_independent() {
        let mut op = nl();
        op.update(&[Some(1.0)]);

        let mut fresh = op.fresh();
        assert_eq!(fresh.update(&[Some(1.0)]), out(Some(0.0)));
        assert_eq!(op.update(&[Some(1.0)]), out(Some(0.0)));
    }

    #[test]
    fn test_invalid_params() {
        assert!(matches!(
            NegLog::new(vec![], vec!["out".into()]).unwrap_err(),
            OryonError::InvalidInput { ref msg } if msg.contains("1 column")
        ));
        assert!(matches!(
            NegLog::new(vec!["a".into(), "b".into()], vec!["out".into()]).unwrap_err(),
            OryonError::InvalidInput { ref msg } if msg.contains("1 column")
        ));
        assert!(matches!(
            NegLog::new(vec!["x".into()], vec![]).unwrap_err(),
            OryonError::InvalidInput { ref msg } if msg.contains("outputs")
        ));
    }
}
