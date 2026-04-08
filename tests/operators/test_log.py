import math

import pytest
import oryon
from oryon import Log


def test_valid_value():
    op = Log(inputs=["x"], outputs=["ln_x"])
    assert abs(op.update([1.0])[0] - 0.0) < 1e-10


def test_valid_value_e():
    op = Log(inputs=["x"], outputs=["ln_x"])
    assert abs(op.update([math.e])[0] - 1.0) < 1e-10


def test_nan_input():
    op = Log(inputs=["x"], outputs=["ln_x"])
    assert math.isnan(op.update([float("nan")])[0])


def test_zero_input():
    op = Log(inputs=["x"], outputs=["ln_x"])
    assert math.isnan(op.update([0.0])[0])


def test_negative_input():
    op = Log(inputs=["x"], outputs=["ln_x"])
    assert math.isnan(op.update([-1.0])[0])


def test_warm_up_period():
    op = Log(inputs=["x"], outputs=["ln_x"])
    assert op.warm_up_period() == 0


def test_input_names():
    op = Log(inputs=["x"], outputs=["ln_x"])
    assert op.input_names() == ["x"]


def test_output_names():
    op = Log(inputs=["x"], outputs=["ln_x"])
    assert op.output_names() == ["ln_x"]


def test_reset_is_noop():
    op = Log(inputs=["x"], outputs=["ln_x"])
    op.update([math.e])
    op.reset()
    assert abs(op.update([1.0])[0] - 0.0) < 1e-10


def test_invalid_inputs():
    with pytest.raises(oryon.InvalidInputError):
        Log(inputs=[], outputs=["ln_x"])


def test_invalid_outputs():
    with pytest.raises(oryon.InvalidInputError):
        Log(inputs=["x"], outputs=[])
