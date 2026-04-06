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

/// Sample skewness (Fisher-Pearson corrected, same as pandas `.skew()`).
///
/// Returns `None` if fewer than 3 values, any is `None`, or standard deviation is 0.
pub fn skewness(data: &[Option<f64>]) -> Option<f64> {
    let n = data.len();
    if n < 3 {
        return None;
    }
    let mean = average(data)?;
    let s = std_dev(data)?;
    if s == 0.0 {
        return None;
    }
    let mut sum: f64 = 0.0;
    for value in data.iter().copied() {
        let v = value?;
        sum += ((v - mean) / s).powi(3);
    }
    let correction = (n as f64) / ((n - 1) as f64 * (n - 2) as f64);
    Some(correction * sum)
}

/// Excess kurtosis (Fisher, same as pandas `.kurt()`).
///
/// Returns `None` if fewer than 4 values, any is `None`, or standard deviation is 0.
pub fn kurtosis(data: &[Option<f64>]) -> Option<f64> {
    let n = data.len();
    if n < 4 {
        return None;
    }
    let mean = average(data)?;
    let s = std_dev(data)?;
    if s == 0.0 {
        return None;
    }
    let mut sum: f64 = 0.0;
    for value in data.iter().copied() {
        let v = value?;
        sum += ((v - mean) / s).powi(4);
    }
    let n = n as f64;
    let term1 = (n * (n + 1.0)) / ((n - 1.0) * (n - 2.0) * (n - 3.0));
    let correction = (3.0 * (n - 1.0).powi(2)) / ((n - 2.0) * (n - 3.0));
    Some(term1 * sum - correction)
}

/// Median
/// Returns `None` if empty or any value is `None`.
pub fn median(data: &[Option<f64>]) -> Option<f64> {
    if data.is_empty() {
        return None;
    }

    let mut sorted: Vec<f64> = data.iter().copied().collect::<Option<Vec<_>>>()?;
    sorted.sort_by(|a, b| a.total_cmp(b));

    let mid = sorted.len() / 2;
    if sorted.len().is_multiple_of(2) {
        Some((sorted[mid - 1] + sorted[mid]) / 2.0)
    } else {
        Some(sorted[mid])
    }
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

    #[test]
    fn test_skewness_symmetric() {
        // Python: skewness([2,4,4,4,5,5,7,9]) = 0.818487553356800
        let data = vec![
            Some(2.0),
            Some(4.0),
            Some(4.0),
            Some(4.0),
            Some(5.0),
            Some(5.0),
            Some(7.0),
            Some(9.0),
        ];
        assert!((skewness(&data).unwrap() - 0.818487553356800).abs() < 1e-10);
    }

    #[test]
    fn test_skewness_linear() {
        // Python: skewness([1,2,3,4,5]) = 0.0
        let data = vec![Some(1.0), Some(2.0), Some(3.0), Some(4.0), Some(5.0)];
        assert!(skewness(&data).unwrap().abs() < 1e-10);
    }

    #[test]
    fn test_skewness_right_skewed() {
        // Python: skewness([1,1,1,2,10]) = 2.171292493874224
        let data = vec![Some(1.0), Some(1.0), Some(1.0), Some(2.0), Some(10.0)];
        assert!((skewness(&data).unwrap() - 2.171292493874224).abs() < 1e-10);
    }

    #[test]
    fn test_skewness_min_n() {
        // Python: skewness([1,2,4]) = 0.935219529582824
        let data = vec![Some(1.0), Some(2.0), Some(4.0)];
        assert!((skewness(&data).unwrap() - 0.935219529582824).abs() < 1e-10);
    }

    #[test]
    fn test_skewness_too_short() {
        assert_eq!(skewness(&[Some(1.0), Some(2.0)]), None);
    }

    #[test]
    fn test_skewness_with_none() {
        assert_eq!(skewness(&[Some(1.0), None, Some(3.0), Some(4.0)]), None);
    }

    #[test]
    fn test_skewness_all_equal() {
        assert_eq!(skewness(&[Some(5.0), Some(5.0), Some(5.0)]), None);
    }

    #[test]
    fn test_kurtosis_symmetric() {
        // Python: kurtosis([2,4,4,4,5,5,7,9]) = 0.940625000000000
        let data = vec![
            Some(2.0),
            Some(4.0),
            Some(4.0),
            Some(4.0),
            Some(5.0),
            Some(5.0),
            Some(7.0),
            Some(9.0),
        ];
        assert!((kurtosis(&data).unwrap() - 0.940625000000000).abs() < 1e-10);
    }

    #[test]
    fn test_kurtosis_linear() {
        // Python: kurtosis([1,2,3,4,5]) = -1.2
        let data = vec![Some(1.0), Some(2.0), Some(3.0), Some(4.0), Some(5.0)];
        assert!((kurtosis(&data).unwrap() - (-1.2)).abs() < 1e-10);
    }

    #[test]
    fn test_kurtosis_min_n() {
        // Python: kurtosis([1,2,4,8]) = 0.757655954631380
        let data = vec![Some(1.0), Some(2.0), Some(4.0), Some(8.0)];
        assert!((kurtosis(&data).unwrap() - 0.757655954631380).abs() < 1e-10);
    }

    #[test]
    fn test_kurtosis_too_short() {
        assert_eq!(kurtosis(&[Some(1.0), Some(2.0), Some(3.0)]), None);
    }

    #[test]
    fn test_kurtosis_with_none() {
        assert_eq!(kurtosis(&[Some(1.0), None, Some(3.0), Some(4.0)]), None);
    }

    #[test]
    fn test_kurtosis_all_equal() {
        assert_eq!(
            kurtosis(&[Some(5.0), Some(5.0), Some(5.0), Some(5.0)]),
            None
        );
    }

    #[test]
    fn test_median_odd() {
        assert_eq!(
            median(&[Some(3.0), Some(2.0), Some(1.0), Some(4.0), Some(5.0)]),
            Some(3.0)
        );
    }

    #[test]
    fn test_median_even() {
        assert_eq!(
            median(&[
                Some(6.0),
                Some(3.0),
                Some(2.0),
                Some(1.0),
                Some(4.0),
                Some(5.0)
            ]),
            Some(3.5)
        );
    }

    #[test]
    fn test_median_unique_value() {
        assert_eq!(median(&[Some(1.0)]), Some(1.0));
    }

    #[test]
    fn test_median_with_none() {
        assert_eq!(median(&[None, Some(2.0), Some(1.0)]), None);
    }

    #[test]
    fn test_median_empty() {
        assert_eq!(median(&[]), None);
    }
}
