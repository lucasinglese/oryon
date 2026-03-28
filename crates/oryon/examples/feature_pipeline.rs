use oryon::features::{Ema, LogReturn, Sma};
use oryon::pipeline::FeaturePipeline;

fn main() {
    let prices: Vec<Option<f64>> = vec![
        Some(100.0),
        Some(101.5),
        Some(103.0),
        Some(102.0),
        Some(104.5),
        Some(106.0),
        Some(105.5),
        Some(107.0),
        Some(109.0),
        Some(108.0),
        Some(110.0),
        Some(111.5),
        Some(113.0),
        Some(112.0),
        Some(114.5),
    ];

    // LogReturn → Sma on returns (chained via DAG)
    let log_ret = LogReturn::new(vec!["close".into()], 1, vec!["close_lr".into()]).unwrap();
    let sma_lr = Sma::new(vec!["close_lr".into()], 5, vec!["close_lr_sma5".into()]).unwrap();
    let ema = Ema::new(vec!["close".into()], 5, vec!["close_ema5".into()]).unwrap();

    let mut pipeline = FeaturePipeline::new(
        vec![Box::new(log_ret), Box::new(sma_lr), Box::new(ema)],
        vec!["close".into()],
    )
    .unwrap();

    // research mode — full dataset at once
    let data: Vec<Vec<Option<f64>>> = prices.iter().map(|&p| vec![p]).collect();
    let matrix = pipeline.run_research(&data);

    println!(
        "{:<5} {:<8} {:<10} {:<12} ema5",
        "bar", "close", "log_ret", "lr_sma5"
    );
    for (i, (row, &price)) in matrix.iter().zip(prices.iter()).enumerate() {
        println!(
            "{:<5} {:<8.2} {:<10} {:<12} {}",
            i,
            price.unwrap(),
            fmt(row[0]),
            fmt(row[1]),
            fmt(row[2])
        );
    }

    // live mode — reset and process bar by bar
    println!("\n--- live mode (reset) ---");
    pipeline.reset();
    for (i, &price) in prices.iter().enumerate() {
        let out = pipeline.update(&[price]);
        println!(
            "bar {i}: close={:.2}  lr={}  lr_sma5={}  ema5={}",
            price.unwrap(),
            fmt(out[0]),
            fmt(out[1]),
            fmt(out[2])
        );
    }
}

fn fmt(v: Option<f64>) -> String {
    v.map_or("—".into(), |x| format!("{x:.6}"))
}
