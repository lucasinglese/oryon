import math

import pytest
from oryon import Mma


def test_warm_up():
    mma = Mma(inputs=["close"], window=3, outputs=["close_mma_3"])
    assert math.isnan(mma.update([100.0])[0])
    assert math.isnan(mma.update([101.0])[0])


def test_valid_value():
    mma = Mma(inputs=["close"], window=3, outputs=["close_mma_3"])
    mma.update([1.0])
    mma.update([3.0])
    result = mma.update([2.0])
    assert abs(result[0] - 2.0) < 1e-10


def test_sliding_window():
    mma = Mma(inputs=["close"], window=3, outputs=["close_mma_3"])
    mma.update([1.0])
    mma.update([3.0])
    mma.update([2.0])
    result = mma.update([5.0])
    assert abs(result[0] - 3.0) < 1e-10


def test_nan_input_propagates():
    mma = Mma(inputs=["close"], window=3, outputs=["close_mma_3"])
    mma.update([1.0])
    mma.update([3.0])
    mma.update([2.0])
    assert math.isnan(mma.update([float("nan")])[0])


def test_reset():
    mma = Mma(inputs=["close"], window=3, outputs=["close_mma_3"])
    mma.update([1.0])
    mma.update([3.0])
    mma.reset()
    assert math.isnan(mma.update([1.0])[0])


def test_input_names():
    mma = Mma(inputs=["close"], window=3, outputs=["close_mma_3"])
    assert mma.input_names() == ["close"]


def test_output_names():
    mma = Mma(inputs=["close"], window=3, outputs=["close_mma_3"])
    assert mma.output_names() == ["close_mma_3"]


def test_warm_up_period():
    mma = Mma(inputs=["close"], window=3, outputs=["close_mma_3"])
    assert mma.warm_up_period() == 2


def test_invalid_window():
    with pytest.raises(ValueError):
        Mma(inputs=["close"], window=0, outputs=["out"])


def test_invalid_inputs():
    with pytest.raises(ValueError):
        Mma(inputs=[], window=3, outputs=["out"])
