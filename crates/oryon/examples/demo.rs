use oryon::features::Sma;
use oryon::pipeline::{FeaturePipeline, TargetPipeline};
use oryon::targets::FutureCTCVolatility;

fn main() {
    let prices: Vec<Option<f64>> = vec![
        Some(100.0), Some(101.5), Some(103.0), Some(102.0), Some(104.5),
        Some(106.0), Some(105.5), Some(107.0), Some(109.0), Some(108.0),
    ];

    // -----------------------------------------------------------------------
    // Features
    // -----------------------------------------------------------------------

    let sma3 = Sma::new(vec!["close".into()], 3, vec!["close_sma_3".into()]).unwrap();
    let sma5 = Sma::new(vec!["close".into()], 5, vec!["close_sma_5".into()]).unwrap();

    let mut feature_pipeline = FeaturePipeline::new(
        vec![Box::new(sma3), Box::new(sma5)],
        vec!["close".into()],
    )
    .unwrap();

    println!("=== Features ===");
    println!(
        "{:<6} {:>10} {:>12} {:>12}",
        "bar", "close", "sma_3", "sma_5"
    );

    let feature_matrix = feature_pipeline.run_research(
        &prices.iter().map(|p| vec![*p]).collect::<Vec<_>>(),
    );

    for (i, (row, price)) in feature_matrix.iter().zip(prices.iter()).enumerate() {
        println!(
            "{:<6} {:>10.2} {:>12} {:>12}",
            i,
            price.unwrap(),
            fmt(row[0]),
            fmt(row[1]),
        );
    }

    // -----------------------------------------------------------------------
    // Targets
    // -----------------------------------------------------------------------

    let vol3 = FutureCTCVolatility::new("close", 3).unwrap();
    let vol5 = FutureCTCVolatility::new("close", 5).unwrap();

    let target_pipeline = TargetPipeline::new(
        vec![Box::new(vol3), Box::new(vol5)],
        vec!["close".into()],
    )
    .unwrap();

    let target_result = target_pipeline.compute(&[&prices]);

    println!("\n=== Targets ===");
    println!(
        "{:<6} {:>10} {:>20} {:>20}",
        "bar", "close", "future_ctc_vol_3", "future_ctc_vol_5"
    );

    for (i, price) in prices.iter().enumerate() {
        println!(
            "{:<6} {:>10.2} {:>20} {:>20}",
            i,
            price.unwrap(),
            fmt(target_result[0][i]),
            fmt(target_result[1][i]),
        );
    }

    // -----------------------------------------------------------------------
    // Live mode (one bar at a time)
    // -----------------------------------------------------------------------

    println!("\n=== Live mode (one bar at a time) ===");
    println!("output names: {:?}", feature_pipeline.output_names());

    feature_pipeline.reset();
    for (i, price) in prices.iter().enumerate() {
        let out = feature_pipeline.update(&[*price]);
        println!("bar {i}: close={:.1}  →  sma_3={}  sma_5={}", price.unwrap(), fmt(out[0]), fmt(out[1]));
    }
}

fn fmt(v: Option<f64>) -> String {
    match v {
        Some(x) => format!("{x:.6}"),
        None => "None".to_string(),
    }
}