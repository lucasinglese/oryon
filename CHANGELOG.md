# Changelog

All notable changes to this project will be documented here.
The format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
This project adheres to [Semantic Versioning](https://semver.org/).

> Oryon is a full rewrite of [Quantreo](https://github.com/Quantreo/quantreo),
> the original pure-Python library. **Quantreo is deprecated in favour of Oryon.**
> The API has been redesigned alongside the rewrite. Migration is not drop-in.
> This changelog starts at v0.2.0, the first public release of Oryon.

---

## [0.2.8] - 2026-04-05

### Added

**Operators** - stateless streaming transforms, `warm_up_period = 0`

- `Subtract` - computes `A - B` from two input columns. Returns `None` if either input is `None`.
- `NegLog` - computes `-ln(x)`. Returns `None` if `x <= 0` or `None`.

**Scalers** - streaming normalization transforms

- `RollingZScore` - rolling `(x - mean) / std` over a sliding window. Live-compatible.
- `FixedZScore` - `(x - mean) / std` with pre-fitted parameters. Stateless, `warm_up_period = 0`.

**Fitting** - batch parameter estimation for pre-fitted scalers

- `fit_standard_scaler(data)` - computes mean and standard deviation from a column, skipping `None` values. Returns `(mean, std)`. Use the result to construct `FixedZScore`.

### Changed

- Internal trait `Feature` renamed to `StreamingTransform`. Covers features, scalers, and operators uniformly. No Python API change.

---

## [0.2.7] - 2026-04-05

### Added

**Datasets**

- `load_sample_bars()` - loads a built-in synthetic OHLCV sample dataset with approximately 14,000 bars
- The returned `DataFrame` uses a datetime index and includes `open`, `high`, `low`, `close`, and `volume`
- Intended for quick experimentation, testing, examples, and documentation

## [0.2.0] - 2026-04-03

### Added

**Features** - streaming transforms, live trading safe

- `Sma` - Simple Moving Average
- `Ema` - Exponential Moving Average (SMA-seeded)
- `Kama` - Kaufman Adaptive Moving Average
- `SimpleReturn` - arithmetic return over a configurable lookback
- `LogReturn` - log return over a configurable lookback
- `Skewness` - Fisher-Pearson corrected skewness (matches `pandas .skew()`)
- `Kurtosis` - Fisher excess kurtosis (matches `pandas .kurt()`)
- `LinearSlope` - OLS slope and R² over a rolling window
- `ParkinsonVolatility` - high-low realized volatility (Parkinson, 1980)
- `RogersSatchellVolatility` - drift-unbiased OHLC volatility (Rogers-Satchell, 1994)

**Targets** - forward labels, research only

- `FutureReturn` - simple return from `t` to `t + horizon`
- `FutureCTCVolatility` - close-to-close realized volatility over the next `horizon` bars
- `FutureLinearSlope` - OLS slope and R² over the next `horizon` bars

**Pipelines**

- `FeaturePipeline` - DAG-resolved orchestration of features, streaming and research modes
- `TargetPipeline` - batch orchestration of targets, research only
- `run_features_pipeline(pipeline, df)` - pandas helper, preserves index
- `run_targets_pipeline(pipeline, df)` - pandas helper, preserves index

**Infrastructure**

- Rust core via PyO3/maturin, pre-built wheels for Linux, macOS, Windows
- Python 3.9 to 3.13 support
- CI: lint (`cargo fmt`, `clippy`), `cargo test`, `pytest` on all supported Python versions
- Criterion.rs benchmarks for all features and targets