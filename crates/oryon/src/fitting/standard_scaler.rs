use crate::error::OryonError;
use crate::ops::{average, std_dev};

/// Parameters produced by [`fit_standard_scaler`].
#[derive(Debug, Clone, Copy)]
pub struct StandardScalerParams {
    /// Column mean.
    pub mean: f64,
    /// Column sample standard deviation.
    pub std: f64,
}

/// Compute mean and standard deviation from a column of data.
///
/// Skips `None` values. Returns `Err` if fewer than 2 valid values
/// or if the standard deviation is zero (all values equal).
pub fn fit_standard_scaler(data: &[Option<f64>]) -> Result<StandardScalerParams, OryonError> {
    let valid: Vec<Option<f64>> = data.iter().filter(|v| v.is_some()).copied().collect();

    let mean = average(&valid).ok_or_else(|| OryonError::InvalidInput {
        msg: "not enough valid values to compute mean".into(),
    })?;

    let std = std_dev(&valid).ok_or_else(|| OryonError::InvalidInput {
        msg: "not enough valid values to compute std (need >= 2)".into(),
    })?;

    if std == 0.0 {
        return Err(OryonError::InvalidInput {
            msg: "standard deviation is zero (all values are equal)".into(),
        });
    }

    Ok(StandardScalerParams { mean, std })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fit_standard_scaler() {
        let data = vec![Some(1.0), Some(2.0), Some(3.0), Some(4.0), Some(5.0)];
        let params = fit_standard_scaler(&data).unwrap();
        assert!((params.mean - 3.0).abs() < 1e-10);
        // std_dev([1,2,3,4,5]) = sqrt(10/4) = sqrt(2.5)
        assert!((params.std - 2.5_f64.sqrt()).abs() < 1e-10);
    }

    #[test]
    fn test_fit_standard_scaler_skips_none() {
        let data = vec![Some(1.0), None, Some(3.0), None, Some(5.0)];
        let params = fit_standard_scaler(&data).unwrap();
        assert!((params.mean - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_fit_standard_scaler_all_none() {
        let data = vec![None, None, None];
        assert!(fit_standard_scaler(&data).is_err());
    }

    #[test]
    fn test_fit_standard_scaler_single_value() {
        let data = vec![Some(5.0)];
        assert!(fit_standard_scaler(&data).is_err());
    }

    #[test]
    fn test_fit_standard_scaler_all_equal() {
        let data = vec![Some(3.0), Some(3.0), Some(3.0)];
        let err = fit_standard_scaler(&data).unwrap_err();
        assert!(matches!(err, OryonError::InvalidInput { ref msg } if msg.contains("zero")));
    }

    #[test]
    fn test_fit_standard_scaler_empty() {
        assert!(fit_standard_scaler(&[]).is_err());
    }
}
