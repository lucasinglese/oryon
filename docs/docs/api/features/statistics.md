# Statistics

Statistical features summarize the distribution of a price series over a
rolling window. All have `forward_period = 0` and are safe for live streaming.

---

## Skewness

<a href="../../../getting-started/streaming-vs-research/#streaming-live-trading" class="oryon-badge oryon-badge--streaming">Streaming</a> <a href="../../../benchmarks/" class="oryon-badge oryon-badge--perf">&lt;1µs/update</a> <a href="../../../getting-started/streaming-vs-research/#research-full-dataset" class="oryon-badge oryon-badge--research">Research</a>

$$
\text{Skew}_t = \frac{n}{(n-1)(n-2)} \sum_{i=1}^{n} \left(\frac{x_i - \bar{x}}{s}\right)^3
$$

Fisher-Pearson corrected skewness over a rolling window, identical to `pandas .skew()`.
Positive values indicate a long right tail (rare large gains), negative values a long left
tail (rare large losses). Useful for detecting regime shifts and tail risk.

=== "Parameters"

    | Name | Type | Constraint | Description |
    |---|---|---|---|
    | `inputs` | `list[str]` | len = 1 | Input column, e.g. `["close"]` |
    | `window` | `int` | >= 3 | Rolling window length |
    | `outputs` | `list[str]` | len = 1 | Output column, e.g. `["close_skew_20"]` |

=== "Output"

    | Column | When valid | Description |
    |---|---|---|
    | `outputs[0]` | `t >= window - 1`, no `NaN` in buffer, not all values equal | Fisher-Pearson sample skewness |

=== "Behavior"

    **Warm-up.** The first `window - 1` bars return `NaN`. A full buffer of `window`
    values is required.

    **`NaN` propagation.** A `NaN` input contaminates the buffer. Output stays `NaN`
    until the `NaN` is evicted after `window` consecutive valid bars.

    **All values equal.** If all `window` values are identical, the standard deviation
    is zero and the output is `NaN`.

    **`reset()`.** Clears the buffer entirely. Call it between backtest folds
    (CPCV, walk-forward) to avoid state leaking across splits.

    **Implementation.** Recomputes over the full buffer on every `update()` (`O(N)` per bar).
    Uses sample standard deviation (N-1 denominator) for the standardization step.

    | Situation | Output |
    |---|---|
    | `t < window - 1` (buffer not full) | `NaN` |
    | Buffer full, all values valid, not all equal | Skewness value |
    | Any `NaN` in the buffer | `NaN` |
    | All values in the buffer are equal | `NaN` |
    | After `reset()` | `NaN` until buffer refills |

=== "Example"

    ```python
    import pandas as pd
    from oryon.features import Skewness
    from oryon import FeaturePipeline, run_features_pipeline

    sk = Skewness(["close"], window=3, outputs=["close_skew_3"])
    fp = FeaturePipeline(features=[sk], input_columns=["close"])

    df = pd.DataFrame({"close": [1.0, 2.0, 4.0, 6.0, 8.0]})
    out = run_features_pipeline(fp, df)
    print(out)
    #    close_skew_3
    # 0           NaN
    # 1           NaN
    # 2          0.94
    # 3          0.00
    # 4          0.00
    ```

    `skew([1, 2, 4]) = 0.935` (right-skewed). `skew([2, 4, 6]) = 0.0` (symmetric -
    evenly spaced values always give zero skewness). Results match `pandas .skew()` exactly.

=== "Source"

    [:octicons-mark-github-16: `crates/oryon/src/features/skewness.rs`](https://github.com/lucasinglese/oryon/blob/main/crates/oryon/src/features/skewness.rs)

---

## Kurtosis

<a href="../../../getting-started/streaming-vs-research/#streaming-live-trading" class="oryon-badge oryon-badge--streaming">Streaming</a> <a href="../../../benchmarks/" class="oryon-badge oryon-badge--perf">&lt;1µs/update</a> <a href="../../../getting-started/streaming-vs-research/#research-full-dataset" class="oryon-badge oryon-badge--research">Research</a>

$$
\text{Kurt}_t = \frac{n(n+1)}{(n-1)(n-2)(n-3)} \sum_{i=1}^{n} \left(\frac{x_i - \bar{x}}{s}\right)^4 - \frac{3(n-1)^2}{(n-2)(n-3)}
$$

Fisher excess kurtosis over a rolling window, identical to `pandas .kurt()`.
Values above 0 indicate fat tails (leptokurtic). A normal distribution gives 0.
Uniformly spaced returns give negative kurtosis (platykurtic).

=== "Parameters"

    | Name | Type | Constraint | Description |
    |---|---|---|---|
    | `inputs` | `list[str]` | len = 1 | Input column, e.g. `["close"]` |
    | `window` | `int` | >= 4 | Rolling window length |
    | `outputs` | `list[str]` | len = 1 | Output column, e.g. `["close_kurt_20"]` |

=== "Output"

    | Column | When valid | Description |
    |---|---|---|
    | `outputs[0]` | `t >= window - 1`, no `NaN` in buffer, not all values equal | Fisher excess kurtosis |

=== "Behavior"

    **Warm-up.** The first `window - 1` bars return `NaN`. A full buffer of `window`
    values is required.

    **`NaN` propagation.** A `NaN` input contaminates the buffer. Output stays `NaN`
    until the `NaN` is evicted after `window` consecutive valid bars.

    **All values equal.** If all `window` values are identical, the standard deviation
    is zero and the output is `NaN`.

    **`reset()`.** Clears the buffer entirely. Call it between backtest folds
    (CPCV, walk-forward) to avoid state leaking across splits.

    **Implementation.** Recomputes over the full buffer on every `update()` (`O(N)` per bar).
    Uses sample standard deviation (N-1 denominator) for the standardization step.

    | Situation | Output |
    |---|---|
    | `t < window - 1` (buffer not full) | `NaN` |
    | Buffer full, all values valid, not all equal | Excess kurtosis value |
    | Any `NaN` in the buffer | `NaN` |
    | All values in the buffer are equal | `NaN` |
    | After `reset()` | `NaN` until buffer refills |

=== "Example"

    ```python
    import pandas as pd
    from oryon.features import Kurtosis
    from oryon import FeaturePipeline, run_features_pipeline

    ku = Kurtosis(["close"], window=4, outputs=["close_kurt_4"])
    fp = FeaturePipeline(features=[ku], input_columns=["close"])

    df = pd.DataFrame({"close": [1.0, 2.0, 4.0, 8.0, 6.0]})
    out = run_features_pipeline(fp, df)
    print(out)
    #    close_kurt_4
    # 0           NaN
    # 1           NaN
    # 2           NaN
    # 3          0.76
    # 4          2.24
    ```

    `kurt([1, 2, 4, 8]) = 0.758` (fat tails from the jump to 8). Results match
    `pandas .kurt()` exactly.

=== "Source"

    [:octicons-mark-github-16: `crates/oryon/src/features/kurtosis.rs`](https://github.com/lucasinglese/oryon/blob/main/crates/oryon/src/features/kurtosis.rs)