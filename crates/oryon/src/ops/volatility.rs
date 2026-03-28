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

/// Single-bar Rogers-Satchell squared term: `ln(H/C)·ln(H/O) + ln(L/C)·ln(L/O)`.
///
/// `data[0]` = high, `data[1]` = low, `data[2]` = open, `data[3]` = close.
///
/// Returns `None` if fewer than 4 values, any is `None` or non-positive, or high < low.
/// Note: the result can be negative for individual bars with unusual price action.
pub fn rogers_satchell_sq(data: &[Option<f64>]) -> Option<f64> {
    if data.len() < 4 {
        return None;
    }
    match (data[0], data[1], data[2], data[3]) {
        (Some(high), Some(low), Some(open), Some(close))
            if high > 0.0 && low > 0.0 && open > 0.0 && close > 0.0 && high >= low =>
        {
            Some((high / close).ln() * (high / open).ln() + (low / close).ln() * (low / open).ln())
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

    #[test]
    fn test_rogers_satchell_sq() {
        // H=108, L=104, O=105, C=107
        // expected = ln(108/107)·ln(108/105) + ln(104/107)·ln(104/105)
        let h = 108.0f64;
        let l = 104.0f64;
        let o = 105.0f64;
        let c = 107.0f64;
        let expected = (h / c).ln() * (h / o).ln() + (l / c).ln() * (l / o).ln();
        let result = rogers_satchell_sq(&[Some(h), Some(l), Some(o), Some(c)]);
        assert!((result.unwrap() - expected).abs() < 1e-14);
    }

    #[test]
    fn test_rogers_satchell_sq_with_none() {
        assert_eq!(
            rogers_satchell_sq(&[None, Some(104.0), Some(105.0), Some(107.0)]),
            None
        );
        assert_eq!(
            rogers_satchell_sq(&[Some(108.0), Some(104.0), None, Some(107.0)]),
            None
        );
    }

    #[test]
    fn test_rogers_satchell_sq_too_short() {
        assert_eq!(
            rogers_satchell_sq(&[Some(108.0), Some(104.0), Some(105.0)]),
            None
        );
    }

    #[test]
    fn test_rogers_satchell_sq_high_less_than_low() {
        assert_eq!(
            rogers_satchell_sq(&[Some(100.0), Some(110.0), Some(105.0), Some(107.0)]),
            None
        );
    }

    #[test]
    fn test_rogers_satchell_sq_zero_price() {
        assert_eq!(
            rogers_satchell_sq(&[Some(108.0), Some(0.0), Some(105.0), Some(107.0)]),
            None
        );
    }
}
