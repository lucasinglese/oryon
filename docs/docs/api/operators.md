# Operators

Operators are stateless StreamingTransforms. They perform arithmetic on one or two
input columns with no internal buffer and no warm-up. Cost is `O(1)` per bar at
any input size.

---

## Add

<a href="../../getting-started/streaming-vs-research/#streaming-live-trading" class="oryon-badge oryon-badge--streaming">Streaming</a> <a href="../../benchmarks/" class="oryon-badge oryon-badge--perf">&lt;1µs/update</a> <a href="../../getting-started/streaming-vs-research/#research-full-dataset" class="oryon-badge oryon-badge--research">Research</a>

$$
\text{out}_t = A_t + B_t
$$

Computes `A + B` from two input columns. Returns `NaN` if either input is `NaN`.

=== "Parameters"

    | Name | Type | Constraint | Description |
    |---|---|---|---|
    | `inputs` | `list[str]` | len = 2 | The two input columns `[A, B]` |
    | `outputs` | `list[str]` | len >= 1 | Output column |

=== "Output"

    | Column | When valid | Description |
    |---|---|---|
    | `outputs[0]` | Both inputs are not `NaN` | `A + B` |

=== "Behavior"

    - **No warm-up.** Output is valid from the first bar.

    - **`NaN` propagation.** If either input is `NaN`, output is `NaN`.

    - **`reset()`.** No-op. There is no state to clear.

    | Situation | Output |
    |---|---|
    | Both inputs valid | `A + B` |
    | Either input is `NaN` | `NaN` |

=== "Example"

    ```python
    import pandas as pd
    from oryon.operators import Add
    from oryon import FeaturePipeline
    from oryon.adapters import run_features_pipeline_pandas

    op = Add(["a", "b"], outputs=["sum"])
    fp = FeaturePipeline(features=[op], input_columns=["a", "b"])

    df = pd.DataFrame({"a": [1.0, 2.0, 3.0], "b": [4.0, 5.0, 6.0]})
    out = run_features_pipeline_pandas(fp, df)
    print(out)
    #    sum
    # 0  5.0
    # 1  7.0
    # 2  9.0
    ```

=== "Source"

    [:octicons-mark-github-16: `crates/oryon/src/operators/add.rs`](https://github.com/lucasinglese/oryon/blob/main/crates/oryon/src/operators/add.rs)

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

## Multiply

<a href="../../getting-started/streaming-vs-research/#streaming-live-trading" class="oryon-badge oryon-badge--streaming">Streaming</a> <a href="../../benchmarks/" class="oryon-badge oryon-badge--perf">&lt;1µs/update</a> <a href="../../getting-started/streaming-vs-research/#research-full-dataset" class="oryon-badge oryon-badge--research">Research</a>

$$
\text{out}_t = A_t \times B_t
$$

Computes `A * B` from two input columns. Returns `NaN` if either input is `NaN`.

=== "Parameters"

    | Name | Type | Constraint | Description |
    |---|---|---|---|
    | `inputs` | `list[str]` | len = 2 | The two input columns `[A, B]` |
    | `outputs` | `list[str]` | len >= 1 | Output column |

=== "Output"

    | Column | When valid | Description |
    |---|---|---|
    | `outputs[0]` | Both inputs are not `NaN` | `A * B` |

=== "Behavior"

    - **No warm-up.** Output is valid from the first bar.

    - **`NaN` propagation.** If either input is `NaN`, output is `NaN`.

    - **`reset()`.** No-op. There is no state to clear.

    | Situation | Output |
    |---|---|
    | Both inputs valid | `A * B` |
    | Either input is `NaN` | `NaN` |

=== "Example"

    ```python
    import pandas as pd
    from oryon.operators import Multiply
    from oryon import FeaturePipeline
    from oryon.adapters import run_features_pipeline_pandas

    op = Multiply(["signal", "weight"], outputs=["weighted_signal"])
    fp = FeaturePipeline(features=[op], input_columns=["signal", "weight"])

    df = pd.DataFrame({"signal": [1.0, -0.5, 2.0], "weight": [0.8, 0.8, 0.8]})
    out = run_features_pipeline_pandas(fp, df)
    print(out)
    #    weighted_signal
    # 0              0.8
    # 1             -0.4
    # 2              1.6
    ```

=== "Source"

    [:octicons-mark-github-16: `crates/oryon/src/operators/multiply.rs`](https://github.com/lucasinglese/oryon/blob/main/crates/oryon/src/operators/multiply.rs)

---

## Divide

<a href="../../getting-started/streaming-vs-research/#streaming-live-trading" class="oryon-badge oryon-badge--streaming">Streaming</a> <a href="../../benchmarks/" class="oryon-badge oryon-badge--perf">&lt;1µs/update</a> <a href="../../getting-started/streaming-vs-research/#research-full-dataset" class="oryon-badge oryon-badge--research">Research</a>

$$
\text{out}_t = \frac{A_t}{B_t}
$$

Computes `A / B` from two input columns. Returns `NaN` if either input is `NaN` or
if `B` is exactly `0`.

=== "Parameters"

    | Name | Type | Constraint | Description |
    |---|---|---|---|
    | `inputs` | `list[str]` | len = 2 | The two input columns `[A, B]` |
    | `outputs` | `list[str]` | len >= 1 | Output column |

=== "Output"

    | Column | When valid | Description |
    |---|---|---|
    | `outputs[0]` | Both inputs are not `NaN` and `B != 0` | `A / B` |

=== "Behavior"

    - **No warm-up.** Output is valid from the first bar.

    - **`NaN` propagation.** If either input is `NaN`, output is `NaN`.

    - **Zero denominator.** If `B` is exactly `0`, output is `NaN`. Near-zero but
      non-zero denominators are computed normally - the result will be a large finite
      number, which is mathematically correct.

    - **`reset()`.** No-op. There is no state to clear.

    | Situation | Output |
    |---|---|
    | Both inputs valid and `B != 0` | `A / B` |
    | `B == 0` | `NaN` |
    | Either input is `NaN` | `NaN` |

=== "Example"

    ```python
    import pandas as pd
    from oryon.operators import Divide
    from oryon import FeaturePipeline
    from oryon.adapters import run_features_pipeline_pandas

    op = Divide(["close", "open"], outputs=["co_ratio"])
    fp = FeaturePipeline(features=[op], input_columns=["close", "open"])

    df = pd.DataFrame({"close": [101.0, 99.0, 100.0], "open": [100.0, 100.0, 0.0]})
    out = run_features_pipeline_pandas(fp, df)
    print(out)
    #    co_ratio
    # 0      1.01
    # 1      0.99
    # 2       NaN   # open == 0
    ```

=== "Source"

    [:octicons-mark-github-16: `crates/oryon/src/operators/divide.rs`](https://github.com/lucasinglese/oryon/blob/main/crates/oryon/src/operators/divide.rs)

---

## Reciprocal

<a href="../../getting-started/streaming-vs-research/#streaming-live-trading" class="oryon-badge oryon-badge--streaming">Streaming</a> <a href="../../benchmarks/" class="oryon-badge oryon-badge--perf">&lt;1µs/update</a> <a href="../../getting-started/streaming-vs-research/#research-full-dataset" class="oryon-badge oryon-badge--research">Research</a>

$$
\text{out}_t = \frac{1}{x_t}
$$

Computes the multiplicative inverse of the input. Returns `NaN` if the input is
`NaN` or exactly `0`.

=== "Parameters"

    | Name | Type | Constraint | Description |
    |---|---|---|---|
    | `inputs` | `list[str]` | len = 1 | Input column |
    | `outputs` | `list[str]` | len >= 1 | Output column |

=== "Output"

    | Column | When valid | Description |
    |---|---|---|
    | `outputs[0]` | `x != 0` | `1 / x` |

=== "Behavior"

    - **No warm-up.** Output is valid from the first bar.

    - **Zero input.** Returns `NaN`.

    - **`NaN` input.** Returns `NaN`.

    - **`reset()`.** No-op. There is no state to clear.

    | Situation | Output |
    |---|---|
    | `x != 0` | `1 / x` |
    | `x == 0` | `NaN` |
    | `NaN` input | `NaN` |

=== "Example"

    ```python
    import pandas as pd
    from oryon.operators import Reciprocal
    from oryon import FeaturePipeline
    from oryon.adapters import run_features_pipeline_pandas

    op = Reciprocal(["period"], outputs=["freq"])
    fp = FeaturePipeline(features=[op], input_columns=["period"])

    df = pd.DataFrame({"period": [2.0, 4.0, 0.0]})
    out = run_features_pipeline_pandas(fp, df)
    print(out)
    #    freq
    # 0   0.5
    # 1  0.25
    # 2   NaN   # x == 0
    ```

=== "Source"

    [:octicons-mark-github-16: `crates/oryon/src/operators/reciprocal.rs`](https://github.com/lucasinglese/oryon/blob/main/crates/oryon/src/operators/reciprocal.rs)

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

---

## Log

<a href="../../getting-started/streaming-vs-research/#streaming-live-trading" class="oryon-badge oryon-badge--streaming">Streaming</a> <a href="../../benchmarks/" class="oryon-badge oryon-badge--perf">&lt;1µs/update</a> <a href="../../getting-started/streaming-vs-research/#research-full-dataset" class="oryon-badge oryon-badge--research">Research</a>

$$
\text{out}_t = \ln(x_t)
$$

Computes the natural logarithm of the input. Returns `NaN` if the input is `NaN`
or `<= 0`.

=== "Parameters"

    | Name | Type | Constraint | Description |
    |---|---|---|---|
    | `inputs` | `list[str]` | len = 1 | Input column |
    | `outputs` | `list[str]` | len >= 1 | Output column |

=== "Output"

    | Column | When valid | Description |
    |---|---|---|
    | `outputs[0]` | `x > 0` | `ln(x)` |

=== "Behavior"

    - **No warm-up.** Output is valid from the first bar.

    - **Domain.** Input must be strictly positive (`x > 0`). Values of `0` or below return `NaN`.

    - **`NaN` input.** Returns `NaN`.

    - **`reset()`.** No-op. There is no state to clear.

    | Situation | Output |
    |---|---|
    | `x > 0` | `ln(x)` |
    | `x <= 0` | `NaN` |
    | `NaN` input | `NaN` |

=== "Example"

    ```python
    import pandas as pd
    from oryon.operators import Log
    from oryon import FeaturePipeline
    from oryon.adapters import run_features_pipeline_pandas

    op = Log(["price"], outputs=["log_price"])
    fp = FeaturePipeline(features=[op], input_columns=["price"])

    df = pd.DataFrame({"price": [1.0, 2.718, 100.0, 0.0]})
    out = run_features_pipeline_pandas(fp, df)
    print(out)
    #    log_price
    # 0      0.000
    # 1      1.000
    # 2      4.605
    # 3        NaN   # x <= 0
    ```

=== "Source"

    [:octicons-mark-github-16: `crates/oryon/src/operators/log.rs`](https://github.com/lucasinglese/oryon/blob/main/crates/oryon/src/operators/log.rs)

---

## Logit

<a href="../../getting-started/streaming-vs-research/#streaming-live-trading" class="oryon-badge oryon-badge--streaming">Streaming</a> <a href="../../benchmarks/" class="oryon-badge oryon-badge--perf">&lt;1µs/update</a> <a href="../../getting-started/streaming-vs-research/#research-full-dataset" class="oryon-badge oryon-badge--research">Research</a>

$$
\text{out}_t = \ln\!\left(\frac{x_t}{1 - x_t}\right)
$$

Computes the logit (log-odds) of the input. Returns `NaN` if the input is `NaN` or
outside the open interval `(0, 1)`.

=== "Parameters"

    | Name | Type | Constraint | Description |
    |---|---|---|---|
    | `inputs` | `list[str]` | len = 1 | Input column. Values must be in `(0, 1)` to produce valid output |
    | `outputs` | `list[str]` | len >= 1 | Output column |

=== "Output"

    | Column | When valid | Description |
    |---|---|---|
    | `outputs[0]` | `0 < x < 1` | `ln(x / (1 - x))` |

=== "Behavior"

    - **No warm-up.** Output is valid from the first bar.

    - **Domain.** Input must be in the open interval `(0, 1)`. The boundaries `0` and
      `1` return `NaN`.

    - **`NaN` input.** Returns `NaN`.

    - **`reset()`.** No-op. There is no state to clear.

    | Situation | Output |
    |---|---|
    | `0 < x < 1` | `ln(x / (1 - x))` |
    | `x <= 0` or `x >= 1` | `NaN` |
    | `NaN` input | `NaN` |

=== "Interpretation"

    - **Symmetry.** `logit(0.5) = 0`. Values above `0.5` map to positive outputs,
      values below `0.5` map to negative outputs.

    - **Unbounded output.** Maps the bounded interval `(0, 1)` to `(-∞, +∞)`, making
      probability features suitable as inputs to linear models.

    - **Common use.** Apply after a feature that produces a probability or a ratio
      bounded in `(0, 1)` (e.g. a sigmoid-transformed score) to undo the compression
      near the boundaries.

=== "Example"

    ```python
    import pandas as pd
    from oryon.operators import Logit
    from oryon import FeaturePipeline
    from oryon.adapters import run_features_pipeline_pandas

    op = Logit(["prob"], outputs=["log_odds"])
    fp = FeaturePipeline(features=[op], input_columns=["prob"])

    df = pd.DataFrame({"prob": [0.1, 0.5, 0.9, 0.0, 1.0]})
    out = run_features_pipeline_pandas(fp, df)
    print(out)
    #    log_odds
    # 0    -2.197   # ln(0.1 / 0.9)
    # 1     0.000   # ln(0.5 / 0.5) = 0
    # 2     2.197   # ln(0.9 / 0.1)
    # 3       NaN   # x <= 0
    # 4       NaN   # x >= 1
    ```

=== "Source"

    [:octicons-mark-github-16: `crates/oryon/src/operators/logit.rs`](https://github.com/lucasinglese/oryon/blob/main/crates/oryon/src/operators/logit.rs)
