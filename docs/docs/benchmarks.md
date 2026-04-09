# Benchmarks

All numbers are **Rust core** measurements. The Python API adds a constant PyO3
boundary cost of roughly 100-200 ns per call on top, independent of feature or
window size. See [Python overhead](#python-overhead) below.

---

## Methodology

| | |
|---|---|
| **Toolchain** | Criterion.rs, bencher output format |
| **Profile** | `--release` (full optimizations) |
| **Hardware** | Apple M-series |
| **Metric** | Median over thousands of iterations |
| **Feature measurement** | Per `update()` call with a warmed-up buffer (steady state) |
| **Target measurement** | Full `run_research()` pass over 1 000 bars |

Benchmarks are located in `crates/oryon/benches/`. See [Reproduce](#reproduce) to
run them on your own hardware.

---

## Streaming Latency

*Measured per `update()` call, buffer full, valid input. This is the cost paid on every live bar.*

### Features

| Feature | w=20 | w=200 |
|---|---|---|
| `Ema` | 4 ns | 4 ns |
| `SimpleReturn` | 4 ns | 4 ns |
| `LogReturn` | 7 ns | 7 ns |
| `Sma` | 14 ns | 151 ns |
| `ParkinsonVolatility` | 17 ns | 164 ns |
| `RogersSatchellVolatility` | 26 ns | 178 ns |
| `AutoCorrelation` (pearson) | 36 ns | 382 ns |
| `Correlation` (pearson) | 39 ns | 384 ns |
| `Skewness` | 36 ns | 507 ns |
| `Kurtosis` | 36 ns | 510 ns |
| `LinearSlope` | 38 ns | 383 ns |
| `Mma` | 156 ns | 576 ns |
| `Kama` | 158 ns | 641 ns |
| `ShannonEntropy` | 262 ns | 951 ns |
| `AutoCorrelation` (spearman) | 279 ns | 1 700 ns |
| `Correlation` (spearman) | 279 ns | 1 687 ns |
| `Adf` | 457 ns | 2 306 ns |
| `AutoCorrelation` (kendall) | 245 ns | 18 039 ns |
| `Correlation` (kendall) | 248 ns | 17 779 ns |

`Ema`, `SimpleReturn`, and `LogReturn` are `O(1)` per update regardless of window
size. Most features (`Sma`, `Skewness`, `Kurtosis`, `LinearSlope`, `Mma`, `Kama`,
`ShannonEntropy`, `ParkinsonVolatility`, `RogersSatchellVolatility`) stay under
1 µs at w=200. Exceptions: `Correlation` and `AutoCorrelation` with `spearman`
(~1.7 µs, O(n log n)) or `kendall` (~18 µs, O(n²)), and `Adf` (~2.3 µs, full OLS
on each bar). Pearson is the default and stays under 400 ns at w=200.

!!! tip "Research mode cost for features"
    `FeaturePipeline.run_research()` calls `update()` in a loop. The total cost to
    process a dataset is simply `update latency × number of bars`.

    Example: `Sma` at w=200 (144 ns/update) over 1 000 000 bars ≈ **144 ms**.

### Scalers

| Scaler | w=20 | w=200  |
|---|---|--------|
| `FixedZScore` | 1 ns | 1 ns   |
| `RollingZScore` | 35 ns | 496 ns |

`FixedZScore` is stateless: fixed parameters mean no buffer to maintain (`O(1)`, constant).
`RollingZScore` recomputes mean and std over the full buffer on each update (`O(N)`) and
scales with window size, matching `Sma` in character.

### Operators

*Operators are stateless. No window parameter. a single latency is reported.*

| Operator | Latency |
|---|---|
| `Reciprocal` | 1 ns |
| `Add` | 2 ns |
| `Subtract` | 2 ns |
| `Multiply` | 2 ns |
| `Divide` | 2 ns |
| `Log` | 4 ns |
| `NegLog` | 4 ns |
| `Logit` | 4 ns |

All operators are `O(1)`: they perform a fixed arithmetic operation with no buffer or state.

---

## Research Throughput

*Measured per `run_research()` call over 1 000 bars. Targets are research-only and never called in live trading.*

| Target | h=20 / 1k bars | h=200 / 1k bars |
|---|---|---|
| `FutureReturn` | 1.9 µs | 1.7 µs |
| `FutureCTCVolatility` | 28 µs | 280 µs |
| `FutureLinearSlope` | 31 µs | 287 µs |

`FutureReturn` is `O(N)` independent of horizon. `FutureCTCVolatility` and
`FutureLinearSlope` are `O(N · h)`: cost scales linearly with both dataset size
and horizon.

---

## Python overhead

The PyO3 boundary adds roughly **100-200 ns per call** on top of the Rust numbers
above. This cost is constant: it does not depend on the feature, the window size,
or the number of outputs.

At w=200, the most expensive feature in Python is `Adf` at roughly 2.5 µs total.
`Correlation` and `AutoCorrelation` with `pearson` or `spearman` stay under 2 µs.
`kendall` at w=200 reaches roughly 18 µs.

---

## Reproduce

```bash
cargo bench --bench features  -- --output-format bencher
cargo bench --bench scalers   -- --output-format bencher
cargo bench --bench operators -- --output-format bencher
cargo bench --bench targets   -- --output-format bencher
cargo bench --bench ops       -- --output-format bencher
```

Or with `make`:

```bash
make bench-rust
```

Re-run after any change to the Rust core. Results will vary by hardware.
