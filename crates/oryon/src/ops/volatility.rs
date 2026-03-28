/// Single-bar Parkinson log-squared: `(ln(data[0] / data[1]))^2`.
/// `data[0]` is high, `data[1]` is low.
/// Returns `None` if fewer than 2 values, any is `None`, either <= 0, or high < low.
pub fn parkinson_log_hl_sq(data: &[Option<f64>]) -> Option<f64> {
    if data.len() < 2 {
        return None;
    }
    match (data[0], data[1]) {
        (Some(high), Some(low)) if high > 0.0 && low > 0.0 && high >= low => {
            let log_hl = (high / low).ln();
            Some(log_hl * log_hl)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parkinson_log_hl_sq() {
        // Python: (ln(102/99))^2 = 0.000891199408816
        let result = parkinson_log_hl_sq(&[Some(102.0), Some(99.0)]);
        assert!((result.unwrap() - 0.000_891_199_408_816).abs() < 1e-14);
    }

    #[test]
    fn test_parkinson_log_hl_sq_equal() {
        // H == L → ln(1) = 0 → sq = 0
        let result = parkinson_log_hl_sq(&[Some(100.0), Some(100.0)]);
        assert_eq!(result, Some(0.0));
    }

    #[test]
    fn test_parkinson_log_hl_sq_with_none() {
        assert_eq!(parkinson_log_hl_sq(&[None, Some(99.0)]), None);
        assert_eq!(parkinson_log_hl_sq(&[Some(102.0), None]), None);
    }

    #[test]
    fn test_parkinson_log_hl_sq_too_short() {
        assert_eq!(parkinson_log_hl_sq(&[Some(102.0)]), None);
    }

    #[test]
    fn test_parkinson_log_hl_sq_high_less_than_low() {
        assert_eq!(parkinson_log_hl_sq(&[Some(99.0), Some(102.0)]), None);
    }

    #[test]
    fn test_parkinson_log_hl_sq_zero_price() {
        assert_eq!(parkinson_log_hl_sq(&[Some(0.0), Some(99.0)]), None);
    }
}