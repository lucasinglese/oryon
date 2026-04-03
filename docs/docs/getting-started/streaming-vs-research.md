# Streaming vs Research

Two modes, one library. The choice is made at call time, not at construction.

---

## Streaming (live trading)

Call `update()` with one bar. The feature updates its internal state and returns
the current output immediately.

```python
from oryon.features import Sma

sma = Sma(["close"], window=3, outputs=["sma_3"])

sma.update([100.0])  # → [nan]    warm-up
sma.update([101.0])  # → [nan]    warm-up
sma.update([102.0])  # → [101.0]  first valid bar
sma.update([103.0])  # → [102.0]
```

No history is retained beyond what is strictly necessary. A `window=3` SMA keeps
exactly 3 values in memory, regardless of how many bars have been processed.

---

## Research (full dataset)

Call `run_research()` on a `FeaturePipeline` with a list of bars. Internally it
resets state and calls `update()` in a loop. The output is identical to calling
`update()` bar by bar from a clean state.

```python
from oryon import FeaturePipeline
from oryon.features import Sma

sma = Sma(["close"], window=3, outputs=["sma_3"])
fp  = FeaturePipeline(features=[sma], input_columns=["close"])

data   = [[100.0], [101.0], [102.0], [103.0], [104.0]]
result = fp.run_research(data)
# [[nan], [nan], [101.0], [102.0], [103.0]]
```

Run several features or targets at once? Check the [Pipelines](../api/pipelines.md).

---

## Features vs Targets

The streaming / research distinction applies differently depending on the type.

| | Features | Targets |
|---|---|---|
| Direction | Backward only | Forward (uses future bars) |
| Streaming (`update()`) | Yes | No |
| Research (`run_research()`) | Yes | Yes |
| NaN location | Beginning (warm-up) | End (horizon) |
| `warm_up_period` | `>= 1` for windowed indicators | `0` |
| `forward_period` | `0` | `>= 1` |
| Role in ML | Model inputs (X) | Model labels (y) |
| Example | SMA, EMA, Kurtosis | FutureReturn, FutureCTCVolatility |

!!! warning "Never use a target in live trading"
    Targets require future bars to compute. `FutureReturn(horizon=5)` needs the
    price 5 bars ahead, which does not exist in a live environment.

---

## reset()

Every feature has a `reset()` method that clears internal state. Call it:

- Between CPCV or walk-forward folds (to prevent state leaking across splits)
- When switching to a new instrument mid-stream

`FeaturePipeline.run_research()` calls `reset()` automatically before processing.
`FeaturePipeline.update()` does not. It is your responsibility to call `reset()`
between folds when running in streaming mode.

```python
for fold_data in walk_forward_splits:
    fp.reset()
    results = fp.run_research(fold_data)
```
