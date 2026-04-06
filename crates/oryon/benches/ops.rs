use criterion::{black_box, criterion_group, criterion_main, Criterion};
use oryon::ops::{
    adf_pvalue, adf_stat, average, kurtosis, linear_slope, log_return, median, parkinson_log_hl_sq,
    rogers_satchell_sq, simple_return, skewness, std_dev, AdfRegression,
};

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

fn bench_median(c: &mut Criterion) {
    let w20 = data(20);
    let w200 = data(200);

    let mut group = c.benchmark_group("median");
    group.bench_function("w20", |b| b.iter(|| median(black_box(&w20))));
    group.bench_function("w200", |b| b.iter(|| median(black_box(&w200))));
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

    c.bench_function("log_return", |b| b.iter(|| log_return(black_box(&pair))));
}

fn bench_simple_return(c: &mut Criterion) {
    let pair = vec![Some(100.0), Some(110.0)];

    c.bench_function("simple_return", |b| {
        b.iter(|| simple_return(black_box(&pair)))
    });
}

fn bench_skewness(c: &mut Criterion) {
    let w20 = data(20);
    let w200 = data(200);

    let mut group = c.benchmark_group("skewness");
    group.bench_function("w20", |b| b.iter(|| skewness(black_box(&w20))));
    group.bench_function("w200", |b| b.iter(|| skewness(black_box(&w200))));
    group.finish();
}

fn bench_kurtosis(c: &mut Criterion) {
    let w20 = data(20);
    let w200 = data(200);

    let mut group = c.benchmark_group("kurtosis");
    group.bench_function("w20", |b| b.iter(|| kurtosis(black_box(&w20))));
    group.bench_function("w200", |b| b.iter(|| kurtosis(black_box(&w200))));
    group.finish();
}

fn bench_parkinson_log_hl_sq(c: &mut Criterion) {
    let pair = vec![Some(108.0), Some(105.0)];

    c.bench_function("parkinson_log_hl_sq", |b| {
        b.iter(|| parkinson_log_hl_sq(black_box(&pair)))
    });
}

fn bench_linear_slope(c: &mut Criterion) {
    let x20 = data(20);
    let y20 = data(20);
    let x200 = data(200);
    let y200 = data(200);

    let mut group = c.benchmark_group("linear_slope");
    group.bench_function("w20", |b| {
        b.iter(|| linear_slope(black_box(&x20), black_box(&y20)))
    });
    group.bench_function("w200", |b| {
        b.iter(|| linear_slope(black_box(&x200), black_box(&y200)))
    });
    group.finish();
}

fn bench_rogers_satchell_sq(c: &mut Criterion) {
    let bar = vec![Some(108.0), Some(104.0), Some(105.0), Some(107.0)];

    c.bench_function("rogers_satchell_sq", |b| {
        b.iter(|| rogers_satchell_sq(black_box(&bar)))
    });
}

fn adf_data(window: usize) -> Vec<Option<f64>> {
    // Sinusoidal series: avoids the degenerate (constant/linear) case that returns None.
    (0..window)
        .map(|i| Some(100.0 + (i as f64 * 1.7).sin() * 5.0))
        .collect()
}

fn bench_adf_stat(c: &mut Criterion) {
    let w20 = adf_data(20);
    let w200 = adf_data(200);

    let mut group = c.benchmark_group("adf_stat");
    group.bench_function("w20", |b| {
        b.iter(|| adf_stat(black_box(&w20), 0, AdfRegression::Constant))
    });
    group.bench_function("w200", |b| {
        b.iter(|| adf_stat(black_box(&w200), 0, AdfRegression::Constant))
    });
    group.finish();
}

fn bench_adf_pvalue(c: &mut Criterion) {
    c.bench_function("adf_pvalue", |b| {
        b.iter(|| adf_pvalue(black_box(-2.5), AdfRegression::Constant))
    });
}

criterion_group!(
    benches,
    bench_adf_stat,
    bench_adf_pvalue,
    bench_average,
    bench_median,
    bench_std_dev,
    bench_log_return,
    bench_parkinson_log_hl_sq,
    bench_rogers_satchell_sq,
    bench_simple_return,
    bench_skewness,
    bench_kurtosis,
    bench_linear_slope
);
criterion_main!(benches);
