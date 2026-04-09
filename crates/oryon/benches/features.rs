use criterion::{black_box, criterion_group, criterion_main, Criterion};
use oryon::features::{
    Adf, BinMethod, Correlation, CorrelationMethod, Ema, Kama, Kurtosis, LinearSlope, LogReturn,
    Mma, ParkinsonVolatility, RogersSatchellVolatility, ShannonEntropy, SimpleReturn, Skewness,
    Sma,
};
use oryon::ops::AdfRegression;
use oryon::traits::StreamingTransform;

fn bench_log_return(c: &mut Criterion) {
    let mut group = c.benchmark_group("log_return_update");

    let mut lr_w20 =
        LogReturn::new(vec!["close".into()], 20, vec!["close_log_return_20".into()]).unwrap();
    group.bench_function("w20", |b| {
        b.iter(|| lr_w20.update(black_box(&[Some(100.0)])))
    });

    let mut lr_w200 = LogReturn::new(
        vec!["close".into()],
        200,
        vec!["close_log_return_200".into()],
    )
    .unwrap();
    group.bench_function("w200", |b| {
        b.iter(|| lr_w200.update(black_box(&[Some(100.0)])))
    });

    group.finish();
}

fn bench_ema(c: &mut Criterion) {
    let mut group = c.benchmark_group("ema_update");

    let mut ema_w20 = Ema::new(vec!["close".into()], 20, vec!["close_ema_20".into()]).unwrap();
    group.bench_function("w20", |b| {
        b.iter(|| ema_w20.update(black_box(&[Some(100.0)])))
    });

    let mut ema_w200 = Ema::new(vec!["close".into()], 200, vec!["close_ema_200".into()]).unwrap();
    group.bench_function("w200", |b| {
        b.iter(|| ema_w200.update(black_box(&[Some(100.0)])))
    });

    group.finish();
}

fn bench_kama(c: &mut Criterion) {
    let mut group = c.benchmark_group("kama_update");

    let mut kama_w20 = Kama::new(
        vec!["close".into()],
        20,
        vec!["close_kama_20".into()],
        2,
        30,
    )
    .unwrap();
    group.bench_function("w20", |b| {
        b.iter(|| kama_w20.update(black_box(&[Some(100.0)])))
    });

    let mut kama_w200 = Kama::new(
        vec!["close".into()],
        200,
        vec!["close_kama_200".into()],
        2,
        30,
    )
    .unwrap();
    group.bench_function("w200", |b| {
        b.iter(|| kama_w200.update(black_box(&[Some(100.0)])))
    });

    group.finish();
}

fn bench_kurtosis(c: &mut Criterion) {
    let mut group = c.benchmark_group("kurtosis_update");

    let mut k_w20 =
        Kurtosis::new(vec!["close".into()], 20, vec!["close_kurtosis_20".into()]).unwrap();
    group.bench_function("w20", |b| {
        b.iter(|| k_w20.update(black_box(&[Some(100.0)])))
    });

    let mut k_w200 =
        Kurtosis::new(vec!["close".into()], 200, vec!["close_kurtosis_200".into()]).unwrap();
    group.bench_function("w200", |b| {
        b.iter(|| k_w200.update(black_box(&[Some(100.0)])))
    });

    group.finish();
}

fn bench_skewness(c: &mut Criterion) {
    let mut group = c.benchmark_group("skewness_update");

    let mut sk_w20 =
        Skewness::new(vec!["close".into()], 20, vec!["close_skewness_20".into()]).unwrap();
    group.bench_function("w20", |b| {
        b.iter(|| sk_w20.update(black_box(&[Some(100.0)])))
    });

    let mut sk_w200 =
        Skewness::new(vec!["close".into()], 200, vec!["close_skewness_200".into()]).unwrap();
    group.bench_function("w200", |b| {
        b.iter(|| sk_w200.update(black_box(&[Some(100.0)])))
    });

    group.finish();
}

fn bench_parkinson_volatility(c: &mut Criterion) {
    let mut group = c.benchmark_group("parkinson_volatility_update");

    let mut pv_w20 =
        ParkinsonVolatility::new(vec!["high".into(), "low".into()], 20, vec!["pv_20".into()])
            .unwrap();
    group.bench_function("w20", |b| {
        b.iter(|| pv_w20.update(black_box(&[Some(108.0), Some(105.0)])))
    });

    let mut pv_w200 = ParkinsonVolatility::new(
        vec!["high".into(), "low".into()],
        200,
        vec!["pv_200".into()],
    )
    .unwrap();
    group.bench_function("w200", |b| {
        b.iter(|| pv_w200.update(black_box(&[Some(108.0), Some(105.0)])))
    });

    group.finish();
}

fn bench_simple_return(c: &mut Criterion) {
    let mut group = c.benchmark_group("simple_return_update");

    let mut sr_w20 = SimpleReturn::new(
        vec!["close".into()],
        20,
        vec!["close_simple_return_20".into()],
    )
    .unwrap();
    group.bench_function("w20", |b| {
        b.iter(|| sr_w20.update(black_box(&[Some(100.0)])))
    });

    let mut sr_w200 = SimpleReturn::new(
        vec!["close".into()],
        200,
        vec!["close_simple_return_200".into()],
    )
    .unwrap();
    group.bench_function("w200", |b| {
        b.iter(|| sr_w200.update(black_box(&[Some(100.0)])))
    });

    group.finish();
}

fn bench_mma(c: &mut Criterion) {
    let mut group = c.benchmark_group("mma_update");

    let mut mma_w20 = Mma::new(vec!["close".into()], 20, vec!["close_mma_20".into()]).unwrap();
    group.bench_function("w20", |b| {
        b.iter(|| mma_w20.update(black_box(&[Some(100.0)])))
    });

    let mut mma_w200 = Mma::new(vec!["close".into()], 200, vec!["close_mma_200".into()]).unwrap();
    group.bench_function("w200", |b| {
        b.iter(|| mma_w200.update(black_box(&[Some(100.0)])))
    });

    group.finish();
}

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

fn bench_linear_slope(c: &mut Criterion) {
    let mut group = c.benchmark_group("linear_slope_update");

    let mut ls_w20 = LinearSlope::new(
        vec!["x".into(), "y".into()],
        20,
        vec!["xy_slope_20".into(), "xy_r2_20".into()],
    )
    .unwrap();
    group.bench_function("w20", |b| {
        b.iter(|| ls_w20.update(black_box(&[Some(3.0), Some(6.0)])))
    });

    let mut ls_w200 = LinearSlope::new(
        vec!["x".into(), "y".into()],
        200,
        vec!["xy_slope_200".into(), "xy_r2_200".into()],
    )
    .unwrap();
    group.bench_function("w200", |b| {
        b.iter(|| ls_w200.update(black_box(&[Some(3.0), Some(6.0)])))
    });

    group.finish();
}

fn bench_rogers_satchell_volatility(c: &mut Criterion) {
    let mut group = c.benchmark_group("rogers_satchell_volatility_update");

    let mut rs_w20 = RogersSatchellVolatility::new(
        vec!["high".into(), "low".into(), "open".into(), "close".into()],
        20,
        vec!["rs_vol_20".into()],
    )
    .unwrap();
    group.bench_function("w20", |b| {
        b.iter(|| {
            rs_w20.update(black_box(&[
                Some(108.0),
                Some(104.0),
                Some(105.0),
                Some(107.0),
            ]))
        })
    });

    let mut rs_w200 = RogersSatchellVolatility::new(
        vec!["high".into(), "low".into(), "open".into(), "close".into()],
        200,
        vec!["rs_vol_200".into()],
    )
    .unwrap();
    group.bench_function("w200", |b| {
        b.iter(|| {
            rs_w200.update(black_box(&[
                Some(108.0),
                Some(104.0),
                Some(105.0),
                Some(107.0),
            ]))
        })
    });

    group.finish();
}

fn bench_adf(c: &mut Criterion) {
    let mut group = c.benchmark_group("adf_update");

    let mut adf_w20 = Adf::new(
        vec!["close".into()],
        20,
        vec!["close_adf_stat_20".into(), "close_adf_pval_20".into()],
        Some(0),
        AdfRegression::Constant,
    )
    .unwrap();
    group.bench_function("w20", |b| {
        b.iter(|| adf_w20.update(black_box(&[Some(100.0)])))
    });

    let mut adf_w200 = Adf::new(
        vec!["close".into()],
        200,
        vec!["close_adf_stat_200".into(), "close_adf_pval_200".into()],
        Some(0),
        AdfRegression::Constant,
    )
    .unwrap();
    group.bench_function("w200", |b| {
        b.iter(|| adf_w200.update(black_box(&[Some(100.0)])))
    });

    group.finish();
}

fn bench_shannon_entropy(c: &mut Criterion) {
    // Cycling data keeps the buffer varied so the full binning + entropy path is exercised.
    // Without this, a constant input produces range=0 on every bar (shortcut path, not representative).
    let data: Vec<f64> = (0..1000).map(|i| ((i % 10) as f64) * 0.5 + 1.0).collect();

    let mut group = c.benchmark_group("shannon_entropy_update");

    let mut se_w20 = ShannonEntropy::new(
        vec!["x".into()],
        20,
        vec!["x_entropy_20".into()],
        BinMethod::FixedCount(5),
        true,
    )
    .unwrap();
    for &v in &data[..20] {
        se_w20.update(&[Some(v)]);
    }
    let mut idx_w20 = 20usize;
    group.bench_function("w20", |b| {
        b.iter(|| {
            let v = data[idx_w20 % data.len()];
            idx_w20 = idx_w20.wrapping_add(1);
            se_w20.update(black_box(&[Some(v)]))
        })
    });

    let mut se_w200 = ShannonEntropy::new(
        vec!["x".into()],
        200,
        vec!["x_entropy_200".into()],
        BinMethod::FixedCount(5),
        true,
    )
    .unwrap();
    for &v in data.iter().cycle().take(200) {
        se_w200.update(&[Some(v)]);
    }
    let mut idx_w200 = 200usize;
    group.bench_function("w200", |b| {
        b.iter(|| {
            let v = data[idx_w200 % data.len()];
            idx_w200 = idx_w200.wrapping_add(1);
            se_w200.update(black_box(&[Some(v)]))
        })
    });

    group.finish();
}

fn bench_correlation(c: &mut Criterion) {
    let mut group = c.benchmark_group("correlation_update");

    // Pearson — O(n)
    let mut pearson_w20 = Correlation::new(
        vec!["x".into(), "y".into()],
        20,
        vec!["xy_corr_20".into()],
        CorrelationMethod::Pearson,
    )
    .unwrap();
    group.bench_function("pearson/w20", |b| {
        b.iter(|| pearson_w20.update(black_box(&[Some(1.0), Some(2.0)])))
    });

    let mut pearson_w200 = Correlation::new(
        vec!["x".into(), "y".into()],
        200,
        vec!["xy_corr_200".into()],
        CorrelationMethod::Pearson,
    )
    .unwrap();
    group.bench_function("pearson/w200", |b| {
        b.iter(|| pearson_w200.update(black_box(&[Some(1.0), Some(2.0)])))
    });

    // Spearman — O(n log n)
    let mut spearman_w20 = Correlation::new(
        vec!["x".into(), "y".into()],
        20,
        vec!["xy_corr_20".into()],
        CorrelationMethod::Spearman,
    )
    .unwrap();
    group.bench_function("spearman/w20", |b| {
        b.iter(|| spearman_w20.update(black_box(&[Some(1.0), Some(2.0)])))
    });

    let mut spearman_w200 = Correlation::new(
        vec!["x".into(), "y".into()],
        200,
        vec!["xy_corr_200".into()],
        CorrelationMethod::Spearman,
    )
    .unwrap();
    group.bench_function("spearman/w200", |b| {
        b.iter(|| spearman_w200.update(black_box(&[Some(1.0), Some(2.0)])))
    });

    // Kendall — O(n^2): expect w200 to exceed the 1µs target
    let mut kendall_w20 = Correlation::new(
        vec!["x".into(), "y".into()],
        20,
        vec!["xy_corr_20".into()],
        CorrelationMethod::Kendall,
    )
    .unwrap();
    group.bench_function("kendall/w20", |b| {
        b.iter(|| kendall_w20.update(black_box(&[Some(1.0), Some(2.0)])))
    });

    let mut kendall_w200 = Correlation::new(
        vec!["x".into(), "y".into()],
        200,
        vec!["xy_corr_200".into()],
        CorrelationMethod::Kendall,
    )
    .unwrap();
    group.bench_function("kendall/w200", |b| {
        b.iter(|| kendall_w200.update(black_box(&[Some(1.0), Some(2.0)])))
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_adf,
    bench_correlation,
    bench_ema,
    bench_kama,
    bench_kurtosis,
    bench_linear_slope,
    bench_log_return,
    bench_mma,
    bench_parkinson_volatility,
    bench_rogers_satchell_volatility,
    bench_shannon_entropy,
    bench_simple_return,
    bench_skewness,
    bench_sma
);
criterion_main!(benches);
