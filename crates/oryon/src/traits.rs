use smallvec::SmallVec;

/// Return type for Feature::update(). Stack-allocated for up to 4 outputs.
pub type Output = SmallVec<[Option<f64>; 4]>;

/// A streaming transformation that processes data one bar at a time.
///
/// Features are backward-looking: they only use past and present data.
/// They work in both live trading and research contexts.
pub trait Feature: Send + Sync {
    /// Process one bar and return the computed value(s).
    fn update(&mut self, state: &[Option<f64>]) -> Output;

    /// Reset internal state to initial values (e.g. between CPCV splits).
    fn reset(&mut self);

    /// Number of bars needed before the first valid output.
    fn warm_up_period(&self) -> usize {
        0
    }

    /// Output column name(s) produced by this feature.
    fn names(&self) -> Vec<String>;

    /// Input column name(s) required by this feature.
    fn required_columns(&self) -> Vec<String>;

    /// Create a new instance with the same config but clean state.
    fn fresh(&self) -> Box<dyn Feature>;
}

/// A batch transformation that requires future data.
///
/// Targets are forward-looking: they label past bars using future information.
/// They only work in research mode (offline), never live.
///
/// Targets are stateless — `compute()` is `&self`, no `reset()` needed.
pub trait Target: Send + Sync {
    /// Compute the target over a full series.
    ///
    /// `columns` contains one slice per entry in `required_columns()`,
    /// in the same order. Returns one `Vec<Option<f64>>` per output name.
    fn compute(&self, columns: &[&[Option<f64>]]) -> Vec<Vec<Option<f64>>>;

    /// Number of bars at the end that will be `None` (future unknown).
    fn forward_period(&self) -> usize;

    /// Number of bars at the start that will be `None`.
    fn warm_up_period(&self) -> usize {
        0
    }

    /// Output column name(s) produced by this target.
    fn names(&self) -> Vec<String>;

    /// Input column name(s) required by this target.
    fn required_columns(&self) -> Vec<String>;
}
