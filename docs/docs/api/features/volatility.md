# Volatility

Volatility features estimate the dispersion or range of a price series using
OHLC data. All have `forward_period = 0` and are safe for live streaming.

---

## Parkinson Volatility

<a href="../../../getting-started/streaming-vs-research/#streaming-live-trading" class="oryon-badge oryon-badge--streaming">Streaming</a> <a href="../../../benchmarks/" class="oryon-badge oryon-badge--perf">&lt;1µs/update</a> <a href="../../../getting-started/streaming-vs-research/#research-full-dataset" class="oryon-badge oryon-badge--research">Research</a>

$$
\sigma_P = \sqrt{\frac{1}{4 N \ln 2} \sum_{i=0}^{N-1} \left(\ln \frac{H_{t-i}}{L_{t-i}}\right)^2}
$$

Uses the high-low range to estimate realized volatility (Parkinson, 1980). More efficient
than close-to-close volatility (roughly 4-5x lower variance) because it uses intra-bar
information. Assumes Brownian motion without drift. Less accurate in trending markets.

!!! tip "Trending markets"
    In strongly trending markets, Rogers-Satchell is more accurate: it accounts for directional drift where Parkinson overestimates.

=== "Parameters"

    | Name | Type | Constraint | Description |
    |---|---|---|---|
    | `inputs` | `list[str]` | len >= 2, order: `[high, low]` | High and low columns, in that order |
    | `window` | `int` | >= 1 | Rolling window length ($N$) |
    | `outputs` | `list[str]` | len = 1 | Output column, e.g. `["parkinson_vol_20"]` |

=== "Output"

    | Column | When valid | Description |
    |---|---|---|
    | `outputs[0]` | `t >= window - 1`, all bars valid | Parkinson volatility (not annualized) |

=== "Behavior"

    **Warm-up.** The first `window - 1` bars return `NaN`. A full buffer is required.

    **`NaN` propagation.** A `NaN` in either high or low contaminates the per-bar term
    stored in the buffer. Output stays `NaN` until that bar is evicted.

    **Invalid bar.** If `high < low` or either value is zero or negative, the per-bar
    term is stored as `NaN`, propagating as described above.

    **`reset()`.** Clears the buffer entirely. Call it between backtest folds
    (CPCV, walk-forward) to avoid state leaking across splits.

    **Implementation.** Stores the per-bar `ln(H/L)^2` term in the buffer, then
    computes `sqrt(mean(buffer) / (4 * ln(2)))` (`O(N)` per bar).

    | Situation | Output |
    |---|---|
    | `t < window - 1` (buffer not full) | `NaN` |
    | Buffer full, all bars valid | Parkinson volatility |
    | Any `NaN` or invalid bar (`high < low`) in buffer | `NaN` |
    | After `reset()` | `NaN` until buffer refills |

=== "Example"

    ```python
    import pandas as pd
    from oryon.features import ParkinsonVolatility
    from oryon import FeaturePipeline, run_features_pipeline

    pv = ParkinsonVolatility(["high", "low"], window=3, outputs=["pv_3"])
    fp = FeaturePipeline(features=[pv], input_columns=["high", "low"])

    df = pd.DataFrame({
        "high": [102.0, 104.0, 103.0, 106.0, 108.0],
        "low":  [ 99.0, 101.0, 100.0, 103.0, 105.0],
    })
    out = run_features_pipeline(fp, df)
    print(out)
    #         pv_3
    # 0        NaN
    # 1        NaN
    # 2     0.0178
    # 3     0.0175
    # 4     0.0173
    ```

=== "Source"

    [:octicons-mark-github-16: `crates/oryon/src/features/parkinson_volatility.rs`](https://github.com/Quantreo/oryon/blob/main/crates/oryon/src/features/parkinson_volatility.rs)

---

## Rogers-Satchell Volatility

<a href="../../../getting-started/streaming-vs-research/#streaming-live-trading" class="oryon-badge oryon-badge--streaming">Streaming</a> <a href="../../../benchmarks/" class="oryon-badge oryon-badge--perf">&lt;1µs/update</a> <a href="../../../getting-started/streaming-vs-research/#research-full-dataset" class="oryon-badge oryon-badge--research">Research</a>

$$
\sigma_{RS} = \sqrt{\frac{1}{N} \sum_{i=0}^{N-1} \left[ \ln\frac{H}{C}\ln\frac{H}{O} + \ln\frac{L}{C}\ln\frac{L}{O} \right]_{t-i}}
$$

Uses all four OHLC prices (Rogers and Satchell, 1994). Unlike Parkinson, it is unbiased
in the presence of a directional drift, making it more accurate in trending markets.
Individual bar terms can be negative for unusual price action. The rolling mean
must be positive for a valid output.

!!! tip "Range-bound markets"
    In mean-reverting or range-bound markets, Parkinson is a simpler alternative: it uses only high and low, has no negative-term edge case, and is slightly cheaper to compute.

=== "Parameters"

    | Name | Type | Constraint | Description |
    |---|---|---|---|
    | `inputs` | `list[str]` | len >= 4, order: `[high, low, open, close]` | OHLC columns, in that exact order |
    | `window` | `int` | >= 1 | Rolling window length ($N$) |
    | `outputs` | `list[str]` | len = 1 | Output column, e.g. `["rs_vol_20"]` |

=== "Output"

    | Column | When valid | Description |
    |---|---|---|
    | `outputs[0]` | `t >= window - 1`, all bars valid, mean > 0 | Rogers-Satchell volatility (not annualized) |

=== "Behavior"

    **Warm-up.** The first `window - 1` bars return `NaN`.

    **`NaN` propagation.** A `NaN` in any of the four OHLC values, or an invalid bar
    (`high < low` or any value zero/negative), stores `NaN` in the buffer. Output
    stays `NaN` until that bar is evicted.

    **Non-positive mean.** If the rolling mean of per-bar terms is zero or negative
    (unusual price action with many reversals), the output is `NaN`.

    **`reset()`.** Clears the buffer entirely. Call it between backtest folds.

    **Implementation.** Stores the per-bar RS term in the buffer, then computes
    `sqrt(mean(buffer))` if positive (`O(N)` per bar).

    | Situation | Output |
    |---|---|
    | `t < window - 1` (buffer not full) | `NaN` |
    | Buffer full, all bars valid, mean > 0 | RS volatility |
    | Any `NaN` or invalid bar in buffer | `NaN` |
    | Rolling mean of terms <= 0 | `NaN` |
    | After `reset()` | `NaN` until buffer refills |

=== "Example"

    ```python
    import pandas as pd
    from oryon.features import RogersSatchellVolatility
    from oryon import FeaturePipeline, run_features_pipeline

    rs = RogersSatchellVolatility(
        ["high", "low", "open", "close"], window=3, outputs=["rs_vol_3"]
    )
    fp = FeaturePipeline(features=[rs], input_columns=["high", "low", "open", "close"])

    df = pd.DataFrame({
        "high":  [108.0, 108.0, 108.0, 110.0],
        "low":   [104.0, 104.0, 104.0, 106.0],
        "open":  [105.0, 105.0, 105.0, 107.0],
        "close": [107.0, 107.0, 107.0, 109.0],
    })
    out = run_features_pipeline(fp, df)
    print(out)
    #     rs_vol_3
    # 0        NaN
    # 1        NaN
    # 2     0.0231
    # 3     0.0231
    ```

    !!! warning "Input order"
        Inputs must be `[high, low, open, close]` in that exact order. Passing
        `[open, high, low, close]` will produce incorrect results silently.

=== "Source"

    [:octicons-mark-github-16: `crates/oryon/src/features/rogers_satchell_volatility.rs`](https://github.com/Quantreo/oryon/blob/main/crates/oryon/src/features/rogers_satchell_volatility.rs)
