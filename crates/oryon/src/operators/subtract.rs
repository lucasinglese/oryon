binary_operator!(
    Subtract,
    "Element-wise subtraction: `A - B`.\n\nReturns `None` if either input is `None`.",
    |a: f64, b: f64| Some(a - b),
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::streaming_transform_contract_tests;
    use crate::traits::StreamingTransform;
    use smallvec::smallvec;

    streaming_transform_contract_tests!(
        Subtract::new(vec!["a".into(), "b".into()], vec!["a_minus_b".into()]).unwrap(),
        vec!["a".to_string(), "b".to_string()],
        vec!["a_minus_b".to_string()],
        0,
        &[Some(1.0), Some(2.0)],
    );

    fn sub() -> Subtract {
        Subtract::new(vec!["a".into(), "b".into()], vec!["a_minus_b".into()]).unwrap()
    }

    fn out(v: Option<f64>) -> crate::traits::Output {
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
    fn test_error_raises_when_inputs_wrong_length() {
        assert!(matches!(
            Subtract::new(vec!["a".into()], vec!["out".into()]).unwrap_err(),
            crate::error::OryonError::InvalidInput { ref msg } if msg.contains("2 columns")
        ));
        assert!(matches!(
            Subtract::new(vec!["a".into(), "b".into(), "c".into()], vec!["out".into()]).unwrap_err(),
            crate::error::OryonError::InvalidInput { ref msg } if msg.contains("2 columns")
        ));
    }

    #[test]
    fn test_error_raises_when_outputs_empty() {
        assert!(matches!(
            Subtract::new(vec!["a".into(), "b".into()], vec![]).unwrap_err(),
            crate::error::OryonError::InvalidInput { ref msg } if msg.contains("outputs")
        ));
    }
}
