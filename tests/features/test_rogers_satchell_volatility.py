import math
import pytest
from oryon.features import RogersSatchellVolatility


def test_warm_up():
    rs = RogersSatchellVolatility(
        inputs=["high", "low", "open", "close"], window=3, outputs=["out"]
    )
    assert math.isnan(rs.update([101.0, 99.0, 100.0, 100.5])[0])
    assert math.isnan(rs.update([102.0, 98.0, 100.5, 101.0])[0])


def test_valid_value():
    rs = RogersSatchellVolatility(
        inputs=["high", "low", "open", "close"], window=1, outputs=["out"]
    )
    result = rs.update([110.0, 90.0, 100.0, 105.0])
    assert not math.isnan(result[0])
    assert result[0] > 0.0


def test_reset():
    rs = RogersSatchellVolatility(
        inputs=["high", "low", "open", "close"], window=3, outputs=["out"]
    )
    for _ in range(3):
        rs.update([101.0, 99.0, 100.0, 100.5])
    rs.reset()
    assert math.isnan(rs.update([101.0, 99.0, 100.0, 100.5])[0])


def test_names():
    rs = RogersSatchellVolatility(
        inputs=["high", "low", "open", "close"], window=20, outputs=["rs_vol_20"]
    )
    assert rs.input_names() == ["high", "low", "open", "close"]
    assert rs.output_names() == ["rs_vol_20"]


def test_warm_up_period():
    rs = RogersSatchellVolatility(
        inputs=["high", "low", "open", "close"], window=20, outputs=["out"]
    )
    assert rs.warm_up_period() == 19


def test_invalid_window():
    with pytest.raises(ValueError):
        RogersSatchellVolatility(
            inputs=["high", "low", "open", "close"], window=0, outputs=["out"]
        )


def test_invalid_inputs_lt_4():
    with pytest.raises(ValueError):
        RogersSatchellVolatility(
            inputs=["high", "low", "open"], window=5, outputs=["out"]
        )