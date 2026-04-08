import math

import pytest
import oryon
from oryon import Multiply


def test_valid_value():
    op = Multiply(inputs=["a", "b"], outputs=["product"])
    assert abs(op.update([3.0, 2.0])[0] - 6.0) < 1e-10


def test_nan_input_a():
    op = Multiply(inputs=["a", "b"], outputs=["product"])
    assert math.isnan(op.update([float("nan"), 2.0])[0])


def test_nan_input_b():
    op = Multiply(inputs=["a", "b"], outputs=["product"])
    assert math.isnan(op.update([3.0, float("nan")])[0])


def test_warm_up_period():
    op = Multiply(inputs=["a", "b"], outputs=["product"])
    assert op.warm_up_period() == 0


def test_input_names():
    op = Multiply(inputs=["a", "b"], outputs=["product"])
    assert op.input_names() == ["a", "b"]


def test_output_names():
    op = Multiply(inputs=["a", "b"], outputs=["product"])
    assert op.output_names() == ["product"]


def test_reset_is_noop():
    op = Multiply(inputs=["a", "b"], outputs=["product"])
    op.update([3.0, 2.0])
    op.reset()
    assert abs(op.update([4.0, 5.0])[0] - 20.0) < 1e-10


def test_invalid_inputs():
    with pytest.raises(oryon.InvalidInputError):
        Multiply(inputs=[], outputs=["product"])


def test_invalid_outputs():
    with pytest.raises(oryon.InvalidInputError):
        Multiply(inputs=["a", "b"], outputs=[])
