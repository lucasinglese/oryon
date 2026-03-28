import math
import pytest
from oryon import FutureReturn, FutureCTCVolatility, TargetPipeline


PRICES = [100.0, 102.0, 105.0, 103.0, 108.0, 107.0, 110.0]


# --- FutureReturn ------------------------------------------------------------


def test_future_return_names():
    t = FutureReturn(inputs=["close"], horizon=2, outputs=["close_fr_2"])
    assert t.input_names() == ["close"]
    assert t.output_names() == ["close_fr_2"]
    assert t.forward_period() == 2


def test_future_return_invalid():
    with pytest.raises(ValueError):
        FutureReturn(inputs=[], horizon=2, outputs=["out"])
    with pytest.raises(ValueError):
        FutureReturn(inputs=["close"], horizon=0, outputs=["out"])
    with pytest.raises(ValueError):
        FutureReturn(inputs=["close"], horizon=2, outputs=[])


# --- FutureCTCVolatility -----------------------------------------------------


def test_future_ctc_vol_auto_name():
    t = FutureCTCVolatility(input="close", horizon=5)
    assert t.input_names() == ["close"]
    assert t.output_names() == ["close_future_ctc_vol_5"]
    assert t.forward_period() == 5


def test_future_ctc_vol_invalid():
    with pytest.raises(ValueError):
        FutureCTCVolatility(input="", horizon=5)
    with pytest.raises(ValueError):
        FutureCTCVolatility(input="close", horizon=0)


# --- TargetPipeline ----------------------------------------------------------


def test_target_pipeline_single():
    t = FutureReturn(inputs=["close"], horizon=2, outputs=["close_fr_2"])
    pipeline = TargetPipeline(targets=[t], input_columns=["close"])
    assert pipeline.output_names() == ["close_fr_2"]
    assert pipeline.forward_period() == 2
    assert len(pipeline) == 1


def test_target_pipeline_compute_shape():
    t = FutureReturn(inputs=["close"], horizon=2, outputs=["close_fr_2"])
    pipeline = TargetPipeline(targets=[t], input_columns=["close"])
    result = pipeline.compute([PRICES])
    assert len(result) == 1
    assert len(result[0]) == len(PRICES)


def test_target_pipeline_forward_none():
    t = FutureReturn(inputs=["close"], horizon=2, outputs=["close_fr_2"])
    pipeline = TargetPipeline(targets=[t], input_columns=["close"])
    result = pipeline.compute([PRICES])
    assert math.isnan(result[0][-1])
    assert math.isnan(result[0][-2])


def test_target_pipeline_valid_value():
    t = FutureReturn(inputs=["close"], horizon=2, outputs=["close_fr_2"])
    pipeline = TargetPipeline(targets=[t], input_columns=["close"])
    result = pipeline.compute([PRICES])
    # bar 0: (105 - 100) / 100 = 0.05
    assert abs(result[0][0] - 0.05) < 1e-10


def test_target_pipeline_multiple_targets():
    t1 = FutureReturn(inputs=["close"], horizon=2, outputs=["close_fr_2"])
    t2 = FutureCTCVolatility(input="close", horizon=3)
    pipeline = TargetPipeline(targets=[t1, t2], input_columns=["close"])
    assert len(pipeline) == 2
    assert pipeline.forward_period() == 3
    result = pipeline.compute([PRICES])
    assert len(result) == 2


def test_target_pipeline_missing_column():
    t = FutureReturn(inputs=["close"], horizon=2, outputs=["close_fr_2"])
    with pytest.raises(ValueError):
        TargetPipeline(targets=[t], input_columns=["volume"])


def test_target_pipeline_duplicate_output():
    t1 = FutureReturn(inputs=["close"], horizon=2, outputs=["same"])
    t2 = FutureReturn(inputs=["close"], horizon=3, outputs=["same"])
    with pytest.raises(ValueError):
        TargetPipeline(targets=[t1, t2], input_columns=["close"])