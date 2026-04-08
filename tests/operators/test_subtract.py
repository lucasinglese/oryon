import math

import pytest
import oryon
from oryon import Subtract


def test_valid_value():
    op = Subtract(inputs=["a", "b"], outputs=["spread"])
    result = op.update([10.0, 3.0])
    assert abs(result[0] - 7.0) < 1e-10


def test_nan_input_a():
    op = Subtract(inputs=["a", "b"], outputs=["spread"])
    assert math.isnan(op.update([float("nan"), 3.0])[0])


def test_nan_input_b():
    op = Subtract(inputs=["a", "b"], outputs=["spread"])
    assert math.isnan(op.update([10.0, float("nan")])[0])


def test_warm_up_period():
    op = Subtract(inputs=["a", "b"], outputs=["spread"])
    assert op.warm_up_period() == 0


def test_input_names():
    op = Subtract(inputs=["a", "b"], outputs=["spread"])
    assert op.input_names() == ["a", "b"]


def test_output_names():
    op = Subtract(inputs=["a", "b"], outputs=["spread"])
    assert op.output_names() == ["spread"]


def test_reset_is_noop():
    op = Subtract(inputs=["a", "b"], outputs=["spread"])
    op.update([10.0, 3.0])
    op.reset()
    result = op.update([5.0, 2.0])
    assert abs(result[0] - 3.0) < 1e-10


def test_invalid_inputs():
    with pytest.raises(oryon.InvalidInputError):
        Subtract(inputs=[], outputs=["spread"])


def test_invalid_outputs():
    with pytest.raises(oryon.InvalidInputError):
        Subtract(inputs=["a", "b"], outputs=[])
