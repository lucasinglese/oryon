# Roadmap

This document outlines the planned development of Oryon. Dates are indicative.
The API is not stable until v1.0.0, breaking changes may occur on any `0.x` release.

---

## v0.2.0 — Current

- 10 features: Sma, Ema, Kama, SimpleReturn, LogReturn, Skewness, Kurtosis, LinearSlope, ParkinsonVolatility, RogersSatchellVolatility
- 3 forward targets: FutureReturn, FutureCTCVolatility, FutureLinearSlope
- FeaturePipeline (DAG) + TargetPipeline
- Python 3.9 to 3.14, pre-built wheels for Linux, macOS, Windows

---

## v0.3.0 — Scale

- Features x50: RSI, ATR, Bollinger Bands, MACD, momentum, rolling beta, rolling correlation, ...
- Targets x10: FutureDirection, FutureMaxDrawdown, ...
- Performance: O(1) running sum for Sma

---

## v0.4.0 — Hardening

- Property-based testing (`proptest`): random inputs, invariants on all features and targets
- Tick-to-bars module (beta)

---

## v1.0.0 — Stable

- Stable API: no breaking changes without deprecation notice from this point
- Pipeline serialization: save and restore state without warm-up
- Tick-to-bars module (stable)