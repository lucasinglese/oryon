import math

import pytest
import oryon
from oryon import Add


def test_valid_value():
    op = Add(inputs=["a", "b"], outputs=["sum"])
    assert abs(op.update([3.0, 2.0])[0] - 5.0) < 1e-10


def test_nan_input_a():
    op = Add(inputs=["a", "b"], outputs=["sum"])
    assert math.isnan(op.update([float("nan"), 2.0])[0])


def test_nan_input_b():
    op = Add(inputs=["a", "b"], outputs=["sum"])
    assert math.isnan(op.update([3.0, float("nan")])[0])


def test_warm_up_period():
    op = Add(inputs=["a", "b"], outputs=["sum"])
    assert op.warm_up_period() == 0


def test_input_names():
    op = Add(inputs=["a", "b"], outputs=["sum"])
    assert op.input_names() == ["a", "b"]


def test_output_names():
    op = Add(inputs=["a", "b"], outputs=["sum"])
    assert op.output_names() == ["sum"]


def test_reset_is_noop():
    op = Add(inputs=["a", "b"], outputs=["sum"])
    op.update([3.0, 2.0])
    op.reset()
    assert abs(op.update([1.0, 4.0])[0] - 5.0) < 1e-10


def test_invalid_inputs():
    with pytest.raises(oryon.InvalidInputError):
        Add(inputs=[], outputs=["sum"])


def test_invalid_outputs():
    with pytest.raises(oryon.InvalidInputError):
        Add(inputs=["a", "b"], outputs=[])
