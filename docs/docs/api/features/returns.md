# Returns

Return features compute price changes over a configurable lookback window.
Both have `forward_period = 0` and are safe for live streaming.

---

## Simple Return

<a href="../../../getting-started/streaming-vs-research/#streaming-live-trading" class="oryon-badge oryon-badge--streaming">Streaming</a> <a href="../../../benchmarks/" class="oryon-badge oryon-badge--perf">&lt;1µs/update</a> <a href="../../../getting-started/streaming-vs-research/#research-full-dataset" class="oryon-badge oryon-badge--research">Research</a>

$$
r_t = \frac{P_t - P_{t-n}}{P_{t-n}}
$$

Arithmetic return from bar `t - window` to bar `t`. Returns `None` if the
reference price is zero or negative.

=== "Parameters"

    | Name | Type | Constraint | Description |
    |---|---|---|---|
    | `inputs` | `list[str]` | len = 1 | Input column, e.g. `["close"]` |
    | `window` | `int` | >= 1 | Lookback in bars ($n$). Use `1` for bar-to-bar return |
    | `outputs` | `list[str]` | len = 1 | Output column, e.g. `["close_ret_1"]` |

=== "Output"

    | Column | When valid | Description |
    |---|---|---|
    | `outputs[0]` | `t >= window`, `P_{t-n} > 0` | `(P_t - P_{t-n}) / P_{t-n}` |

=== "Behavior"

    - **Warm-up.** The first `window` bars return `NaN`. Both `P_{t-window}` and `P_t`
    must be in the buffer before a return can be computed.

    - **`NaN` propagation.** Only the two endpoints are used: `P_{t-window}` (lookback)
    and `P_t` (current). A `NaN` at either endpoint returns `NaN` for that bar. A `NaN`
    at an intermediate position does not affect the output — the return is still computed
    from the two valid endpoints.

    - **Zero or negative reference.** If `P_{t-window} <= 0`, the output is `NaN`
    for that bar only (the buffer is not affected).

    - **`reset()`.** Clears the buffer entirely. Call it between backtest folds
    (CPCV, walk-forward) to avoid state leaking across splits. After reset,
    the full `window` warm-up applies again.

    - **Implementation.** Maintains a rolling buffer of size `window + 1`.
    Computes `(buffer[window] - buffer[0]) / buffer[0]` on each valid bar (`O(1)` per update).

    | Situation | Output |
    |---|---|
    | `t < window` (buffer not full) | `NaN` |
    | Buffer full, `P_{t-n} > 0` | Simple return value |
    | `P_{t-n} <= 0` | `NaN` |
    | `P_t` or `P_{t-n}` is `NaN` | `NaN` |
    | `NaN` at intermediate position | Value computed from endpoints |
    | After `reset()` | `NaN` until buffer refills |

=== "Interpretation"

    - **Signal.** The actual percentage change over the lookback. Used when the
    percentage scale matters directly: portfolio attribution, position sizing, PnL.

    - **Not additive.** Compounding multiple periods requires multiplication, not
    addition. `r(A→C) ≠ r(A→B) + r(B→C)`.

=== "Example"

    ```python
    import pandas as pd
    from oryon.features import SimpleReturn
    from oryon import FeaturePipeline
from oryon.adapters import run_features_pipeline_pandas

    sr = SimpleReturn(["close"], window=1, outputs=["close_ret"])
    fp = FeaturePipeline(features=[sr], input_columns=["close"])

    df = pd.DataFrame({"close": [100.0, 102.0, 105.0, 103.0, 108.0]})
    out = run_features_pipeline_pandas(fp, df)
    print(out)
    #    close_ret
    # 0        NaN
    # 1     0.0200
    # 2     0.0294
    # 3    -0.0190
    # 4     0.0485
    ```

    With `window=1`, bar 0 is `NaN` (warm-up). From bar 1 onwards:
    `r_t = (P_t - P_{t-1}) / P_{t-1}`.

=== "Source"

    [:octicons-mark-github-16: `crates/oryon/src/features/simple_return.rs`](https://github.com/lucasinglese/oryon/blob/main/crates/oryon/src/features/simple_return.rs)

---

## Log Return

<a href="../../../getting-started/streaming-vs-research/#streaming-live-trading" class="oryon-badge oryon-badge--streaming">Streaming</a> <a href="../../../benchmarks/" class="oryon-badge oryon-badge--perf">&lt;1µs/update</a> <a href="../../../getting-started/streaming-vs-research/#research-full-dataset" class="oryon-badge oryon-badge--research">Research</a>

$$
r_t = \ln\frac{P_t}{P_{t-n}}
$$

Natural log of the price ratio from bar `t - window` to bar `t`. Log returns are
additive over time and better suited for statistical modeling than simple returns.
Returns `None` if either price is zero or negative.

=== "Parameters"

    | Name | Type | Constraint | Description |
    |---|---|---|---|
    | `inputs` | `list[str]` | len = 1 | Input column, e.g. `["close"]` |
    | `window` | `int` | >= 1 | Lookback in bars ($n$). Use `1` for bar-to-bar return |
    | `outputs` | `list[str]` | len = 1 | Output column, e.g. `["close_log_ret_1"]` |

=== "Output"

    | Column | When valid | Description |
    |---|---|---|
    | `outputs[0]` | `t >= window`, both prices `> 0` | `ln(P_t / P_{t-n})` |

=== "Behavior"

    - **Warm-up.** The first `window` bars return `NaN`. Both `P_{t-window}` and `P_t`
    must be in the buffer before a return can be computed.

    - **`NaN` propagation.** Only the two endpoints are used: `P_{t-window}` (lookback)
    and `P_t` (current). A `NaN` at either endpoint returns `NaN` for that bar. A `NaN`
    at an intermediate position does not affect the output — the return is still computed
    from the two valid endpoints.

    - **Zero or negative prices.** If either `P_{t-window} <= 0` or `P_t <= 0`, the
    output is `NaN` for that bar only (the buffer is not affected).

    - **`reset()`.** Clears the buffer entirely. Call it between backtest folds
    (CPCV, walk-forward) to avoid state leaking across splits. After reset,
    the full `window` warm-up applies again.

    - **Implementation.** Rolling buffer of size `window + 1`. `O(1)` per update,
    `O(N)` memory.

    | Situation | Output |
    |---|---|
    | `t < window` (buffer not full) | `NaN` |
    | Buffer full, both prices `> 0` | Log return value |
    | Either price `<= 0` | `NaN` |
    | `P_t` or `P_{t-n}` is `NaN` | `NaN` |
    | `NaN` at intermediate position | Value computed from endpoints |
    | After `reset()` | `NaN` until buffer refills |

=== "Interpretation"

    - **Additive.** `ln(P_t / P_0) = Σ ln(P_{t_i} / P_{t_{i-1}})` exactly. The sum
    of bar-to-bar log returns equals the log return over the full period. Simple
    returns do not have this property.

    - **Statistical modeling.** For small moves, log returns are better approximated
    by a normal distribution and are closer to stationary than price levels.

=== "Example"

    ```python
    import pandas as pd
    from oryon.features import LogReturn
    from oryon import FeaturePipeline
from oryon.adapters import run_features_pipeline_pandas

    lr = LogReturn(["close"], window=1, outputs=["close_log_ret"])
    fp = FeaturePipeline(features=[lr], input_columns=["close"])

    df = pd.DataFrame({"close": [100.0, 102.0, 105.0, 103.0, 108.0]})
    out = run_features_pipeline_pandas(fp, df)
    print(out)
    #    close_log_ret
    # 0            NaN
    # 1         0.0198
    # 2         0.0290
    # 3        -0.0190
    # 4         0.0473
    ```

    With `window=1`, bar 0 is `NaN`. Log returns are approximately equal to
    simple returns for small moves, but are strictly additive: summing bars 1 to 4
    gives `ln(108/100)`.

=== "Source"

    [:octicons-mark-github-16: `crates/oryon/src/features/log_return.rs`](https://github.com/lucasinglese/oryon/blob/main/crates/oryon/src/features/log_return.rs)