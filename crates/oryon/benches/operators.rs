use criterion::{black_box, criterion_group, criterion_main, Criterion};
use oryon::operators::{Add, Divide, Log, Logit, Multiply, NegLog, Reciprocal, Subtract};
use oryon::traits::StreamingTransform;

fn bench_add(c: &mut Criterion) {
    let mut op = Add::new(vec!["a".into(), "b".into()], vec!["out".into()]).unwrap();
    c.bench_function("add_update", |b| {
        b.iter(|| op.update(black_box(&[Some(3.0), Some(2.0)])))
    });
}

fn bench_subtract(c: &mut Criterion) {
    let mut op = Subtract::new(vec!["a".into(), "b".into()], vec!["out".into()]).unwrap();
    c.bench_function("subtract_update", |b| {
        b.iter(|| op.update(black_box(&[Some(10.0), Some(3.0)])))
    });
}

fn bench_multiply(c: &mut Criterion) {
    let mut op = Multiply::new(vec!["a".into(), "b".into()], vec!["out".into()]).unwrap();
    c.bench_function("multiply_update", |b| {
        b.iter(|| op.update(black_box(&[Some(3.0), Some(2.0)])))
    });
}

fn bench_divide(c: &mut Criterion) {
    let mut op = Divide::new(vec!["a".into(), "b".into()], vec!["out".into()]).unwrap();
    c.bench_function("divide_update", |b| {
        b.iter(|| op.update(black_box(&[Some(10.0), Some(3.0)])))
    });
}

fn bench_reciprocal(c: &mut Criterion) {
    let mut op = Reciprocal::new(vec!["x".into()], vec!["out".into()]).unwrap();
    c.bench_function("reciprocal_update", |b| {
        b.iter(|| op.update(black_box(&[Some(2.0)])))
    });
}

fn bench_neg_log(c: &mut Criterion) {
    let mut op = NegLog::new(vec!["x".into()], vec!["out".into()]).unwrap();
    c.bench_function("neg_log_update", |b| {
        b.iter(|| op.update(black_box(&[Some(0.05)])))
    });
}

fn bench_log(c: &mut Criterion) {
    let mut op = Log::new(vec!["x".into()], vec!["out".into()]).unwrap();
    c.bench_function("log_update", |b| {
        b.iter(|| op.update(black_box(&[Some(std::f64::consts::E)])))
    });
}

fn bench_logit(c: &mut Criterion) {
    let mut op = Logit::new(vec!["x".into()], vec!["out".into()]).unwrap();
    c.bench_function("logit_update", |b| {
        b.iter(|| op.update(black_box(&[Some(0.7)])))
    });
}

criterion_group!(
    benches,
    bench_add,
    bench_subtract,
    bench_multiply,
    bench_divide,
    bench_reciprocal,
    bench_neg_log,
    bench_log,
    bench_logit,
);
criterion_main!(benches);
