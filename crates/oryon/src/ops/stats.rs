/// Arithmetic mean. Returns `None` if empty or any value is `None`.
pub fn average(data: &[Option<f64>]) -> Option<f64> {
    if data.is_empty() {
        return None;
    }
    let mut sum: f64 = 0.0;
    for value in data.iter().copied() {
        sum += value?;
    }
    Some(sum / (data.len() as f64))
}

/// Sample standard deviation (N-1). Returns `None` if fewer than 2 values or any is `None`.
pub fn std_dev(data: &[Option<f64>]) -> Option<f64> {
    if data.len() < 2 {
        return None;
    }
    let avg = average(data)?;
    let mut sum_sq: f64 = 0.0;
    for value in data.iter().copied() {
        let v = value?;
        sum_sq += (v - avg).powi(2);
    }
    Some((sum_sq / ((data.len() - 1) as f64)).sqrt())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_average() {
        assert_eq!(average(&[Some(1.0), Some(2.0), Some(3.0)]), Some(2.0));
    }

    #[test]
    fn test_average_single() {
        assert_eq!(average(&[Some(1.0)]), Some(1.0));
    }

    #[test]
    fn test_average_empty() {
        assert_eq!(average(&[]), None);
    }

    #[test]
    fn test_average_with_none() {
        assert_eq!(average(&[Some(1.0), None, Some(3.0)]), None);
    }

    #[test]
    fn test_std_dev() {
        // std_dev([1, 3]) = sqrt(((1-2)^2 + (3-2)^2) / 1) = sqrt(2)
        let result = std_dev(&[Some(1.0), Some(3.0)]);
        assert!((result.unwrap() - std::f64::consts::SQRT_2).abs() < 1e-10);
    }

    #[test]
    fn test_std_dev_single() {
        assert_eq!(std_dev(&[Some(1.0)]), None);
    }

    #[test]
    fn test_std_dev_empty() {
        assert_eq!(std_dev(&[]), None);
    }

    #[test]
    fn test_std_dev_with_none() {
        assert_eq!(std_dev(&[Some(1.0), None, Some(3.0)]), None);
    }
}