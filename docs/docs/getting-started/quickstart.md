# Quick Start

## The problem

Most quant libraries compute indicators as a batch over a full DataFrame. That
means your research code and your live trading code are fundamentally different
objects. Any inconsistency between them is a live trading bug waiting to happen.

## The Oryon model

Every feature is a stateful object. It keeps just enough history to compute its
output. You feed it one bar at a time, or the whole dataset. Same object, same
logic, same output.

```python
from oryon import FeaturePipeline
from oryon.features import Sma

sma = Sma(["close"], window=20, outputs=["close_sma_20"])
fp  = FeaturePipeline(features=[sma], input_columns=["close"])

# Live: one bar at a time
result = fp.update([109.5])   # [nan] during warm-up, [value] once ready

# Research: same object, same code
results = fp.run_research([[100.0], [101.5], [103.0], ...])
```

There is no separate research implementation. The same Rust code runs in both
contexts.

---

## Build a dataset in 10 lines

```python
import pandas as pd
from oryon import FeaturePipeline, TargetPipeline
from oryon import run_features_pipeline, run_targets_pipeline
from oryon.features import Ema, LogReturn, ParkinsonVolatility
from oryon.targets import FutureReturn

df = pd.read_parquet("btc_1h.parquet")  # your OHLCV data

fp = FeaturePipeline(
    features=[
        Ema(["close"], window=20, outputs=["ema_20"]),
        LogReturn(["close"], window=1, outputs=["log_ret"]),
        ParkinsonVolatility(["high", "low"], window=20, outputs=["pvol_20"]),
    ],
    input_columns=["close", "high", "low"],
)
tp = TargetPipeline(
    targets=[FutureReturn(inputs=["close"], horizon=5, outputs=["ret_5"])],
    input_columns=["close"],
)

X = run_features_pipeline(fp, df)
y = run_targets_pipeline(tp, df)
dataset = pd.concat([X, y], axis=1).dropna()
```

The same `fp` pipeline can then be used in live trading. Call `fp.update(bar)`
on every new tick. No rewrite, no translation layer.

---

## Performance

Features run under 1µs per update in Python at `window=200`
(measured via pytest-benchmark on Apple M-series):

| Feature | Median (w=200) |
|---|---|
| EMA, SimpleReturn, LogReturn | `<200ns` |
| Parkinson, Rogers-Satchell, SMA | `<500ns` |
| LinearSlope, Skewness, Kurtosis | `<1µs` |
| KAMA | `<2µs` |

See the full table on the [Benchmarks](../benchmarks/) page.

---

## Next

- [Streaming vs Research](streaming-vs-research.md): the core Feature / Target model
- [NaN Convention](nan-convention.md): warm-up gaps, horizon gaps, model-ready zone
- [API Reference](../api/features/trend.md): every feature and target documented
