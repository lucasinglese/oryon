use oryon::features::{LinearSlope, ParkinsonVolatility, SimpleReturn};
use oryon::pipeline::FeaturePipeline;

// Inputs: high, low
// ParkinsonVolatility(high, low, w=5)  → pv5
// SimpleReturn(high, w=1)              → high_ret
// SimpleReturn(low,  w=1)              → low_ret
// LinearSlope(high_ret, low_ret, w=5)  → [hl_slope, hl_r2]  (chained)

fn main() {
    let highs: Vec<Option<f64>> = vec![
        Some(102.0),
        Some(104.0),
        Some(103.0),
        Some(106.0),
        Some(108.0),
        Some(107.0),
        Some(109.0),
        Some(111.0),
        Some(110.0),
        Some(113.0),
        Some(115.0),
        Some(114.0),
        Some(116.0),
        Some(118.0),
        Some(117.0),
    ];
    let lows: Vec<Option<f64>> = vec![
        Some(99.0),
        Some(101.0),
        Some(100.0),
        Some(103.0),
        Some(105.0),
        Some(104.0),
        Some(106.0),
        Some(108.0),
        Some(107.0),
        Some(110.0),
        Some(112.0),
        Some(111.0),
        Some(113.0),
        Some(115.0),
        Some(114.0),
    ];

    let pv =
        ParkinsonVolatility::new(vec!["high".into(), "low".into()], 5, vec!["pv5".into()]).unwrap();
    let rh = SimpleReturn::new(vec!["high".into()], 1, vec!["high_ret".into()]).unwrap();
    let rl = SimpleReturn::new(vec!["low".into()], 1, vec!["low_ret".into()]).unwrap();
    let slope = LinearSlope::new(
        vec!["high_ret".into(), "low_ret".into()],
        5,
        vec!["hl_slope".into(), "hl_r2".into()],
    )
    .unwrap();

    let mut pipeline = FeaturePipeline::new(
        vec![Box::new(pv), Box::new(rh), Box::new(rl), Box::new(slope)],
        vec!["high".into(), "low".into()],
    )
    .unwrap();

    println!(
        "{:<5} {:<8} {:<8} {:<12} {:<12} hl_r2",
        "bar", "high", "low", "pv5", "hl_slope"
    );
    for i in 0..highs.len() {
        let out = pipeline.update(&[highs[i], lows[i]]).unwrap();
        println!(
            "{:<5} {:<8.1} {:<8.1} {:<12} {:<12} {}",
            i,
            highs[i].unwrap(),
            lows[i].unwrap(),
            fmt(out[0]),
            fmt(out[2]),
            fmt(out[3]),
        );
    }
}

fn fmt(v: Option<f64>) -> String {
    v.map_or("—".into(), |x| format!("{x:.6}"))
}
