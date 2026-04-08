import math

import pytest
import oryon
from oryon.features import ShannonEntropy

# Reference data: 20 bars, scipy-verified (same binning logic as Rust).
DATA = [
    1.0, 2.0, 1.5, 3.0, 2.5, 1.0, 4.0, 3.5, 2.0, 1.5,
    2.0, 3.0, 1.0, 4.0, 3.0, 2.5, 1.5, 2.0, 3.5, 4.0,
]


def test_warm_up():
    se = ShannonEntropy(inputs=["x"], window=10, outputs=["out"])
    for _ in range(9):
        assert math.isnan(se.update([1.0])[0])


def test_valid_value():
    # scipy: window=[1,2,1.5,3,2.5,1,4,3.5,2,1.5], n_bins=Sturges(10)=5
    # normalize=True (default), verified against scipy.stats.entropy
    se = ShannonEntropy(inputs=["x"], window=10, outputs=["out"])
    for v in DATA[:9]:
        se.update([v])
    result = se.update([DATA[9]])
    assert not math.isnan(result[0])
    assert 0.0 <= result[0] <= 1.0


def test_valid_value_fixed_bins_normalize_false():
    # scipy: window=[1,2,1.5,3,2.5,1,4,3.5,2,1.5], n_bins=2 -> counts=[7,3]
    # H = 0.6730116670092565
    se = ShannonEntropy(inputs=["x"], window=10, outputs=["out"], bins=2, normalize=False)
    for v in DATA[:9]:
        se.update([v])
    result = se.update([DATA[9]])
    assert abs(result[0] - 0.6730116670092565) < 1e-10


def test_valid_value_fixed_bins_normalized():
    # scipy: H_norm(bar 19, n_bins=2) = 1.0
    se = ShannonEntropy(inputs=["x"], window=10, outputs=["out"], bins=2, normalize=True)
    for v in DATA[:18]:
        se.update([v])
    result = se.update([DATA[18]])
    assert abs(result[0] - 1.0) < 1e-10


def test_nan_input_propagates():
    se = ShannonEntropy(inputs=["x"], window=10, outputs=["out"], bins=2, normalize=False)
    for v in DATA[:10]:
        se.update([v])
    assert math.isnan(se.update([float("nan")])[0])


def test_nan_stays_until_rolled_out():
    se = ShannonEntropy(inputs=["x"], window=10, outputs=["out"], bins=2, normalize=False)
    for v in DATA[:10]:
        se.update([v])
    se.update([float("nan")])
    # NaN stays in window for 9 more bars (window-1 pushes keep it inside)
    for v in DATA[11:20]:
        assert math.isnan(se.update([v])[0])
    # 10th push after NaN: NaN finally rolled out
    assert not math.isnan(se.update([2.0])[0])


def test_all_equal_values_returns_zero():
    se = ShannonEntropy(inputs=["x"], window=4, outputs=["out"], bins=2, normalize=False)
    for _ in range(5):
        result = se.update([5.0])
    assert result[0] == 0.0


def test_reset():
    se = ShannonEntropy(inputs=["x"], window=4, outputs=["out"])
    for v in [1.0, 2.0, 3.0, 4.0]:
        se.update([v])
    se.reset()
    assert math.isnan(se.update([1.0])[0])


def test_input_names():
    se = ShannonEntropy(inputs=["returns"], window=20, outputs=["entropy_20"])
    assert se.input_names() == ["returns"]


def test_output_names():
    se = ShannonEntropy(inputs=["returns"], window=20, outputs=["entropy_20"])
    assert se.output_names() == ["entropy_20"]


def test_warm_up_period():
    se = ShannonEntropy(inputs=["x"], window=20, outputs=["out"])
    assert se.warm_up_period() == 19


def test_invalid_window():
    with pytest.raises(oryon.InvalidInputError):
        ShannonEntropy(inputs=["x"], window=1, outputs=["out"])


def test_invalid_bins():
    with pytest.raises(oryon.InvalidInputError):
        ShannonEntropy(inputs=["x"], window=4, outputs=["out"], bins=1)


def test_invalid_empty_inputs():
    with pytest.raises(oryon.InvalidInputError):
        ShannonEntropy(inputs=[], window=4, outputs=["out"])
