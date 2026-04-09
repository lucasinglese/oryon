import math
import pytest
import oryon
from oryon.features import Correlation


def test_warm_up_pearson():
    c = Correlation(inputs=["x", "y"], window=3, outputs=["corr"], method="pearson")
    assert math.isnan(c.update([1.0, 1.0])[0])
    assert math.isnan(c.update([2.0, 2.0])[0])


def test_warm_up_spearman():
    c = Correlation(inputs=["x", "y"], window=3, outputs=["corr"], method="spearman")
    assert math.isnan(c.update([1.0, 1.0])[0])
    assert math.isnan(c.update([2.0, 2.0])[0])


def test_warm_up_kendall():
    c = Correlation(inputs=["x", "y"], window=3, outputs=["corr"], method="kendall")
    assert math.isnan(c.update([1.0, 1.0])[0])
    assert math.isnan(c.update([2.0, 2.0])[0])


def test_valid_pearson_perfect_positive():
    c = Correlation(inputs=["x", "y"], window=3, outputs=["corr"], method="pearson")
    c.update([1.0, 1.0])
    c.update([2.0, 2.0])
    result = c.update([3.0, 3.0])
    assert abs(result[0] - 1.0) < 1e-10


def test_valid_pearson_perfect_negative():
    c = Correlation(inputs=["x", "y"], window=3, outputs=["corr"], method="pearson")
    c.update([1.0, 3.0])
    c.update([2.0, 2.0])
    result = c.update([3.0, 1.0])
    assert abs(result[0] + 1.0) < 1e-10


def test_valid_pearson_non_trivial():
    # x=[1,2,3], y=[1,3,2]: Sxy=1, Sxx=2, Syy=2 → r = 0.5
    c = Correlation(inputs=["x", "y"], window=3, outputs=["corr"], method="pearson")
    c.update([1.0, 1.0])
    c.update([2.0, 3.0])
    result = c.update([3.0, 2.0])
    assert abs(result[0] - 0.5) < 1e-10


def test_valid_spearman_perfect_positive():
    c = Correlation(inputs=["x", "y"], window=4, outputs=["corr"], method="spearman")
    c.update([1.0, 1.0])
    c.update([2.0, 2.0])
    c.update([3.0, 3.0])
    result = c.update([4.0, 4.0])
    assert abs(result[0] - 1.0) < 1e-10


def test_valid_spearman_non_trivial():
    # x=[1,2,3,4], y=[1,3,2,4]: ranks identical, Pearson of ranks = 0.8
    c = Correlation(inputs=["x", "y"], window=4, outputs=["corr"], method="spearman")
    c.update([1.0, 1.0])
    c.update([2.0, 3.0])
    c.update([3.0, 2.0])
    result = c.update([4.0, 4.0])
    assert abs(result[0] - 0.8) < 1e-10


def test_valid_kendall_perfect_positive():
    c = Correlation(inputs=["x", "y"], window=4, outputs=["corr"], method="kendall")
    c.update([1.0, 1.0])
    c.update([2.0, 2.0])
    c.update([3.0, 3.0])
    result = c.update([4.0, 4.0])
    assert abs(result[0] - 1.0) < 1e-10


def test_valid_kendall_non_trivial():
    # x=[1,2,3,4], y=[1,3,2,4]: C=5, D=1, n0=6 → τ = 4/6 = 2/3
    c = Correlation(inputs=["x", "y"], window=4, outputs=["corr"], method="kendall")
    c.update([1.0, 1.0])
    c.update([2.0, 3.0])
    c.update([3.0, 2.0])
    result = c.update([4.0, 4.0])
    assert abs(result[0] - 2.0 / 3.0) < 1e-10


def test_nan_input_propagates():
    c = Correlation(inputs=["x", "y"], window=3, outputs=["corr"], method="pearson")
    c.update([1.0, 1.0])
    c.update([2.0, 2.0])
    c.update([3.0, 3.0])
    assert math.isnan(c.update([float("nan"), 4.0])[0])


def test_constant_series_returns_nan():
    # y constant → correlation undefined → NaN
    c = Correlation(inputs=["x", "y"], window=3, outputs=["corr"], method="pearson")
    c.update([1.0, 5.0])
    c.update([2.0, 5.0])
    assert math.isnan(c.update([3.0, 5.0])[0])


def test_reset():
    c = Correlation(inputs=["x", "y"], window=3, outputs=["corr"], method="pearson")
    c.update([1.0, 1.0])
    c.update([2.0, 2.0])
    c.update([3.0, 3.0])
    c.reset()
    assert math.isnan(c.update([1.0, 1.0])[0])


def test_input_names():
    c = Correlation(inputs=["close", "volume"], window=20, outputs=["corr_20"], method="pearson")
    assert c.input_names() == ["close", "volume"]


def test_output_names():
    c = Correlation(inputs=["x", "y"], window=20, outputs=["xy_corr_20"], method="pearson")
    assert c.output_names() == ["xy_corr_20"]


def test_warm_up_period():
    c = Correlation(inputs=["x", "y"], window=20, outputs=["corr"], method="pearson")
    assert c.warm_up_period() == 19


def test_invalid_window_zero():
    with pytest.raises(oryon.InvalidInputError):
        Correlation(inputs=["x", "y"], window=0, outputs=["corr"], method="pearson")


def test_invalid_window_one():
    with pytest.raises(oryon.InvalidInputError):
        Correlation(inputs=["x", "y"], window=1, outputs=["corr"], method="pearson")


def test_invalid_inputs_lt_2():
    with pytest.raises(oryon.InvalidInputError):
        Correlation(inputs=["x"], window=5, outputs=["corr"], method="pearson")


def test_invalid_outputs_not_1():
    with pytest.raises(oryon.InvalidInputError):
        Correlation(inputs=["x", "y"], window=5, outputs=["c1", "c2"], method="pearson")


def test_invalid_method():
    with pytest.raises(oryon.InvalidConfigError):
        Correlation(inputs=["x", "y"], window=5, outputs=["corr"], method="cosine")
