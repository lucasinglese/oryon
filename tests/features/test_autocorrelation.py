import math
import pytest
import oryon
from oryon.features import AutoCorrelation


def test_warm_up_pearson():
    c = AutoCorrelation(inputs=["x"], window=3, outputs=["autocorr"], lag=1, method="pearson")
    assert math.isnan(c.update([1.0])[0])
    assert math.isnan(c.update([2.0])[0])
    assert math.isnan(c.update([3.0])[0])


def test_warm_up_spearman():
    c = AutoCorrelation(inputs=["x"], window=3, outputs=["autocorr"], lag=1, method="spearman")
    assert math.isnan(c.update([1.0])[0])
    assert math.isnan(c.update([2.0])[0])
    assert math.isnan(c.update([3.0])[0])


def test_warm_up_kendall():
    c = AutoCorrelation(inputs=["x"], window=3, outputs=["autocorr"], lag=1, method="kendall")
    assert math.isnan(c.update([1.0])[0])
    assert math.isnan(c.update([2.0])[0])
    assert math.isnan(c.update([3.0])[0])


def test_valid_value_perfect_positive():
    # Series [1,2,3,4]: recent=[2,3,4], lagged=[1,2,3] -> r=1.0
    c = AutoCorrelation(inputs=["x"], window=3, outputs=["autocorr"], lag=1)
    c.update([1.0])
    c.update([2.0])
    c.update([3.0])
    result = c.update([4.0])
    assert abs(result[0] - 1.0) < 1e-10


def test_valid_value_perfect_negative():
    # Alternating series: recent=[-1,1,-1], lagged=[1,-1,1] -> r=-1.0
    c = AutoCorrelation(inputs=["x"], window=3, outputs=["autocorr"], lag=1)
    c.update([1.0])
    c.update([-1.0])
    c.update([1.0])
    result = c.update([-1.0])
    assert abs(result[0] + 1.0) < 1e-10


def test_valid_value_non_trivial():
    # buf=[1,3,2,4]: recent=[3,2,4], lagged=[1,3,2]
    # mean_recent=3, mean_lagged=2
    # dx=[0,-1,1], dy=[-1,1,0]  Sxy=-1, Sxx=2, Syy=2  r=-0.5
    c = AutoCorrelation(inputs=["x"], window=3, outputs=["autocorr"], lag=1)
    c.update([1.0])
    c.update([3.0])
    c.update([2.0])
    result = c.update([4.0])
    assert abs(result[0] + 0.5) < 1e-10


def test_valid_value_lag2():
    # Series [1,2,3,4,5] with lag=2: recent=[3,4,5], lagged=[1,2,3] -> r=1.0
    c = AutoCorrelation(inputs=["x"], window=3, outputs=["autocorr"], lag=2)
    c.update([1.0])
    c.update([2.0])
    c.update([3.0])
    c.update([4.0])
    result = c.update([5.0])
    assert abs(result[0] - 1.0) < 1e-10


def test_valid_value_spearman():
    # Same monotone series -> Spearman r=1.0
    c = AutoCorrelation(inputs=["x"], window=3, outputs=["autocorr"], lag=1, method="spearman")
    c.update([1.0])
    c.update([2.0])
    c.update([3.0])
    result = c.update([4.0])
    assert abs(result[0] - 1.0) < 1e-10


def test_valid_value_kendall():
    # Same monotone series -> Kendall tau=1.0
    c = AutoCorrelation(inputs=["x"], window=3, outputs=["autocorr"], lag=1, method="kendall")
    c.update([1.0])
    c.update([2.0])
    c.update([3.0])
    result = c.update([4.0])
    assert abs(result[0] - 1.0) < 1e-10


def test_nan_input_propagates():
    c = AutoCorrelation(inputs=["x"], window=3, outputs=["autocorr"], lag=1)
    c.update([1.0])
    c.update([2.0])
    c.update([3.0])
    c.update([4.0])
    assert math.isnan(c.update([float("nan")])[0])


def test_constant_series_returns_nan():
    # Both sub-windows are constant -> sigma=0 -> NaN
    c = AutoCorrelation(inputs=["x"], window=3, outputs=["autocorr"], lag=1)
    c.update([5.0])
    c.update([5.0])
    c.update([5.0])
    assert math.isnan(c.update([5.0])[0])


def test_reset():
    c = AutoCorrelation(inputs=["x"], window=3, outputs=["autocorr"], lag=1)
    c.update([1.0])
    c.update([2.0])
    c.update([3.0])
    c.update([4.0])
    c.reset()
    assert math.isnan(c.update([1.0])[0])


def test_input_names():
    c = AutoCorrelation(inputs=["close"], window=20, outputs=["autocorr_20"], lag=1)
    assert c.input_names() == ["close"]


def test_output_names():
    c = AutoCorrelation(inputs=["x"], window=20, outputs=["x_autocorr_20_1"], lag=1)
    assert c.output_names() == ["x_autocorr_20_1"]


def test_warm_up_period():
    # warm_up_period = window + lag - 1
    c = AutoCorrelation(inputs=["x"], window=20, outputs=["autocorr"], lag=3)
    assert c.warm_up_period() == 22


def test_warm_up_period_lag1():
    c = AutoCorrelation(inputs=["x"], window=20, outputs=["autocorr"], lag=1)
    assert c.warm_up_period() == 20


def test_invalid_window_zero():
    with pytest.raises(oryon.InvalidInputError):
        AutoCorrelation(inputs=["x"], window=0, outputs=["autocorr"], lag=1)


def test_invalid_window_one():
    with pytest.raises(oryon.InvalidInputError):
        AutoCorrelation(inputs=["x"], window=1, outputs=["autocorr"], lag=1)


def test_invalid_lag_zero():
    with pytest.raises(oryon.InvalidInputError):
        AutoCorrelation(inputs=["x"], window=3, outputs=["autocorr"], lag=0)


def test_invalid_inputs_not_1():
    with pytest.raises(oryon.InvalidInputError):
        AutoCorrelation(inputs=["x", "y"], window=3, outputs=["autocorr"], lag=1)


def test_invalid_outputs_not_1():
    with pytest.raises(oryon.InvalidInputError):
        AutoCorrelation(inputs=["x"], window=3, outputs=["c1", "c2"], lag=1)


def test_invalid_method():
    with pytest.raises(oryon.InvalidConfigError):
        AutoCorrelation(inputs=["x"], window=3, outputs=["autocorr"], lag=1, method="cosine")
