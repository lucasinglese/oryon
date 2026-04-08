import math

import pytest
import oryon
from oryon import TargetPipeline
from oryon.targets import FutureLinearSlope

N = 20
TIME_IDX = [float(i) for i in range(N)]
PRICES = [100.0 + i * 0.5 for i in range(N)]  # perfect linear trend


def test_names():
    t = FutureLinearSlope(
        inputs=["t", "close"], horizon=5, outputs=["slope_5", "r2_5"]
    )
    assert t.input_names() == ["t", "close"]
    assert t.output_names() == ["slope_5", "r2_5"]
    assert t.forward_period() == 5


def test_compute_shape():
    t = FutureLinearSlope(
        inputs=["t", "close"], horizon=5, outputs=["slope_5", "r2_5"]
    )
    pipeline = TargetPipeline(targets=[t], input_columns=["t", "close"])
    result = pipeline.run_research([TIME_IDX, PRICES])
    assert len(result) == 2
    assert len(result[0]) == N
    assert len(result[1]) == N


def test_forward_none():
    # FutureLinearSlope(horizon=h): last h-1 bars are None.
    # Window [t, t+h) needs t+h <= N → last valid t = N-h → last h-1 bars are None.
    t = FutureLinearSlope(
        inputs=["t", "close"], horizon=5, outputs=["slope_5", "r2_5"]
    )
    pipeline = TargetPipeline(targets=[t], input_columns=["t", "close"])
    result = pipeline.run_research([TIME_IDX, PRICES])
    for col in result:
        assert math.isnan(col[-1])
        assert math.isnan(col[-2])
        assert math.isnan(col[-3])
        assert math.isnan(col[-4])
        assert not math.isnan(col[-5])  # t=N-h is valid


def test_perfect_linear_trend():
    # y = 0.5 * t → slope = 0.5, r2 = 1.0
    t = FutureLinearSlope(
        inputs=["t", "close"], horizon=5, outputs=["slope_5", "r2_5"]
    )
    pipeline = TargetPipeline(targets=[t], input_columns=["t", "close"])
    result = pipeline.run_research([TIME_IDX, PRICES])
    assert abs(result[0][0] - 0.5) < 1e-10   # slope
    assert abs(result[1][0] - 1.0) < 1e-10   # r2


def test_invalid_horizon_lt_2():
    with pytest.raises(oryon.InvalidInputError):
        FutureLinearSlope(inputs=["t", "close"], horizon=1, outputs=["slope", "r2"])


def test_invalid_inputs_lt_2():
    with pytest.raises(oryon.InvalidInputError):
        FutureLinearSlope(inputs=["close"], horizon=5, outputs=["slope", "r2"])


def test_invalid_outputs_not_2():
    with pytest.raises(oryon.InvalidInputError):
        FutureLinearSlope(inputs=["t", "close"], horizon=5, outputs=["slope"])