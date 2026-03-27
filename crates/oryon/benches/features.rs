use criterion::{black_box, criterion_group, criterion_main, Criterion};
use oryon::features::Sma;
use oryon::traits::Feature;

fn bench_sma(c: &mut Criterion) {
    let mut group = c.benchmark_group("sma_update");

    let mut sma_w20 = Sma::new(vec!["close".into()], 20, vec!["close_sma_20".into()]).unwrap();
    group.bench_function("w20", |b| {
        b.iter(|| sma_w20.update(black_box(&[Some(100.0)])))
    });

    let mut sma_w200 = Sma::new(vec!["close".into()], 200, vec!["close_sma_200".into()]).unwrap();
    group.bench_function("w200", |b| {
        b.iter(|| sma_w200.update(black_box(&[Some(100.0)])))
    });

    group.finish();
}

criterion_group!(benches, bench_sma);
criterion_main!(benches);