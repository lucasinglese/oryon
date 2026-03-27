use smallvec::SmallVec;

/// Return type for Feature::update(). Stack-allocated for up to 4 outputs.
pub type Output = SmallVec<[Option<f64>; 4]>;

/// A streaming transformation that processes data one bar at a time.
///
/// Features are backward-looking: they only use past and present data.
/// They work in both live trading and research contexts.
pub trait Feature: Send + Sync {
    /// Input name(s) this feature reads from the bar state.
    fn input_names(&self) -> Vec<String>;

    /// Output name(s) produced by this feature.
    fn output_names(&self) -> Vec<String>;

    /// Number of bars needed before the first valid output.
    fn warm_up_period(&self) -> usize {
        0
    }

    /// Create a new instance with the same config but clean state.
    fn fresh(&self) -> Box<dyn Feature>;

    /// Reset internal state to initial values (e.g. between CPCV splits).
    fn reset(&mut self);

    /// Process one bar and return the computed value(s).
    fn update(&mut self, state: &[Option<f64>]) -> Output;
}
/// A batch transformation that requires future data.
///
/// Targets are forward-looking: they label past bars using future information.
/// They only work in research mode (offline), never live.
///
/// Targets are stateless — `compute()` is `&self`, no `reset()` needed.
pub trait Target: Send + Sync {
    /// Input name(s) this target reads from the dataset.
    fn input_names(&self) -> Vec<String>;

    /// Output name(s) produced by this target.
    fn output_names(&self) -> Vec<String>;

    /// Number of bars at the end that will be `None` (future unknown).
    fn forward_period(&self) -> usize;

    /// Number of bars at the start that will be `None`.
    fn warm_up_period(&self) -> usize {
        0
    }

    /// Compute the target over a full series.
    ///
    /// `columns` contains one slice per entry in `input_names()`,
    /// in the same order. Returns one `Vec<Option<f64>>` per output name.
    fn compute(&self, columns: &[&[Option<f64>]]) -> Vec<Vec<Option<f64>>>;
}
