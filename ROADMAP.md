# Roadmap

## v1 — Core (in progress)

- [ ] Features: EMA, RSI, ATR, Bollinger, MACD, ...
- [ ] Targets: FutureReturn, FutureDirection, ...
- [ ] Python facade (PyO3/maturin) — after ~20 features
- [ ] Documentation (rustdoc + examples)
- [ ] CI (cargo test + cargo clippy on push)

## v2 — Production

- [ ] Pipeline serialization — save/load internal state (buffers) to disk so a live pipeline can resume after a restart without waiting for warm-up
- [ ] Tick-to-bars module