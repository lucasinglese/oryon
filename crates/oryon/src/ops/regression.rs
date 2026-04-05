/// OLS slope of `y` regressed on `x`.
///
/// Computes `Sxy / Sxx` where:
/// - `Sxy = Σ (x_i - x̄)(y_i - ȳ)`
/// - `Sxx = Σ (x_i - x̄)²`
///
/// Returns `None` if:
/// - slices have different lengths or fewer than 2 points,
/// - any value in `x` or `y` is `None`,
/// - `x` is constant (`Sxx == 0`).
pub fn linear_slope(x: &[Option<f64>], y: &[Option<f64>]) -> Option<f64> {
    let n = x.len();
    if n < 2 || n != y.len() {
        return None;
    }

    let mut x_sum = 0.0f64;
    let mut y_sum = 0.0f64;
    for i in 0..n {
        x_sum += x[i]?;
        y_sum += y[i]?;
    }
    let x_mean = x_sum / n as f64;
    let y_mean = y_sum / n as f64;

    let mut sxx = 0.0f64;
    let mut sxy = 0.0f64;
    for i in 0..n {
        let dx = x[i].unwrap() - x_mean;
        let dy = y[i].unwrap() - y_mean;
        sxx += dx * dx;
        sxy += dx * dy;
    }

    if sxx == 0.0 {
        return None;
    }

    Some(sxy / sxx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_slope() {
        // Python reference: slope([1,2,3,4,5], [2.1,3.9,6.2,7.8,10.1]) = 1.99
        let x = [Some(1.0), Some(2.0), Some(3.0), Some(4.0), Some(5.0)];
        let y = [Some(2.1), Some(3.9), Some(6.2), Some(7.8), Some(10.1)];
        assert!((linear_slope(&x, &y).unwrap() - 1.99).abs() < 1e-10);
    }

    #[test]
    fn test_linear_slope_perfect() {
        // y = 2x → slope = 2.0
        let x = [Some(1.0), Some(2.0), Some(3.0)];
        let y = [Some(2.0), Some(4.0), Some(6.0)];
        assert!((linear_slope(&x, &y).unwrap() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_linear_slope_with_none() {
        let x = [Some(1.0), None, Some(3.0)];
        let y = [Some(2.0), Some(4.0), Some(6.0)];
        assert_eq!(linear_slope(&x, &y), None);
    }

    #[test]
    fn test_linear_slope_with_none_in_y() {
        let x = [Some(1.0), Some(2.0), Some(3.0)];
        let y = [Some(2.0), None, Some(6.0)];
        assert_eq!(linear_slope(&x, &y), None);
    }

    #[test]
    fn test_linear_slope_empty() {
        assert_eq!(linear_slope(&[], &[]), None);
    }

    #[test]
    fn test_linear_slope_constant_x() {
        // x constant → Sxx == 0 → None
        let x = [Some(3.0), Some(3.0), Some(3.0)];
        let y = [Some(1.0), Some(2.0), Some(3.0)];
        assert_eq!(linear_slope(&x, &y), None);
    }

    #[test]
    fn test_linear_slope_mismatched_lengths() {
        let x = [Some(1.0), Some(2.0)];
        let y = [Some(1.0), Some(2.0), Some(3.0)];
        assert_eq!(linear_slope(&x, &y), None);
    }
}
