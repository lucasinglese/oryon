from ._oryon import FeaturePipeline, TargetPipeline
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
from .targets import FutureCTCVolatility, FutureLinearSlope, FutureReturn


def run_dataframe(pipeline: FeaturePipeline, df):
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


def run_target_dataframe(pipeline: TargetPipeline, df):
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
    result = pipeline.compute(data)
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
    # targets
    "FutureReturn",
    "FutureCTCVolatility",
    "FutureLinearSlope",
    # pipelines
    "FeaturePipeline",
    "TargetPipeline",
    # helpers
    "run_dataframe",
    "run_target_dataframe",
]
