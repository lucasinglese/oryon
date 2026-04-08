import math

import pytest
import oryon
from oryon import FixedZScore
from oryon.scalers import fit_standard_scaler


def test_valid_value():
    scaler = FixedZScore(inputs=["x"], outputs=["x_z"], mean=2.0, std=1.0)
    result = scaler.update([3.0])
    assert abs(result[0] - 1.0) < 1e-10


def test_no_warm_up():
    scaler = FixedZScore(inputs=["x"], outputs=["x_z"], mean=0.0, std=1.0)
    result = scaler.update([1.0])
    assert not math.isnan(result[0])


def test_nan_input_propagates():
    scaler = FixedZScore(inputs=["x"], outputs=["x_z"], mean=0.0, std=1.0)
    assert math.isnan(scaler.update([float("nan")])[0])


def test_reset_is_noop():
    scaler = FixedZScore(inputs=["x"], outputs=["x_z"], mean=2.0, std=1.0)
    scaler.update([3.0])
    scaler.reset()
    result = scaler.update([3.0])
    assert abs(result[0] - 1.0) < 1e-10


def test_warm_up_period():
    scaler = FixedZScore(inputs=["x"], outputs=["x_z"], mean=0.0, std=1.0)
    assert scaler.warm_up_period() == 0


def test_input_names():
    scaler = FixedZScore(inputs=["x"], outputs=["x_z"], mean=0.0, std=1.0)
    assert scaler.input_names() == ["x"]


def test_output_names():
    scaler = FixedZScore(inputs=["x"], outputs=["x_z"], mean=0.0, std=1.0)
    assert scaler.output_names() == ["x_z"]


def test_invalid_std_zero():
    with pytest.raises(oryon.InvalidInputError):
        FixedZScore(inputs=["x"], outputs=["x_z"], mean=0.0, std=0.0)


def test_invalid_inputs():
    with pytest.raises(oryon.InvalidInputError):
        FixedZScore(inputs=[], outputs=["x_z"], mean=0.0, std=1.0)


def test_fit_standard_scaler_roundtrip():
    data = [1.0, 2.0, 3.0, 4.0, 5.0]
    mean, std = fit_standard_scaler(data)
    scaler = FixedZScore(inputs=["x"], outputs=["x_z"], mean=mean, std=std)
    result = scaler.update([mean])
    assert abs(result[0] - 0.0) < 1e-10
