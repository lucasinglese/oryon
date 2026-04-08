import math

import pytest
import oryon
from oryon import Divide


def test_valid_value():
    op = Divide(inputs=["a", "b"], outputs=["ratio"])
    assert abs(op.update([10.0, 2.0])[0] - 5.0) < 1e-10


def test_nan_input_a():
    op = Divide(inputs=["a", "b"], outputs=["ratio"])
    assert math.isnan(op.update([float("nan"), 2.0])[0])


def test_nan_input_b():
    op = Divide(inputs=["a", "b"], outputs=["ratio"])
    assert math.isnan(op.update([10.0, float("nan")])[0])


def test_divide_by_zero():
    op = Divide(inputs=["a", "b"], outputs=["ratio"])
    assert math.isnan(op.update([10.0, 0.0])[0])


def test_warm_up_period():
    op = Divide(inputs=["a", "b"], outputs=["ratio"])
    assert op.warm_up_period() == 0


def test_input_names():
    op = Divide(inputs=["a", "b"], outputs=["ratio"])
    assert op.input_names() == ["a", "b"]


def test_output_names():
    op = Divide(inputs=["a", "b"], outputs=["ratio"])
    assert op.output_names() == ["ratio"]


def test_reset_is_noop():
    op = Divide(inputs=["a", "b"], outputs=["ratio"])
    op.update([10.0, 2.0])
    op.reset()
    assert abs(op.update([6.0, 3.0])[0] - 2.0) < 1e-10


def test_invalid_inputs():
    with pytest.raises(oryon.InvalidInputError):
        Divide(inputs=[], outputs=["ratio"])


def test_invalid_outputs():
    with pytest.raises(oryon.InvalidInputError):
        Divide(inputs=["a", "b"], outputs=[])
