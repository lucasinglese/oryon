import math

import pytest
from oryon import NegLog


def test_valid_value():
    op = NegLog(inputs=["pvalue"], outputs=["neg_log_pvalue"])
    result = op.update([math.e])
    assert abs(result[0] - (-1.0)) < 1e-10


def test_nan_input():
    op = NegLog(inputs=["pvalue"], outputs=["neg_log_pvalue"])
    assert math.isnan(op.update([float("nan")])[0])


def test_zero_input():
    op = NegLog(inputs=["pvalue"], outputs=["neg_log_pvalue"])
    assert math.isnan(op.update([0.0])[0])


def test_negative_input():
    op = NegLog(inputs=["pvalue"], outputs=["neg_log_pvalue"])
    assert math.isnan(op.update([-1.0])[0])


def test_warm_up_period():
    op = NegLog(inputs=["pvalue"], outputs=["neg_log_pvalue"])
    assert op.warm_up_period() == 0


def test_input_names():
    op = NegLog(inputs=["pvalue"], outputs=["neg_log_pvalue"])
    assert op.input_names() == ["pvalue"]


def test_output_names():
    op = NegLog(inputs=["pvalue"], outputs=["neg_log_pvalue"])
    assert op.output_names() == ["neg_log_pvalue"]


def test_reset_is_noop():
    op = NegLog(inputs=["pvalue"], outputs=["neg_log_pvalue"])
    op.update([math.e])
    op.reset()
    result = op.update([math.e])
    assert abs(result[0] - (-1.0)) < 1e-10


def test_invalid_inputs():
    with pytest.raises(ValueError):
        NegLog(inputs=[], outputs=["out"])


def test_invalid_outputs():
    with pytest.raises(ValueError):
        NegLog(inputs=["pvalue"], outputs=[])
