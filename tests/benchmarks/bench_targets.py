from oryon import TargetPipeline
from oryon.targets import FutureCTCVolatility, FutureLinearSlope, FutureReturn

PRICES = [float(i) for i in range(1000)]
TIME_IDX = list(range(1000))


# --- FutureReturn ------------------------------------------------------------


def test_future_return_h5(benchmark):
    t = FutureReturn(inputs=["close"], horizon=5, outputs=["out"])
    pipeline = TargetPipeline(targets=[t], input_columns=["close"])
    benchmark(pipeline.compute, [PRICES])


def test_future_return_h20(benchmark):
    t = FutureReturn(inputs=["close"], horizon=20, outputs=["out"])
    pipeline = TargetPipeline(targets=[t], input_columns=["close"])
    benchmark(pipeline.compute, [PRICES])


# --- FutureCTCVolatility -----------------------------------------------------


def test_ctc_vol_h5(benchmark):
    t = FutureCTCVolatility(input="close", horizon=5)
    pipeline = TargetPipeline(targets=[t], input_columns=["close"])
    benchmark(pipeline.compute, [PRICES])


def test_ctc_vol_h20(benchmark):
    t = FutureCTCVolatility(input="close", horizon=20)
    pipeline = TargetPipeline(targets=[t], input_columns=["close"])
    benchmark(pipeline.compute, [PRICES])


# --- FutureLinearSlope -------------------------------------------------------


def test_future_linear_slope_h5(benchmark):
    t = FutureLinearSlope(
        inputs=["t", "close"], horizon=5, outputs=["slope", "r2"]
    )
    pipeline = TargetPipeline(targets=[t], input_columns=["t", "close"])
    benchmark(pipeline.compute, [TIME_IDX, PRICES])


def test_future_linear_slope_h20(benchmark):
    t = FutureLinearSlope(
        inputs=["t", "close"], horizon=20, outputs=["slope", "r2"]
    )
    pipeline = TargetPipeline(targets=[t], input_columns=["t", "close"])
    benchmark(pipeline.compute, [TIME_IDX, PRICES])
