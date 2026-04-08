unary_operator!(
    Reciprocal,
    "Multiplicative inverse: `1 / x`.\n\nReturns `None` if the input is `None` or `0`.",
    |x: f64| if x != 0.0 { Some(1.0 / x) } else { None },
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::streaming_transform_contract_tests;
    use crate::traits::StreamingTransform;
    use smallvec::smallvec;

    streaming_transform_contract_tests!(
        Reciprocal::new(vec!["x".into()], vec!["inv_x".into()]).unwrap(),
        vec!["x".to_string()],
        vec!["inv_x".to_string()],
        0,
        &[Some(1.0)],
    );

    fn rec() -> Reciprocal {
        Reciprocal::new(vec!["x".into()], vec!["inv_x".into()]).unwrap()
    }

    fn out(v: Option<f64>) -> crate::traits::Output {
        smallvec![v]
    }

    #[test]
    fn test_update() {
        let mut op = rec();
        assert_eq!(op.update(&[Some(2.0)]), out(Some(0.5)));
        assert_eq!(op.update(&[Some(4.0)]), out(Some(0.25)));
        assert_eq!(op.update(&[Some(1.0)]), out(Some(1.0)));
    }

    #[test]
    fn test_update_none_input() {
        let mut op = rec();
        assert_eq!(op.update(&[None]), out(None));
    }

    #[test]
    fn test_update_zero() {
        let mut op = rec();
        assert_eq!(op.update(&[Some(0.0)]), out(None));
    }

    #[test]
    fn test_fresh_instances_are_independent() {
        let mut op = rec();
        op.update(&[Some(2.0)]);

        let mut fresh = op.fresh();
        assert_eq!(fresh.update(&[Some(4.0)]), out(Some(0.25)));
        assert_eq!(op.update(&[Some(2.0)]), out(Some(0.5)));
    }

    #[test]
    fn test_error_raises_when_inputs_wrong_length() {
        assert!(matches!(
            Reciprocal::new(vec![], vec!["out".into()]).unwrap_err(),
            crate::error::OryonError::InvalidInput { ref msg } if msg.contains("1 column")
        ));
        assert!(matches!(
            Reciprocal::new(vec!["a".into(), "b".into()], vec!["out".into()]).unwrap_err(),
            crate::error::OryonError::InvalidInput { ref msg } if msg.contains("1 column")
        ));
    }

    #[test]
    fn test_error_raises_when_outputs_empty() {
        assert!(matches!(
            Reciprocal::new(vec!["x".into()], vec![]).unwrap_err(),
            crate::error::OryonError::InvalidInput { ref msg } if msg.contains("outputs")
        ));
    }
}
