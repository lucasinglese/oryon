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

    - **Warm-up.** The first `window - 1` bars return `NaN`. A full buffer of `window`
    values is required.

    - **`NaN` propagation.** A `NaN` input contaminates the buffer. Output stays `NaN`
    until the `NaN` is evicted after `window` consecutive valid bars.

    - **All values equal.** If all `window` values are identical, the standard deviation
    is zero and the output is `NaN`.

    - **`reset()`.** Clears the buffer entirely. Call it between backtest folds
    (CPCV, walk-forward) to avoid state leaking across splits.

    - **Implementation.** Recomputes over the full buffer on every `update()` (`O(N)` per bar).
    Uses sample standard deviation (N-1 denominator) for the standardization step.

    | Situation | Output |
    |---|---|
    | `t < window - 1` (buffer not full) | `NaN` |
    | Buffer full, all values valid, not all equal | Skewness value |
    | Any `NaN` in the buffer | `NaN` |
    | All values in the buffer are equal | `NaN` |
    | After `reset()` | `NaN` until buffer refills |

=== "Interpretation"

    - **Signal.** Positive: right tail dominates - extreme positive deviations are
    farther from the mean than extreme negative ones. Negative: left tail dominates.
    Zero: the distribution is symmetric over the window.

    - **Rolling.** Changes in sign or magnitude capture distributional shifts in the
    series. A transition from positive to negative skewness signals that the left tail
    is growing relative to the right.

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

    - **Warm-up.** The first `window - 1` bars return `NaN`. A full buffer of `window`
    values is required.

    - **`NaN` propagation.** A `NaN` input contaminates the buffer. Output stays `NaN`
    until the `NaN` is evicted after `window` consecutive valid bars.

    - **All values equal.** If all `window` values are identical, the standard deviation
    is zero and the output is `NaN`.

    - **`reset()`.** Clears the buffer entirely. Call it between backtest folds
    (CPCV, walk-forward) to avoid state leaking across splits.

    - **Implementation.** Recomputes over the full buffer on every `update()` (`O(N)` per bar).
    Uses sample standard deviation (N-1 denominator) for the standardization step.

    | Situation | Output |
    |---|---|
    | `t < window - 1` (buffer not full) | `NaN` |
    | Buffer full, all values valid, not all equal | Excess kurtosis value |
    | Any `NaN` in the buffer | `NaN` |
    | All values in the buffer are equal | `NaN` |
    | After `reset()` | `NaN` until buffer refills |

=== "Interpretation"

    - **Signal.** Excess kurtosis > 0: more probability mass in the tails than a
    normal distribution (leptokurtic). = 0: consistent with normal. < 0: lighter
    tails than normal (platykurtic).

    - **Rolling.** Spikes in excess kurtosis indicate concentration of extreme
    observations within the window, the tail behavior is changing.

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

---

## Median Moving Average

<a href="../../../getting-started/streaming-vs-research/#streaming-live-trading" class="oryon-badge oryon-badge--streaming">Streaming</a> <a href="../../../benchmarks/" class="oryon-badge oryon-badge--perf">&lt;1µs/update</a> <a href="../../../getting-started/streaming-vs-research/#research-full-dataset" class="oryon-badge oryon-badge--research">Research</a>

$$
\text{MMA}_t = \text{median}(x_{t-n+1}, \ldots, x_t)
$$

Rolling median over the last `window` bars. More robust to outliers than the SMA -
a single spike does not shift the output, making it useful as a pre-filter before
applying trend or signal detection indicators.

=== "Parameters"

    | Name | Type | Constraint | Description |
    |---|---|---|---|
    | `inputs` | `list[str]` | len = 1 | Input column, e.g. `["close"]` |
    | `window` | `int` | >= 1 | Rolling window length |
    | `outputs` | `list[str]` | len = 1 | Output column, e.g. `["close_mma_20"]` |

=== "Output"

    | Column | When valid | Description |
    |---|---|---|
    | `outputs[0]` | `t >= window - 1`, no `NaN` in buffer | Rolling median of the last `window` values |

=== "Properties"

    | Property | Value |
    |---|---|
    | `warm_up_period` | `window - 1` |
    | `forward_period` | `0` |

=== "Behavior"

    - **Warm-up.** The first `window - 1` bars return `NaN`.

    - **`NaN` propagation.** A `NaN` input enters the buffer and contaminates the output.
    Output stays `NaN` until the `NaN` is evicted after `window` consecutive valid bars.

    - **`reset()`.** Clears the buffer entirely.

    | Situation | Output |
    |---|---|
    | `t < window - 1` (buffer not full) | `NaN` |
    | Buffer full, all values valid | Median value |
    | Any `NaN` in the buffer | `NaN` |
    | After `reset()` | `NaN` until buffer refills |

=== "Interpretation"

    - **Signal.** Tracks the central value of the series while ignoring extreme bars.
    A single spike does not shift the output, unlike the SMA or EMA.

    - **Window size.** Small windows (`3`-`5`) remove isolated outliers while preserving
    local structure. Large windows smooth too aggressively and lag significantly behind
    real price moves.

    - **Use case.** Pre-filter before applying trend or signal detection indicators when
    the raw series contains frequent outliers (bad ticks, gaps, illiquid bars).

=== "Example"

    ```python
    from oryon.features import Mma

    mma = Mma(inputs=["close"], window=3, outputs=["close_mma_3"])

    mma.update([1.0])  # -> [NaN]
    mma.update([3.0])  # -> [NaN]
    mma.update([2.0])  # -> [2.0]  median([1, 3, 2]) = 2.0
    mma.update([5.0])  # -> [3.0]  median([3, 2, 5]) = 3.0
    ```

=== "Source"

    [:octicons-mark-github-16: `crates/oryon/src/features/mma.rs`](https://github.com/lucasinglese/oryon/blob/main/crates/oryon/src/features/mma.rs)