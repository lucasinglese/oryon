use thiserror::Error;

/// Central error type for the Oryon library.
#[derive(Debug, Error)]
pub enum OryonError {
    /// Two features or targets produce the same output key.
    #[error("Duplicate output key '{key}': produced by index {idx_a} and {idx_b}")]
    DuplicateOutputKey {
        /// The duplicate output column name.
        key: String,
        /// Index of the first producer.
        idx_a: usize,
        /// Index of the second producer.
        idx_b: usize,
    },

    /// The feature dependency graph contains a cycle.
    #[error("Cyclic dependency detected among features")]
    CyclicDependency,

    /// One or more required input columns are not provided.
    #[error("Missing input column(s): {missing:?}")]
    MissingInputColumn {
        /// Names of the missing columns.
        missing: Vec<String>,
    },

    /// Invalid configuration passed to a constructor.
    #[error("Invalid configuration: {msg}")]
    InvalidConfig {
        /// Human-readable description of the problem.
        msg: String,
    },

    /// Invalid input passed to a computation function.
    #[error("Invalid input: {msg}")]
    InvalidInput {
        /// Human-readable description of the problem.
        msg: String,
    },
}
