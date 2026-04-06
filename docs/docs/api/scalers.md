# Scalers

Scalers are StreamingTransforms that normalize values. They implement the same interface
as features and are fully pipeline-compatible.

| | `RollingZScore` | `FixedZScore` |
|---|---|---|
| Warm-up | `window - 1` bars | None |
| Parameters | Recomputed each bar from buffer | Fixed at construction |
| Use case | Exploratory / adaptive | Train-then-deploy |

---

## Rolling Z-Score

<a href="../../getting-started/streaming-vs-research/#streaming-live-trading" class="oryon-badge oryon-badge--streaming">Streaming</a> <a href="../../benchmarks/" class="oryon-badge oryon-badge--perf">&lt;1µs/update</a> <a href="../../getting-started/streaming-vs-research/#research-full-dataset" class="oryon-badge oryon-badge--research">Research</a>

$$
z_t = \frac{x_t - \bar{x}_w}{\sigma_w}
$$

Streaming z-score computed over a sliding window. Mean and standard deviation are
recalculated every bar from the buffer. Returns `NaN` during warm-up, if any value
in the window is `NaN`, or if the standard deviation is zero (all window values are equal).

=== "Parameters"

    | Name | Type | Constraint | Description |
    |---|---|---|---|
    | `inputs` | `list[str]` | len = 1 | Input column, e.g. `["close_sma_20"]` |
    | `window` | `int` | >= 2 | Number of bars for rolling statistics |
    | `outputs` | `list[str]` | len >= 1 | Output column, e.g. `["close_sma_20_z"]` |

=== "Output"

    | Column | When valid | Description |
    |---|---|---|
    | `outputs[0]` | `t >= window - 1`, no `NaN` in buffer, std > 0 | `(x - rolling_mean) / rolling_std` over the last `window` bars |

=== "Behavior"

    - **Warm-up.** The first `window - 1` bars return `NaN`. A full window is required
    before mean and std can be computed.

    - **`NaN` propagation.** A `NaN` input contaminates the buffer. Output stays `NaN`
    until that value is evicted, i.e. until `window` consecutive valid bars have been seen.

    - **Zero std.** If all values in the window are equal, std is zero and output is `NaN`.

    - **`reset()`.** Clears the buffer entirely. After reset, the full `window - 1`
    warm-up applies again.

    | Situation | Output |
    |---|---|
    | `t < window - 1` (buffer not full) | `NaN` |
    | Buffer full, all values valid, std > 0 | z-score |
    | Any `NaN` in buffer | `NaN` |
    | All window values equal (std = 0) | `NaN` |
    | After `reset()` | `NaN` until buffer refills |

=== "Interpretation"

    - **Scale normalization.** Brings the feature to a mean=0, std=1 scale relative
    to its recent history. Makes features with different absolute magnitudes comparable
    in the same model without static assumptions about the distribution.

    - **Adaptive.** Parameters update every bar. The normalization tracks the current
    regime rather than a fixed training distribution. Useful when the series is
    non-stationary.

=== "Example"

    ```python
    import pandas as pd
    from oryon.scalers import RollingZScore
    from oryon import FeaturePipeline, run_features_pipeline

    rz = RollingZScore(["x"], window=3, outputs=["x_z"])
    fp = FeaturePipeline(features=[rz], input_columns=["x"])

    df = pd.DataFrame({"x": [1.0, 2.0, 3.0, 4.0, 5.0, 5.0, 5.0]})
    out = run_features_pipeline(fp, df)
    print(out)
    #      x_z
    # 0    NaN
    # 1    NaN
    # 2   1.00   # (3 - 2) / 1.0 = 1.0
    # 3   1.00   # (4 - 3) / 1.0 = 1.0
    # 4   1.00   # (5 - 4) / 1.0 = 1.0
    # 5   0.58   # (5 - 4.67) / 0.577 ≈ 0.58
    # 6    NaN   # std = 0: [5, 5, 5] are all equal
    ```

    Step-by-step with `window = 3`:

    | Bar | Input | Buffer | mean | std | Output |
    |-----|-------|--------|------|-----|--------|
    | 0 | 1.0 | `[1]` | - | - | `NaN` |
    | 1 | 2.0 | `[1, 2]` | - | - | `NaN` |
    | 2 | 3.0 | `[1, 2, 3]` | 2.0 | 1.0 | **1.00** |
    | 3 | 4.0 | `[2, 3, 4]` | 3.0 | 1.0 | **1.00** |
    | 4 | 5.0 | `[3, 4, 5]` | 4.0 | 1.0 | **1.00** |
    | 5 | 5.0 | `[4, 5, 5]` | 4.67 | 0.577 | **0.58** |
    | 6 | 5.0 | `[5, 5, 5]` | 5.0 | 0.0 | `NaN` |

=== "Source"

    [:octicons-mark-github-16: `crates/oryon/src/scalers/rolling_zscore.rs`](https://github.com/lucasinglese/oryon/blob/main/crates/oryon/src/scalers/rolling_zscore.rs)

---

## Fixed Z-Score

<a href="../../getting-started/streaming-vs-research/#streaming-live-trading" class="oryon-badge oryon-badge--streaming">Streaming</a> <a href="../../benchmarks/" class="oryon-badge oryon-badge--perf">&lt;1µs/update</a> <a href="../../getting-started/streaming-vs-research/#research-full-dataset" class="oryon-badge oryon-badge--research">Research</a>

$$
z = \frac{x - \mu}{\sigma}
$$

Stateless z-score normalization using pre-fitted mean and standard deviation. Parameters
are fixed at construction time and never change. No warm-up, no buffer. Returns `NaN`
only if the input is `NaN`.

Fit the parameters with [`fit_standard_scaler`](fitting.md) on a training dataset, then
construct `FixedZScore` once and apply it at inference time.

=== "Parameters"

    | Name | Type | Constraint | Description |
    |---|---|---|---|
    | `inputs` | `list[str]` | len = 1 | Input column, e.g. `["returns"]` |
    | `outputs` | `list[str]` | len >= 1 | Output column, e.g. `["returns_z"]` |
    | `mean` | `float` | - | Pre-fitted mean (from `fit_standard_scaler`) |
    | `std` | `float` | > 0 | Pre-fitted standard deviation (from `fit_standard_scaler`) |

=== "Output"

    | Column | When valid | Description |
    |---|---|---|
    | `outputs[0]` | input is not `NaN` | `(x - mean) / std` |

=== "Behavior"

    - **No warm-up.** Output is valid from the first bar.

    - **`NaN` input.** Returns `NaN`. No state is affected.

    - **`reset()`.** No-op. There is no internal state to clear.

    - **Train/test split.** Fit on training data only. Fitting on the full dataset leaks
    future distribution information and overstates out-of-sample performance.

    | Situation | Output |
    |---|---|
    | Valid input | `(x - mean) / std` |
    | `NaN` input | `NaN` |

=== "Example"

    ```python
    import pandas as pd
    from oryon.scalers import fit_standard_scaler, FixedZScore
    from oryon import FeaturePipeline, run_features_pipeline

    # Fit on training data
    train = [1.0, 2.0, 3.0, 4.0, 5.0]
    mean, std = fit_standard_scaler(train)
    # mean = 3.0, std ≈ 1.5811

    fz = FixedZScore(["x"], ["x_z"], mean=mean, std=std)
    fp = FeaturePipeline(features=[fz], input_columns=["x"])

    df = pd.DataFrame({"x": [1.0, 3.0, 5.0]})
    out = run_features_pipeline(fp, df)
    print(out)
    #       x_z
    # 0  -1.265   # (1 - 3) / 1.5811
    # 1   0.000   # (3 - 3) / 1.5811
    # 2   1.265   # (5 - 3) / 1.5811
    ```

=== "Source"

    [:octicons-mark-github-16: `crates/oryon/src/scalers/fixed_zscore.rs`](https://github.com/lucasinglese/oryon/blob/main/crates/oryon/src/scalers/fixed_zscore.rs)