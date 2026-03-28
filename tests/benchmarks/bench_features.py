from oryon.features import (
    Ema,
    Kama,
    Kurtosis,
    LinearSlope,
    LogReturn,
    ParkinsonVolatility,
    RogersSatchellVolatility,
    SimpleReturn,
    Skewness,
    Sma,
)

# --- Sma ---------------------------------------------------------------------


def test_sma_update_w20(benchmark):
    sma = Sma(inputs=["close"], window=20, outputs=["out"])
    benchmark(sma.update, [100.0])


def test_sma_update_w200(benchmark):
    sma = Sma(inputs=["close"], window=200, outputs=["out"])
    benchmark(sma.update, [100.0])


# --- Ema ---------------------------------------------------------------------


def test_ema_update_w20(benchmark):
    ema = Ema(inputs=["close"], window=20, outputs=["out"])
    benchmark(ema.update, [100.0])


def test_ema_update_w200(benchmark):
    ema = Ema(inputs=["close"], window=200, outputs=["out"])
    benchmark(ema.update, [100.0])


# --- Kama --------------------------------------------------------------------


def test_kama_update_w10(benchmark):
    kama = Kama(inputs=["close"], window=10, outputs=["out"])
    benchmark(kama.update, [100.0])


def test_kama_update_w20(benchmark):
    kama = Kama(inputs=["close"], window=20, outputs=["out"])
    benchmark(kama.update, [100.0])


# --- SimpleReturn ------------------------------------------------------------


def test_simple_return_update_w1(benchmark):
    sr = SimpleReturn(inputs=["close"], window=1, outputs=["out"])
    benchmark(sr.update, [100.0])


def test_simple_return_update_w20(benchmark):
    sr = SimpleReturn(inputs=["close"], window=20, outputs=["out"])
    benchmark(sr.update, [100.0])


# --- LogReturn ---------------------------------------------------------------


def test_log_return_update_w1(benchmark):
    lr = LogReturn(inputs=["close"], window=1, outputs=["out"])
    benchmark(lr.update, [100.0])


def test_log_return_update_w20(benchmark):
    lr = LogReturn(inputs=["close"], window=20, outputs=["out"])
    benchmark(lr.update, [100.0])


# --- Skewness ----------------------------------------------------------------


def test_skewness_update_w20(benchmark):
    sk = Skewness(inputs=["close"], window=20, outputs=["out"])
    benchmark(sk.update, [100.0])


def test_skewness_update_w200(benchmark):
    sk = Skewness(inputs=["close"], window=200, outputs=["out"])
    benchmark(sk.update, [100.0])


# --- Kurtosis ----------------------------------------------------------------


def test_kurtosis_update_w20(benchmark):
    ku = Kurtosis(inputs=["close"], window=20, outputs=["out"])
    benchmark(ku.update, [100.0])


def test_kurtosis_update_w200(benchmark):
    ku = Kurtosis(inputs=["close"], window=200, outputs=["out"])
    benchmark(ku.update, [100.0])


# --- LinearSlope -------------------------------------------------------------


def test_linear_slope_update_w20(benchmark):
    ls = LinearSlope(inputs=["t", "close"], window=20, outputs=["slope", "r2"])
    benchmark(ls.update, [1.0, 100.0])


def test_linear_slope_update_w200(benchmark):
    ls = LinearSlope(inputs=["t", "close"], window=200, outputs=["slope", "r2"])
    benchmark(ls.update, [1.0, 100.0])


# --- ParkinsonVolatility -----------------------------------------------------


def test_parkinson_update_w20(benchmark):
    pv = ParkinsonVolatility(inputs=["high", "low"], window=20, outputs=["out"])
    benchmark(pv.update, [101.0, 99.0])


def test_parkinson_update_w200(benchmark):
    pv = ParkinsonVolatility(inputs=["high", "low"], window=200, outputs=["out"])
    benchmark(pv.update, [101.0, 99.0])


# --- RogersSatchellVolatility ------------------------------------------------


def test_rogers_satchell_update_w20(benchmark):
    rs = RogersSatchellVolatility(
        inputs=["high", "low", "open", "close"], window=20, outputs=["out"]
    )
    benchmark(rs.update, [101.0, 99.0, 100.0, 100.5])


def test_rogers_satchell_update_w200(benchmark):
    rs = RogersSatchellVolatility(
        inputs=["high", "low", "open", "close"], window=200, outputs=["out"]
    )
    benchmark(rs.update, [101.0, 99.0, 100.0, 100.5])
