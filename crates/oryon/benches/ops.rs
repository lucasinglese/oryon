use criterion::{black_box, criterion_group, criterion_main, Criterion};
use oryon::ops::{average, log_return, simple_return, std_dev};

fn data(window: usize) -> Vec<Option<f64>> {
    (0..window).map(|i| Some(100.0 + i as f64 * 0.01)).collect()
}

fn bench_average(c: &mut Criterion) {
    let w20 = data(20);
    let w200 = data(200);

    let mut group = c.benchmark_group("average");
    group.bench_function("w20", |b| b.iter(|| average(black_box(&w20))));
    group.bench_function("w200", |b| b.iter(|| average(black_box(&w200))));
    group.finish();
}

fn bench_std_dev(c: &mut Criterion) {
    let w20 = data(20);
    let w200 = data(200);

    let mut group = c.benchmark_group("std_dev");
    group.bench_function("w20", |b| b.iter(|| std_dev(black_box(&w20))));
    group.bench_function("w200", |b| b.iter(|| std_dev(black_box(&w200))));
    group.finish();
}

fn bench_log_return(c: &mut Criterion) {
    let pair = vec![Some(101.0), Some(100.0)];

    c.bench_function("log_return", |b| {
        b.iter(|| log_return(black_box(&pair)))
    });
}

fn bench_simple_return(c: &mut Criterion) {
    let pair = vec![Some(100.0), Some(110.0)];

    c.bench_function("simple_return", |b| {
        b.iter(|| simple_return(black_box(&pair)))
    });
}

criterion_group!(benches, bench_average, bench_std_dev, bench_log_return, bench_simple_return);
criterion_main!(benches);