use criterion::{black_box, criterion_group, criterion_main, Criterion};
use oryon::operators::{NegLog, Subtract};
use oryon::traits::StreamingTransform;

fn bench_subtract(c: &mut Criterion) {
    let mut op = Subtract::new(vec!["a".into(), "b".into()], vec!["a_minus_b".into()]).unwrap();
    c.bench_function("subtract_update", |b| {
        b.iter(|| op.update(black_box(&[Some(10.0), Some(3.0)])))
    });
}

fn bench_neg_log(c: &mut Criterion) {
    let mut op = NegLog::new(vec!["x".into()], vec!["neg_log_x".into()]).unwrap();
    c.bench_function("neg_log_update", |b| {
        b.iter(|| op.update(black_box(&[Some(0.05)])))
    });
}

criterion_group!(benches, bench_subtract, bench_neg_log);
criterion_main!(benches);
