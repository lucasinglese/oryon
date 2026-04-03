# Regressions

Regression targets produce continuous labels that represent future price
behavior. All have `warm_up_period = 0` and `forward_period > 0`.

---

## Future Return

<a href="../../../getting-started/streaming-vs-research/#research-full-dataset" class="oryon-badge oryon-badge--research">Research only</a> <a href="../../../benchmarks/" class="oryon-badge oryon-badge--perf">&lt;2µs/1k bars</a>

$$
y_t = \frac{P_{t+h} - P_t}{P_t}
$$

Simple return from bar `t` to bar `t + horizon`. Identical formula to
`SimpleReturn` but looks forward instead of backward.

=== "Parameters"

    | Name | Type | Constraint | Description |
    |---|---|---|---|
    | `inputs` | `list[str]` | len >= 1 | Price column, e.g. `["close"]` |
    | `horizon` | `int` | >= 1 | Number of bars to look ahead ($h$) |
    | `outputs` | `list[str]` | len >= 1 | Output column name |

=== "Output"

    | Column | When valid | Description |
    |---|---|---|
    | `outputs[0]` | `t <= N - horizon - 1` | `(P_{t+h} - P_t) / P_t` |

=== "Properties"

    | Property | Value |
    |---|---|
    | `warm_up_period` | `0` |
    | `forward_period` | `horizon` |

=== "Behavior"

    **Forward None.** The last `horizon` values are `None` because the future
    window is not complete. The first valid value appears at bar 0.

    **None in prices.** If any price in the window is `None` or the reference
    price `P_t` is zero or negative, the output is `None` for that bar.

    **Stateless.** `run_research()` has no internal state. Calling it twice with
    the same input always returns the same output.

    **Implementation.** Shifts the price series by `-horizon`, then applies
    `simple_return` pairwise: `(P_{t+h} - P_t) / P_t` (`O(N)` total).

=== "Example"

    ```python
    import pandas as pd
    from oryon.targets import FutureReturn
    from oryon import TargetPipeline, run_targets_pipeline

    t = FutureReturn(inputs=["close"], horizon=2, outputs=["close_fr_2"])
    tp = TargetPipeline(targets=[t], input_columns=["close"])

    df = pd.DataFrame({
        "close": [100.0, 102.0, 105.0, 103.0, 108.0],
    })
    out = run_targets_pipeline(tp, df)
    print(out)
    #    close_fr_2
    # 0      0.0500
    # 1      0.0098
    # 2      0.0286
    # 3         NaN
    # 4         NaN
    ```

    Bar 0: `(105 - 100) / 100 = 0.05`. Bar 1: `(103 - 102) / 102 ≈ 0.0098`.
    Bars 3 and 4 are `NaN` because the future window extends beyond the series.

=== "Source"

    [:octicons-mark-github-16: `crates/oryon/src/targets/future_return.rs`](https://github.com/lucasinglese/oryon/blob/main/crates/oryon/src/targets/future_return.rs)

---

## Future Linear Slope

<a href="../../../getting-started/streaming-vs-research/#research-full-dataset" class="oryon-badge oryon-badge--research">Research only</a> <a href="../../../benchmarks/" class="oryon-badge oryon-badge--perf">&lt;300µs/1k bars</a>

$$
\hat{a} = \frac{S_{xy}}{S_{xx}}, \qquad R^2 = \frac{S_{xy}^2}{S_{xx} \cdot S_{yy}}
$$

$$
S_{xy} = \sum_{i=0}^{h-1}(x_{t+i} - \bar{x})(y_{t+i} - \bar{y}), \quad
S_{xx} = \sum_{i=0}^{h-1}(x_{t+i} - \bar{x})^2, \quad
S_{yy} = \sum_{i=0}^{h-1}(y_{t+i} - \bar{y})^2
$$

OLS regression of `y` on `x` over the next `h` bars. Outputs the slope
and R² for each bar `t`. Both `x` and `y` are real input columns. `x` is
typically a time index or cumulative volume, `y` is the price series.

!!! tip "Choosing x"
    Use a simple integer index `[0, 1, 2, ...]` as `x` to get slope in price-per-bar units. If you pass timestamps, the slope becomes price-per-nanosecond and is much harder to read.

=== "Parameters"

    | Name | Type | Constraint | Description |
    |---|---|---|---|
    | `inputs` | `list[str]` | len >= 2 | Two columns in order: `[x_col, y_col]` |
    | `horizon` | `int` | >= 2 | Number of bars in the regression window ($h$) |
    | `outputs` | `list[str]` | len = 2 | Two output names: `[slope_col, r2_col]` |

=== "Output"

    | Column | When valid | Description |
    |---|---|---|
    | `outputs[0]` | `t <= N - horizon - 1`, `x` not constant | OLS slope over the next `horizon` bars |
    | `outputs[1]` | same as above, `y` not constant | R² of the regression |

=== "Properties"

    | Property | Value |
    |---|---|
    | `warm_up_period` | `0` |
    | `forward_period` | `horizon` |

    !!! note "Trailing None count"
        Despite `forward_period()` returning `horizon`, only the last `horizon - 1`
        values are `None`. The window `[t, t+h)` is valid whenever `t + h <= N`,
        so the last valid bar is `N - h`, leaving `h - 1` trailing `None` values.

=== "Behavior"

    **Forward None.** The last `horizon - 1` values are `None` for both outputs.

    **None in inputs.** Any `None` within the `horizon` window produces `None`
    for that bar in both outputs.

    **Constant x (`Sxx = 0`).** If `x` is constant over the window, the slope
    is undefined. Both slope and R² are `None`.

    **Constant y (`Syy = 0`).** If `y` is constant over the window, the slope
    is `0.0` (the regression is exact but flat). R² is `None` (variance is zero,
    so the coefficient of determination is undefined).

    **Stateless.** `run_research()` has no internal state. Calling it twice with
    the same input always returns the same output.

    **Implementation.** Two-pass computation per window: first pass computes means
    and validates inputs, second pass computes `Sxx`, `Sxy`, `Syy`. (`O(N · h)` total).

    | Situation | Slope | R² |
    |---|---|---|
    | `t + horizon > N` | `None` | `None` |
    | Any `None` in window | `None` | `None` |
    | `Sxx = 0` (constant x) | `None` | `None` |
    | `Syy = 0` (constant y) | `0.0` | `None` |
    | Normal case | `Sxy / Sxx` | `Sxy² / (Sxx · Syy)` |

=== "Example"

    ```python
    import numpy as np
    import pandas as pd
    from oryon.targets import FutureLinearSlope
    from oryon import TargetPipeline, run_targets_pipeline

    t = FutureLinearSlope(
        inputs=["time_idx", "close"],
        horizon=3,
        outputs=["close_slope_3", "close_r2_3"],
    )
    tp = TargetPipeline(targets=[t], input_columns=["time_idx", "close"])

    df = pd.DataFrame({
        "time_idx": [0.0, 1.0, 2.0, 3.0, 4.0],
        "close":    [100.0, 101.0, 103.0, 102.0, 105.0],
    })
    out = run_targets_pipeline(tp, df)
    print(out)
    #    close_slope_3  close_r2_3
    # 0           1.50      0.9643
    # 1           0.50      0.2500
    # 2           1.00      0.4286
    # 3            NaN         NaN
    # 4            NaN         NaN
    ```

    Bar 0: regresses `y=[100, 101, 103]` on `x=[0, 1, 2]`. Slope = 1.5, R² = 27/28 ≈ 0.9643.
    Bar 1: `y=[101, 103, 102]` on `x=[1, 2, 3]`. Slope = 0.5, R² = 0.25.
    Bars 3 and 4 are `NaN` (`horizon - 1 = 2` trailing `NaN` values).

=== "Source"

    [:octicons-mark-github-16: `crates/oryon/src/targets/future_linear_slope.rs`](https://github.com/lucasinglese/oryon/blob/main/crates/oryon/src/targets/future_linear_slope.rs)