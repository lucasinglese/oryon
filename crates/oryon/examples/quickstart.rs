use oryon::features::Sma;
use oryon::pipeline::FeaturePipeline;

fn main() {
    let prices: Vec<Option<f64>> = vec![
        Some(100.0), Some(101.5), Some(103.0), Some(102.0), Some(104.5),
        Some(106.0), Some(105.5), Some(107.0),
    ];

    let sma = Sma::new(vec!["close".into()], 3, vec!["close_sma_3".into()]).unwrap();
    let mut pipeline = FeaturePipeline::new(vec![Box::new(sma)], vec!["close".into()]).unwrap();

    println!("{:<5} {:<8} {}", "bar", "close", "sma_3");
    for (i, &price) in prices.iter().enumerate() {
        let out = pipeline.update(&[price]);
        println!("{:<5} {:<8.2} {}", i, price.unwrap(), fmt(out[0]));
    }
}

fn fmt(v: Option<f64>) -> String {
    v.map_or("—".into(), |x| format!("{x:.4}"))
}