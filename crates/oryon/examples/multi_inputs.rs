use oryon::features::{ParkinsonVolatility, Sma};
use oryon::pipeline::FeaturePipeline;

fn main() {
    let highs: Vec<Option<f64>> = vec![
        Some(102.0), Some(104.0), Some(103.0), Some(106.0), Some(108.0),
        Some(107.0), Some(109.0), Some(111.0), Some(110.0), Some(113.0),
    ];
    let lows: Vec<Option<f64>> = vec![
        Some(99.0), Some(101.0), Some(100.0), Some(103.0), Some(105.0),
        Some(104.0), Some(106.0), Some(108.0), Some(107.0), Some(110.0),
    ];

    let pv = ParkinsonVolatility::new(
        vec!["high".into(), "low".into()],
        3,
        vec!["parkinson_vol_3".into()],
    )
    .unwrap();

    let sma_high = Sma::new(vec!["high".into()], 2, vec!["high_sma_2".into()]).unwrap();

    // chained: reads the output of ParkinsonVolatility
    let sma_vol = Sma::new(vec!["parkinson_vol_3".into()], 2, vec!["parkinson_vol_3_sma_2".into()]).unwrap();

    let mut pipeline = FeaturePipeline::new(
        vec![Box::new(pv), Box::new(sma_high), Box::new(sma_vol)],
        vec!["high".into(), "low".into()],
    )
    .unwrap();

    println!("output names: {:?}", pipeline.output_names());
    println!("\n{:<5} {:<7} {:<7} {:<14} {:<14} {}", "bar", "high", "low", "pv_3", "high_sma_2", "pv_3_sma_2");

    for i in 0..highs.len() {
        let out = pipeline.update(&[highs[i], lows[i]]);
        println!(
            "{:<5} {:<7.1} {:<7.1} {:<14} {:<14} {}",
            i,
            highs[i].unwrap(),
            lows[i].unwrap(),
            fmt(out[0]),
            fmt(out[1]),
            fmt(out[2]),
        );
    }
}

fn fmt(v: Option<f64>) -> String {
    match v {
        Some(x) => format!("{x:.8}"),
        None => "None".to_string(),
    }
}
