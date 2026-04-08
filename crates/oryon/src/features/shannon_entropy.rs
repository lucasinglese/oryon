use crate::error::OryonError;
use crate::ops::shannon_entropy;
use crate::traits::{Output, StreamingTransform};
use smallvec::smallvec;
use std::collections::VecDeque;

/// Method used to determine the number of histogram bins for entropy estimation.
#[derive(Debug, Clone, Copy)]
pub enum BinMethod {
    /// Divide the value range into `n` equal-width bins. `n` must be >= 2.
    FixedCount(usize),
    /// Sturges' rule: `k = ceil(1 + log2(window))`. Computed once at construction.
    Sturges,
}

/// Rolling Shannon entropy over a fixed window.
///
/// Discretizes the last `window` values into equal-width bins and computes
/// `H = -sum(p_i * ln(p_i))` where `p_i` is the fraction of values in each bin.
/// Uses the convention `0 * ln(0) = 0`. When all values in the window are identical,
/// all mass falls in one bin and entropy is `0.0` (maximum certainty, not `None`).
/// When `normalize` is `true`, outputs `H / ln(n_bins)` in [0, 1].
///
/// Returns `None` during warm-up (first `window - 1` bars) and while any `None`
/// value remains in the rolling window.
#[derive(Debug)]
pub struct ShannonEntropy {
    inputs: Vec<String>,
    window: usize,
    outputs: Vec<String>,
    method: BinMethod,
    normalize: bool,
    n_bins: usize,
    buffer: VecDeque<Option<f64>>,
}

impl ShannonEntropy {
    /// Create a new `ShannonEntropy`.
    ///
    /// - `inputs` - name of the input column (e.g. `["returns"]`).
    /// - `window` - number of bars. Must be >= 2.
    /// - `outputs` - name of the output column (e.g. `["returns_entropy_20"]`).
    /// - `method` - binning strategy. `BinMethod::FixedCount(n)` requires `n >= 2`.
    /// - `normalize` - if `true`, output is `H / ln(n_bins)` in [0, 1].
    pub fn new(
        inputs: Vec<String>,
        window: usize,
        outputs: Vec<String>,
        method: BinMethod,
        normalize: bool,
    ) -> Result<Self, OryonError> {
        if inputs.is_empty() {
            return Err(OryonError::InvalidInput {
                msg: "inputs must not be empty".into(),
            });
        }
        if outputs.is_empty() {
            return Err(OryonError::InvalidInput {
                msg: "outputs must not be empty".into(),
            });
        }
        if window < 2 {
            return Err(OryonError::InvalidInput {
                msg: "window must be >= 2".into(),
            });
        }
        let n_bins = match method {
            BinMethod::FixedCount(n) => {
                if n < 2 {
                    return Err(OryonError::InvalidInput {
                        msg: "FixedCount bin count must be >= 2".into(),
                    });
                }
                n
            }
            BinMethod::Sturges => (1.0_f64 + (window as f64).log2()).ceil() as usize,
        };
        Ok(ShannonEntropy {
            inputs,
            window,
            outputs,
            method,
            normalize,
            n_bins,
            buffer: VecDeque::with_capacity(window),
        })
    }
}

impl StreamingTransform for ShannonEntropy {
    fn input_names(&self) -> Vec<String> {
        self.inputs.clone()
    }

    fn output_names(&self) -> Vec<String> {
        self.outputs.clone()
    }

    fn warm_up_period(&self) -> usize {
        self.window - 1
    }

    fn fresh(&self) -> Box<dyn StreamingTransform> {
        Box::new(
            ShannonEntropy::new(
                self.inputs.clone(),
                self.window,
                self.outputs.clone(),
                self.method,
                self.normalize,
            )
            .expect("fresh: config was already validated at construction"),
        )
    }

    fn reset(&mut self) {
        self.buffer.clear();
    }

    fn update(&mut self, state: &[Option<f64>]) -> Output {
        self.buffer.push_back(state[0]);
        if self.buffer.len() > self.window {
            self.buffer.pop_front();
        }
        if self.buffer.len() == self.window {
            // None propagation: any None in window -> None output.
            let slice = self.buffer.make_contiguous();
            let values: Option<Vec<f64>> = slice.iter().copied().collect::<Option<Vec<_>>>();
            let values = match values {
                Some(v) => v,
                None => return smallvec![None],
            };

            let min = values.iter().copied().fold(f64::INFINITY, f64::min);
            let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            let range = max - min;

            let mut counts = vec![0usize; self.n_bins];
            if range == 0.0 {
                // All values identical: all mass in bin 0, entropy = 0.
                counts[0] = self.window;
            } else {
                for &v in &values {
                    let raw_idx = ((v - min) / range * self.n_bins as f64) as usize;
                    counts[raw_idx.min(self.n_bins - 1)] += 1;
                }
            }

            let n = self.window as f64;
            let probs: Vec<f64> = counts.iter().map(|&c| c as f64 / n).collect();

            let h = shannon_entropy(&probs);
            let result = if self.normalize {
                h / (self.n_bins as f64).ln()
            } else {
                h
            };

            smallvec![Some(result)]
        } else {
            smallvec![None]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::streaming_transform_contract_tests;
    use smallvec::smallvec;

    streaming_transform_contract_tests!(
        ShannonEntropy::new(
            vec!["x".into()],
            4,
            vec!["x_entropy_4".into()],
            BinMethod::FixedCount(2),
            false
        )
        .unwrap(),
        vec!["x".to_string()],
        vec!["x_entropy_4".to_string()],
        3,
        &[Some(1.0)],
    );

    fn se_w10_fixed2() -> ShannonEntropy {
        ShannonEntropy::new(
            vec!["x".into()],
            10,
            vec!["x_entropy_10".into()],
            BinMethod::FixedCount(2),
            false,
        )
        .unwrap()
    }

    fn out(v: Option<f64>) -> Output {
        smallvec![v]
    }

    // Reference data: 20 bars, verified with scipy.stats.entropy using the same
    // binning logic (equal-width bins, natural log).
    const DATA: [f64; 20] = [
        1.0, 2.0, 1.5, 3.0, 2.5, 1.0, 4.0, 3.5, 2.0, 1.5, 2.0, 3.0, 1.0, 4.0, 3.0, 2.5, 1.5, 2.0,
        3.5, 4.0,
    ];

    #[test]
    fn test_update() {
        let mut se = se_w10_fixed2();

        // Warm-up: first 9 bars return None.
        for &v in &DATA[..9] {
            assert_eq!(se.update(&[Some(v)]), out(None));
        }

        // scipy: window=[1,2,1.5,3,2.5,1,4,3.5,2,1.5], n_bins=2
        //   min=1 max=4 range=3 -> counts=[7,3] -> H=0.6730116670092565
        let r = se.update(&[Some(DATA[9])]);
        assert!((r[0].unwrap() - 0.6730116670092565).abs() < 1e-10);

        // scipy: window=[2,1.5,3,2.5,1,4,3.5,2,1.5,2], n_bins=2 -> counts=[7,3]
        assert!((se.update(&[Some(DATA[10])])[0].unwrap() - 0.6730116670092565).abs() < 1e-10);

        // scipy: window=[1.5,3,2.5,1,4,3.5,2,1.5,2,3], n_bins=2 -> counts=[5,5] -> H=ln(2)
        assert!((se.update(&[Some(DATA[11])])[0].unwrap() - std::f64::consts::LN_2).abs() < 1e-10);
    }

    #[test]
    fn test_update_none_input() {
        let mut se = ShannonEntropy::new(
            vec!["x".into()],
            4,
            vec!["out".into()],
            BinMethod::FixedCount(2),
            false,
        )
        .unwrap();

        se.update(&[Some(1.0)]);
        se.update(&[Some(2.0)]);
        se.update(&[Some(3.0)]);
        assert!(se.update(&[Some(4.0)])[0].is_some());

        // Inject None — propagates for window (4) bars.
        assert_eq!(se.update(&[None]), out(None));
        assert_eq!(se.update(&[Some(2.0)]), out(None));
        assert_eq!(se.update(&[Some(3.0)]), out(None));
        assert_eq!(se.update(&[Some(4.0)]), out(None));
        // None flushed out — window=[2,3,4,5], all valid.
        assert!(se.update(&[Some(5.0)])[0].is_some());
    }

    #[test]
    fn test_update_normalized() {
        let mut se = ShannonEntropy::new(
            vec!["x".into()],
            10,
            vec!["x_entropy_10".into()],
            BinMethod::FixedCount(2),
            true,
        )
        .unwrap();

        for &v in &DATA[..17] {
            se.update(&[Some(v)]);
        }

        // scipy: window bar 18 -> H_norm=0.9709505944546688
        assert!((se.update(&[Some(DATA[17])])[0].unwrap() - 0.9709505944546688).abs() < 1e-10);
        // window bar 19 -> H_norm=1.0 (uniform)
        assert!((se.update(&[Some(DATA[18])])[0].unwrap() - 1.0).abs() < 1e-10);
        // window bar 20 -> H_norm=0.9709505944546688
        assert!((se.update(&[Some(DATA[19])])[0].unwrap() - 0.9709505944546688).abs() < 1e-10);
    }

    #[test]
    fn test_update_all_equal_values() {
        let mut se = ShannonEntropy::new(
            vec!["x".into()],
            4,
            vec!["x_entropy_4".into()],
            BinMethod::FixedCount(2),
            false,
        )
        .unwrap();
        for _ in 0..4 {
            se.update(&[Some(5.0)]);
        }
        // range=0 -> all mass in one bin -> H=0.0, not None.
        assert_eq!(se.update(&[Some(5.0)]), out(Some(0.0)));
    }

    #[test]
    fn test_sturges_n_bins() {
        // window=4: ceil(1+log2(4))=3, window=10: ceil(1+log2(10))=5
        let se4 = ShannonEntropy::new(
            vec!["x".into()],
            4,
            vec!["out".into()],
            BinMethod::Sturges,
            false,
        )
        .unwrap();
        assert_eq!(se4.n_bins, 3);

        let se10 = ShannonEntropy::new(
            vec!["x".into()],
            10,
            vec!["out".into()],
            BinMethod::Sturges,
            false,
        )
        .unwrap();
        assert_eq!(se10.n_bins, 5);
    }

    #[test]
    fn test_window_size_two() {
        // Minimum valid window. Sturges: k=ceil(1+log2(2))=2 bins.
        let mut se = ShannonEntropy::new(
            vec!["x".into()],
            2,
            vec!["out".into()],
            BinMethod::FixedCount(2),
            false,
        )
        .unwrap();
        assert_eq!(se.update(&[Some(1.0)]), out(None)); // warm-up
                                                        // window=[1.0, 4.0]: counts=[1,1] -> H = ln(2)
        assert!((se.update(&[Some(4.0)])[0].unwrap() - std::f64::consts::LN_2).abs() < 1e-10);
        // window=[4.0, 4.0]: range=0 -> H = 0.0
        assert_eq!(se.update(&[Some(4.0)]), out(Some(0.0)));
    }

    #[test]
    fn test_reset_clears_buffer() {
        let mut se = se_w10_fixed2();
        se.update(&[Some(1.0)]);
        se.reset();
        assert_eq!(se.buffer.len(), 0);
        assert_eq!(se.buffer.capacity(), se.window);
    }

    #[test]
    fn test_fresh_instances_are_independent() {
        let mut original = se_w10_fixed2();
        for &v in &DATA[..10] {
            original.update(&[Some(v)]);
        }

        let mut fresh = original.fresh();

        // fresh starts from scratch
        for &v in &DATA[..9] {
            assert_eq!(fresh.update(&[Some(v)]), out(None));
        }
        assert!((fresh.update(&[Some(DATA[9])])[0].unwrap() - 0.6730116670092565).abs() < 1e-10);

        // original continues its own state
        assert!(
            (original.update(&[Some(DATA[10])])[0].unwrap() - 0.6730116670092565).abs() < 1e-10
        );
    }

    #[test]
    fn test_error_raises_when_empty_inputs() {
        let err = ShannonEntropy::new(
            vec![],
            4,
            vec!["out".into()],
            BinMethod::FixedCount(2),
            false,
        )
        .unwrap_err();
        assert!(matches!(err, OryonError::InvalidInput { ref msg } if msg.contains("inputs")));
    }

    #[test]
    fn test_error_raises_when_empty_outputs() {
        let err = ShannonEntropy::new(vec!["x".into()], 4, vec![], BinMethod::FixedCount(2), false)
            .unwrap_err();
        assert!(matches!(err, OryonError::InvalidInput { ref msg } if msg.contains("outputs")));
    }

    #[test]
    fn test_error_raises_when_window_too_small() {
        let err = ShannonEntropy::new(
            vec!["x".into()],
            1,
            vec!["out".into()],
            BinMethod::FixedCount(2),
            false,
        )
        .unwrap_err();
        assert!(matches!(err, OryonError::InvalidInput { ref msg } if msg.contains("window")));
    }

    #[test]
    fn test_error_raises_when_fixed_count_too_small() {
        let err = ShannonEntropy::new(
            vec!["x".into()],
            4,
            vec!["out".into()],
            BinMethod::FixedCount(1),
            false,
        )
        .unwrap_err();
        assert!(matches!(err, OryonError::InvalidInput { ref msg } if msg.contains("bin")));
    }
}
