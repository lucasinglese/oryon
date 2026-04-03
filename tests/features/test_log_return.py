import math

import pytest
from oryon import LogReturn


def test_warm_up():
    lr = LogReturn(inputs=["close"], window=2, outputs=["close_lr_2"])
    assert math.isnan(lr.update([1.0])[0])
    assert math.isnan(lr.update([1.0])[0])


def test_valid_value():
    import math as m

    lr = LogReturn(inputs=["close"], window=1, outputs=["close_lr_1"])
    assert math.isnan(lr.update([m.exp(1.0)])[0])
    result = lr.update([m.exp(1.1)])
    assert abs(result[0] - 0.1) < 1e-10


def test_reset():
    lr = LogReturn(inputs=["close"], window=1, outputs=["close_lr_1"])
    import math as m

    lr.update([m.exp(1.0)])
    lr.reset()
    assert math.isnan(lr.update([m.exp(1.0)])[0])


def test_names():
    lr = LogReturn(inputs=["close"], window=2, outputs=["close_lr_2"])
    assert lr.input_names() == ["close"]
    assert lr.output_names() == ["close_lr_2"]


def test_warm_up_period():
    lr = LogReturn(inputs=["close"], window=2, outputs=["close_lr_2"])
    assert lr.warm_up_period() == 2


def test_invalid_window():
    with pytest.raises(ValueError):
        LogReturn(inputs=["close"], window=0, outputs=["out"])