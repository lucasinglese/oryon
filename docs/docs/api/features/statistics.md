# Statistics

Statistical features summarize the distribution of a price series over a
rolling window. All have `forward_period = 0` and are safe for live streaming.

---

## Adf

<a href="../../../getting-started/streaming-vs-research/#streaming-live-trading" class="oryon-badge oryon-badge--streaming">Streaming</a> <a href="../../../benchmarks/" class="oryon-badge oryon-badge--perf">&lt;3µs/update</a> <a href="../../../getting-started/streaming-vs-research/#research-full-dataset" class="oryon-badge oryon-badge--research">Research</a>

The Augmented Dickey-Fuller test measures whether a rolling window of prices
behaves like a stationary (mean-reverting) process or a unit-root (random walk)
process. Two outputs are produced per bar: the ADF test statistic and its
approximate p-value.

The regression model estimated by OLS on each window is:

$$
\Delta y_t = \alpha + \gamma \, y_{t-1} + \sum_{j=1}^{k} \delta_j \, \Delta y_{t-j} + \varepsilon_t
\quad \text{(regression='c')}
$$

$$
\Delta y_t = \alpha + \beta t + \gamma \, y_{t-1} + \sum_{j=1}^{k} \delta_j \, \Delta y_{t-j} + \varepsilon_t
\quad \text{(regression='ct')}
$$

The test statistic is $\hat{\gamma} / \text{SE}(\hat{\gamma})$. Under H0 (unit root) it
follows the Dickey-Fuller distribution, not Student's t. P-values are computed via
linear interpolation over a 45-point table derived from MacKinnon (2010).

**H0:** $\gamma = 0$ (unit root - non-stationary)
**H1:** $\gamma < 0$ (no unit root - stationary)

=== "Parameters"

    | Name | Type | Constraint | Description |
    |---|---|---|---|
    | `inputs` | `list[str]` | len = 1 | Input column, e.g. `["close"]` |
    | `window` | `int` | `> 3 + 2 * lags` | Rolling window length |
    | `outputs` | `list[str]` | len = 2 | Output columns `[adf_stat_col, adf_pval_col]` |
    | `lags` | `int \| None` | >= 0 | Lagged differences. `None` applies Schwert's rule |
    | `regression` | `str` | `'c'` or `'ct'` | `'c'`: constant only. `'ct'`: constant + trend |

    **Schwert's rule** (default when `lags=None`): $k = \lfloor 12 \cdot (n/100)^{0.25} \rfloor$.
    Applied once at construction. For `window=100` this gives `k=12`.

=== "Output"

    | Column | When valid | Description |
    |---|---|---|
    | `outputs[0]` | `t >= window - 1`, no `NaN` in buffer, OLS non-singular | ADF test statistic |
    | `outputs[1]` | Same as above | Approximate p-value (MacKinnon 2010, asymptotic) |

=== "Behavior"

    - **Warm-up.** The first `window - 1` bars return `[NaN, NaN]`.

    - **`NaN` propagation.** A `NaN` input enters the buffer. Both outputs are `NaN`
    until the `NaN` is evicted after `window` consecutive valid bars.

    - **Singular OLS.** When the OLS system is degenerate (e.g. all values in the
    buffer are identical), both outputs return `NaN`.

    - **`reset()`.** Clears the buffer entirely. Call it between backtest folds
    (CPCV, walk-forward) to avoid state leaking across splits.

    - **P-value accuracy.** P-values use the asymptotic (large-sample) MacKinnon
    distribution. For `window < 100` the asymptotic approximation becomes less
    accurate - the true significance level may differ by a few percent from the
    reported p-value. Prefer `window >= 100` for reliable inference.

    - **Implementation.** Full OLS via Gaussian elimination on every `update()`,
    `O(window)` per bar.

    | Situation | Output |
    |---|---|
    | `t < window - 1` (buffer not full) | `[NaN, NaN]` |
    | Buffer full, all values valid, OLS non-singular | `[stat, pvalue]` |
    | Any `NaN` in the buffer | `[NaN, NaN]` |
    | OLS singular (e.g. constant series) | `[NaN, NaN]` |
    | After `reset()` | `[NaN, NaN]` until buffer refills |

=== "Interpretation"

    - **Stat below -3.5 (`'c'`)** or **-4.0 (`'ct'`)**: strong evidence of
    stationarity. The series is likely mean-reverting in this window.

    - **P-value below 0.05**: reject H0 (unit root) at the 5% level.

    - **Rolling use.** A series that switches from high p-values to low p-values
    across time is transitioning from a trending/random-walk regime to a
    mean-reverting regime - a common signal in pairs trading and stat-arb.

    - **Regression choice.** Use `'c'` when the series oscillates around a
    non-zero level. Use `'ct'` when you expect a linear trend and want to test
    stationarity around that trend.

=== "Example"

    ```python
    from statsmodels.tsa.stattools import adfuller
    from oryon.features import Adf

    # Reference: adfuller(x, regression='c', maxlag=0, autolag=None)[0] = -5.656
    x = [0.0, 0.5087, -0.1558, 0.2507, 0.8633, 0.3085, 0.5017, 0.4578,
         -0.2826, 0.1437, 0.4694, -0.0066, 0.2960, 0.6133, -0.1088,
         0.3521, 0.3786, 0.1477, 0.5707, 0.1324]

    adf = Adf(inputs=["close"], window=20, outputs=["adf_stat", "adf_pval"],
              lags=0, regression="c")

    for v in x[:-1]:
        adf.update([v])  # returns [NaN, NaN] during warm-up

    stat, pval = adf.update([x[-1]])
    print(f"stat={stat:.4f}, pval={pval:.2e}")
    # stat=-5.6565, pval=1.04e-06  → strong evidence of stationarity
    ```

=== "Contributing"

    - **Additional test series.** The current Rust test suite validates `adf_stat`
    against statsmodels on a single 20-bar reference series. Adding 2-3 more series
    (random walk, strong mean-reversion, high-volatility) would increase confidence
    across the full range of the statistic.

    - **Finite-sample p-values.** P-values currently use the asymptotic MacKinnon
    distribution (`N=1` in `mackinnonp`). A finite-sample correction (separate lookup
    tables for `N=30`, `50`, `100`, `250`) would improve accuracy for short windows.
    This requires Monte Carlo simulation or MacKinnon (1994) coefficient tables not
    available through statsmodels.

    - **Performance: incremental XtX update.** Each `update()` currently rebuilds the
    full OLS system from scratch - `O(window * p^2)` per bar, where `p = 3 + lags` is
    the number of regressors. At `window=200` with `lags=0` this costs ~3µs, and grows
    further with larger windows or more lags. The bottleneck is avoidable: the `Adf`
    struct could maintain `XtX` and `Xty` as running state, updating them incrementally
    in `O(p^2)` per bar by adding the new row and subtracting the evicted row. This
    would reduce per-bar cost to sub-1µs regardless of window size.

=== "Source"

    [:octicons-mark-github-16: `crates/oryon/src/features/adf.rs`](https://github.com/lucasinglese/oryon/blob/main/crates/oryon/src/features/adf.rs)

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
    from oryon import FeaturePipeline
    from oryon.adapters import run_features_pipeline_pandas

    sk = Skewness(["close"], window=3, outputs=["close_skew_3"])
    fp = FeaturePipeline(features=[sk], input_columns=["close"])

    df = pd.DataFrame({"close": [1.0, 2.0, 4.0, 6.0, 8.0]})
    out = run_features_pipeline_pandas(fp, df)
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
    from oryon import FeaturePipeline
    from oryon.adapters import run_features_pipeline_pandas

    ku = Kurtosis(["close"], window=4, outputs=["close_kurt_4"])
    fp = FeaturePipeline(features=[ku], input_columns=["close"])

    df = pd.DataFrame({"close": [1.0, 2.0, 4.0, 8.0, 6.0]})
    out = run_features_pipeline_pandas(fp, df)
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

---

## ShannonEntropy

<a href="../../../getting-started/streaming-vs-research/#streaming-live-trading" class="oryon-badge oryon-badge--streaming">Streaming</a> <a href="../../../benchmarks/" class="oryon-badge oryon-badge--perf">&lt;1µs/update</a> <a href="../../../getting-started/streaming-vs-research/#research-full-dataset" class="oryon-badge oryon-badge--research">Research</a>

$$
H_t = -\sum_{i=1}^{k} p_i \ln p_i
$$

$$
H^*_t = \frac{H_t}{\ln k} \in [0,\,1] \quad \text{(when \texttt{normalize=True})}
$$

Rolling Shannon entropy over the last `window` bars. Values are discretized into
`k` equal-width bins; $p_i$ is the fraction of observations in bin $i$.
High entropy means the distribution is spread across bins (disordered market).
Low entropy means mass is concentrated in a few bins (directional or calm regime).
When all values in the window are identical, range is zero and entropy is `0.0` (not `NaN`).

=== "Parameters"

    | Name | Type | Constraint | Description |
    |---|---|---|---|
    | `inputs` | `list[str]` | len = 1 | Input column, e.g. `["returns"]` |
    | `window` | `int` | >= 2 | Rolling window length |
    | `outputs` | `list[str]` | len = 1 | Output column, e.g. `["returns_entropy_20"]` |
    | `bins` | `int \| None` | >= 2 or `None` | Number of bins. `None` applies Sturges' rule. Default: `None` |
    | `normalize` | `bool` | - | If `True`, output is `H / ln(bins)` in [0, 1]. Default: `True` |

    **Sturges' rule** (default when `bins=None`): $k = \lceil 1 + \log_2(\text{window}) \rceil$.
    Computed once at construction. For `window=20` this gives `k=5`, for `window=200`, `k=9`.

=== "Output"

    | Column | When valid | Description |
    |---|---|---|
    | `outputs[0]` | `t >= window - 1`, no `NaN` in buffer | Shannon entropy in nats (`normalize=False`) or [0, 1] (`normalize=True`) |

=== "Behavior"

    - **Warm-up.** The first `window - 1` bars return `NaN`. A full buffer of `window`
    values is required before the first output.

    - **`NaN` propagation.** A `NaN` input enters the buffer. Output stays `NaN` until
    the `NaN` is evicted after `window` consecutive valid bars.

    - **Identical values.** When all `window` values are equal, range is zero and all
    mass falls in the first bin - entropy is `0.0`, not `NaN`. This represents
    maximum certainty (minimum disorder).

    - **`reset()`.** Clears the buffer entirely. Call it between backtest folds
    (CPCV, walk-forward) to avoid state leaking across splits.

    - **Implementation.** Rebuilds bin counts over the full buffer on every `update()`
    (`O(window)` per bar). Bins use equal-width partitioning of `[min, max]`.

    | Situation | Output |
    |---|---|
    | `t < window - 1` (buffer not full) | `NaN` |
    | Buffer full, all values valid | Entropy value |
    | Any `NaN` in the buffer | `NaN` |
    | All values identical (range = 0) | `0.0` |
    | After `reset()` | `NaN` until buffer refills |

=== "Interpretation"

    - **High entropy (near 1.0).** Returns are spread across the value range -
    no dominant direction, diffuse distribution. Associated with choppy or
    transitional regimes.

    - **Low entropy (near 0.0).** Returns are concentrated in a narrow region -
    the distribution is peaked. Associated with trending or calm regimes where
    most observations cluster together.

    - **Bin choice.** `bins=2` captures a simple high/low split and is the most
    stable. More bins (Sturges or explicit) resolve finer structure but add noise
    for small windows. As a rule: `bins <= window / 5` avoids empty bins on
    most real distributions.

    - **Normalize.** Use `normalize=True` (default) when comparing across assets
    or across different `bins` configs. Use `normalize=False` when you want the
    raw value in nats for downstream math (e.g. KL divergence).

=== "Example"

    ```python
    from oryon.features import ShannonEntropy

    se = ShannonEntropy(inputs=["x"], window=4, outputs=["entropy_4"],
                        bins=2, normalize=True)

    se.update([1.0])  # [nan]  - warm-up
    se.update([1.0])  # [nan]
    se.update([4.0])  # [nan]
    se.update([4.0])  # [1.0]  - window=[1,1,4,4]: 2 low, 2 high -> max entropy
    se.update([4.0])  # [0.811] - window=[1,4,4,4]: 1 low, 3 high -> entropy drops
    se.update([4.0])  # [0.0]  - window=[4,4,4,4]: range=0 -> minimum entropy
    ```

    Entropy falls from `1.0` (uniform split) to `0.811` (skewed 1/4 vs 3/4) to `0.0`
    (constant series). The `0.811` value equals $H^*([0.25, 0.75]) = 0.5623 / \ln 2$.

    ```python
    import pandas as pd
    from oryon.features import ShannonEntropy
    from oryon import FeaturePipeline
    from oryon.adapters import run_features_pipeline_pandas

    se = ShannonEntropy(["returns"], window=4, outputs=["returns_entropy_4"],
                        bins=2, normalize=True)
    fp = FeaturePipeline(features=[se], input_columns=["returns"])

    df = pd.DataFrame({"returns": [0.01, -0.02, 0.03, -0.01, 0.02, 0.03, 0.01]})
    out = run_features_pipeline_pandas(fp, df)
    print(out)
    #    returns_entropy_4
    # 0                NaN
    # 1                NaN
    # 2                NaN
    # 3              1.000   # counts=[2,2] -> uniform split -> max entropy
    # 4              1.000   # counts=[2,2]
    # 5              0.811   # counts=[1,3] -> skewed split -> entropy drops
    # 6              0.811   # counts=[1,3]
    ```

=== "Contributing"

    - **Incremental range tracking.** Each `update()` scans the full buffer to find
    `min` and `max` (`O(window)` extra). A sliding-window min/max structure (e.g.
    monotone deque) would reduce this to `O(1)` amortized, cutting the constant factor
    roughly in half for large windows.

    - **Additional bin methods.** Freedman-Diaconis (`h = 2 * IQR * n^{-1/3}`) and
    equal-frequency (quantile) bins are natural extensions. Equal-frequency bins
    in particular avoid empty bins and produce more stable entropy estimates on
    fat-tailed financial series. The `BinMethod` enum in Rust is already structured
    to accommodate new variants without changing existing behavior.

=== "Source"

    [:octicons-mark-github-16: `crates/oryon/src/features/shannon_entropy.rs`](https://github.com/lucasinglese/oryon/blob/main/crates/oryon/src/features/shannon_entropy.rs)

---

## Correlation

<a href="../../../getting-started/streaming-vs-research/#streaming-live-trading" class="oryon-badge oryon-badge--streaming">Streaming</a> <a href="../../../benchmarks/" class="oryon-badge oryon-badge--perf">&lt;2µs/update</a> <a href="../../../getting-started/streaming-vs-research/#research-full-dataset" class="oryon-badge oryon-badge--research">Research</a>

Rolling pairwise correlation between two series over a sliding window. Three methods
are supported, differing in what relationship they measure and their computational cost.

**Pearson** - product-moment correlation, measures linear co-movement:

$$
r = \frac{\sum_{i=1}^{n}(x_i - \bar{x})(y_i - \bar{y})}{\sqrt{\sum_{i=1}^{n}(x_i - \bar{x})^2 \cdot \sum_{i=1}^{n}(y_i - \bar{y})^2}}
$$

**Spearman** - Pearson correlation applied to the ranks of each series (average rank for ties):

$$
\rho = r\!\left(\text{rank}(x),\, \text{rank}(y)\right)
$$

**Kendall tau-b** - fraction of concordant minus discordant pairs, adjusted for ties:

$$
\tau_b = \frac{C - D}{\sqrt{(n_0 - n_1)(n_0 - n_2)}}
$$

where $n_0 = n(n-1)/2$, $n_1$ = pairs tied in $x$, $n_2$ = pairs tied in $y$,
$C$ = concordant pairs (same ordering in both series), $D$ = discordant pairs.

=== "Parameters"

    | Name | Type | Constraint | Description |
    |---|---|---|---|
    | `inputs` | `list[str]` | len >= 2 | Two input columns `[x, y]` |
    | `window` | `int` | >= 2 | Rolling window length |
    | `outputs` | `list[str]` | len = 1 | Output column, e.g. `["xy_corr_20"]` |
    | `method` | `str` | `'pearson'`, `'spearman'`, `'kendall'` | Correlation method. Default: `'pearson'` |

=== "Output"

    | Column | When valid | Description |
    |---|---|---|
    | `outputs[0]` | `t >= window - 1`, no `NaN` in buffer, neither series constant | Correlation coefficient in [-1, 1] |

=== "Behavior"

    - **Warm-up.** The first `window - 1` bars return `NaN`. A full buffer of `window`
    values is required for the first output.

    - **`NaN` propagation.** A `NaN` input enters the buffer. Output stays `NaN` until
    the `NaN` is evicted after `window` consecutive valid bars.

    - **Constant series.** If either series has zero variance over the window, the
    correlation is mathematically undefined and the output is `NaN`.

    - **`reset()`.** Clears both buffers entirely. Call between backtest folds
    (CPCV, walk-forward) to avoid state leaking across splits.

    - **Performance.** Measured at `w200` on Apple M-series:

    | Method | Complexity | w20 | w200 | Live-safe |
    |---|---|---|---|---|
    | `'pearson'` | O(n) | 38 ns | 373 ns | Yes |
    | `'spearman'` | O(n log n) | 273 ns | 1 648 ns | Yes (small-to-mid windows) |
    | `'kendall'` | O(n^2) | 247 ns | 17 372 ns | Small windows only (<=30) |

    | Situation | Output |
    |---|---|
    | `t < window - 1` (buffer not full) | `NaN` |
    | Buffer full, all values valid, neither series constant | Correlation value in [-1, 1] |
    | Any `NaN` in the buffer | `NaN` |
    | Either series constant over the window | `NaN` |
    | After `reset()` | `NaN` until buffer refills |

=== "Interpretation"

    - **Pearson.** Measures linear co-movement. Use for pairs trading (do two price
    series move together linearly?), factor exposure (is a return series linearly
    related to a risk factor?). Sensitive to outliers.

    - **Spearman.** Measures monotonic co-movement regardless of linearity. Ranks the
    values before correlating, so a consistently higher value in one series paired with
    a consistently higher value in the other scores +1, even if the relationship is
    exponential rather than linear. More robust to outliers than Pearson.

    - **Kendall.** Measures concordance: what fraction of pairs agree on ordering?
    Conceptually simpler than Spearman, has better statistical properties at small
    samples and is more robust to outliers, but is significantly slower. Prefer
    Spearman for large windows.

    - **Choosing a method.** For most quant use cases, Pearson is sufficient and fastest.
    Use Spearman when the relationship may be non-linear or the series are fat-tailed.
    Use Kendall for small windows when you need concordance-based statistics (e.g.
    comparing strategy rankings).

=== "Example"

    ```python
    from oryon.features import Correlation

    # Pearson: close prices vs volume — do they move together linearly?
    corr = Correlation(inputs=["close", "volume"], window=20, outputs=["cv_corr_20"])
    corr.update([100.0, 1_200_000.0])  # -> [NaN] (warm-up)
    # ... feed 20 bars ...

    # Spearman: rank correlation between two return series
    corr_sp = Correlation(inputs=["ret_a", "ret_b"], window=60,
                          outputs=["ret_corr_60"], method="spearman")

    # Kendall: concordance for small windows only
    corr_k = Correlation(inputs=["x", "y"], window=20,
                         outputs=["xy_tau_20"], method="kendall")
    ```

    Manual verification (window=3, `method='pearson'`):

    ```python
    corr = Correlation(inputs=["x", "y"], window=3, outputs=["corr"])
    corr.update([1.0, 1.0])   # [NaN]
    corr.update([2.0, 3.0])   # [NaN]
    result = corr.update([3.0, 2.0])
    # x=[1,2,3], y=[1,3,2]: Sxy=1, Sxx=2, Syy=2 -> r = 1/sqrt(4) = 0.5
    assert abs(result[0] - 0.5) < 1e-10
    ```

=== "Contributing"

    **Pearson - incremental O(1) is possible.**
    The current implementation recomputes over the full buffer on every bar (`O(n)`).
    An incremental version would maintain five running sums for the sliding window:
    $S_x$, $S_y$, $S_{xx}$, $S_{yy}$, $S_{xy}$. When bar $t$ enters and bar $t-n$
    is evicted, each sum is updated in O(1):

    $$r = \frac{n \cdot S_{xy} - S_x S_y}{\sqrt{(n \cdot S_{xx} - S_x^2)(n \cdot S_{yy} - S_y^2)}}$$

    This would reduce Pearson from 373 ns to sub-10 ns at `w200` - roughly on par with
    EMA. The main caveat is numerical stability: when $n \cdot S_{xx} \approx S_x^2$
    (near-constant series), catastrophic cancellation can occur. A Welford-style
    compensated accumulator mitigates this but adds implementation complexity. The
    two-pass approach currently used is unconditionally stable, which is why it was
    chosen first.

    **Spearman - no sub-O(n) approach known for a sliding window.**
    Every time one bar enters and one leaves, all ranks in the window can shift.
    There is no way to avoid touching all `n` ranks on each update. With an
    order-statistic tree (e.g. a Fenwick tree on compressed rank indices), individual
    rank lookups become O(log n) each, reducing the ranking step to O(n log n) with
    better cache behavior - but the asymptotic complexity stays O(n log n). The
    downstream Pearson step on ranks is still O(n). Bottom line: Spearman at `w200`
    is bounded below by O(n) and is unlikely to drop below a few hundred nanoseconds.

    **Kendall - incremental O(n) is possible (vs current O(n^2)).**
    When the window slides by one bar, only O(n) pairs change: the n-1 pairs
    involving the evicted element are removed, and n-1 new pairs involving the
    incoming element are added. A sliding-window implementation would maintain
    running concordance and tie counts, updating them in O(n) rather than
    recomputing all $n(n-1)/2$ pairs from scratch. This should significantly
    reduce the per-bar cost at large windows, with the goal of approaching the
    1-2 µs range at `w200`.

=== "Source"

    [:octicons-mark-github-16: `crates/oryon/src/features/correlation.rs`](https://github.com/lucasinglese/oryon/blob/main/crates/oryon/src/features/correlation.rs)