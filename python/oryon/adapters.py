"""DataFrame adapters for FeaturePipeline and TargetPipeline.

Provides pandas and polars helpers that handle the conversion between
DataFrame columns and the raw list-of-lists API used by the pipelines.
Neither pandas nor polars is a required dependency - an ImportError is
raised at call time if the relevant library is not installed.
"""

from __future__ import annotations

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from ._oryon import FeaturePipeline, TargetPipeline


# --- pandas ------------------------------------------------------------------


def run_features_pipeline_pandas(pipeline: FeaturePipeline, df):
    """Run a FeaturePipeline on a pandas DataFrame, preserving the index.

    Args:
        pipeline: A FeaturePipeline instance.
        df: A pandas DataFrame with at least the columns in
            ``pipeline.input_names()``.

    Returns:
        A pandas DataFrame with ``pipeline.output_names()`` as columns
        and the same index as ``df``.
    """
    import pandas as pd

    data = df[pipeline.input_names()].values.tolist()
    result = pipeline.run_research(data)
    return pd.DataFrame(result, index=df.index, columns=pipeline.output_names())


def run_targets_pipeline_pandas(pipeline: TargetPipeline, df):
    """Run a TargetPipeline on a pandas DataFrame, preserving the index.

    Args:
        pipeline: A TargetPipeline instance.
        df: A pandas DataFrame with at least the columns in
            ``pipeline.input_names()``.

    Returns:
        A pandas DataFrame with ``pipeline.output_names()`` as columns
        and the same index as ``df``.
    """
    import pandas as pd

    data = [df[col].tolist() for col in pipeline.input_names()]
    result = pipeline.run_research(data)
    return pd.DataFrame(dict(zip(pipeline.output_names(), result)), index=df.index)


# --- polars ------------------------------------------------------------------


def run_features_pipeline_polars(pipeline: FeaturePipeline, df):
    """Run a FeaturePipeline on a polars DataFrame.

    Args:
        pipeline: A FeaturePipeline instance.
        df: A polars DataFrame with at least the columns in
            ``pipeline.input_names()``.

    Returns:
        A polars DataFrame with ``pipeline.output_names()`` as columns.
    """
    import polars as pl

    data = df.select(pipeline.input_names()).rows()
    result = pipeline.run_research([list(row) for row in data])
    return pl.DataFrame(result, schema=pipeline.output_names(), orient="row")


def run_targets_pipeline_polars(pipeline: TargetPipeline, df):
    """Run a TargetPipeline on a polars DataFrame.

    Args:
        pipeline: A TargetPipeline instance.
        df: A polars DataFrame with at least the columns in
            ``pipeline.input_names()``.

    Returns:
        A polars DataFrame with ``pipeline.output_names()`` as columns.
    """
    import polars as pl

    data = [df[col].to_list() for col in pipeline.input_names()]
    result = pipeline.run_research(data)
    return pl.DataFrame(dict(zip(pipeline.output_names(), result)))


__all__ = [
    "run_features_pipeline_pandas",
    "run_targets_pipeline_pandas",
    "run_features_pipeline_polars",
    "run_targets_pipeline_polars",
]
