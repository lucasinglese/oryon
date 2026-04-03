import math

import pytest
from oryon import Sma


def test_warm_up():
    sma = Sma(inputs=["close"], window=3, outputs=["close_sma_3"])
    assert math.isnan(sma.update([100.0])[0])
    assert math.isnan(sma.update([101.0])[0])


def test_valid_value():
    sma = Sma(inputs=["close"], window=3, outputs=["close_sma_3"])
    sma.update([100.0])
    sma.update([101.0])
    result = sma.update([102.0])
    assert abs(result[0] - 101.0) < 1e-10


def test_nan_input_propagates():
    sma = Sma(inputs=["close"], window=3, outputs=["close_sma_3"])
    sma.update([100.0])
    sma.update([101.0])
    assert math.isnan(sma.update([float("nan")])[0])


def test_reset():
    sma = Sma(inputs=["close"], window=3, outputs=["close_sma_3"])
    sma.update([100.0])
    sma.update([101.0])
    sma.reset()
    assert math.isnan(sma.update([100.0])[0])


def test_input_names():
    sma = Sma(inputs=["close"], window=3, outputs=["close_sma_3"])
    assert sma.input_names() == ["close"]


def test_output_names():
    sma = Sma(inputs=["close"], window=3, outputs=["close_sma_3"])
    assert sma.output_names() == ["close_sma_3"]


def test_warm_up_period():
    sma = Sma(inputs=["close"], window=3, outputs=["close_sma_3"])
    assert sma.warm_up_period() == 2


def test_invalid_window():
    with pytest.raises(ValueError):
        Sma(inputs=["close"], window=0, outputs=["out"])


def test_invalid_inputs():
    with pytest.raises(ValueError):
        Sma(inputs=[], window=3, outputs=["out"])