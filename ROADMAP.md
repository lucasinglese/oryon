# Roadmap

## v1 — Core (done)

- [x] Features: Sma, Ema, Kama, SimpleReturn, LogReturn, Skewness, Kurtosis, LinearSlope, ParkinsonVolatility, RogersSatchellVolatility
- [x] Targets: FutureReturn, FutureCTCVolatility, FutureLinearSlope
- [x] FeaturePipeline (DAG) + TargetPipeline
- [x] Python facade (PyO3/maturin) — submodules oryon.features / oryon.targets
- [x] Type stubs (.pyi) + py.typed
- [x] CI — lint + cargo test + pytest (Python 3.9 → 3.14)
- [x] Benchmarks Rust + Python
- [x] Makefile

## v2 — Scale

- [ ] Features x50 — RSI, ATR, Bollinger, MACD, momentum, rolling beta/corr, ...
- [ ] Targets x10 — FutureDirection, FutureMaxDrawdown, ...
- [ ] CHANGELOG — remplir à chaque release, suivre Keep a Changelog
- [ ] CD — publier les wheels automatiquement sur tag (maturin publish / PyPI privé)
- [ ] Property-based testing (`proptest`) — inputs aléatoires, invariants sur tous les features

## v3 — Production

- [ ] Pipeline serialization — save/load state pour reprendre sans warm-up
- [ ] Tick-to-bars module