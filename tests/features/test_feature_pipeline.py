import math

import pytest
from oryon import FeaturePipeline, Sma


def test_single_feature():
    sma = Sma(inputs=["close"], window=3, outputs=["close_sma_3"])
    pipeline = FeaturePipeline(features=[sma], input_columns=["close"])
    assert math.isnan(pipeline.update([100.0])[0])
    assert math.isnan(pipeline.update([101.0])[0])
    result = pipeline.update([102.0])
    assert abs(result[0] - 101.0) < 1e-10


def test_run_research():
    sma = Sma(inputs=["close"], window=3, outputs=["close_sma_3"])
    pipeline = FeaturePipeline(features=[sma], input_columns=["close"])
    data = [[100.0], [101.0], [102.0], [103.0]]
    result = pipeline.run_research(data)
    assert len(result) == 4
    assert math.isnan(result[0][0])
    assert math.isnan(result[1][0])
    assert abs(result[2][0] - 101.0) < 1e-10
    assert abs(result[3][0] - 102.0) < 1e-10


def test_reset():
    sma = Sma(inputs=["close"], window=3, outputs=["close_sma_3"])
    pipeline = FeaturePipeline(features=[sma], input_columns=["close"])
    pipeline.update([100.0])
    pipeline.update([101.0])
    pipeline.update([102.0])
    pipeline.reset()
    assert math.isnan(pipeline.update([100.0])[0])


def test_feature_state_is_independent():
    # modifying the original feature after pipeline creation has no effect
    sma = Sma(inputs=["close"], window=3, outputs=["close_sma_3"])
    pipeline = FeaturePipeline(features=[sma], input_columns=["close"])
    sma.update([999.0])
    sma.update([999.0])
    assert math.isnan(pipeline.update([100.0])[0])


def test_output_names():
    sma = Sma(inputs=["close"], window=3, outputs=["close_sma_3"])
    pipeline = FeaturePipeline(features=[sma], input_columns=["close"])
    assert pipeline.output_names() == ["close_sma_3"]


def test_warm_up_period():
    sma = Sma(inputs=["close"], window=3, outputs=["close_sma_3"])
    pipeline = FeaturePipeline(features=[sma], input_columns=["close"])
    assert pipeline.warm_up_period() == 2


def test_len():
    sma = Sma(inputs=["close"], window=3, outputs=["close_sma_3"])
    pipeline = FeaturePipeline(features=[sma], input_columns=["close"])
    assert len(pipeline) == 1


def test_missing_input_column():
    sma = Sma(inputs=["close"], window=3, outputs=["close_sma_3"])
    with pytest.raises(ValueError):
        FeaturePipeline(features=[sma], input_columns=["volume"])


def test_unsupported_feature_type():
    with pytest.raises((ValueError, TypeError)):
        FeaturePipeline(features=["not_a_feature"], input_columns=["close"])