# Volatility

Volatility targets compute realized volatility over a future window.
They have `warm_up_period = 0` and `forward_period = horizon`.

---

## Future CTC Volatility

<a href="../../../getting-started/streaming-vs-research/#research-full-dataset" class="oryon-badge oryon-badge--research">Research only</a> <a href="../../../benchmarks/" class="oryon-badge oryon-badge--perf">&lt;300µs/1k bars</a>

$$
\sigma_t = \text{std}\!\left(\ln\frac{x_{t+1}}{x_t},\, \ln\frac{x_{t+2}}{x_{t+1}},\, \ldots,\, \ln\frac{x_{t+h}}{x_{t+h-1}}\right)
$$

Close-to-close realized volatility over the next `horizon` bars. Computes the
sample standard deviation of log returns over the forward window. The output
name is auto-generated as `{input}_future_ctc_vol_{horizon}`.

=== "Parameters"

    | Name | Type | Constraint | Description |
    |---|---|---|---|
    | `input` | `str` | non-empty | Price column, e.g. `"close"` |
    | `horizon` | `int` | >= 1 | Number of bars to look ahead ($h$) |

=== "Output"

    | Column | When valid | Description |
    |---|---|---|
    | `{input}_future_ctc_vol_{horizon}` | `t <= N - horizon - 1` | Sample std of `horizon` log returns starting at `t+1` |

=== "Properties"

    | Property | Value |
    |---|---|
    | `warm_up_period` | `0` |
    | `forward_period` | `horizon` |

=== "Behavior"

    - **Forward NaN.** The last `horizon` values are `NaN` because the future window
    is not complete. The first valid value appears at bar 0.

    - **`NaN` in prices.** If any price in the forward window is `NaN` or non-positive,
    the corresponding log return is `NaN`, and the standard deviation returns `NaN`.

    - **Stateless.** `run_research()` has no internal state. Calling it twice with the
    same input always returns the same output.

    - **Implementation.** Computes log returns via a pairwise pass, then applies a
    rolling sample standard deviation with `shift` to align results to bar `t`.

=== "Interpretation"

    - **Label.** The standard label for realized volatility forecasting. Captures
    how much the price moved over the next `horizon` bars, regardless of direction.

    - **Units.** Log-return scale, not annualized. To annualize, multiply by
    `sqrt(periods_per_year)`.

=== "Example"

    ```python
    import pandas as pd
    from oryon.targets import FutureCTCVolatility
    from oryon import TargetPipeline
from oryon.adapters import run_targets_pipeline_pandas

    t = FutureCTCVolatility(input="close", horizon=3)
    print(t.output_names())  # ['close_future_ctc_vol_3']

    tp = TargetPipeline(targets=[t], input_columns=["close"])
    df = pd.DataFrame({
        "close": [100.0, 101.0, 103.0, 102.0, 105.0, 107.0, 106.0],
    })
    out = run_targets_pipeline_pandas(tp, df)
    print(out)
    #    close_future_ctc_vol_3
    # 0                  0.0150
    # 1                  0.0202
    # 2                  0.0201
    # 3                  0.0199
    # 4                     NaN
    # 5                     NaN
    # 6                     NaN
    ```

=== "Source"

    [:octicons-mark-github-16: `crates/oryon/src/targets/future_ctc_volatility.rs`](https://github.com/lucasinglese/oryon/blob/main/crates/oryon/src/targets/future_ctc_volatility.rs)
