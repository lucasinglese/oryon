# Documentation Templates

When adding a new feature or target, append its entry to the correct API Reference page.

The source code is the single source of truth. Before writing any statement about behavior, parameters, formulas, or output, read the Rust implementation in `crates/oryon/src/`.

You can paste the Rust source and the template below into an LLM to generate the entry.

---

=== "Feature"

    Append to the matching category page in `docs/docs/api/features/<category>.md`.

    ````markdown
    ## Full Name

    <a href="../../../getting-started/streaming-vs-research/" class="oryon-badge oryon-badge--streaming">Streaming</a> <a href="../../../getting-started/streaming-vs-research/#research-full-dataset" class="oryon-badge oryon-badge--research">Research</a> <a href="../../../benchmarks/" class="oryon-badge oryon-badge--perf">&lt;Xµs/1k bars</a>

    $$
    \text{formula}
    $$

    One or two sentences describing what the feature computes and when outputs are `None`.

    === "Parameters"

        | Name | Type | Constraint | Description |
        |---|---|---|---|
        | `inputs` | `list[str]` | non-empty | Input column names, e.g. `["close"]` |
        | `window` | `int` | >= 1 | Rolling window size |
        | `outputs` | `list[str]` | non-empty | Output column names, e.g. `["close_name_20"]` |

    === "Output"

        | Column | When valid | Description |
        |---|---|---|
        | `outputs[0]` | `t >= warm_up_period` | What the value represents |

    === "Properties"

        | Property | Value |
        |---|---|
        | `warm_up_period` | `window - 1` |
        | `forward_period` | `0` |

    === "Behavior"

        **Warm-up.** The first `window - 1` outputs are `NaN`.

        **`NaN` input.** Describe how `None` inputs propagate through the window.

        **`reset()`.** Describe what is cleared.

    === "Example"

        ```python
        from oryon.features import YourFeature

        f = YourFeature(inputs=["close"], window=3, outputs=["close_your_feature_3"])

        f.update([100.0])  # -> [NaN]
        f.update([101.0])  # -> [NaN]
        f.update([102.0])  # -> [expected value]
        ```

    === "Source"

        [:octicons-mark-github-16: `crates/oryon/src/features/your_feature.rs`](https://github.com/lucasinglese/oryon/blob/main/crates/oryon/src/features/your_feature.rs)
    ````

=== "Target"

    Append to the matching category page in `docs/docs/api/targets/<category>.md`.

    ````markdown
    ## Full Name

    <a href="../../../getting-started/streaming-vs-research/#research-full-dataset" class="oryon-badge oryon-badge--research">Research only</a> <a href="../../../benchmarks/" class="oryon-badge oryon-badge--perf">&lt;Xµs/1k bars</a>

    $$
    \text{formula}
    $$

    One or two sentences describing what the target computes and what kind of prediction task it supports.

    === "Parameters"

        | Name | Type | Constraint | Description |
        |---|---|---|---|
        | `inputs` | `list[str]` | non-empty | Input column names, e.g. `["close"]` |
        | `horizon` | `int` | >= 1 | Number of bars to look ahead |
        | `outputs` | `list[str]` | non-empty | Output column names |

    === "Output"

        | Column | When valid | Description |
        |---|---|---|
        | `outputs[0]` | `t <= N - horizon - 1` | What the value represents |

    === "Properties"

        | Property | Value |
        |---|---|
        | `warm_up_period` | `0` |
        | `forward_period` | `horizon` |

    === "Behavior"

        **Forward NaN.** The last `horizon` values are `NaN` because the future window is incomplete.

        **`NaN` in inputs.** Describe how `None` values in the input series affect output.

        **Stateless.** `run_research()` has no internal state. Calling it twice with the same input always returns the same output.

    === "Example"

        ```python
        import pandas as pd
        from oryon.targets import YourTarget
        from oryon import TargetPipeline, run_targets_pipeline

        t = YourTarget(inputs=["close"], horizon=3, outputs=["close_your_target_3"])

        tp = TargetPipeline(targets=[t], input_columns=["close"])
        df = pd.DataFrame({"close": [100.0, 101.0, 103.0, 102.0, 105.0, 107.0]})
        out = run_targets_pipeline(tp, df)
        print(out)
        ```

    === "Source"

        [:octicons-mark-github-16: `crates/oryon/src/targets/your_target.rs`](https://github.com/lucasinglese/oryon/blob/main/crates/oryon/src/targets/your_target.rs)
    ````

=== "Scaler"

    Append to `docs/docs/api/scalers.md`.

    ````markdown
    ## Full Name

    <a href="../../getting-started/streaming-vs-research/#streaming-live-trading" class="oryon-badge oryon-badge--streaming">Streaming</a> <a href="../../benchmarks/" class="oryon-badge oryon-badge--perf">&lt;1µs/update</a> <a href="../../getting-started/streaming-vs-research/#research-full-dataset" class="oryon-badge oryon-badge--research">Research</a>

    $$
    \text{formula}
    $$

    One or two sentences describing what the scaler computes and when outputs are `None`.

    === "Parameters"

        | Name | Type | Constraint | Description |
        |---|---|---|---|
        | `inputs` | `list[str]` | len = 1 | Input column, e.g. `["close"]` |
        | `window` | `int` | >= 2 | Rolling window size (omit if stateless) |
        | `outputs` | `list[str]` | len >= 1 | Output column, e.g. `["close_scaled"]` |

    === "Output"

        | Column | When valid | Description |
        |---|---|---|
        | `outputs[0]` | `t >= warm_up_period`, no `NaN` in buffer | What the value represents |

    === "Behavior"

        - **Warm-up.** The first `warm_up_period` bars return `NaN`.

        - **`NaN` propagation.** Describe how `None` inputs propagate.

        - **`reset()`.** Describe what is cleared (or "No-op" if stateless).

    === "Example"

        ```python
        from oryon.scalers import YourScaler

        s = YourScaler(["close"], window=3, outputs=["close_scaled"])
        s.update([100.0])  # -> [NaN]
        s.update([101.0])  # -> [NaN]
        s.update([102.0])  # -> [expected value]
        ```

    === "Source"

        [:octicons-mark-github-16: `crates/oryon/src/scalers/your_scaler.rs`](https://github.com/lucasinglese/oryon/blob/main/crates/oryon/src/scalers/your_scaler.rs)
    ````

=== "Operator"

    Append to `docs/docs/api/operators.md`.

    ````markdown
    ## Full Name

    <a href="../../getting-started/streaming-vs-research/#streaming-live-trading" class="oryon-badge oryon-badge--streaming">Streaming</a> <a href="../../benchmarks/" class="oryon-badge oryon-badge--perf">&lt;1µs/update</a> <a href="../../getting-started/streaming-vs-research/#research-full-dataset" class="oryon-badge oryon-badge--research">Research</a>

    $$
    \text{formula}
    $$

    One sentence describing what the operator computes and when output is `None`.

    === "Parameters"

        | Name | Type | Constraint | Description |
        |---|---|---|---|
        | `inputs` | `list[str]` | len = N | Input columns |
        | `outputs` | `list[str]` | len >= 1 | Output column |

    === "Output"

        | Column | When valid | Description |
        |---|---|---|
        | `outputs[0]` | All inputs not `NaN` | What the value represents |

    === "Behavior"

        - **No warm-up.** Output is valid from the first bar.

        - **`NaN` propagation.** Describe which `None` inputs trigger `None` output.

        - **`reset()`.** No-op. There is no state to clear.

    === "Example"

        ```python
        from oryon.operators import YourOperator

        op = YourOperator(["a", "b"], outputs=["result"])
        op.update([10.0, 3.0])  # -> [expected value]
        ```

    === "Source"

        [:octicons-mark-github-16: `crates/oryon/src/operators/your_operator.rs`](https://github.com/lucasinglese/oryon/blob/main/crates/oryon/src/operators/your_operator.rs)
    ````

---

## Checklist before submitting

- [ ] Every statement about behavior is verified against the Rust source
- [ ] Formula matches the implementation, not a textbook definition
- [ ] `warm_up_period` and `forward_period` values are exact
- [ ] All `None` conditions are listed in the Behavior tab
- [ ] The example runs without error and the output comments are correct
- [ ] No em dash in prose - use a hyphen or rewrite the sentence
