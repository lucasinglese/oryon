unary_operator!(
    Logit,
    "Logit function: `ln(x / (1 - x))`.\n\nReturns `None` if the input is `None` or outside the open interval `(0, 1)`.",
    |x: f64| if x > 0.0 && x < 1.0 { Some((x / (1.0 - x)).ln()) } else { None },
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::streaming_transform_contract_tests;
    use crate::traits::StreamingTransform;
    use smallvec::smallvec;

    streaming_transform_contract_tests!(
        Logit::new(vec!["x".into()], vec!["logit_x".into()]).unwrap(),
        vec!["x".to_string()],
        vec!["logit_x".to_string()],
        0,
        &[Some(0.5)],
    );

    fn logit() -> Logit {
        Logit::new(vec!["x".into()], vec!["logit_x".into()]).unwrap()
    }

    fn out(v: Option<f64>) -> crate::traits::Output {
        smallvec![v]
    }

    #[test]
    fn test_update() {
        let mut op = logit();
        // logit(0.5) = ln(1) = 0
        assert_eq!(op.update(&[Some(0.5)]), out(Some(0.0)));
        // logit(0.7) = ln(7/3) ≈ 0.8473
        assert!((op.update(&[Some(0.7)])[0].unwrap() - (7.0_f64 / 3.0).ln()).abs() < 1e-10);
        // logit(0.3) = ln(3/7) ≈ -0.8473 (symmetric)
        assert!((op.update(&[Some(0.3)])[0].unwrap() - (3.0_f64 / 7.0).ln()).abs() < 1e-10);
    }

    #[test]
    fn test_update_none_input() {
        let mut op = logit();
        assert_eq!(op.update(&[None]), out(None));
    }

    #[test]
    fn test_update_out_of_domain() {
        let mut op = logit();
        assert_eq!(op.update(&[Some(0.0)]), out(None));
        assert_eq!(op.update(&[Some(1.0)]), out(None));
        assert_eq!(op.update(&[Some(-0.1)]), out(None));
        assert_eq!(op.update(&[Some(1.1)]), out(None));
    }

    #[test]
    fn test_fresh_instances_are_independent() {
        let mut op = logit();
        op.update(&[Some(0.5)]);

        let mut fresh = op.fresh();
        assert_eq!(fresh.update(&[Some(0.5)]), out(Some(0.0)));
        assert_eq!(op.update(&[Some(0.5)]), out(Some(0.0)));
    }

    #[test]
    fn test_error_raises_when_inputs_wrong_length() {
        assert!(matches!(
            Logit::new(vec![], vec!["out".into()]).unwrap_err(),
            crate::error::OryonError::InvalidInput { ref msg } if msg.contains("1 column")
        ));
        assert!(matches!(
            Logit::new(vec!["a".into(), "b".into()], vec!["out".into()]).unwrap_err(),
            crate::error::OryonError::InvalidInput { ref msg } if msg.contains("1 column")
        ));
    }

    #[test]
    fn test_error_raises_when_outputs_empty() {
        assert!(matches!(
            Logit::new(vec!["x".into()], vec![]).unwrap_err(),
            crate::error::OryonError::InvalidInput { ref msg } if msg.contains("outputs")
        ));
    }
}
