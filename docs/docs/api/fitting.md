# Fitting

Fitting functions estimate distribution parameters from a batch dataset. Use them on a
training set, then pass the results to pre-fitted scalers like [`FixedZScore`](../scalers/#fixed-z-score).

---

## fit_standard_scaler

Computes mean and sample standard deviation from a column of data, skipping `NaN` values.
Returns `(mean, std)` as a tuple. Raises `ValueError` if fewer than 2 valid values remain
or if all valid values are equal (std would be zero).

=== "Signature"

    ```python
    fit_standard_scaler(data: list[float]) -> tuple[float, float]
    ```

    | Parameter | Type | Description |
    |---|---|---|
    | `data` | `list[float]` | Values to fit on. Use `float('nan')` for missing entries |

    Returns `(mean, std)`.

=== "Behavior"

    **`NaN` handling.** `NaN` entries are skipped before computing statistics. Only
    non-NaN values contribute to mean and std.

    **Sample std.** Standard deviation uses `n - 1` denominator, matching `numpy.std(ddof=1)`
    and `pandas .std()`.

    **Error conditions.** Raises `ValueError` if:

    - Fewer than 2 valid values remain after skipping `NaN`
    - All valid values are equal (std would be zero - `FixedZScore` would divide by zero)

    | Situation | Result |
    |---|---|
    | >= 2 distinct valid values | `(mean, std)` |
    | < 2 valid values | `ValueError` |
    | All valid values equal | `ValueError` (std = 0) |

=== "Example"

    ```python
    import pandas as pd
    from oryon.scalers import fit_standard_scaler, FixedZScore
    from oryon import FeaturePipeline, run_features_pipeline

    df_train = pd.DataFrame({"returns": [0.01, -0.02, 0.03, -0.01, 0.02]})
    df_live  = pd.DataFrame({"returns": [0.015, -0.005, 0.025]})

    # Fit on training data only
    mean, std = fit_standard_scaler(df_train["returns"].tolist())

    # Apply fixed parameters at inference time
    fz = FixedZScore(["returns"], ["returns_z"], mean=mean, std=std)
    fp = FeaturePipeline(features=[fz], input_columns=["returns"])

    out = run_features_pipeline(fp, df_live)
    print(out)
    ```

    !!! warning "Fit on training data only"
        Never call `fit_standard_scaler` on the full dataset before splitting.
        Fitting on test data leaks future distribution information and overstates
        out-of-sample performance.

=== "Source"

    [:octicons-mark-github-16: `crates/oryon/src/fitting/standard_scaler.rs`](https://github.com/lucasinglese/oryon/blob/main/crates/oryon/src/fitting/standard_scaler.rs)
