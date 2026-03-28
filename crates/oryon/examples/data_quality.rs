use oryon::checks::is_valid;
use oryon::diagnostics::{has_nan, null_rate, valid_rate};
use oryon::features::Sma;
use oryon::pipeline::FeaturePipeline;

fn main() {
    let prices: Vec<Option<f64>> = vec![
        None, Some(101.5), Some(f64::NAN), Some(102.0), Some(104.5),
        Some(106.0), None, Some(107.0), Some(109.0), Some(108.0),
    ];

    // pre-flight diagnostics on the raw column
    println!("null_rate:  {:.0}%", null_rate(&prices) * 100.0);
    println!("valid_rate: {:.0}%", valid_rate(&prices) * 100.0);
    println!("has_nan:    {}\n", has_nan(&prices));

    let sma = Sma::new(vec!["close".into()], 3, vec!["close_sma_3".into()]).unwrap();
    let mut pipeline = FeaturePipeline::new(vec![Box::new(sma)], vec!["close".into()]).unwrap();

    // per-bar check on feature output
    println!("{:<5} {:<10} {:<10} {}", "bar", "close", "sma_3", "valid");
    for (i, &price) in prices.iter().enumerate() {
        let out = pipeline.update(&[price]);
        println!("{:<5} {:<10} {:<10} {}", i, fmt(price), fmt(out[0]), is_valid(out[0]));
    }
}

fn fmt(v: Option<f64>) -> String {
    v.map_or("—".into(), |x| {
        if x.is_nan() { "NaN".into() } else { format!("{x:.4}") }
    })
}