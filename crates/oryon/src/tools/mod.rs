use std::collections::VecDeque;

/// Apply `op` over a rolling window of `window_size`.
/// Returns `None` for positions where the window is incomplete.
pub fn rolling<F>(data: &[Option<f64>], window_size: usize, op: F) -> Vec<Option<f64>>
where
    F: Fn(&[Option<f64>]) -> Option<f64>,
{
    let mut result = Vec::with_capacity(data.len());

    if window_size == 0 || data.is_empty() {
        return result;
    }

    let mut buffer: VecDeque<Option<f64>> = VecDeque::with_capacity(window_size);

    for &value in data {
        buffer.push_back(value);

        if buffer.len() > window_size {
            buffer.pop_front();
        }

        if buffer.len() == window_size {
            let slice = buffer.make_contiguous();
            result.push(op(slice));
        } else {
            result.push(None);
        }
    }
    result
}

/// Shift a series by `offset` positions.
/// Positive = forward (inserts `None` at the start).
/// Negative = backward (inserts `None` at the end).
pub fn shift(data: &[Option<f64>], offset: isize) -> Vec<Option<f64>> {
    let n = data.len();
    let mut result = vec![None; n];
    if offset >= 0 {
        let off = offset as usize;
        for i in off..n {
            result[i] = data[i - off];
        }
    } else {
        let off = (-offset) as usize;
        if off < n {
            for i in 0..(n - off) {
                result[i] = data[i + off];
            }
        }
    }
    result
}

/// Zip two series element-wise and apply `op` to each pair.
/// `op` receives `&[a_i, b_i]`. Output length is `min(a.len(), b.len())`.
pub fn pairwise<F>(a: &[Option<f64>], b: &[Option<f64>], op: F) -> Vec<Option<f64>>
where
    F: Fn(&[Option<f64>]) -> Option<f64>,
{
    let n = a.len().min(b.len());
    let mut result = Vec::with_capacity(n);
    for i in 0..n {
        result.push(op(&[a[i], b[i]]));
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ops::{average, log_return};

    // --- rolling ---

    #[test]
    fn test_rolling_average() {
        let data = vec![Some(1.0), Some(2.0), Some(3.0), Some(4.0), Some(5.0)];
        let result = rolling(&data, 3, average);
        assert_eq!(result, vec![None, None, Some(2.0), Some(3.0), Some(4.0)]);
    }

    #[test]
    fn test_rolling_with_none() {
        let data = vec![Some(1.0), Some(2.0), None, Some(4.0), Some(5.0)];
        let result = rolling(&data, 3, average);
        assert_eq!(result, vec![None, None, None, None, None]);
    }

    #[test]
    fn test_rolling_window_zero() {
        let result = rolling(&[Some(1.0)], 0, average);
        assert!(result.is_empty());
    }

    // --- shift ---

    #[test]
    fn test_shift_positive() {
        let data = vec![Some(1.0), Some(2.0), Some(3.0)];
        assert_eq!(shift(&data, 1), vec![None, Some(1.0), Some(2.0)]);
    }

    #[test]
    fn test_shift_negative() {
        let data = vec![Some(1.0), Some(2.0), Some(3.0)];
        assert_eq!(shift(&data, -1), vec![Some(2.0), Some(3.0), None]);
    }

    #[test]
    fn test_shift_zero() {
        let data = vec![Some(1.0), Some(2.0)];
        assert_eq!(shift(&data, 0), vec![Some(1.0), Some(2.0)]);
    }

    // --- pairwise ---

    #[test]
    fn test_pairwise_log_return() {
        let prices = vec![Some(100.0), Some(110.0), Some(105.0)];
        let shifted = shift(&prices, 1);
        let result = pairwise(&prices, &shifted, log_return);

        assert_eq!(result[0], None);
        assert!((result[1].unwrap() - (110.0_f64 / 100.0).ln()).abs() < 1e-10);
        assert!((result[2].unwrap() - (105.0_f64 / 110.0).ln()).abs() < 1e-10);
    }
}
