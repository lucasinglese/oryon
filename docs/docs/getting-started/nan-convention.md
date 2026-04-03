# NaN Convention

Oryon uses `float('nan')` in Python to represent missing values. In Rust, missing
values are `Option<f64>` (`None`). The conversion is transparent at the PyO3
boundary, and you never deal with it explicitly.

There are exactly two sources of NaN in any output column.

---

## Warm-up (features)

A windowed feature cannot produce a valid output until its buffer is full.
The first `warm_up_period` bars are always `NaN`.

```
SMA(window=3)

bar 0  →  NaN    buffer not full
bar 1  →  NaN    buffer not full
bar 2  →  101.0  first valid output
bar 3  →  102.0
bar 4  →  103.0
```

`warm_up_period = window - 1` for most features. The API Reference page for each
feature lists the exact value.

---

## Horizon (targets)

A target cannot compute its label when the forward window extends beyond the end
of the dataset. The last `forward_period` bars are always `NaN`.

```
FutureReturn(horizon=2)  —  5 bars

bar 0  →  0.0500
bar 1  →  0.0098
bar 2  →  0.0286
bar 3  →  NaN    needs bar 5, which doesn't exist
bar 4  →  NaN    needs bar 6, which doesn't exist
```

---

## Model-ready zone

Combining features and targets produces NaN at both ends. The rows in between
are your training data.

```
       close_sma_3   close_fr_2
bar 0       NaN          0.05    ← feature warm-up
bar 1       NaN          0.0098  ← feature warm-up
bar 2      101.0         0.0286
bar 3      102.0         NaN     ← target horizon
bar 4      103.0         NaN     ← target horizon
```

Isolate the model-ready zone with a single `dropna()`:

```python
X = run_features_pipeline(fp, df)
y = run_targets_pipeline(tp, df)
dataset = pd.concat([X, y], axis=1).dropna()
```

---

## NaN propagation in features

A `NaN` input contaminates a feature's buffer. The output stays `NaN` until
that bar is evicted, i.e. after `window` consecutive valid bars have been seen.
Each feature's Behavior tab documents the exact propagation rules, including
edge cases specific to that feature (e.g. EMA resets entirely on `NaN`).
