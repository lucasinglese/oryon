# Changelog

All notable changes to this project will be documented here.
The format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
This project adheres to [Semantic Versioning](https://semver.org/).

> Oryon is a full rewrite of [Quantreo](https://github.com/Quantreo/quantreo),
> the original pure-Python library. **Quantreo is deprecated in favour of Oryon.**
> The API has been redesigned alongside the rewrite. Migration is not drop-in.
> This changelog starts at v0.2.0, the first public release of Oryon.

---

## [0.2.10] - 2026-04-08

### Added

**Features**

- `Correlation` - rolling pairwise correlation between two series over a sliding window. Supports three methods via the `method` parameter:
  - `'pearson'` - product-moment linear correlation. O(n) per bar. Live trading safe.
  - `'spearman'` - rank correlation, captures monotonic relationships. O(n log n) per bar. Live trading safe.
  - `'kendall'` - Kendall tau-b rank correlation, robust to outliers. O(n^2) per bar. Prefer small windows (`window <= 30`) for live trading.
  - Returns `None` when either series is constant over the window.

**Operators** - 6 new stateless streaming transforms

- `Add` - computes `A + B`. Returns `None` if either input is `None`.
- `Multiply` - computes `A * B`. Returns `None` if either input is `None`.
- `Divide` - computes `A / B`. Returns `None` if either input is `None` or `B == 0`.
- `Reciprocal` - computes `1 / x`. Returns `None` if `x == 0` or `None`.
- `Log` - computes `ln(x)`. Returns `None` if `x <= 0` or `None`.
- `Logit` - computes `ln(x / (1 - x))`. Returns `None` if `x` is outside `(0, 1)` or `None`.

### Changed

- Operators are now generated via internal `binary_operator!` and `unary_operator!` macros, eliminating boilerplate while keeping one file per operator and explicit tests per edge case. No API change.

---

## [0.2.9] - 2026-04-08

### Added

**Features**

- `Adf` - rolling Augmented Dickey-Fuller test. Produces ADF statistic and approximate p-value (MacKinnon 2010) over a sliding window. Supports constant and constant+trend regressions, configurable lag count via Schwert's rule.
- `Mma` - Median Moving Average over a rolling window.
- `ShannonEntropy` - rolling Shannon entropy over a fixed window. Discretizes values into equal-width bins, supports Sturges' rule or fixed bin count, optional normalization to [0, 1].

**Python exceptions** - typed error hierarchy replacing the flat `ValueError`

- `OryonError` - base exception for all Oryon errors
- `InvalidConfigError` - invalid constructor parameter (e.g. bad regression type)
- `InvalidInputError` - invalid value passed at construction or runtime (e.g. `window=0`)
- `MissingInputColumnError` - required input column not found in the pipeline
- `DuplicateOutputKeyError` - two features produce the same output key
- `CyclicDependencyError` - cyclic dependency detected in the feature DAG

**Adapters** - DataFrame helpers moved to `oryon.adapters`

- `run_features_pipeline_pandas(pipeline, df)` - runs a `FeaturePipeline` on a pandas DataFrame, preserves index
- `run_targets_pipeline_pandas(pipeline, df)` - runs a `TargetPipeline` on a pandas DataFrame, preserves index
- `run_features_pipeline_polars(pipeline, df)` - runs a `FeaturePipeline` on a polars DataFrame
- `run_targets_pipeline_polars(pipeline, df)` - runs a `TargetPipeline` on a polars DataFrame

### Changed

- `run_features_pipeline` and `run_targets_pipeline` (previously at top-level) are removed. Use `oryon.adapters` instead.

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
- `run_features_pipeline_pandas(pipeline, df)` - pandas helper, preserves index
- `run_targets_pipeline_pandas(pipeline, df)` - pandas helper, preserves index

**Infrastructure**

- Rust core via PyO3/maturin, pre-built wheels for Linux, macOS, Windows
- Python 3.9 to 3.13 support
- CI: lint (`cargo fmt`, `clippy`), `cargo test`, `pytest` on all supported Python versions
- Criterion.rs benchmarks for all features and targets