# Philosophy

Most quants who deploy systematic strategies hit the same wall. A feature backtests beautifully in a Jupyter notebook. It gets translated into production code, deployed alongside a live strategy, and the numbers don't match. Sometimes the divergence is obvious and gets caught. More often, it's silent. The strategy quietly underperforms its backtest by 20-30%, and nobody understands why.

The root cause is almost never the strategy itself. It's the feature computation that diverges between research and production. Research code computes features over a full historical DataFrame. Production code computes them one bar at a time. Two codebases for the same logic, written at different times, by different people, with different assumptions about state management, edge cases, and timing. They drift apart. Not dramatically, just enough to erode every edge you thought you had.

**Oryon exists to make this problem structurally impossible.**

## One Object, Two Modes

The central design decision in Oryon is that each feature is a single Python object that operates in two modes.

In research mode, you call `run_research()` on a full historical dataset and get a complete feature column back. In live mode, you call `update()` with one new bar at a time and get the current feature value. The critical guarantee is that both modes produce bit-for-bit identical results, because they execute the same underlying computation.

!!! info "Key guarantee"
    `run_research()` is not a separate implementation. It calls `update()` internally, once per bar, in sequence. Same code path, same state, same results.

There is no "research version" and "production version" of a feature. There is one feature, two interfaces. This eliminates research-to-production drift by construction, not by discipline. You don't need code reviews to catch divergence. You don't need integration tests comparing two implementations. The divergence cannot exist because the second implementation does not exist.

## Causal by Construction

Oryon enforces a strict separation between features and targets.

Features are causal: they use only past and present data. When you call `update()` with the current bar, the feature has access to that bar and everything before it. Nothing else. This is enforced by the streaming API itself - you cannot pass future data to a feature because the interface accepts one bar at a time.

Targets are anti-causal: they use future data to produce labels for ML training. `FutureReturn`, `FutureCTCVolatility`, `FutureLinearSlope` - these exist in a separate module, have a different API (`compute()` over a full dataset), and are explicitly marked as research-only. They cannot be used in streaming mode. The separation is architectural, not conventional.

!!! warning "Look-ahead bias"
    Look-ahead bias is structurally impossible in feature computation. The streaming API accepts one bar at a time - you cannot pass future data to a feature because the interface does not allow it.

If you need forward-looking data, you reach for a target, and the code makes it explicit that you are in research-only territory.

## Contract-Tested for Silent Errors

Every transformation in Oryon ships with 20+ contract tests. These are not happy-path tests that verify a moving average produces roughly the right number. They cover the edge cases that are commonly mishandled in quant code: division by zero, logarithm of zero or negative values, NaN propagation through chains of computations, missing values at arbitrary positions, warm-up period behavior, state independence between instances, and reset behavior across walk-forward folds.

!!! danger "Why this matters"
    Silent errors in feature computation don't crash your system. A division by zero that returns `inf` instead of `NaN` propagates through your model, generates a position, and loses money. These bugs don't announce themselves. They compound.

A feature that leaks state between instances corrupts your entire pipeline without raising an exception.

Catching them via contract tests at the library level means you can trust the computation without having to verify it yourself. When you use an Oryon feature, the edge cases have already been handled and tested. Your responsibility starts at the strategy level, not the feature level.

## Performance That Doesn't Get In The Way

Python for the interface, because that's where quant research happens. Rust for the core, because Oryon needed predictable sub-microsecond latency without GIL contention, garbage collection pauses, or JIT warmup.

The Rust internals are invisible to users. You write Python, you import from `oryon`, you call methods on Python objects. The PyO3 bridge handles the rest.

!!! tip "Concrete numbers"
    Under 1 microsecond per feature update on a single bar. A pipeline of 50 features updates in under 1 millisecond per bar. Feature computation will never be the bottleneck for intraday, daily, or most sub-second strategies.

That said, Oryon is not designed for ultra-high-frequency trading where sub-100-nanosecond latency is required. At that level, you need custom C++ with direct memory-mapped I/O and kernel bypass networking. For everything above that threshold, Oryon is more performance than you will use - and that margin is intentional. Headroom means your feature pipeline stays fast as it grows.

## What Oryon Is Not

!!! note "Positioning"
    - **Not a backtest engine.** It computes features, not strategies.
    - **Not a strategy framework.** What you do with the features is your decision.
    - **Not another TA library.** It's a transformation engine designed for production deployment.
    - **Not for HFT.** If you need nanosecond latency, build it custom in C++.

## Who Oryon Is For

!!! success "Target users"
    - **Quants** deploying systematic strategies in live trading who want to eliminate research-to-production drift.
    - **ML/AI engineers** building trading models who need bit-for-bit consistency between training and inference.
    - **Researchers** tired of maintaining two codebases for the same logic.
    - **Independent traders** who want production-grade infrastructure without building it from scratch.

## A Living Project

Oryon is in active development. The current release ships with transformations across trend, volatility, statistics, and returns, plus forward-looking targets for ML labeling, scalers, operators, fitting utilities, and full pipeline orchestration. The roadmap targets 100+ transformations, additional bar types, multi-asset transformations, and intra-bar features. Contributions are welcome via the [GitHub repository](https://github.com/lucasinglese/oryon). The direction is clear: make feature engineering for live trading a solved problem.