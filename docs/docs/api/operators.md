# Operators

Operators are stateless StreamingTransforms. They perform arithmetic on one or two
input columns with no internal buffer and no warm-up. Cost is `O(1)` per bar at
any input size.

---

## Subtract

<a href="../../getting-started/streaming-vs-research/#streaming-live-trading" class="oryon-badge oryon-badge--streaming">Streaming</a> <a href="../../benchmarks/" class="oryon-badge oryon-badge--perf">&lt;1µs/update</a> <a href="../../getting-started/streaming-vs-research/#research-full-dataset" class="oryon-badge oryon-badge--research">Research</a>

$$
\text{out}_t = A_t - B_t
$$

Computes `A - B` from two input columns. Returns `NaN` if either input is `NaN`.

=== "Parameters"

    | Name | Type | Constraint | Description |
    |---|---|---|---|
    | `inputs` | `list[str]` | len = 2 | The two input columns `[A, B]`, e.g. `["ema_fast", "ema_slow"]` |
    | `outputs` | `list[str]` | len >= 1 | Output column, e.g. `["ema_spread"]` |

=== "Output"

    | Column | When valid | Description |
    |---|---|---|
    | `outputs[0]` | Both inputs are not `NaN` | `A - B` |

=== "Behavior"

    - **No warm-up.** Output is valid from the first bar.

    - **`NaN` propagation.** If either input is `NaN`, output is `NaN`.

    - **`reset()`.** No-op. There is no state to clear.

    | Situation | Output |
    |---|---|
    | Both inputs valid | `A - B` |
    | Either input is `NaN` | `NaN` |

=== "Example"

    ```python
    import pandas as pd
    from oryon.operators import Subtract
    from oryon import FeaturePipeline
from oryon.adapters import run_features_pipeline_pandas

    sub = Subtract(["a", "b"], outputs=["spread"])
    fp = FeaturePipeline(features=[sub], input_columns=["a", "b"])

    df = pd.DataFrame({"a": [10.0, 5.0, 1.0], "b": [3.0, 5.0, 4.0]})
    out = run_features_pipeline_pandas(fp, df)
    print(out)
    #    spread
    # 0     7.0
    # 1     0.0
    # 2    -3.0
    ```

    **Chaining with features.** A common use is spreading two moving averages computed
    upstream in the same pipeline:

    ```python
    from oryon.features import Ema
    from oryon.operators import Subtract
    from oryon import FeaturePipeline

    fp = FeaturePipeline(
        features=[
            Ema(["close"], window=5,  outputs=["ema_5"]),
            Ema(["close"], window=20, outputs=["ema_20"]),
            Subtract(["ema_5", "ema_20"], outputs=["ema_spread"]),
        ],
        input_columns=["close"],
    )
    # The pipeline resolves the DAG automatically: both Ema run before Subtract.
    ```

=== "Source"

    [:octicons-mark-github-16: `crates/oryon/src/operators/subtract.rs`](https://github.com/lucasinglese/oryon/blob/main/crates/oryon/src/operators/subtract.rs)

---

## NegLog

<a href="../../getting-started/streaming-vs-research/#streaming-live-trading" class="oryon-badge oryon-badge--streaming">Streaming</a> <a href="../../benchmarks/" class="oryon-badge oryon-badge--perf">&lt;1µs/update</a> <a href="../../getting-started/streaming-vs-research/#research-full-dataset" class="oryon-badge oryon-badge--research">Research</a>

$$
\text{out}_t = -\ln(x_t)
$$

Computes the negative natural logarithm of the input. Returns `NaN` if the input
is `NaN` or `<= 0`.

A common use is transforming p-values: small p-values become large positive scores,
making them linearly separable for ML models.

=== "Parameters"

    | Name | Type | Constraint | Description |
    |---|---|---|---|
    | `inputs` | `list[str]` | len = 1 | Input column, e.g. `["pvalue"]` |
    | `outputs` | `list[str]` | len >= 1 | Output column, e.g. `["neg_log_pvalue"]` |

=== "Output"

    | Column | When valid | Description |
    |---|---|---|
    | `outputs[0]` | `x > 0` | `-ln(x)` |

=== "Behavior"

    - **No warm-up.** Output is valid from the first bar.

    - **Domain.** Input must be strictly positive (`x > 0`). Values of `0` or below return `NaN`.

    - **`NaN` input.** Returns `NaN`.

    - **`reset()`.** No-op. There is no state to clear.

    | Situation | Output |
    |---|---|
    | `x > 0` | `-ln(x)` |
    | `x <= 0` | `NaN` |
    | `NaN` input | `NaN` |

=== "Interpretation"

    - **Domain mapping.** Maps `(0, 1]` to `[0, +∞)` and `(1, +∞)` to `(-∞, 0)`.
    Monotonically decreasing: the ordering is reversed but preserved. Smaller inputs
    produce larger outputs.

    - **P-value use case.** A p-value of `0.001` becomes `6.9`, a p-value of `0.05`
    becomes `3.0`. This expands the scale near zero where differences matter most,
    making small p-values linearly separable in ML models.

=== "Example"

    ```python
    import pandas as pd
    from oryon.operators import NegLog
    from oryon import FeaturePipeline
from oryon.adapters import run_features_pipeline_pandas

    nl = NegLog(["pvalue"], outputs=["neg_log_pvalue"])
    fp = FeaturePipeline(features=[nl], input_columns=["pvalue"])

    df = pd.DataFrame({"pvalue": [1.0, 0.5, 0.05, 0.0, None]})
    out = run_features_pipeline_pandas(fp, df)
    print(out)
    #    neg_log_pvalue
    # 0           0.000   # -ln(1.0) = 0
    # 1           0.693   # -ln(0.5) = ln(2)
    # 2           2.996   # -ln(0.05) = ln(20)
    # 3             NaN   # 0.0 is not > 0
    # 4             NaN   # NaN input
    ```

=== "Source"

    [:octicons-mark-github-16: `crates/oryon/src/operators/neg_log.rs`](https://github.com/lucasinglese/oryon/blob/main/crates/oryon/src/operators/neg_log.rs)