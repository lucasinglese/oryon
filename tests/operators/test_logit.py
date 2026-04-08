import math

import pytest
import oryon
from oryon import Logit


def test_valid_value_half():
    op = Logit(inputs=["x"], outputs=["logit_x"])
    # logit(0.5) = ln(1) = 0
    assert abs(op.update([0.5])[0] - 0.0) < 1e-10


def test_valid_value():
    op = Logit(inputs=["x"], outputs=["logit_x"])
    # logit(0.7) = ln(7/3)
    expected = math.log(7.0 / 3.0)
    assert abs(op.update([0.7])[0] - expected) < 1e-10


def test_nan_input():
    op = Logit(inputs=["x"], outputs=["logit_x"])
    assert math.isnan(op.update([float("nan")])[0])


def test_zero_boundary():
    op = Logit(inputs=["x"], outputs=["logit_x"])
    assert math.isnan(op.update([0.0])[0])


def test_one_boundary():
    op = Logit(inputs=["x"], outputs=["logit_x"])
    assert math.isnan(op.update([1.0])[0])


def test_out_of_domain_negative():
    op = Logit(inputs=["x"], outputs=["logit_x"])
    assert math.isnan(op.update([-0.1])[0])


def test_out_of_domain_above_one():
    op = Logit(inputs=["x"], outputs=["logit_x"])
    assert math.isnan(op.update([1.1])[0])


def test_warm_up_period():
    op = Logit(inputs=["x"], outputs=["logit_x"])
    assert op.warm_up_period() == 0


def test_input_names():
    op = Logit(inputs=["x"], outputs=["logit_x"])
    assert op.input_names() == ["x"]


def test_output_names():
    op = Logit(inputs=["x"], outputs=["logit_x"])
    assert op.output_names() == ["logit_x"]


def test_reset_is_noop():
    op = Logit(inputs=["x"], outputs=["logit_x"])
    op.update([0.7])
    op.reset()
    assert abs(op.update([0.5])[0] - 0.0) < 1e-10


def test_invalid_inputs():
    with pytest.raises(oryon.InvalidInputError):
        Logit(inputs=[], outputs=["logit_x"])


def test_invalid_outputs():
    with pytest.raises(oryon.InvalidInputError):
        Logit(inputs=["x"], outputs=[])
