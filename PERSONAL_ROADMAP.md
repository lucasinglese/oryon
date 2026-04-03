# Roadmap

## v0.2.0 — Publication (current)

- [x] Features: Sma, Ema, Kama, SimpleReturn, LogReturn, Skewness, Kurtosis, LinearSlope, ParkinsonVolatility, RogersSatchellVolatility
- [x] Targets: FutureReturn, FutureCTCVolatility, FutureLinearSlope
- [x] FeaturePipeline (DAG) + TargetPipeline
- [x] Python facade (PyO3/maturin) — submodules oryon.features / oryon.targets
- [x] Type stubs (.pyi) + py.typed
- [x] CI — lint + cargo test + pytest (Python 3.9 → 3.14)
- [x] Benchmarks Rust + Python
- [x] Makefile
- [x] Documentation (MkDocs Material)
- [ ] README.md
- [ ] CHANGELOG.md — v0.2.0 entry
- [ ] GitHub repo: description + topics (python, rust, quantitative-finance, feature-engineering, algo-trading)
- [ ] GitHub social preview image — shown when sharing the repo link
- [ ] MkDocs social cards — og:image auto-generated for each doc page when sharing the docs link
- [ ] .github/PULL_REQUEST_TEMPLATE.md
- [ ] .github/ISSUE_TEMPLATE — bug report + feature request
- [ ] CONTRIBUTING.md at root (pointer to docs)
- [ ] CD — publish wheels automatically on tag (maturin publish / PyPI)

## v0.3.0+ — Scale

- [ ] Features x50 — RSI, ATR, Bollinger, MACD, momentum, rolling beta/corr, ...
- [ ] Targets x10 — FutureDirection, FutureMaxDrawdown, ...
- [ ] Running sum for Sma (O(N) to O(1) per update)
- [ ] Property-based testing (`proptest`) — random inputs, invariants on all features
- [ ] llms.txt
- [ ] Doc compliance prompt — LLM check against source for badges, tabs, NaN propagation, em dashes, obsolete Contribute tabs

## v1.0.0 — Production

- [ ] Pipeline serialization — save/load state to resume without warm-up
- [ ] Tick-to-bars module
