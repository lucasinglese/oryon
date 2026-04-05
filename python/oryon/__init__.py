from __future__ import annotations

from typing import TYPE_CHECKING

from ._oryon import FeaturePipeline, TargetPipeline
from .datasets import load_sample_bars
from .features import (
    Ema,
    Kama,
    Kurtosis,
    LinearSlope,
    LogReturn,
    ParkinsonVolatility,
    RogersSatchellVolatility,
    SimpleReturn,
    Skewness,
    Sma,
)
from .operators import NegLog, Subtract
from .scalers import FixedZScore, RollingZScore, fit_standard_scaler
from .targets import FutureCTCVolatility, FutureLinearSlope, FutureReturn

if TYPE_CHECKING:
    from pandas import DataFrame


def run_features_pipeline(pipeline: FeaturePipeline, df: DataFrame) -> DataFrame:
    """Run a FeaturePipeline on a pandas DataFrame, preserving the index.

    Args:
        pipeline: A FeaturePipeline instance.
        df: A pandas DataFrame with at least the columns in ``pipeline.input_names()``.

    Returns:
        A pandas DataFrame with ``pipeline.output_names()`` as columns
        and the same index as ``df``.
    """
    import pandas as pd

    data = df[pipeline.input_names()].values.tolist()
    result = pipeline.run_research(data)
    return pd.DataFrame(result, index=df.index, columns=pipeline.output_names())


def run_targets_pipeline(pipeline: TargetPipeline, df: DataFrame) -> DataFrame:
    """Run a TargetPipeline on a pandas DataFrame, preserving the index.

    Args:
        pipeline: A TargetPipeline instance.
        df: A pandas DataFrame with at least the columns in ``pipeline.input_names()``.

    Returns:
        A pandas DataFrame with ``pipeline.output_names()`` as columns
        and the same index as ``df``.
    """
    import pandas as pd

    data = [df[col].tolist() for col in pipeline.input_names()]
    result = pipeline.run_research(data)
    return pd.DataFrame(
        dict(zip(pipeline.output_names(), result)),
        index=df.index,
    )


__all__ = [
    # features
    "Sma",
    "Ema",
    "Kama",
    "SimpleReturn",
    "LogReturn",
    "Skewness",
    "Kurtosis",
    "LinearSlope",
    "ParkinsonVolatility",
    "RogersSatchellVolatility",
    # operators
    "Subtract",
    "NegLog",
    # scalers
    "RollingZScore",
    "FixedZScore",
    "fit_standard_scaler",
    # targets
    "FutureReturn",
    "FutureCTCVolatility",
    "FutureLinearSlope",
    # pipelines
    "FeaturePipeline",
    "TargetPipeline",
    # helpers
    "run_features_pipeline",
    "run_targets_pipeline",
    # datasets
    "load_sample_bars",
]
