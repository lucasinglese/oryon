<p align="center">
  <img src="docs/assets/figures/oryon_logo_text.svg" alt="Oryon" width="380">
</p>

<p align="center">
  <a href="https://pypi.org/project/oryon/"><img src="https://img.shields.io/pypi/v/oryon.svg" alt="PyPI"></a>
  <img src="https://img.shields.io/badge/python-3.9--3.13-blue" alt="Python versions">
  <a href="https://github.com/lucasinglese/oryon/actions/workflows/ci.yml"><img src="https://github.com/lucasinglese/oryon/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://github.com/lucasinglese/oryon/blob/main/LICENSE"><img src="https://img.shields.io/github/license/lucasinglese/oryon.svg" alt="License"></a>
  <img src="https://img.shields.io/github/last-commit/lucasinglese/oryon" alt="Last commit">
</p>

<p align="center">
  Production-grade feature and forward target engineering for quantitative research.<br>
  Rust core. Python API. Streaming and batch, same object.
</p>

---

## The problem

Most feature engineering libraries take a full DataFrame and return a DataFrame. That works in research. In live trading, it forces you to keep a growing history in memory and recompute every feature on every new bar. This doesn't scale and isn't how production systems work.

A second, quieter problem: research code and live code diverge. Any inconsistency between them is a bug waiting to surface in production.

Oryon solves both. Every feature is a stateful object with a fixed memory footprint. You feed it one bar at a time in live trading, or pass the full dataset in research. Same object, same Rust code, same output.

---

## Install

```bash
pip install oryon
```

No Rust toolchain required. Pre-built wheels for Linux, macOS, and Windows.

---

## Quick start

**Live trading, one bar at a time:**

```python
from oryon.features import Ema, ParkinsonVolatility
from oryon import FeaturePipeline

fp = FeaturePipeline(
    features=[
        Ema(["close"], window=20, outputs=["ema_20"]),
        ParkinsonVolatility(["high", "low"], window=20, outputs=["pvol_20"]),
    ],
    input_columns=["close", "high", "low"],
)

# on each new bar from your data feed
result = fp.update([bar.close, bar.high, bar.low])
# [nan, nan]     during warm-up
# [102.4, 0.018] once ready
```

**Research, full dataset at once:**

```python
import pandas as pd
from oryon import run_features_pipeline, run_targets_pipeline, TargetPipeline
from oryon.targets import FutureReturn

X = run_features_pipeline(fp, df)
y = run_targets_pipeline(
    TargetPipeline(
        targets=[FutureReturn(inputs=["close"], horizon=5, outputs=["ret_5"])],
        input_columns=["close"],
    ),
    df,
)
dataset = pd.concat([X, y], axis=1).dropna()
```

The same `fp` used in live trading builds your training dataset. No rewrite, no translation layer.

---

## Benchmarks

Rust core, Apple M-series. Python adds a constant ~150 ns per call on top.

**Features: per `update()` call**

| Feature | w=20 | w=200 |
|---|---|---|
| `Ema`, `SimpleReturn`, `LogReturn` | < 10 ns | < 10 ns |
| `Sma`, `ParkinsonVolatility`, `RogersSatchellVolatility` | < 30 ns | < 175 ns |
| `Skewness`, `Kurtosis`, `LinearSlope` | < 40 ns | < 510 ns |
| `Kama` | 164 ns | 870 ns |

Every feature under **1 µs** at w=200.

**Targets: per `run_research()` call over 1 000 bars**

| Target | h=20 | h=200 |
|---|---|---|
| `FutureReturn` | 1.9 µs | 1.7 µs |
| `FutureCTCVolatility` | 28 µs | 280 µs |
| `FutureLinearSlope` | 31 µs | 287 µs |

---

## Why not build it yourself?

Custom pipelines accumulate silent bugs:

- **Look-ahead bias.** A feature that accidentally reads future data produces results impossible to replicate in live trading. It will never raise an error.
- **State leakage between folds.** In cross-validation, state from one fold contaminates the next if not explicitly reset. The numbers look plausible. The model is wrong.
- **Research / live divergence.** Batch and streaming implementations drift over time. A subtle difference in edge case handling is enough to break a live strategy.

Every feature and target in Oryon ships with contract tests that enforce `warm_up_period`, `forward_period`, `None` propagation, reset correctness, and instance independence. The test infrastructure is part of the public API. Contributions must pass the same contracts.

---

## Documentation

Full API reference, guides, and benchmarks at **[oryonlib.dev](https://oryonlib.dev)**.

---

## Contributing

Contributions of features and targets are welcome. See the [contributing guide](https://oryonlib.dev/contributing/guide/) for the full workflow and checklist.

---

## License

MIT. See [LICENSE](LICENSE).

---

<p align="center">Developed by <a href="https://www.linkedin.com/in/lucas-inglese-75574817b/">Lucas Inglese</a></p>