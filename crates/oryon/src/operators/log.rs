unary_operator!(
    Log,
    "Natural logarithm: `ln(x)`.\n\nReturns `None` if the input is `None` or `<= 0`.",
    |x: f64| if x > 0.0 { Some(x.ln()) } else { None },
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::streaming_transform_contract_tests;
    use crate::traits::StreamingTransform;
    use smallvec::smallvec;

    streaming_transform_contract_tests!(
        Log::new(vec!["x".into()], vec!["ln_x".into()]).unwrap(),
        vec!["x".to_string()],
        vec!["ln_x".to_string()],
        0,
        &[Some(1.0)],
    );

    fn lg() -> Log {
        Log::new(vec!["x".into()], vec!["ln_x".into()]).unwrap()
    }

    fn out(v: Option<f64>) -> crate::traits::Output {
        smallvec![v]
    }

    #[test]
    fn test_update() {
        let mut op = lg();
        // ln(1) = 0
        assert_eq!(op.update(&[Some(1.0)]), out(Some(0.0)));
        // ln(e) = 1
        assert!((op.update(&[Some(std::f64::consts::E)])[0].unwrap() - 1.0).abs() < 1e-10);
        // ln(e^2) = 2
        assert!((op.update(&[Some(std::f64::consts::E.powi(2))])[0].unwrap() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_update_none_input() {
        let mut op = lg();
        assert_eq!(op.update(&[None]), out(None));
    }

    #[test]
    fn test_update_zero_and_negative() {
        let mut op = lg();
        assert_eq!(op.update(&[Some(0.0)]), out(None));
        assert_eq!(op.update(&[Some(-1.0)]), out(None));
    }

    #[test]
    fn test_fresh_instances_are_independent() {
        let mut op = lg();
        op.update(&[Some(1.0)]);

        let mut fresh = op.fresh();
        assert_eq!(fresh.update(&[Some(1.0)]), out(Some(0.0)));
        assert_eq!(op.update(&[Some(1.0)]), out(Some(0.0)));
    }

    #[test]
    fn test_error_raises_when_inputs_wrong_length() {
        assert!(matches!(
            Log::new(vec![], vec!["out".into()]).unwrap_err(),
            crate::error::OryonError::InvalidInput { ref msg } if msg.contains("1 column")
        ));
        assert!(matches!(
            Log::new(vec!["a".into(), "b".into()], vec!["out".into()]).unwrap_err(),
            crate::error::OryonError::InvalidInput { ref msg } if msg.contains("1 column")
        ));
    }

    #[test]
    fn test_error_raises_when_outputs_empty() {
        assert!(matches!(
            Log::new(vec!["x".into()], vec![]).unwrap_err(),
            crate::error::OryonError::InvalidInput { ref msg } if msg.contains("outputs")
        ));
    }
}
