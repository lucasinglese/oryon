/// Returns `true` if the value is `None`.
pub fn is_none(v: Option<f64>) -> bool {
    v.is_none()
}

/// Returns `true` if the value is infinite (positive or negative).
pub fn is_inf(v: f64) -> bool {
    v.is_infinite()
}

/// Returns `true` if the value is `Some` and finite.
pub fn is_valid(v: Option<f64>) -> bool {
    matches!(v, Some(x) if x.is_finite())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_none() {
        assert!(is_none(None));
        assert!(!is_none(Some(1.0)));
    }

    #[test]
    fn test_is_inf() {
        assert!(is_inf(f64::INFINITY));
        assert!(is_inf(f64::NEG_INFINITY));
        assert!(!is_inf(1.0));
    }

    #[test]
    fn test_is_valid() {
        assert!(is_valid(Some(1.0)));
        assert!(!is_valid(None));
        assert!(!is_valid(Some(f64::INFINITY)));
        assert!(!is_valid(Some(f64::NAN)));
    }
}
