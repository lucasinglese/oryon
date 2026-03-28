use criterion::{black_box, criterion_group, criterion_main, Criterion};
use oryon::diagnostics::{has_inf, has_nan, null_rate, valid_rate};

fn col(n: usize) -> Vec<Option<f64>> {
    (0..n).map(|i| Some(100.0 + i as f64 * 0.01)).collect()
}

fn bench_null_rate(c: &mut Criterion) {
    let mut group = c.benchmark_group("null_rate");
    let c1000 = col(1000);
    let c10000 = col(10000);
    group.bench_function("1000", |b| b.iter(|| null_rate(black_box(&c1000))));
    group.bench_function("10000", |b| b.iter(|| null_rate(black_box(&c10000))));
    group.finish();
}

fn bench_has_inf(c: &mut Criterion) {
    let mut group = c.benchmark_group("has_inf");
    let c1000 = col(1000);
    let c10000 = col(10000);
    group.bench_function("1000", |b| b.iter(|| has_inf(black_box(&c1000))));
    group.bench_function("10000", |b| b.iter(|| has_inf(black_box(&c10000))));
    group.finish();
}

fn bench_has_nan(c: &mut Criterion) {
    let mut group = c.benchmark_group("has_nan");
    let c1000 = col(1000);
    let c10000 = col(10000);
    group.bench_function("1000", |b| b.iter(|| has_nan(black_box(&c1000))));
    group.bench_function("10000", |b| b.iter(|| has_nan(black_box(&c10000))));
    group.finish();
}

fn bench_valid_rate(c: &mut Criterion) {
    let mut group = c.benchmark_group("valid_rate");
    let c1000 = col(1000);
    let c10000 = col(10000);
    group.bench_function("1000", |b| b.iter(|| valid_rate(black_box(&c1000))));
    group.bench_function("10000", |b| b.iter(|| valid_rate(black_box(&c10000))));
    group.finish();
}

criterion_group!(
    benches,
    bench_null_rate,
    bench_has_inf,
    bench_has_nan,
    bench_valid_rate
);
criterion_main!(benches);
