/// DAG resolution for feature execution order.
pub mod dag;
/// Streaming feature pipeline (live and research mode).
pub mod feature_pipeline;
/// Batch target pipeline (research mode only).
pub mod target_pipeline;

pub use feature_pipeline::FeaturePipeline;
pub use target_pipeline::TargetPipeline;
