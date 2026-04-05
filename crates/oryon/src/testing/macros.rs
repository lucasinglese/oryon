/// Generates the standard contract tests for any [`Target`] implementation.
///
/// Checks:
/// - `input_names()` matches expected value
/// - `output_names()` matches expected value
/// - `forward_period()` matches expected value
/// - `warm_up_period()` matches expected value
/// - smoke: `run_research()` returns correct number of columns, each with correct length
///
/// # Usage
/// ```ignore
/// target_contract_tests!(
///     FutureCTCVolatility::new("close", 3).unwrap(),
///     vec!["close".to_string()],
///     vec!["close_future_ctc_vol_3".to_string()],
///     3,  // forward_period
///     0,  // warm_up_period
///     &prices,
/// );
/// ```
#[macro_export]
macro_rules! target_contract_tests {
    ($target:expr, $input_names:expr, $output_names:expr, $forward:expr, $warm_up:expr, $data:expr $(,)?) => {
        #[test]
        fn test_contract_input_names() {
            assert_eq!($target.input_names(), $input_names);
        }

        #[test]
        fn test_contract_output_names() {
            assert_eq!($target.output_names(), $output_names);
        }

        #[test]
        fn test_contract_forward_period() {
            assert_eq!($target.forward_period(), $forward);
        }

        #[test]
        fn test_contract_warm_up_period() {
            assert_eq!($target.warm_up_period(), $warm_up);
        }

        #[test]
        fn test_contract_compute_shape() {
            let t = $target;
            let result = t.run_research(&[$data]);
            assert_eq!(result.len(), t.output_names().len());
            assert!(result.iter().all(|col| col.len() == $data.len()));
        }
    };
}

/// Generates the standard contract tests for any [`StreamingTransform`] implementation.
///
/// Checks:
/// - `input_names()` matches expected value
/// - `output_names()` matches expected value
/// - `warm_up_period()` matches expected value
/// - smoke: `update()` output length matches `output_names()` length
/// - reset: state is cleared (skipped if warm_up == 0)
/// - fresh: new instance is independent (skipped if warm_up == 0)
///
/// # Usage
/// ```ignore
/// feature_contract_tests!(
///     Sma::new(vec!["close".into()], 3, vec!["close_sma_3".into()]).unwrap(),
///     vec!["close".to_string()],
///     vec!["close_sma_3".to_string()],
///     2,
///     &[Some(1.0)],
/// );
/// ```
#[macro_export]
macro_rules! feature_contract_tests {
    ($feature:expr, $input_names:expr, $output_names:expr, $warm_up:expr, $state:expr $(,)?) => {
        #[test]
        fn test_contract_input_names() {
            assert_eq!($feature.input_names(), $input_names);
        }

        #[test]
        fn test_contract_output_names() {
            assert_eq!($feature.output_names(), $output_names);
        }

        #[test]
        fn test_contract_warm_up_period() {
            assert_eq!($feature.warm_up_period(), $warm_up);
        }

        #[test]
        fn test_contract_update_output_shape() {
            let mut f = $feature;
            let out = f.update($state);
            assert_eq!(out.len(), f.output_names().len());
        }

        #[test]
        fn test_contract_reset() {
            if $warm_up > 0 {
                let mut f = $feature;
                #[allow(clippy::reversed_empty_ranges)]
                for _ in 0..$warm_up {
                    f.update($state);
                }
                f.reset();
                assert_eq!(f.update($state)[0], None);
            }
        }

        #[test]
        fn test_contract_fresh() {
            if $warm_up > 0 {
                let mut f = $feature;
                #[allow(clippy::reversed_empty_ranges)]
                for _ in 0..$warm_up {
                    f.update($state);
                }
                let mut fresh = f.fresh();
                assert_eq!(fresh.update($state)[0], None);
            }
        }
    };
}
