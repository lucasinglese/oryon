use criterion::{black_box, criterion_group, criterion_main, Criterion};
use oryon::targets::FutureCTCVolatility;
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

criterion_group!(benches, bench_future_ctc_volatility);
criterion_main!(benches);
