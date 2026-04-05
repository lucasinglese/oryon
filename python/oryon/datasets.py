from importlib.resources import files

import pandas as pd


def load_sample_bars() -> pd.DataFrame:
    """Load the built-in OHLCV sample dataset.

    Returns a DataFrame with columns: open, high, low, close, volume.
    The time column is parsed as datetime and set as the index.
    Contains ~14 000 bars suitable for testing features and targets.
    """
    data = files("oryon._data").joinpath("sample_bars.csv")
    df = pd.read_csv(data, parse_dates=["time"], index_col="time")
    return df