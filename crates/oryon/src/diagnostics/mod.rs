use crate::checks;

/// Fraction of `None` values in the column. Returns `0.0` if empty.
pub fn null_rate(col: &[Option<f64>]) -> f64 {
    if col.is_empty() {
        return 0.0;
    }
    let count = col.iter().filter(|v| checks::is_none(**v)).count();
    count as f64 / col.len() as f64
}

/// Returns `true` if any value is infinite.
pub fn has_inf(col: &[Option<f64>]) -> bool {
    col.iter()
        .any(|v| matches!(v, Some(x) if checks::is_inf(*x)))
}

/// Returns `true` if any value is `NaN`.
pub fn has_nan(col: &[Option<f64>]) -> bool {
    col.iter()
        .any(|v| matches!(v, Some(x) if x.is_nan()))
}

/// Fraction of values that are `Some` and finite.
/// Complement of `null_rate` but also excludes `Inf` and `NaN`.
pub fn valid_rate(col: &[Option<f64>]) -> f64 {
    if col.is_empty() {
        return 0.0;
    }
    let count = col.iter().filter(|v| checks::is_valid(**v)).count();
    count as f64 / col.len() as f64
}


#[cfg(test)]
mod tests {
    use super::*;

    fn col() -> Vec<Option<f64>> {
        vec![None, None, Some(1.0), Some(2.0), Some(f64::INFINITY), None]
    }

    #[test]
    fn test_null_rate() {
        // 3 None out of 6
        assert!((null_rate(&col()) - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_null_rate_empty() {
        assert_eq!(null_rate(&[]), 0.0);
    }

    #[test]
    fn test_has_inf() {
        assert!(has_inf(&col()));
        assert!(!has_inf(&[Some(1.0), Some(2.0)]));
    }

    #[test]
    fn test_has_nan() {
        assert!(has_nan(&[Some(f64::NAN)]));
        assert!(!has_nan(&[Some(1.0), None]));
    }

    #[test]
    fn test_valid_rate() {
        // Some(1.0) and Some(2.0) are valid → 2/6
        assert!((valid_rate(&col()) - 2.0 / 6.0).abs() < 1e-10);
    }
}