import math
import pytest
from oryon.features import Skewness


def test_warm_up():
    sk = Skewness(inputs=["close"], window=5, outputs=["out"])
    for _ in range(4):
        assert math.isnan(sk.update([100.0])[0])


def test_valid_value():
    sk = Skewness(inputs=["close"], window=3, outputs=["out"])
    sk.update([1.0])
    sk.update([2.0])
    result = sk.update([3.0])
    assert not math.isnan(result[0])


def test_reset():
    sk = Skewness(inputs=["close"], window=3, outputs=["out"])
    for price in [1.0, 2.0, 3.0]:
        sk.update([price])
    sk.reset()
    assert math.isnan(sk.update([1.0])[0])


def test_names():
    sk = Skewness(inputs=["close"], window=20, outputs=["close_skew_20"])
    assert sk.input_names() == ["close"]
    assert sk.output_names() == ["close_skew_20"]


def test_warm_up_period():
    sk = Skewness(inputs=["close"], window=20, outputs=["out"])
    assert sk.warm_up_period() == 19


def test_invalid_window_lt_3():
    with pytest.raises(ValueError):
        Skewness(inputs=["close"], window=2, outputs=["out"])
