# Trend

Trend features are backward-looking. They only use past and present data
and are safe for live streaming (`forward_period = 0`).

---

## Simple Moving Average (SMA)

<a href="../../../getting-started/streaming-vs-research/#streaming-live-trading" class="oryon-badge oryon-badge--streaming">Streaming</a> <a href="../../../benchmarks/" class="oryon-badge oryon-badge--perf">&lt;1µs/update</a> <a href="../../../getting-started/streaming-vs-research/#research-full-dataset" class="oryon-badge oryon-badge--research">Research</a>

$$
\text{SMA}_t = \frac{1}{N} \sum_{i=0}^{N-1} x_{t-i}
$$

The arithmetic mean of the last $N$ bars, with equal weight on every observation.
The simplest trend smoother: it reduces noise, but it lags by construction. The
larger the window, the smoother and the slower.

=== "Parameters"

    | Name | Type | Constraint | Description |
    |---|---|---|---|
    | `inputs` | `list[str]` | len = 1 | Input column, e.g. `["close"]` |
    | `window` | `int` | >= 1 | Number of bars to average ($N$) |
    | `outputs` | `list[str]` | len = 1 | Output column, e.g. `["close_sma_20"]` |

=== "Output"

    | Column | When valid | Description |
    |---|---|---|
    | `outputs[0]` | `t >= window - 1`, no `NaN` in buffer | Rolling arithmetic mean over the last `window` bars |

=== "Behavior"

    - **Warm-up.** The first `window - 1` bars return `NaN`. The buffer must hold
    exactly `window` values before a mean can be computed.

    - **`NaN` propagation.** A single `NaN` input contaminates the buffer.
    The output stays `NaN` until that value is evicted, i.e. until `window`
    consecutive valid bars have been seen.

    - **`window = 1`.** No warm-up. Output equals the input at every bar.

    - **`reset()`.** Clears the buffer entirely. Call it between backtest folds
    (CPCV, walk-forward) to avoid state leaking across splits. After reset,
    the full `window - 1` warm-up applies again.

    - **Implementation.** Recomputes the sum over the full buffer on every `update()` call
    (`O(N)` per bar, `O(N)` memory). For typical window sizes the overhead is negligible;
    see [Benchmarks](../../../benchmarks/). A running-sum approach would
    bring this to `O(1)` per bar with no change to output.

    | Situation | Output |
    |---|---|
    | `t < window - 1` (buffer not full) | `NaN` |
    | Buffer full, all values valid | SMA value |
    | Any value in the buffer is `NaN` | `NaN` |
    | `window = 1` | Input value (immediate, no warm-up) |
    | After `reset()` | `NaN` until buffer refills |

=== "Interpretation"

    - **Signal.** Price above SMA suggests an uptrend; price below suggests a downtrend.
    A crossover of two SMAs with different windows is one of the most widely traded
    trend signals.

    - **Lag.** Equal weight on every bar in the window. The larger the window, the
    smoother the output but the more it lags behind the actual price. This lag is
    structural and by design.

=== "Example"

    ```python
    import pandas as pd
    from oryon.features import Sma
    from oryon import FeaturePipeline, run_features_pipeline

    sma = Sma(["close"], window=3, outputs=["close_sma_3"])
    fp  = FeaturePipeline(features=[sma], input_columns=["close"])

    df = pd.DataFrame({"close": [100.0, 101.0, 102.0, 103.0, None, 104.0, 105.0, 106.0]})
    out = run_features_pipeline(fp, df)
    print(out)
    #    close_sma_3
    # 0          NaN
    # 1          NaN
    # 2       101.00
    # 3       102.00
    # 4          NaN
    # 5          NaN
    # 6          NaN
    # 7       105.00
    ```

    Step-by-step with `window = 3`:

    | Bar | Input | Buffer | Output |
    |-----|-------|--------|--------|
    | 0 | 100.0 | `[100]` | `NaN` |
    | 1 | 101.0 | `[100, 101]` | `NaN` |
    | 2 | 102.0 | `[100, 101, 102]` | **101.0** |
    | 3 | 103.0 | `[101, 102, 103]` | **102.0** |
    | 4 | `NaN` | `[102, 103, NaN]` | `NaN` |
    | 5 | 104.0 | `[103, NaN, 104]` | `NaN` |
    | 6 | 105.0 | `[NaN, 104, 105]` | `NaN` |
    | 7 | 106.0 | `[104, 105, 106]` | **105.0** |

    Bars 4-6 are `NaN` because the `NaN` at bar 4 remains in the buffer until bar 7 evicts it.

=== "Source"

    [:octicons-mark-github-16: `crates/oryon/src/features/sma.rs`](https://github.com/lucasinglese/oryon/blob/main/crates/oryon/src/features/sma.rs)

=== "Contribute"

    **O(N) to O(1) per update.**
    `update()` currently recomputes the sum over the full buffer on every bar
    by delegating to `average()` in `ops/stats.rs`. A running-sum approach would
    maintain a single accumulator: add the incoming value, subtract the evicted one.
    This brings the per-update cost from `O(N)` to `O(1)` with no change to numerical output.

---

## Exponential Moving Average (EMA)

<a href="../../../getting-started/streaming-vs-research/#streaming-live-trading" class="oryon-badge oryon-badge--streaming">Streaming</a> <a href="../../../benchmarks/" class="oryon-badge oryon-badge--perf">&lt;1µs/update</a> <a href="../../../getting-started/streaming-vs-research/#research-full-dataset" class="oryon-badge oryon-badge--research">Research</a>

$$
\text{EMA}_t = \alpha \cdot x_t + (1 - \alpha) \cdot \text{EMA}_{t-1}, \quad \alpha = \frac{2}{N+1}
$$

Weights recent observations more heavily than older ones using an exponential decay factor.
It reacts faster to price changes than the SMA and maintains only a single state value, making it `O(1)` per update after seeding.

=== "Parameters"

    | Name | Type | Constraint | Description |
    |---|---|---|---|
    | `inputs` | `list[str]` | len = 1 | Input column, e.g. `["close"]` |
    | `window` | `int` | >= 1 | Span for the smoothing factor ($N$) |
    | `outputs` | `list[str]` | len = 1 | Output column, e.g. `["close_ema_20"]` |

=== "Output"

    | Column | When valid | Description |
    |---|---|---|
    | `outputs[0]` | `t >= window - 1`, no `NaN` since last reset | Exponentially weighted mean |

=== "Behavior"

    - **Warm-up.** The first `window` bars are used to seed the EMA with their SMA.
    The first valid output appears at bar `window - 1`.

    - **`NaN` propagation.** A `NaN` input fully resets the state: `prev_ema` is cleared
    and the seeding phase restarts from scratch. `window` consecutive valid bars are
    required after any `NaN` before output resumes.

    - **`window = 1`.** Alpha equals `1.0`. Output equals the input at every bar, no warm-up.

    - **`reset()`.** Clears `prev_ema` and the seed buffer entirely. Call it between
    backtest folds (CPCV, walk-forward) to avoid state leaking across splits. After
    reset, the full `window - 1` warm-up applies again.

    - **Implementation.** After seeding, only `prev_ema` is maintained in memory (`O(1)`
    per update, `O(1)` memory in the recursive phase). The seed buffer is cleared once
    seeding completes.

    | Situation | Output |
    |---|---|
    | `t < window - 1` (seeding) | `NaN` |
    | `t == window - 1` (seed bar) | SMA of first `window` bars |
    | Recursive phase, valid input | EMA value |
    | Any `NaN` input | `NaN` + full state reset |
    | `window = 1` | Input value (immediate, no warm-up) |
    | After `reset()` | `NaN` until reseeded |

=== "Interpretation"

    - **Signal.** Exponentially weighted smoother with recency bias. Recent bars have
    more influence than older ones, making EMA faster at tracking regime changes than
    equal-weight smoothers.

    - **Production.** Recursive update with a single state variable: `O(1)` time and
    memory after seeding. The cheapest smoother to run in a live pipeline.

=== "Example"

    ```python
    import pandas as pd
    from oryon.features import Ema
    from oryon import FeaturePipeline, run_features_pipeline

    ema = Ema(["close"], window=3, outputs=["close_ema_3"])
    fp  = FeaturePipeline(features=[ema], input_columns=["close"])

    df = pd.DataFrame({"close": [100.0, 101.0, 102.0, 103.0, None, 104.0, 105.0, 106.0]})
    out = run_features_pipeline(fp, df)
    print(out)
    #    close_ema_3
    # 0          NaN
    # 1          NaN
    # 2       101.00
    # 3       102.00
    # 4          NaN
    # 5          NaN
    # 6          NaN
    # 7       105.00
    ```

    Step-by-step with `window = 3`, `alpha = 0.5`:

    | Bar | Input | Phase | State | Output |
    |-----|-------|-------|-------|--------|
    | 0 | 100.0 | Seeding | buffer=`[100]` | `NaN` |
    | 1 | 101.0 | Seeding | buffer=`[100, 101]` | `NaN` |
    | 2 | 102.0 | Seeding | seed = SMA(`[100, 101, 102]`) = 101.0 | **101.0** |
    | 3 | 103.0 | Recursive | 0.5×103 + 0.5×101 = 102.0 | **102.0** |
    | 4 | `NaN` | Reset | state cleared | `NaN` |
    | 5 | 104.0 | Seeding | buffer=`[104]` | `NaN` |
    | 6 | 105.0 | Seeding | buffer=`[104, 105]` | `NaN` |
    | 7 | 106.0 | Seeding | seed = SMA(`[104, 105, 106]`) = 105.0 | **105.0** |

    At bar 4, `NaN` triggers a full state reset. The EMA reseeds from scratch rather
    than waiting for the `NaN` to be evicted from a buffer.

=== "Source"

    [:octicons-mark-github-16: `crates/oryon/src/features/ema.rs`](https://github.com/lucasinglese/oryon/blob/main/crates/oryon/src/features/ema.rs)

---

## Kaufman Adaptive Moving Average (KAMA)

<a href="../../../getting-started/streaming-vs-research/#streaming-live-trading" class="oryon-badge oryon-badge--streaming">Streaming</a> <a href="../../../benchmarks/" class="oryon-badge oryon-badge--perf">&lt;1µs/update</a> <a href="../../../getting-started/streaming-vs-research/#research-full-dataset" class="oryon-badge oryon-badge--research">Research</a>

$$
\text{ER}_t = \frac{|P_t - P_{t-N}|}{\sum_{i=1}^{N} |P_i - P_{i-1}|}
$$

$$
\text{SC}_t = \bigl(\text{ER}_t \cdot (\alpha_f - \alpha_s) + \alpha_s\bigr)^2
$$

$$
\text{KAMA}_t = \text{KAMA}_{t-1} + \text{SC}_t \cdot (P_t - \text{KAMA}_{t-1})
$$

Adapts its smoothing speed based on market efficiency. In trending markets the
Efficiency Ratio (ER) approaches 1 and KAMA tracks price closely. In choppy
markets ER approaches 0 and KAMA barely moves, suppressing noise.

!!! tip "Default parameters"
    `fast=2, slow=30` match Kaufman's original 1998 paper and are well market-tested. Only change them if you have a specific calibration reason.

=== "Parameters"

    | Name | Type | Constraint | Description |
    |---|---|---|---|
    | `inputs` | `list[str]` | len = 1 | Input column, e.g. `["close"]` |
    | `window` | `int` | >= 1 | ER lookback ($N$). Kaufman default: `10` |
    | `outputs` | `list[str]` | len = 1 | Output column, e.g. `["close_kama_10"]` |
    | `fast` | `int` | >= 1 | Fast smoothing period ($\alpha_f = 2/(fast+1)$). Default: `2` |
    | `slow` | `int` | > `fast` | Slow smoothing period ($\alpha_s = 2/(slow+1)$). Default: `30` |

=== "Output"

    | Column | When valid | Description |
    |---|---|---|
    | `outputs[0]` | `t >= window`, no `NaN` in window | Adaptive smoothed value |

=== "Behavior"

    - **Warm-up.** Requires `window + 1` bars to compute the first ER. The first valid
    output appears at bar `window` (one bar later than SMA/EMA with the same `window`).

    - **`NaN` propagation.** A `NaN` anywhere in the current window resets `prev_kama`
    and returns `None`. Once the window no longer contains the `NaN`, KAMA re-seeds
    automatically using `prices[window - 1]` as the starting value.

    - **`reset()`.** Clears `prev_kama` and the price buffer. Call it between backtest
    folds (CPCV, walk-forward) to avoid state leaking across splits. After reset,
    the full `window` warm-up applies again.

    - **Implementation.** Iterates over the full buffer on every `update()` to compute
    direction, volatility, and ER (`O(N)` per bar, `O(N)` memory).

    | Situation | Output |
    |---|---|
    | `t < window` (buffer not full) | `NaN` |
    | Buffer full, all values valid | KAMA value |
    | Any `NaN` in the window | `NaN` + `prev_kama` reset |
    | After `reset()` | `NaN` until buffer refills |

=== "Interpretation"

    - **Signal.** Adaptive smoother. Smoothing speed adjusts automatically based on
    the Efficiency Ratio (ER): fast during directional moves (ER near 1), slow
    during noisy/ranging markets (ER near 0). This built-in regime detection
    reduces false signals compared to fixed-speed smoothers.

    - **The ER as a standalone feature.** The Efficiency Ratio that KAMA uses internally
    is independently useful. It quantifies how directional the market is over a window,
    a direct measure of signal-to-noise ratio in price movement.

    - **References.** Kaufman, P.J. (1995), *Smarter Trading*. Expanded in *Trading
    Systems and Methods* (6th ed., 2020).

=== "Example"

    ```python
    import pandas as pd
    from oryon.features import Kama
    from oryon import FeaturePipeline, run_features_pipeline

    kama = Kama(["close"], window=3, outputs=["close_kama_3"], fast=2, slow=5)
    fp   = FeaturePipeline(features=[kama], input_columns=["close"])

    df = pd.DataFrame({"close": [100.0, 101.0, 103.0, 102.0, 105.0, 107.0, 106.0]})
    out = run_features_pipeline(fp, df)
    print(out)
    #    close_kama_3
    # 0           NaN
    # 1           NaN
    # 2           NaN
    # 3        102.75
    # 4        103.44
    # 5        104.54
    # 6        104.99
    ```

    With `window=3`, `fast=2`, `slow=5` (α_f=2/3, α_s=1/3). Bars 0-2 are `NaN`
    because `window + 1 = 4` bars are needed. At bar 3: ER=0.5, SC=0.25,
    seed=103.0 → KAMA = 103.0 + 0.25×(102-103) = **102.75**.

=== "Source"

    [:octicons-mark-github-16: `crates/oryon/src/features/kama.rs`](https://github.com/lucasinglese/oryon/blob/main/crates/oryon/src/features/kama.rs)

---

## Linear Slope

<a href="../../../getting-started/streaming-vs-research/#streaming-live-trading" class="oryon-badge oryon-badge--streaming">Streaming</a> <a href="../../../benchmarks/" class="oryon-badge oryon-badge--perf">&lt;1µs/update</a> <a href="../../../getting-started/streaming-vs-research/#research-full-dataset" class="oryon-badge oryon-badge--research">Research</a>

$$
\hat{\beta} = \frac{S_{xy}}{S_{xx}}, \quad R^2 = \frac{S_{xy}^2}{S_{xx} \cdot S_{yy}}
$$

$$
S_{xy} = \sum (x_i - \bar{x})(y_i - \bar{y})
$$

Fits an OLS regression of `y` on `x` over a rolling window and outputs the slope and
R² at each bar. Useful for quantifying trend direction, strength, and linearity of price movement.

!!! tip "Choosing x"
    Pass a simple integer index `[0, 1, 2, ...]` as `x` to get slope in price-per-bar units. If you pass timestamps, the slope becomes price-per-nanosecond and is much harder to read.

=== "Parameters"

    | Name | Type | Constraint | Description |
    |---|---|---|---|
    | `inputs` | `list[str]` | len >= 2 | Two columns in order: `[x_col, y_col]` |
    | `window` | `int` | >= 2 | Rolling window length |
    | `outputs` | `list[str]` | len = 2 | Output columns: `[slope_col, r2_col]` |

=== "Output"

    | Column | When valid | Description |
    |---|---|---|
    | `outputs[0]` | `t >= window - 1`, no `NaN`, `x` not constant | OLS slope over the last `window` bars |
    | `outputs[1]` | Same as slope, and `y` not constant | Coefficient of determination R² |

=== "Behavior"

    - **Warm-up.** The first `window - 1` bars return `NaN` for both outputs. A full
    window of `x` and `y` values is required before regression can be computed.

    - **`NaN` propagation.** A `NaN` in either `x` or `y` contaminates both outputs.
    They stay `NaN` until that bar is evicted, i.e. until `window` consecutive valid
    pairs have been seen.

    - **Degenerate cases.** If `x` is constant over the window (`S_xx = 0`), both outputs
    are `NaN`. If `y` is constant (`S_yy = 0`), slope is valid (returns `0.0`) but R²
    is `NaN`.

    - **`reset()`.** Clears both the `x` and `y` buffers entirely. Call it between
    backtest folds (CPCV, walk-forward) to avoid state leaking across splits. After
    reset, the full `window - 1` warm-up applies again.

    - **Implementation.** Two passes over the window on every `update()`: one to compute
    means, one for $S_{xx}$, $S_{xy}$, $S_{yy}$ (`O(N)` per bar, `O(N)` memory).

    | Situation | Slope | R² |
    |---|---|---|
    | `t < window - 1` | `NaN` | `NaN` |
    | Buffer full, all values valid | OLS slope | R² value |
    | Any `NaN` in either buffer | `NaN` | `NaN` |
    | `x` constant (`S_xx = 0`) | `NaN` | `NaN` |
    | `y` constant (`S_yy = 0`) | `0.0` | `NaN` |
    | After `reset()` | `NaN` | `NaN` |

=== "Interpretation"

    - **Signal.** Slope and R² together give a richer picture than any moving average.
    Slope captures trend direction and magnitude. R² captures trend quality - how
    linear the movement is over the window. What counts as a "high" R² depends
    entirely on the inputs: time-vs-price regressions can reach 0.9, while
    volume-vs-price rarely exceeds 0.2.

=== "Example"

    ```python
    import pandas as pd
    from oryon.features import LinearSlope
    from oryon import FeaturePipeline, run_features_pipeline

    ls = LinearSlope(
        ["time_idx", "close"], window=3,
        outputs=["close_slope_3", "close_r2_3"],
    )
    fp = FeaturePipeline(features=[ls], input_columns=["time_idx", "close"])

    df = pd.DataFrame({
        "time_idx": [0.0, 1.0, 2.0, 3.0, 4.0],
        "close":    [100.0, 103.0, 106.0, 109.0, 112.0],
    })
    out = run_features_pipeline(fp, df)
    print(out)
    #    close_slope_3  close_r2_3
    # 0            NaN         NaN
    # 1            NaN         NaN
    # 2            3.0         1.0
    # 3            3.0         1.0
    # 4            3.0         1.0
    ```

    Price increases by exactly 3.0 per bar, so slope=3.0 and R²=1.0 (perfect linear fit)
    at every valid bar.

=== "Source"

    [:octicons-mark-github-16: `crates/oryon/src/features/linear_slope.rs`](https://github.com/lucasinglese/oryon/blob/main/crates/oryon/src/features/linear_slope.rs)

=== "Contribute"

    **O(N) to O(1) per update.**
    `update()` currently runs two full passes over the window to compute means then
    $S_{xx}$, $S_{xy}$, $S_{yy}$. An incremental approach would maintain five running
    sums (`n`, `sum_x`, `sum_y`, `sum_xx`, `sum_xy`, `sum_yy`) updated in `O(1)` by
    adding the incoming pair and subtracting the evicted one, similar to a Welford-style
    online algorithm.
