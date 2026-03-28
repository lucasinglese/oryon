use criterion::{black_box, criterion_group, criterion_main, Criterion};
use oryon::targets::{FutureCTCVolatility, FutureLinearSlope};
use oryon::traits::Target;

fn prices(n: usize) -> Vec<Option<f64>> {
    (0..n).map(|i| Some(100.0 + i as f64 * 0.01)).collect()
}

fn bench_future_ctc_volatility(c: &mut Criterion) {
    let p1000 = prices(1000);

    let mut group = c.benchmark_group("future_ctc_vol_compute");

    let vol_h20 = FutureCTCVolatility::new("close", 20).unwrap();
    group.bench_function("h20/1000_bars", |b| {
        b.iter(|| vol_h20.compute(black_box(&[&p1000])))
    });

    let vol_h200 = FutureCTCVolatility::new("close", 200).unwrap();
    group.bench_function("h200/1000_bars", |b| {
        b.iter(|| vol_h200.compute(black_box(&[&p1000])))
    });

    group.finish();
}

fn bench_future_linear_slope(c: &mut Criterion) {
    let x: Vec<Option<f64>> = (0..1000).map(|i| Some(i as f64)).collect();
    let p = prices(1000);

    let mut group = c.benchmark_group("future_linear_slope_compute");

    let fls_h20 = FutureLinearSlope::new(vec!["t".into(), "close".into()], 20, vec!["slope_20".into(), "r2_20".into()]).unwrap();
    group.bench_function("h20/1000_bars", |b| {
        b.iter(|| fls_h20.compute(black_box(&[&x, &p])))
    });

    let fls_h200 = FutureLinearSlope::new(vec!["t".into(), "close".into()], 200, vec!["slope_200".into(), "r2_200".into()]).unwrap();
    group.bench_function("h200/1000_bars", |b| {
        b.iter(|| fls_h200.compute(black_box(&[&x, &p])))
    });

    group.finish();
}

criterion_group!(benches, bench_future_ctc_volatility, bench_future_linear_slope);
criterion_main!(benches);
