# Pipelines

Pipelines orchestrate features and targets over a dataset. They handle input
routing, execution order, and state management so you only declare what to
compute, not how.

| | `FeaturePipeline` | `TargetPipeline` |
|---|---|---|
| Mode | Streaming + Research | Research only |
| State | Stateful (reset between folds) | Stateless |
| DAG | Yes (auto-resolved) | No (targets are independent) |
| Input orientation | Row (one bar per call) | Column (one list per column) |

---

## FeaturePipeline

Orchestrates features in DAG-resolved order. Dependencies between features are
inferred automatically: if feature B reads a column produced by feature A, B
is scheduled after A. Features with no shared dependency run at the same level.

!!! tip "DAG chaining"
    Any feature's output can be used as input to another feature. Pass them
    in any order. The pipeline resolves the execution sequence automatically.

=== "Constructor"

    ```python
    FeaturePipeline(features: list, input_columns: list[str])
    ```

    | Parameter | Type | Description |
    |---|---|---|
    | `features` | `list[Feature]` | Feature instances in any order |
    | `input_columns` | `list[str]` | Raw input columns provided at each `update()` call, in order |

    Raises `ValueError` if output keys are duplicated, a cyclic dependency is
    detected, or a required input column is missing from `input_columns`.

=== "Methods"

    | Method | Signature | Description |
    |---|---|---|
    | `update` | `(values: list[float]) -> list[float]` | Process one bar. Returns one output value per feature output |
    | `run_research` | `(data: list[list[float]]) -> list[list[float]]` | Batch mode: resets state, processes all bars, returns row-oriented output |
    | `reset` | `() -> None` | Clear all feature states. Call between CPCV / walk-forward folds |
    | `output_names` | `() -> list[str]` | Output column names in execution order |
    | `input_names` | `() -> list[str]` | Input columns in the order expected by `update()` |
    | `warm_up_period` | `() -> int` | Maximum warm-up period across all features |
    | `__len__` | `() -> int` | Number of features |

=== "Behavior"

    **Data orientation.** `update()` and `run_research()` are both row-oriented:
    each bar is a flat list of input values in `input_columns` order. `run_research()`
    returns a list of rows, each row matching `output_names()`.

    **`run_research()` auto-resets.** Calling `run_research()` resets all feature
    states before processing. You do not need to call `reset()` explicitly before it.

    **`reset()` between folds.** Call `reset()` between CPCV or walk-forward
    splits when using `update()` in a loop. This prevents state from the previous
    fold leaking into the next one.

    **`warm_up_period`.** Returns the maximum `warm_up_period` across all features.
    The first `warm_up_period` rows of any output may be `NaN`.

    **Missing column at construction.** If a feature requires a column not present
    in `input_columns` (and not produced by another feature), construction fails
    immediately with a `ValueError`. Misconfiguration is caught early, not at runtime.

=== "Example"

    The pipeline below chains `Sma` into `LogReturn`. `LogReturn` reads
    `close_sma_3`, which is produced by `Sma`. Pass them in any order; the
    DAG resolves the sequence automatically.

    **Live mode, one bar at a time:**

    ```python
    from oryon import FeaturePipeline
    from oryon.features import Sma, LogReturn

    fp = FeaturePipeline(
        features=[
            Sma(["close"], window=3, outputs=["close_sma_3"]),
            LogReturn(["close_sma_3"], window=1, outputs=["sma_log_ret"]),
        ],
        input_columns=["close"],
    )

    print(fp.update([100.0]))  # [nan, nan]
    print(fp.update([101.0]))  # [nan, nan]
    print(fp.update([102.0]))  # [101.0, nan]
    print(fp.update([103.0]))  # [102.0, 0.0099]
    print(fp.update([104.0]))  # [103.0, 0.0098]

    print(fp.output_names())    # ['close_sma_3', 'sma_log_ret']
    print(fp.warm_up_period())  # 3
    ```

    **Research mode, full dataset at once:**

    ```python
    from oryon import FeaturePipeline
    from oryon.features import Sma, LogReturn

    fp = FeaturePipeline(
        features=[
            Sma(["close"], window=3, outputs=["close_sma_3"]),
            LogReturn(["close_sma_3"], window=1, outputs=["sma_log_ret"]),
        ],
        input_columns=["close"],
    )

    # Each inner list is one bar: [close]
    data = [[100.0], [101.0], [102.0], [103.0], [104.0], [105.0]]
    result = fp.run_research(data)
    # result is row-oriented: one list per bar, columns match output_names()
    # [[nan, nan], [nan, nan], [101.0, nan], [102.0, 0.0099], [103.0, 0.0098], [104.0, 0.0097]]

    print(fp.output_names())  # ['close_sma_3', 'sma_log_ret']
    ```

    **`reset()` between folds:**

    ```python
    fold_1 = [[100.0], [101.0], [102.0], [103.0]]
    fold_2 = [[200.0], [201.0], [202.0], [203.0]]

    result_1 = fp.run_research(fold_1)
    fp.reset()
    result_2 = fp.run_research(fold_2)
    ```

=== "Source"

    [:octicons-mark-github-16: `crates/oryon/src/pipeline/feature_pipeline.rs`](https://github.com/Quantreo/oryon/blob/main/crates/oryon/src/pipeline/feature_pipeline.rs)
    · [:octicons-mark-github-16: `crates/oryon/src/pipeline/dag.rs`](https://github.com/Quantreo/oryon/blob/main/crates/oryon/src/pipeline/dag.rs)

---

## TargetPipeline

Orchestrates targets over a complete dataset. Targets are independent and
stateless: no DAG is needed and there is no `update()` method.

!!! warning "Column-oriented input"
    `run_research()` expects **one list per column**, not one list per bar.
    This is the opposite of `FeaturePipeline.run_research()`. The pandas helper
    `run_targets_pipeline()` handles this automatically.

=== "Constructor"

    ```python
    TargetPipeline(targets: list, input_columns: list[str])
    ```

    | Parameter | Type | Description |
    |---|---|---|
    | `targets` | `list[Target]` | Target instances |
    | `input_columns` | `list[str]` | Raw input columns provided to `run_research()`, in order |

    Raises `ValueError` if output keys are duplicated or a required input column
    is missing from `input_columns`.

=== "Methods"

    | Method | Signature | Description |
    |---|---|---|
    | `run_research` | `(data: list[list[float]]) -> list[list[float]]` | Run all targets. Input and output are column-oriented |
    | `output_names` | `() -> list[str]` | Output column names in order |
    | `input_names` | `() -> list[str]` | Input columns in the order expected by `run_research()` |
    | `forward_period` | `() -> int` | Maximum forward period across all targets |
    | `__len__` | `() -> int` | Number of targets |

=== "Behavior"

    **Column-oriented data.** `run_research(data)` expects `data` as a list of
    columns: `data[0]` is the full series for `input_columns[0]`, `data[1]` for
    `input_columns[1]`, and so on. Output follows the same layout: one list per
    output column in `output_names()` order.

    **Stateless.** Targets have no internal state. Calling `run_research()` twice
    with the same data always returns the same result. There is no `reset()`.

    **`forward_period`.** Returns the maximum `forward_period` across all targets.
    The last `forward_period` rows of any output are `NaN` because the future
    window extends beyond the series.

=== "Example"

    ```python
    from oryon import TargetPipeline
    from oryon.targets import FutureReturn, FutureCTCVolatility

    tp = TargetPipeline(
        targets=[
            FutureReturn(inputs=["close"], horizon=2, outputs=["close_fr_2"]),
            FutureCTCVolatility(input="close", horizon=3),
        ],
        input_columns=["close"],
    )

    # Input is column-oriented: one list per column
    closes = [100.0, 101.0, 103.0, 102.0, 105.0, 107.0, 106.0]
    result = tp.run_research([closes])
    # result is column-oriented: one list per output column
    # result[0] → close_fr_2:              [0.0300, 0.0099, 0.0194, 0.0490, 0.0095, nan, nan]
    # result[1] → close_future_ctc_vol_3:  [0.0150, 0.0202, 0.0201, 0.0199, nan,    nan, nan]

    print(tp.forward_period())  # 3
    print(tp.output_names())    # ['close_fr_2', 'close_future_ctc_vol_3']
    ```

    `FutureReturn(h=2)` leaves the last 2 rows as `NaN`. `FutureCTCVolatility(h=3)`
    leaves the last 3 rows as `NaN`. `forward_period()` returns the maximum: `3`.

=== "Source"

    [:octicons-mark-github-16: `crates/oryon/src/pipeline/target_pipeline.rs`](https://github.com/Quantreo/oryon/blob/main/crates/oryon/src/pipeline/target_pipeline.rs)

---

## Pandas helpers

Two functions wrap the pipelines for pandas DataFrames. They handle data
extraction, orientation, and index alignment automatically.

```python
from oryon import run_features_pipeline, run_targets_pipeline
```

| Function | Pipeline | Input orientation | Returns |
|---|---|---|---|
| `run_features_pipeline(pipeline, df)` | `FeaturePipeline` | Rows extracted from `df` | DataFrame aligned to `df.index` |
| `run_targets_pipeline(pipeline, df)` | `TargetPipeline` | Columns extracted from `df` | DataFrame aligned to `df.index` |

Both functions read the columns listed in `pipeline.input_names()` from `df`
and return a new DataFrame with `pipeline.output_names()` as columns, preserving
the original index (integer, datetime, or otherwise).

=== "Example"

    ```python
    import pandas as pd
    from oryon import (
        FeaturePipeline, TargetPipeline,
        run_features_pipeline, run_targets_pipeline,
    )
    from oryon.features import Sma, LogReturn
    from oryon.targets import FutureReturn

    df = pd.DataFrame(
        {"close": [100.0, 101.0, 102.0, 103.0, 104.0, 105.0, 106.0, 107.0]},
        index=pd.date_range("2024-01-01", periods=8),
    )

    fp = FeaturePipeline(
        features=[
            Sma(["close"], window=3, outputs=["close_sma_3"]),
            LogReturn(["close"], window=1, outputs=["log_ret"]),
        ],
        input_columns=["close"],
    )
    tp = TargetPipeline(
        targets=[FutureReturn(inputs=["close"], horizon=2, outputs=["close_fr_2"])],
        input_columns=["close"],
    )

    features_df = run_features_pipeline(fp, df)
    targets_df  = run_targets_pipeline(tp, df)
    result = pd.concat([df, features_df, targets_df], axis=1)
    print(result)
    #              close  close_sma_3  log_ret  close_fr_2
    # 2024-01-01   100.0          NaN      NaN      0.0200
    # 2024-01-02   101.0          NaN     0.0100     0.0198
    # 2024-01-03   102.0       101.0     0.0099     0.0196
    # 2024-01-04   103.0       102.0     0.0098     0.0194
    # 2024-01-05   104.0       103.0     0.0097     0.0192
    # 2024-01-06   105.0       104.0     0.0096     0.0190
    # 2024-01-07   106.0       105.0     0.0095        NaN
    # 2024-01-08   107.0       106.0     0.0094        NaN
    ```

    The datetime index is preserved across both helpers. Features and targets
    can be concatenated directly onto the source DataFrame.

=== "Source"

    [:octicons-mark-github-16: `python/oryon/__init__.py`](https://github.com/Quantreo/oryon/blob/main/python/oryon/__init__.py)
