import math

import pytest
import oryon
from oryon.features import LinearSlope


def test_warm_up():
    ls = LinearSlope(inputs=["t", "close"], window=3, outputs=["slope", "r2"])
    assert math.isnan(ls.update([0.0, 100.0])[0])
    assert math.isnan(ls.update([1.0, 101.0])[0])


def test_valid_value():
    # perfect line y = x → slope=1, r2=1
    ls = LinearSlope(inputs=["t", "close"], window=3, outputs=["slope", "r2"])
    ls.update([0.0, 0.0])
    ls.update([1.0, 1.0])
    result = ls.update([2.0, 2.0])
    assert abs(result[0] - 1.0) < 1e-10  # slope
    assert abs(result[1] - 1.0) < 1e-10  # r2


def test_two_outputs():
    ls = LinearSlope(inputs=["t", "close"], window=3, outputs=["slope", "r2"])
    for i in range(3):
        result = ls.update([float(i), float(i)])
    assert len(result) == 2


def test_reset():
    ls = LinearSlope(inputs=["t", "close"], window=3, outputs=["slope", "r2"])
    for i in range(3):
        ls.update([float(i), float(i)])
    ls.reset()
    assert math.isnan(ls.update([0.0, 100.0])[0])


def test_names():
    ls = LinearSlope(inputs=["t", "close"], window=20, outputs=["slope_20", "r2_20"])
    assert ls.input_names() == ["t", "close"]
    assert ls.output_names() == ["slope_20", "r2_20"]


def test_warm_up_period():
    ls = LinearSlope(inputs=["t", "close"], window=20, outputs=["slope", "r2"])
    assert ls.warm_up_period() == 19


def test_invalid_window_lt_2():
    with pytest.raises(oryon.InvalidInputError):
        LinearSlope(inputs=["t", "close"], window=1, outputs=["slope", "r2"])


def test_invalid_outputs_not_2():
    with pytest.raises(oryon.InvalidInputError):
        LinearSlope(inputs=["t", "close"], window=5, outputs=["slope"])


def test_invalid_inputs_lt_2():
    with pytest.raises(oryon.InvalidInputError):
        LinearSlope(inputs=["close"], window=5, outputs=["slope", "r2"])
