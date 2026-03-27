use oryon::diagnostics::{has_nan, valid_rate};
use oryon::features::Sma;
use oryon::pipeline::FeaturePipeline;

fn main (){
    let prices: Vec<Option<f64>> = vec![
        None, Some(f64::NAN), None, Some(102.0), Some(104.5),
        Some(106.0), Some(105.5), Some(107.0), Some(109.0), Some(108.0),
    ];

    let sma3 = Sma::new(vec!["close".into()], 3, vec!["close_3_sma".into()]).unwrap();
    let sma5 = Sma::new(vec!["close".into()], 5, vec!["close_5_sma".into()]).unwrap();
    let mut feature_pipeline = FeaturePipeline::new(
        vec![Box::new(sma3), Box::new(sma5)],
        vec!["close".into()],
    ).unwrap();
    println!("{:?}", feature_pipeline.output_names());

    println!("{:?}, {:?}", valid_rate(&prices), has_nan(&prices));

    let feature_matrix = feature_pipeline.run_research(
        &prices.iter().map(|x| vec![*x]).collect::<Vec<_>>()
    );
    println!("{:?}", feature_matrix);
}