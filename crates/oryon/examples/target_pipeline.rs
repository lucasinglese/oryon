use oryon::pipeline::TargetPipeline;
use oryon::targets::{FutureCTCVolatility, FutureLinearSlope};

fn main() {
    let n = 12;
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
    ];
    // time index used as x-axis for linear slope
    let t: Vec<Option<f64>> = (0..n).map(|i| Some(i as f64)).collect();

    let vol = FutureCTCVolatility::new("close", 3).unwrap();
    let slope = FutureLinearSlope::new(
        vec!["t".into(), "close".into()],
        4,
        vec!["close_slope_4".into(), "close_r2_4".into()],
    )
    .unwrap();

    let pipeline = TargetPipeline::new(
        vec![Box::new(vol), Box::new(slope)],
        vec!["t".into(), "close".into()],
    )
    .unwrap();

    let result = pipeline.compute(&[&t, &prices]);

    println!(
        "{:<5} {:<8} {:<16} {:<14} r2_4",
        "bar", "close", "ctc_vol_3", "slope_4"
    );
    for i in 0..n {
        println!(
            "{:<5} {:<8.2} {:<16} {:<14} {}",
            i,
            prices[i].unwrap(),
            fmt(result[0][i]),
            fmt(result[1][i]),
            fmt(result[2][i]),
        );
    }
}

fn fmt(v: Option<f64>) -> String {
    v.map_or("-".into(), |x| format!("{x:.6}"))
}
