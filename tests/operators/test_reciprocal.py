import math

import pytest
import oryon
from oryon import Reciprocal


def test_valid_value():
    op = Reciprocal(inputs=["x"], outputs=["inv_x"])
    assert abs(op.update([2.0])[0] - 0.5) < 1e-10


def test_nan_input():
    op = Reciprocal(inputs=["x"], outputs=["inv_x"])
    assert math.isnan(op.update([float("nan")])[0])


def test_zero_input():
    op = Reciprocal(inputs=["x"], outputs=["inv_x"])
    assert math.isnan(op.update([0.0])[0])


def test_warm_up_period():
    op = Reciprocal(inputs=["x"], outputs=["inv_x"])
    assert op.warm_up_period() == 0


def test_input_names():
    op = Reciprocal(inputs=["x"], outputs=["inv_x"])
    assert op.input_names() == ["x"]


def test_output_names():
    op = Reciprocal(inputs=["x"], outputs=["inv_x"])
    assert op.output_names() == ["inv_x"]


def test_reset_is_noop():
    op = Reciprocal(inputs=["x"], outputs=["inv_x"])
    op.update([2.0])
    op.reset()
    assert abs(op.update([4.0])[0] - 0.25) < 1e-10


def test_invalid_inputs():
    with pytest.raises(oryon.InvalidInputError):
        Reciprocal(inputs=[], outputs=["inv_x"])


def test_invalid_outputs():
    with pytest.raises(oryon.InvalidInputError):
        Reciprocal(inputs=["x"], outputs=[])
