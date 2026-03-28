use criterion::{black_box, criterion_group, criterion_main, Criterion};
use oryon::checks::{is_inf, is_none, is_valid};

fn bench_is_none(c: &mut Criterion) {
    c.bench_function("is_none", |b| b.iter(|| is_none(black_box(Some(100.0)))));
}

fn bench_is_inf(c: &mut Criterion) {
    c.bench_function("is_inf", |b| b.iter(|| is_inf(black_box(100.0))));
}

fn bench_is_valid(c: &mut Criterion) {
    c.bench_function("is_valid", |b| b.iter(|| is_valid(black_box(Some(100.0)))));
}

criterion_group!(benches, bench_is_none, bench_is_inf, bench_is_valid);
criterion_main!(benches);
