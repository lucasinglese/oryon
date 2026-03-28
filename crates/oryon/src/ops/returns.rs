/// Log return: `ln(data[1] / data[0])`.
/// `data[0]` is previous, `data[1]` is current.
/// Returns `None` if fewer than 2 values, any is `None`, or denominator <= 0.
pub fn log_return(data: &[Option<f64>]) -> Option<f64> {
    if data.len() < 2 {
        return None;
    }
    match (data[0], data[1]) {
        (Some(previous), Some(current)) if previous > 0.0 => Some((current / previous).ln()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_return() {
        let result = log_return(&[Some(100.0), Some(110.0)]);
        assert!((result.unwrap() - (1.1_f64).ln()).abs() < 1e-10);
    }

    #[test]
    fn test_log_return_with_none() {
        assert_eq!(log_return(&[None, Some(100.0)]), None);
    }

    #[test]
    fn test_log_return_too_short() {
        assert_eq!(log_return(&[Some(100.0)]), None);
    }

    #[test]
    fn test_log_return_zero_denom() {
        assert_eq!(log_return(&[Some(0.0), Some(100.0)]), None);
    }
}