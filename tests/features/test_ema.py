import math

import pytest
from oryon import Ema


def test_warm_up():
    ema = Ema(inputs=["close"], window=3, outputs=["close_ema_3"])
    assert math.isnan(ema.update([100.0])[0])
    assert math.isnan(ema.update([200.0])[0])


def test_valid_value():
    # alpha = 2/(3+1) = 0.5; seed = SMA([100,200,300]) = 200
    ema = Ema(inputs=["close"], window=3, outputs=["close_ema_3"])
    ema.update([100.0])
    ema.update([200.0])
    result = ema.update([300.0])
    assert abs(result[0] - 200.0) < 1e-10
    # 0.5*400 + 0.5*200 = 300
    assert abs(ema.update([400.0])[0] - 300.0) < 1e-10


def test_nan_input_resets():
    ema = Ema(inputs=["close"], window=3, outputs=["close_ema_3"])
    ema.update([100.0])
    ema.update([200.0])
    ema.update([300.0])  # seeded
    assert math.isnan(ema.update([float("nan")])[0])
    # must re-seed from scratch
    assert math.isnan(ema.update([100.0])[0])


def test_reset():
    ema = Ema(inputs=["close"], window=3, outputs=["close_ema_3"])
    ema.update([100.0])
    ema.update([200.0])
    ema.update([300.0])
    ema.reset()
    assert math.isnan(ema.update([100.0])[0])


def test_names():
    ema = Ema(inputs=["close"], window=3, outputs=["close_ema_3"])
    assert ema.input_names() == ["close"]
    assert ema.output_names() == ["close_ema_3"]


def test_warm_up_period():
    ema = Ema(inputs=["close"], window=3, outputs=["close_ema_3"])
    assert ema.warm_up_period() == 2


def test_invalid_window():
    with pytest.raises(ValueError):
        Ema(inputs=["close"], window=0, outputs=["out"])