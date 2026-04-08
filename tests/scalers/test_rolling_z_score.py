import math

import pytest
import oryon
from oryon import RollingZScore


def test_warm_up():
    scaler = RollingZScore(inputs=["x"], window=3, outputs=["x_z"])
    assert math.isnan(scaler.update([1.0])[0])
    assert math.isnan(scaler.update([2.0])[0])


def test_valid_value():
    scaler = RollingZScore(inputs=["x"], window=3, outputs=["x_z"])
    scaler.update([1.0])
    scaler.update([2.0])
    result = scaler.update([3.0])
    # mean=2.0, std=1.0 (sample) -> z = (3 - 2) / 1 = 1.0
    assert abs(result[0] - 1.0) < 1e-10


def test_nan_input_propagates():
    scaler = RollingZScore(inputs=["x"], window=3, outputs=["x_z"])
    scaler.update([1.0])
    scaler.update([2.0])
    assert math.isnan(scaler.update([float("nan")])[0])


def test_reset():
    scaler = RollingZScore(inputs=["x"], window=3, outputs=["x_z"])
    scaler.update([1.0])
    scaler.update([2.0])
    scaler.reset()
    assert math.isnan(scaler.update([1.0])[0])


def test_warm_up_period():
    scaler = RollingZScore(inputs=["x"], window=3, outputs=["x_z"])
    assert scaler.warm_up_period() == 2


def test_input_names():
    scaler = RollingZScore(inputs=["x"], window=3, outputs=["x_z"])
    assert scaler.input_names() == ["x"]


def test_output_names():
    scaler = RollingZScore(inputs=["x"], window=3, outputs=["x_z"])
    assert scaler.output_names() == ["x_z"]


def test_invalid_window():
    with pytest.raises(oryon.InvalidInputError):
        RollingZScore(inputs=["x"], window=0, outputs=["x_z"])


def test_invalid_inputs():
    with pytest.raises(oryon.InvalidInputError):
        RollingZScore(inputs=[], window=3, outputs=["x_z"])
