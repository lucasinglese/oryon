import math
import pytest
from oryon.features import ParkinsonVolatility


def test_warm_up():
    pv = ParkinsonVolatility(inputs=["high", "low"], window=3, outputs=["out"])
    assert math.isnan(pv.update([101.0, 99.0])[0])
    assert math.isnan(pv.update([102.0, 98.0])[0])


def test_valid_value():
    pv = ParkinsonVolatility(inputs=["high", "low"], window=1, outputs=["out"])
    result = pv.update([110.0, 90.0])
    assert not math.isnan(result[0])
    assert result[0] > 0.0


def test_reset():
    pv = ParkinsonVolatility(inputs=["high", "low"], window=3, outputs=["out"])
    for _ in range(3):
        pv.update([101.0, 99.0])
    pv.reset()
    assert math.isnan(pv.update([101.0, 99.0])[0])


def test_names():
    pv = ParkinsonVolatility(inputs=["high", "low"], window=20, outputs=["park_vol_20"])
    assert pv.input_names() == ["high", "low"]
    assert pv.output_names() == ["park_vol_20"]


def test_warm_up_period():
    pv = ParkinsonVolatility(inputs=["high", "low"], window=20, outputs=["out"])
    assert pv.warm_up_period() == 19


def test_invalid_window():
    with pytest.raises(ValueError):
        ParkinsonVolatility(inputs=["high", "low"], window=0, outputs=["out"])


def test_invalid_inputs_lt_2():
    with pytest.raises(ValueError):
        ParkinsonVolatility(inputs=["high"], window=5, outputs=["out"])