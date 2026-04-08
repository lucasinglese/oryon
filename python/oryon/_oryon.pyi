from typing import List, Optional

# ---------------------------------------------------------------------------
# Features
# ---------------------------------------------------------------------------

class Sma:
    """Simple Moving Average over a rolling window.

    Computes the arithmetic mean of the last ``window`` bars.
    Returns ``NaN`` during warm-up (first ``window - 1`` bars) and on ``NaN`` input.
    """

    def __init__(self, inputs: List[str], window: int, outputs: List[str]) -> None:
        """Create a new Sma.

        Args:
            inputs: Name of the input column (e.g. ``["close"]``).
            window: Number of bars in the rolling window. Must be >= 1.
            outputs: Name of the output column (e.g. ``["close_sma_20"]``).

        Raises:
            ValueError: If ``window`` is 0 or ``inputs``/``outputs`` are empty.
        """
        ...

    def update(self, values: List[float]) -> List[float]:
        """Process one bar and return ``[sma]``. Returns ``[NaN]`` during warm-up."""
        ...

    def reset(self) -> None:
        """Reset internal state (e.g. between CPCV splits)."""
        ...

    def input_names(self) -> List[str]:
        """Input column names."""
        ...

    def output_names(self) -> List[str]:
        """Output column names."""
        ...

    def warm_up_period(self) -> int:
        """Number of bars before the first valid output (``window - 1``)."""
        ...

    def __repr__(self) -> str: ...


class Ema:
    """Exponential Moving Average with SMA seeding.

    Uses ``α = 2 / (window + 1)``. Seeds on the SMA of the first ``window`` bars,
    then applies ``EMA_t = α * P_t + (1 - α) * EMA_{t-1}``.
    A ``NaN`` input resets the state and restarts the warm-up.
    """

    def __init__(self, inputs: List[str], window: int, outputs: List[str]) -> None:
        """Create a new Ema.

        Args:
            inputs: Name of the input column (e.g. ``["close"]``).
            window: Number of bars for seeding and smoothing factor. Must be >= 1.
            outputs: Name of the output column (e.g. ``["close_ema_20"]``).

        Raises:
            ValueError: If ``window`` is 0 or ``inputs``/``outputs`` are empty.
        """
        ...

    def update(self, values: List[float]) -> List[float]:
        """Process one bar and return ``[ema]``. Returns ``[NaN]`` during warm-up."""
        ...

    def reset(self) -> None:
        """Reset internal state (e.g. between CPCV splits)."""
        ...

    def input_names(self) -> List[str]:
        """Input column names."""
        ...

    def output_names(self) -> List[str]:
        """Output column names."""
        ...

    def warm_up_period(self) -> int:
        """Number of bars before the first valid output (``window - 1``)."""
        ...

    def __repr__(self) -> str: ...


class Kama:
    """Kaufman's Adaptive Moving Average.

    Adapts its smoothing speed based on the Efficiency Ratio (ER).
    Tracks price closely in trending markets, barely moves in choppy ones.
    Kaufman defaults: ``window=10``, ``fast=2``, ``slow=30``.
    """

    def __init__(
        self,
        inputs: List[str],
        window: int,
        outputs: List[str],
        fast: int = 2,
        slow: int = 30,
    ) -> None:
        """Create a new Kama.

        Args:
            inputs: Name of the input column (e.g. ``["close"]``).
            window: Lookback for the Efficiency Ratio. Must be >= 1.
            outputs: Name of the output column (e.g. ``["close_kama_10"]``).
            fast: Period for the fast smoothing constant. Must be >= 1. Default: 2.
            slow: Period for the slow smoothing constant. Must be > fast. Default: 30.

        Raises:
            ValueError: If params are invalid (window=0, slow <= fast, etc.).
        """
        ...

    def update(self, values: List[float]) -> List[float]:
        """Process one bar and return ``[kama]``. Returns ``[NaN]`` during warm-up."""
        ...

    def reset(self) -> None:
        """Reset internal state (e.g. between CPCV splits)."""
        ...

    def input_names(self) -> List[str]:
        """Input column names."""
        ...

    def output_names(self) -> List[str]:
        """Output column names."""
        ...

    def warm_up_period(self) -> int:
        """Number of bars before the first valid output (``window``)."""
        ...

    def __repr__(self) -> str: ...


class SimpleReturn:
    """Simple (arithmetic) return over a configurable lookback window.

    Computes ``(P_t - P_{t-n}) / P_{t-n}``.
    Returns ``NaN`` during warm-up or if the previous price is <= 0.
    """

    def __init__(self, inputs: List[str], window: int, outputs: List[str]) -> None:
        """Create a new SimpleReturn.

        Args:
            inputs: Name of the input column (e.g. ``["close"]``).
            window: Lookback in bars. Must be >= 1.
            outputs: Name of the output column (e.g. ``["close_simple_return_5"]``).

        Raises:
            ValueError: If ``window`` is 0 or ``inputs``/``outputs`` are empty.
        """
        ...

    def update(self, values: List[float]) -> List[float]:
        """Process one bar and return ``[simple_return]``. Returns ``[NaN]`` during warm-up."""
        ...

    def reset(self) -> None:
        """Reset internal state (e.g. between CPCV splits)."""
        ...

    def input_names(self) -> List[str]:
        """Input column names."""
        ...

    def output_names(self) -> List[str]:
        """Output column names."""
        ...

    def warm_up_period(self) -> int:
        """Number of bars before the first valid output (``window``)."""
        ...

    def __repr__(self) -> str: ...


class LogReturn:
    """Log return over a configurable lookback window.

    Computes ``ln(P_t / P_{t-n})``.
    Returns ``NaN`` during warm-up (first ``window`` bars).
    """

    def __init__(self, inputs: List[str], window: int, outputs: List[str]) -> None:
        """Create a new LogReturn.

        Args:
            inputs: Name of the input column (e.g. ``["close"]``).
            window: Lookback in bars. Must be >= 1.
            outputs: Name of the output column (e.g. ``["close_log_return_5"]``).

        Raises:
            ValueError: If ``window`` is 0 or ``inputs``/``outputs`` are empty.
        """
        ...

    def update(self, values: List[float]) -> List[float]:
        """Process one bar and return ``[log_return]``. Returns ``[NaN]`` during warm-up."""
        ...

    def reset(self) -> None:
        """Reset internal state (e.g. between CPCV splits)."""
        ...

    def input_names(self) -> List[str]:
        """Input column names."""
        ...

    def output_names(self) -> List[str]:
        """Output column names."""
        ...

    def warm_up_period(self) -> int:
        """Number of bars before the first valid output (``window``)."""
        ...

    def __repr__(self) -> str: ...


class Skewness:
    """Rolling sample skewness (Fisher-Pearson corrected, same as pandas ``.skew()``).

    Returns ``NaN`` during warm-up (first ``window - 1`` bars)
    or if all values in the window are equal (std = 0).
    """

    def __init__(self, inputs: List[str], window: int, outputs: List[str]) -> None:
        """Create a new Skewness.

        Args:
            inputs: Name of the input column (e.g. ``["close"]``).
            window: Number of bars. Must be >= 3.
            outputs: Name of the output column (e.g. ``["close_skewness_20"]``).

        Raises:
            ValueError: If ``window`` < 3 or ``inputs``/``outputs`` are empty.
        """
        ...

    def update(self, values: List[float]) -> List[float]:
        """Process one bar and return ``[skewness]``. Returns ``[NaN]`` during warm-up."""
        ...

    def reset(self) -> None:
        """Reset internal state (e.g. between CPCV splits)."""
        ...

    def input_names(self) -> List[str]:
        """Input column names."""
        ...

    def output_names(self) -> List[str]:
        """Output column names."""
        ...

    def warm_up_period(self) -> int:
        """Number of bars before the first valid output (``window - 1``)."""
        ...

    def __repr__(self) -> str: ...


class Kurtosis:
    """Rolling excess kurtosis (Fisher, same as pandas ``.kurt()``).

    Returns ``NaN`` during warm-up (first ``window - 1`` bars)
    or if all values in the window are equal (std = 0).
    """

    def __init__(self, inputs: List[str], window: int, outputs: List[str]) -> None:
        """Create a new Kurtosis.

        Args:
            inputs: Name of the input column (e.g. ``["close"]``).
            window: Number of bars. Must be >= 4.
            outputs: Name of the output column (e.g. ``["close_kurtosis_20"]``).

        Raises:
            ValueError: If ``window`` < 4 or ``inputs``/``outputs`` are empty.
        """
        ...

    def update(self, values: List[float]) -> List[float]:
        """Process one bar and return ``[kurtosis]``. Returns ``[NaN]`` during warm-up."""
        ...

    def reset(self) -> None:
        """Reset internal state (e.g. between CPCV splits)."""
        ...

    def input_names(self) -> List[str]:
        """Input column names."""
        ...

    def output_names(self) -> List[str]:
        """Output column names."""
        ...

    def warm_up_period(self) -> int:
        """Number of bars before the first valid output (``window - 1``)."""
        ...

    def __repr__(self) -> str: ...


class LinearSlope:
    """Rolling OLS regression: slope and R² of y on x over a sliding window.

    Returns two outputs ``[slope, r2]``.
    Both are ``NaN`` during warm-up (first ``window - 1`` bars),
    if any value is ``NaN``, or if x is constant over the window.
    R² is additionally ``NaN`` if y is constant.
    """

    def __init__(self, inputs: List[str], window: int, outputs: List[str]) -> None:
        """Create a new LinearSlope.

        Args:
            inputs: Names of the x and y columns in that order
                (e.g. ``["time_idx", "close"]``). Must contain >= 2 entries.
            window: Number of bars in the rolling window. Must be >= 2.
            outputs: Names of the two output columns ``[slope_name, r2_name]``.
                Must contain exactly 2 entries.

        Raises:
            ValueError: If ``inputs`` has fewer than 2 entries, ``window`` < 2,
                or ``outputs`` does not have exactly 2 entries.
        """
        ...

    def update(self, values: List[float]) -> List[float]:
        """Process one bar and return ``[slope, r2]``. Returns ``[NaN, NaN]`` during warm-up."""
        ...

    def reset(self) -> None:
        """Reset internal state (e.g. between CPCV splits)."""
        ...

    def input_names(self) -> List[str]:
        """Input column names (x, y)."""
        ...

    def output_names(self) -> List[str]:
        """Output column names (slope, r2)."""
        ...

    def warm_up_period(self) -> int:
        """Number of bars before the first valid output (``window - 1``)."""
        ...

    def __repr__(self) -> str: ...


class ParkinsonVolatility:
    """Rolling Parkinson volatility estimator using high and low prices.

    More efficient than close-to-close for assets with significant intraday movement.
    Formula: ``sqrt(mean(ln(H/L)²) / (4·ln(2)))``.
    Returns ``NaN`` during warm-up or if any high/low pair is invalid.
    """

    def __init__(self, inputs: List[str], window: int, outputs: List[str]) -> None:
        """Create a new ParkinsonVolatility.

        Args:
            inputs: Names of the high and low columns in that order
                (e.g. ``["high", "low"]``). Must contain >= 2 entries.
            window: Number of bars in the rolling window. Must be >= 1.
            outputs: Name of the output column (e.g. ``["parkinson_vol_20"]``).

        Raises:
            ValueError: If ``inputs`` has fewer than 2 entries or ``window`` is 0.
        """
        ...

    def update(self, values: List[float]) -> List[float]:
        """Process one bar ``[high, low]`` and return ``[volatility]``."""
        ...

    def reset(self) -> None:
        """Reset internal state (e.g. between CPCV splits)."""
        ...

    def input_names(self) -> List[str]:
        """Input column names (high, low)."""
        ...

    def output_names(self) -> List[str]:
        """Output column names."""
        ...

    def warm_up_period(self) -> int:
        """Number of bars before the first valid output (``window - 1``)."""
        ...

    def __repr__(self) -> str: ...


class RogersSatchellVolatility:
    """Rolling Rogers-Satchell volatility estimator using OHLC prices.

    Unbiased in the presence of a directional drift — more suitable than
    Parkinson for trending markets.
    Formula: ``sqrt(mean(ln(H/C)·ln(H/O) + ln(L/C)·ln(L/O)))``.
    Returns ``NaN`` during warm-up or if any OHLC value is invalid.
    """

    def __init__(self, inputs: List[str], window: int, outputs: List[str]) -> None:
        """Create a new RogersSatchellVolatility.

        Args:
            inputs: Names of the high, low, open, and close columns in that order
                (e.g. ``["high", "low", "open", "close"]``). Must contain >= 4 entries.
            window: Number of bars in the rolling window. Must be >= 1.
            outputs: Name of the output column (e.g. ``["rs_vol_20"]``).

        Raises:
            ValueError: If ``inputs`` has fewer than 4 entries or ``window`` is 0.
        """
        ...

    def update(self, values: List[float]) -> List[float]:
        """Process one bar ``[high, low, open, close]`` and return ``[volatility]``."""
        ...

    def reset(self) -> None:
        """Reset internal state (e.g. between CPCV splits)."""
        ...

    def input_names(self) -> List[str]:
        """Input column names (high, low, open, close)."""
        ...

    def output_names(self) -> List[str]:
        """Output column names."""
        ...

    def warm_up_period(self) -> int:
        """Number of bars before the first valid output (``window - 1``)."""
        ...

    def __repr__(self) -> str: ...


class ShannonEntropy:
    """Rolling Shannon entropy over a fixed window.

    Discretizes the last ``window`` values into equal-width bins and computes
    ``H = -sum(p_i * ln(p_i))`` in nats. When ``normalize`` is ``True``,
    outputs ``H / ln(n_bins)`` in [0, 1].
    When all values in the window are identical, entropy is ``0.0`` (not ``NaN``).
    Returns ``NaN`` during warm-up (first ``window - 1`` bars) and while any
    ``NaN`` value remains in the rolling window.
    """

    def __init__(
        self,
        inputs: List[str],
        window: int,
        outputs: List[str],
        bins: Optional[int] = None,
        normalize: bool = True,
    ) -> None:
        """Create a new ShannonEntropy.

        Args:
            inputs: Name of the input column (e.g. ``["returns"]``).
            window: Number of bars in the rolling window. Must be >= 2.
            outputs: Name of the output column (e.g. ``["returns_entropy_20"]``).
            bins: Number of equal-width bins. Must be >= 2. ``None`` applies
                Sturges' rule ``k = ceil(1 + log2(window))``. Default: ``None``.
            normalize: If ``True``, output is ``H / ln(bins)`` in [0, 1]. Default: ``True``.

        Raises:
            ValueError: If ``window`` < 2, ``bins`` < 2, or ``inputs``/``outputs`` are empty.
        """
        ...

    def update(self, values: List[float]) -> List[float]:
        """Process one bar and return ``[entropy]``. Returns ``[NaN]`` during warm-up."""
        ...

    def reset(self) -> None:
        """Reset internal state (e.g. between CPCV splits)."""
        ...

    def input_names(self) -> List[str]:
        """Input column names."""
        ...

    def output_names(self) -> List[str]:
        """Output column names."""
        ...

    def warm_up_period(self) -> int:
        """Number of bars before the first valid output (``window - 1``)."""
        ...

    def __repr__(self) -> str: ...


# ---------------------------------------------------------------------------
# Targets
# ---------------------------------------------------------------------------

class FutureReturn:
    """Future simple return over ``horizon`` bars.

    For each bar t, computes ``(price[t + horizon] - price[t]) / price[t]``.
    The last ``horizon`` bars are ``NaN`` (future unknown).
    """

    def __init__(self, inputs: List[str], horizon: int, outputs: List[str]) -> None:
        """Create a new FutureReturn target.

        Args:
            inputs: Price series name (e.g. ``["close"]``).
            horizon: Number of bars to look ahead. Must be >= 1.
            outputs: Name of the output column (e.g. ``["close_future_return_5"]``).

        Raises:
            ValueError: If ``horizon`` is 0 or ``inputs``/``outputs`` are empty.
        """
        ...

    def input_names(self) -> List[str]:
        """Input column names."""
        ...

    def output_names(self) -> List[str]:
        """Output column names."""
        ...

    def forward_period(self) -> int:
        """Number of bars at the end that will be ``NaN``."""
        ...

    def __repr__(self) -> str: ...


class FutureCTCVolatility:
    """Future close-to-close realized volatility over ``horizon`` bars.

    For each bar t, computes the rolling std of log-returns over
    ``[t, t + horizon)``. The last ``horizon`` bars are ``NaN``.
    Output name is auto-generated as ``"{input}_future_ctc_vol_{horizon}"``.
    """

    def __init__(self, input: str, horizon: int) -> None:
        """Create a new FutureCTCVolatility target.

        Args:
            input: Price series name (e.g. ``"close"``).
            horizon: Number of bars to look ahead. Must be >= 1.

        Raises:
            ValueError: If ``horizon`` is 0 or ``input`` is empty.
        """
        ...

    def input_names(self) -> List[str]:
        """Input column names."""
        ...

    def output_names(self) -> List[str]:
        """Output column names (auto-generated: ``"{input}_future_ctc_vol_{horizon}"``)."""
        ...

    def forward_period(self) -> int:
        """Number of bars at the end that will be ``NaN``."""
        ...

    def __repr__(self) -> str: ...


class FutureLinearSlope:
    """Future OLS slope and R² of y regressed on x over the next ``horizon`` bars.

    Returns two outputs ``[slope, r2]``.
    The last ``horizon - 1`` bars are ``NaN`` (future unknown).
    """

    def __init__(self, inputs: List[str], horizon: int, outputs: List[str]) -> None:
        """Create a new FutureLinearSlope target.

        Args:
            inputs: Names of the x and y columns in that order
                (e.g. ``["time_idx", "close"]``). Must contain >= 2 entries.
            horizon: Number of bars to look ahead. Must be >= 2.
            outputs: Names of the two output columns ``[slope_name, r2_name]``.
                Must contain exactly 2 entries.

        Raises:
            ValueError: If ``horizon`` < 2, ``inputs`` has fewer than 2 entries,
                or ``outputs`` does not have exactly 2 entries.
        """
        ...

    def input_names(self) -> List[str]:
        """Input column names (x, y)."""
        ...

    def output_names(self) -> List[str]:
        """Output column names (slope, r2)."""
        ...

    def forward_period(self) -> int:
        """Horizon used at construction."""
        ...

    def __repr__(self) -> str: ...


# ---------------------------------------------------------------------------
# Pipelines
# ---------------------------------------------------------------------------

class FeaturePipeline:
    """Orchestrates features in DAG-resolved order.

    Dependencies between features are inferred automatically from their
    ``input_names()`` and ``output_names()``. Pass features in any order.

    Use ``run_research()`` for batch mode (full dataset at once)
    or ``update()`` for live mode (one bar at a time).
    """

    def __init__(
        self,
        features: list,
        input_columns: List[str],
    ) -> None:
        """Create a new FeaturePipeline.

        Args:
            features: List of feature objects (Sma, Ema, etc.). Dependencies
                are resolved automatically via DAG.
            input_columns: Column names in the order passed to ``update()``.

        Raises:
            ValueError: If there are duplicate output keys, cyclic dependencies,
                or required input columns are missing from ``input_columns``.
        """
        ...

    def update(self, values: List[float]) -> List[float]:
        """Process one bar (live mode).

        Args:
            values: One float per input column. Use ``float('nan')`` for missing.

        Returns:
            Flat list of output values matching ``output_names()``.
        """
        ...

    def run_research(self, data: List[List[float]]) -> List[List[float]]:
        """Process a full dataset bar by bar (research mode).

        Args:
            data: List of bars, each bar is a list of input values.

        Returns:
            List of bars, each bar is a list of output values.
        """
        ...

    def reset(self) -> None:
        """Reset all features (e.g. between CPCV splits)."""
        ...

    def output_names(self) -> List[str]:
        """Output column names in execution order."""
        ...

    def input_names(self) -> List[str]:
        """Input columns in the order expected by ``update()``."""
        ...

    def warm_up_period(self) -> int:
        """Maximum warm-up period across all features."""
        ...

    def __len__(self) -> int:
        """Number of features in the pipeline."""
        ...


class TargetPipeline:
    """Orchestrates multiple targets over a full dataset.

    Targets are stateless and independent — no DAG needed.
    Use ``run_research()`` to label an entire dataset at once (research only).
    """

    def __init__(
        self,
        targets: list,
        input_columns: List[str],
    ) -> None:
        """Create a new TargetPipeline.

        Args:
            targets: List of target objects (FutureReturn, FutureCTCVolatility, etc.).
            input_columns: Column names in the order passed to ``run_research()``.

        Raises:
            ValueError: If there are duplicate output keys or required input
                columns are missing from ``input_columns``.
        """
        ...

    def run_research(self, data: List[List[float]]) -> List[List[float]]:
        """Run all targets over the full dataset (research mode).

        Args:
            data: One list per input column, each containing one float per bar.
                Use ``float('nan')`` for missing values.

        Returns:
            One list per output column, in ``output_names()`` order.
        """
        ...

    def output_names(self) -> List[str]:
        """Output column names, in order."""
        ...

    def input_names(self) -> List[str]:
        """Input columns in the order expected by ``run_research()``."""
        ...

    def forward_period(self) -> int:
        """Maximum forward period across all targets."""
        ...

    def __len__(self) -> int:
        """Number of targets in the pipeline."""
        ...
