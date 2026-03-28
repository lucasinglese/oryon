import math
import pytest
from oryon.features import SimpleReturn


def test_warm_up():
    sr = SimpleReturn(inputs=["close"], window=2, outputs=["out"])
    assert math.isnan(sr.update([100.0])[0])
    assert math.isnan(sr.update([101.0])[0])


def test_valid_value():
    # (110 - 100) / 100 = 0.1
    sr = SimpleReturn(inputs=["close"], window=1, outputs=["out"])
    sr.update([100.0])
    result = sr.update([110.0])
    assert abs(result[0] - 0.1) < 1e-10


def test_reset():
    sr = SimpleReturn(inputs=["close"], window=1, outputs=["out"])
    sr.update([100.0])
    sr.reset()
    assert math.isnan(sr.update([100.0])[0])


def test_names():
    sr = SimpleReturn(inputs=["close"], window=5, outputs=["close_sr_5"])
    assert sr.input_names() == ["close"]
    assert sr.output_names() == ["close_sr_5"]


def test_warm_up_period():
    sr = SimpleReturn(inputs=["close"], window=5, outputs=["out"])
    assert sr.warm_up_period() == 5


def test_invalid_window():
    with pytest.raises(ValueError):
        SimpleReturn(inputs=["close"], window=0, outputs=["out"])