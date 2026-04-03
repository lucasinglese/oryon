import math

import pytest
from oryon.features import Kama


def test_warm_up():
    kama = Kama(inputs=["close"], window=3, outputs=["out"])
    assert math.isnan(kama.update([100.0])[0])
    assert math.isnan(kama.update([101.0])[0])
    assert math.isnan(kama.update([102.0])[0])


def test_valid_value():
    kama = Kama(inputs=["close"], window=3, outputs=["out"])
    for price in [100.0, 101.0, 102.0]:
        kama.update([price])
    result = kama.update([103.0])
    assert not math.isnan(result[0])


def test_reset():
    kama = Kama(inputs=["close"], window=3, outputs=["out"])
    for price in [100.0, 101.0, 102.0, 103.0]:
        kama.update([price])
    kama.reset()
    assert math.isnan(kama.update([100.0])[0])


def test_names():
    kama = Kama(inputs=["close"], window=10, outputs=["close_kama_10"])
    assert kama.input_names() == ["close"]
    assert kama.output_names() == ["close_kama_10"]


def test_warm_up_period():
    kama = Kama(inputs=["close"], window=10, outputs=["out"])
    assert kama.warm_up_period() == 10


def test_default_fast_slow():
    # defaults: fast=2, slow=30 — should not raise
    kama = Kama(inputs=["close"], window=10, outputs=["out"])
    assert kama is not None


def test_custom_fast_slow():
    kama = Kama(inputs=["close"], window=10, outputs=["out"], fast=3, slow=20)
    assert kama is not None


def test_invalid_window():
    with pytest.raises(ValueError):
        Kama(inputs=["close"], window=0, outputs=["out"])


def test_invalid_slow_lte_fast():
    with pytest.raises(ValueError):
        Kama(inputs=["close"], window=10, outputs=["out"], fast=5, slow=5)