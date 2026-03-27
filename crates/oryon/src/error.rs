use thiserror::Error;

/// Central error type for the Oryon library.
#[derive(Debug, Error)]
pub enum OryonError {
    #[error("Duplicate output key '{key}': produced by index {idx_a} and {idx_b}")]
    DuplicateOutputKey {
        key: String,
        idx_a: usize,
        idx_b: usize,
    },

    #[error("Cyclic dependency detected among features")]
    CyclicDependency,

    #[error("Missing input column(s): {missing:?}")]
    MissingInputColumn { missing: Vec<String> },

    #[error("Invalid configuration: {msg}")]
    InvalidConfig { msg: String },

    #[error("Invalid input: {msg}")]
    InvalidInput { msg: String },
}
