use oryon::checks::{is_none};
use oryon::features::{Sma, LogReturn};
use oryon::pipeline::FeaturePipeline;


fn main() {
    let prices: Vec<Option<f64>> = vec![
        Some(100.0), Some(101.5), Some(103.0), Some(102.0), Some(104.5),
        Some(106.0), Some(105.5), Some(107.0), Some(109.0), Some(108.0),
    ];

    let sma3 = Sma::new(vec!["close".into()], 3, vec!["close_3_sma".into()]).unwrap();
    let logret5 = LogReturn::new(vec!["close".into()], 5, vec!["close_log_return_5".into()]).unwrap();

    let mut feature_pipeline = FeaturePipeline::new(
        vec![Box::new(sma3), Box::new(logret5)],
        vec!["close".into()],
    ).unwrap();
    println!("{:?}", feature_pipeline.output_names());


    println!("\n=== Live mode (one bar at a time) ===");
    println!("output names: {:?}", feature_pipeline.output_names());

    for (i, price) in prices.iter().enumerate() {
        let out = feature_pipeline.update(&[*price]);
        let check_vec = out.iter().map(|x| is_none(*x)).collect::<Vec<_>>();
        println!("bar {i}: close={:.1}  →  sma_3={}  log_ret_5={} \t Verif: {}, {}", price.unwrap(), fmt(out[0]), fmt(out[1]), check_vec[0], check_vec[1]);
    }
}

fn fmt(v: Option<f64>) -> String {
    match v {
        Some(x) => format!("{x:.6}"),
        None => "None".to_string(),
    }
}