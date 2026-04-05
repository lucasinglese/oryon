use criterion::{black_box, criterion_group, criterion_main, Criterion};
use oryon::fitting::StandardScalerParams;
use oryon::scalers::{FixedZScore, RollingZScore};
use oryon::traits::StreamingTransform;

fn bench_rolling_zscore(c: &mut Criterion) {
    let mut group = c.benchmark_group("rolling_zscore_update");

    let mut rz_w20 =
        RollingZScore::new(vec!["x".into()], 20, vec!["x_z".into()]).unwrap();
    group.bench_function("w20", |b| {
        b.iter(|| rz_w20.update(black_box(&[Some(100.0)])))
    });

    let mut rz_w200 =
        RollingZScore::new(vec!["x".into()], 200, vec!["x_z".into()]).unwrap();
    group.bench_function("w200", |b| {
        b.iter(|| rz_w200.update(black_box(&[Some(100.0)])))
    });

    group.finish();
}

fn bench_fixed_zscore(c: &mut Criterion) {
    let params = StandardScalerParams {
        mean: 100.0,
        std: 5.0,
    };
    let mut fz =
        FixedZScore::new(vec!["x".into()], vec!["x_z".into()], params).unwrap();
    c.bench_function("fixed_zscore_update", |b| {
        b.iter(|| fz.update(black_box(&[Some(102.0)])))
    });
}

criterion_group!(benches, bench_rolling_zscore, bench_fixed_zscore);
criterion_main!(benches);