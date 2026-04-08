import math

import pytest
import oryon
from oryon import Adf

# Reference series — 20 bars, verified against statsmodels.
REF_X = [
    0.0000, 0.5087, -0.1558, 0.2507, 0.8633, 0.3085, 0.5017, 0.4578,
    -0.2826, 0.1437, 0.4694, -0.0066, 0.2960, 0.6133, -0.1088, 0.3521,
    0.3786, 0.1477, 0.5707, 0.1324,
]


def make_adf(window=20, lags=0, regression="c"):
    return Adf(
        inputs=["close"],
        window=window,
        outputs=["close_adf_stat", "close_adf_pval"],
        lags=lags,
        regression=regression,
    )


def feed(adf, series):
    out = [float("nan"), float("nan")]
    for v in series:
        out = adf.update([v])
    return out


def test_warm_up():
    adf = make_adf()
    for v in REF_X[:19]:
        out = adf.update([v])
        assert math.isnan(out[0])
        assert math.isnan(out[1])


def test_valid_value():
    # statsmodels: adfuller(REF_X, regression='c', maxlag=0, autolag=None)
    #   stat = -5.656496589965162
    adf = make_adf()
    out = feed(adf, REF_X)
    assert not math.isnan(out[0])
    assert abs(out[0] - (-5.656496589965162)) < 1e-8
    assert not math.isnan(out[1])
    assert 0.0 < out[1] < 0.01


def test_valid_value_ct():
    # statsmodels: adfuller(REF_X, regression='ct', maxlag=0, autolag=None)
    #   stat = -5.465768323608186
    adf = make_adf(regression="ct")
    out = feed(adf, REF_X)
    assert abs(out[0] - (-5.465768323608186)) < 1e-8


def test_nan_input_propagates():
    adf = make_adf()
    for v in REF_X[:19]:
        adf.update([v])
    out = adf.update([float("nan")])
    assert math.isnan(out[0])
    assert math.isnan(out[1])


def test_reset():
    adf = make_adf()
    feed(adf, REF_X)
    adf.reset()
    out = adf.update([1.0])
    assert math.isnan(out[0])
    assert math.isnan(out[1])


def test_input_names():
    adf = make_adf()
    assert adf.input_names() == ["close"]


def test_output_names():
    adf = make_adf()
    assert adf.output_names() == ["close_adf_stat", "close_adf_pval"]


def test_warm_up_period():
    adf = make_adf(window=20)
    assert adf.warm_up_period() == 19


def test_schwert_rule_default():
    # lags=None → Schwert's rule; window=100 → k=12 → warm_up=99
    adf = Adf(
        inputs=["close"],
        window=100,
        outputs=["s", "p"],
    )
    assert adf.warm_up_period() == 99


def test_invalid_window():
    with pytest.raises(oryon.InvalidInputError):
        make_adf(window=0)


def test_invalid_window_too_small_for_lags():
    # lags=5 → need window > 13; window=13 should fail
    with pytest.raises(oryon.InvalidInputError):
        Adf(inputs=["close"], window=13, outputs=["s", "p"], lags=5)


def test_invalid_regression():
    with pytest.raises(oryon.InvalidConfigError):
        Adf(inputs=["close"], window=20, outputs=["s", "p"], regression="nc")


def test_invalid_inputs():
    with pytest.raises(oryon.InvalidInputError):
        Adf(inputs=[], window=20, outputs=["s", "p"])


def test_invalid_outputs_count():
    with pytest.raises(oryon.InvalidInputError):
        Adf(inputs=["close"], window=20, outputs=["only_one"])
