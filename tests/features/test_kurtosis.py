import math
import pytest
from oryon.features import Kurtosis


def test_warm_up():
    ku = Kurtosis(inputs=["close"], window=5, outputs=["out"])
    for _ in range(4):
        assert math.isnan(ku.update([100.0])[0])


def test_valid_value():
    ku = Kurtosis(inputs=["close"], window=4, outputs=["out"])
    ku.update([1.0])
    ku.update([2.0])
    ku.update([3.0])
    result = ku.update([4.0])
    assert not math.isnan(result[0])


def test_reset():
    ku = Kurtosis(inputs=["close"], window=4, outputs=["out"])
    for price in [1.0, 2.0, 3.0, 4.0]:
        ku.update([price])
    ku.reset()
    assert math.isnan(ku.update([1.0])[0])


def test_names():
    ku = Kurtosis(inputs=["close"], window=20, outputs=["close_kurt_20"])
    assert ku.input_names() == ["close"]
    assert ku.output_names() == ["close_kurt_20"]


def test_warm_up_period():
    ku = Kurtosis(inputs=["close"], window=20, outputs=["out"])
    assert ku.warm_up_period() == 19


def test_invalid_window_lt_4():
    with pytest.raises(ValueError):
        Kurtosis(inputs=["close"], window=3, outputs=["out"])
