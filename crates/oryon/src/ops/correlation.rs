/// Pearson correlation coefficient between two equal-length slices.
///
/// Computes `r = Sxy / sqrt(Sxx * Syy)` using a two-pass algorithm for
/// numerical stability (compute means first, then centred sums of squares).
///
/// Returns `None` if:
/// - either slice contains a `None` value,
/// - either series is constant over the window (`Sxx == 0` or `Syy == 0`),
/// - `n < 2` or the two slices have different lengths.
pub fn pearson_correlation(x: &[Option<f64>], y: &[Option<f64>]) -> Option<f64> {
    let n = x.len();
    if n < 2 || n != y.len() {
        return None;
    }

    // First pass: compute means, bail on any None.
    let mut x_sum = 0.0f64;
    let mut y_sum = 0.0f64;
    for i in 0..n {
        match (x[i], y[i]) {
            (Some(xi), Some(yi)) => {
                x_sum += xi;
                y_sum += yi;
            }
            _ => return None,
        }
    }
    let nf = n as f64;
    let x_mean = x_sum / nf;
    let y_mean = y_sum / nf;

    // Second pass: compute Sxy, Sxx, Syy.
    let mut sxy = 0.0f64;
    let mut sxx = 0.0f64;
    let mut syy = 0.0f64;
    for i in 0..n {
        let dx = x[i].unwrap() - x_mean;
        let dy = y[i].unwrap() - y_mean;
        sxy += dx * dy;
        sxx += dx * dx;
        syy += dy * dy;
    }

    let denom = sxx * syy;
    if denom == 0.0 {
        return None;
    }
    Some(sxy / denom.sqrt())
}

/// Spearman rank correlation coefficient between two equal-length slices.
///
/// Ranks both series (average rank for ties), then computes the Pearson
/// correlation on the ranks. Complexity: O(n log n) per call.
///
/// Returns `None` if:
/// - either slice contains a `None` value,
/// - both series produce identical ranks (`sigma == 0`),
/// - `n < 2` or the two slices have different lengths.
pub fn spearman_correlation(x: &[Option<f64>], y: &[Option<f64>]) -> Option<f64> {
    let n = x.len();
    if n < 2 || n != y.len() {
        return None;
    }
    let rx = rank(x)?;
    let ry = rank(y)?;
    pearson_on_ranks(&rx, &ry)
}

/// Kendall tau-b rank correlation coefficient between two equal-length slices.
///
/// `tau_b = (C - D) / sqrt((n0 - n1) * (n0 - n2))`
///
/// where `n0 = n(n-1)/2` total pairs, `n1` pairs tied in `x`, `n2` pairs
/// tied in `y`, `C` concordant pairs, `D` discordant pairs.
///
/// **Complexity: O(n^2) per call.** Suitable for small windows in live
/// trading (window <= 30, ~435 pairs). For larger windows use `spearman_correlation`
/// instead - it is O(n log n) and sufficient for most rank-based correlation use cases.
///
/// Returns `None` if:
/// - either slice contains a `None` value,
/// - the denominator is zero (e.g. one series is constant),
/// - `n < 2` or the two slices have different lengths.
pub fn kendall_correlation(x: &[Option<f64>], y: &[Option<f64>]) -> Option<f64> {
    let n = x.len();
    if n < 2 || n != y.len() {
        return None;
    }

    // Collect concrete values, bail on any None.
    let mut xv = Vec::with_capacity(n);
    let mut yv = Vec::with_capacity(n);
    for i in 0..n {
        match (x[i], y[i]) {
            (Some(xi), Some(yi)) => {
                xv.push(xi);
                yv.push(yi);
            }
            _ => return None,
        }
    }

    let mut concordant: i64 = 0;
    let mut discordant: i64 = 0;
    let mut tied_x: i64 = 0;
    let mut tied_y: i64 = 0;

    for i in 0..n {
        for j in (i + 1)..n {
            let dx = xv[j] - xv[i];
            let dy = yv[j] - yv[i];
            match (dx == 0.0, dy == 0.0) {
                (true, true) => {
                    // Tied in both — counts toward n1 and n2 but not C or D.
                    tied_x += 1;
                    tied_y += 1;
                }
                (true, false) => tied_x += 1,
                (false, true) => tied_y += 1,
                (false, false) => {
                    if dx.signum() == dy.signum() {
                        concordant += 1;
                    } else {
                        discordant += 1;
                    }
                }
            }
        }
    }

    let n0 = (n as i64) * (n as i64 - 1) / 2;
    let denom_sq = ((n0 - tied_x) * (n0 - tied_y)) as f64;
    if denom_sq <= 0.0 {
        return None;
    }
    Some((concordant - discordant) as f64 / denom_sq.sqrt())
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Compute fractional (average) ranks for a slice of optional values.
/// Returns `None` if any element is `None`.
fn rank(data: &[Option<f64>]) -> Option<Vec<f64>> {
    let n = data.len();
    let mut vals = Vec::with_capacity(n);
    for v in data {
        match v {
            Some(x) => vals.push(*x),
            None => return None,
        }
    }

    // Sort indices by value.
    let mut indices: Vec<usize> = (0..n).collect();
    indices.sort_by(|&a, &b| {
        vals[a]
            .partial_cmp(&vals[b])
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Assign average rank to each tie group.
    let mut ranks = vec![0.0f64; n];
    let mut i = 0;
    while i < n {
        let mut j = i + 1;
        while j < n && vals[indices[j]] == vals[indices[i]] {
            j += 1;
        }
        // 1-indexed average rank: positions i..j map to ranks (i+1)..(j+1).
        let avg_rank = (i + 1 + j) as f64 / 2.0;
        for k in i..j {
            ranks[indices[k]] = avg_rank;
        }
        i = j;
    }
    Some(ranks)
}

/// Pearson correlation on pre-ranked (concrete) slices. Used by Spearman.
fn pearson_on_ranks(x: &[f64], y: &[f64]) -> Option<f64> {
    let n = x.len();
    let nf = n as f64;
    let x_mean = x.iter().sum::<f64>() / nf;
    let y_mean = y.iter().sum::<f64>() / nf;

    let mut sxy = 0.0f64;
    let mut sxx = 0.0f64;
    let mut syy = 0.0f64;
    for i in 0..n {
        let dx = x[i] - x_mean;
        let dy = y[i] - y_mean;
        sxy += dx * dy;
        sxx += dx * dx;
        syy += dy * dy;
    }

    let denom = sxx * syy;
    if denom == 0.0 {
        return None;
    }
    Some(sxy / denom.sqrt())
}

// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- pearson_correlation --------------------------------------------------

    #[test]
    fn test_pearson_correlation() {
        // Perfect positive: r = 1.0
        let x = [Some(1.0), Some(2.0), Some(3.0)];
        let y = [Some(1.0), Some(2.0), Some(3.0)];
        assert!((pearson_correlation(&x, &y).unwrap() - 1.0).abs() < 1e-10);

        // Perfect negative: r = -1.0
        let y_neg = [Some(3.0), Some(2.0), Some(1.0)];
        assert!((pearson_correlation(&x, &y_neg).unwrap() + 1.0).abs() < 1e-10);

        // Non-trivial: x=[1,2,3], y=[1,3,2]
        // Sxy=1, Sxx=2, Syy=2 → r = 1/sqrt(4) = 0.5
        let y_partial = [Some(1.0), Some(3.0), Some(2.0)];
        assert!((pearson_correlation(&x, &y_partial).unwrap() - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_pearson_correlation_with_none() {
        let x = [Some(1.0), None, Some(3.0)];
        let y = [Some(1.0), Some(2.0), Some(3.0)];
        assert_eq!(pearson_correlation(&x, &y), None);

        let x2 = [Some(1.0), Some(2.0), Some(3.0)];
        let y2 = [Some(1.0), Some(2.0), None];
        assert_eq!(pearson_correlation(&x2, &y2), None);
    }

    #[test]
    fn test_pearson_correlation_constant_series() {
        // x constant → r = None
        let x = [Some(2.0), Some(2.0), Some(2.0)];
        let y = [Some(1.0), Some(2.0), Some(3.0)];
        assert_eq!(pearson_correlation(&x, &y), None);

        // y constant → r = None
        let x2 = [Some(1.0), Some(2.0), Some(3.0)];
        let y2 = [Some(5.0), Some(5.0), Some(5.0)];
        assert_eq!(pearson_correlation(&x2, &y2), None);
    }

    #[test]
    fn test_pearson_correlation_empty() {
        assert_eq!(pearson_correlation(&[], &[]), None);
        assert_eq!(pearson_correlation(&[Some(1.0)], &[Some(1.0)]), None);
    }

    // --- spearman_correlation -------------------------------------------------

    #[test]
    fn test_spearman_correlation() {
        // Perfect positive
        let x = [Some(1.0), Some(2.0), Some(3.0), Some(4.0)];
        let y = [Some(1.0), Some(2.0), Some(3.0), Some(4.0)];
        assert!((spearman_correlation(&x, &y).unwrap() - 1.0).abs() < 1e-10);

        // Perfect negative
        let y_neg = [Some(4.0), Some(3.0), Some(2.0), Some(1.0)];
        assert!((spearman_correlation(&x, &y_neg).unwrap() + 1.0).abs() < 1e-10);

        // x=[1,2,3,4], y=[1,3,2,4]: ranks identical → Pearson of [1,2,3,4] and [1,3,2,4]
        // Sxy=4, Sxx=5, Syy=5 → r = 4/5 = 0.8
        let y_partial = [Some(1.0), Some(3.0), Some(2.0), Some(4.0)];
        assert!((spearman_correlation(&x, &y_partial).unwrap() - 0.8).abs() < 1e-10);
    }

    #[test]
    fn test_spearman_correlation_with_ties() {
        // x=[1,1,3,4]: tied values 1 get avg rank 1.5
        let x = [Some(1.0), Some(1.0), Some(3.0), Some(4.0)];
        let y = [Some(1.0), Some(2.0), Some(3.0), Some(4.0)];
        let r = spearman_correlation(&x, &y).unwrap();
        // ranks_x=[1.5,1.5,3,4], ranks_y=[1,2,3,4]
        // x_mean=2.5, y_mean=2.5
        // dx=[-1,-1,0.5,1.5], dy=[-1.5,-0.5,0.5,1.5]
        // Sxy=1.5+0.5+0.25+2.25=4.5, Sxx=1+1+0.25+2.25=4.5, Syy=2.25+0.25+0.25+2.25=5.0
        // r = 4.5/sqrt(4.5*5.0) = 4.5/sqrt(22.5)
        let expected = 4.5f64 / (4.5f64 * 5.0f64).sqrt();
        assert!((r - expected).abs() < 1e-10);
    }

    #[test]
    fn test_spearman_correlation_with_none() {
        let x = [Some(1.0), None, Some(3.0)];
        let y = [Some(1.0), Some(2.0), Some(3.0)];
        assert_eq!(spearman_correlation(&x, &y), None);
    }

    #[test]
    fn test_spearman_correlation_empty() {
        assert_eq!(spearman_correlation(&[], &[]), None);
    }

    // --- kendall_correlation --------------------------------------------------

    #[test]
    fn test_kendall_correlation() {
        // Perfect positive: all 6 pairs concordant → τ = 1.0
        let x = [Some(1.0), Some(2.0), Some(3.0), Some(4.0)];
        let y = [Some(1.0), Some(2.0), Some(3.0), Some(4.0)];
        assert!((kendall_correlation(&x, &y).unwrap() - 1.0).abs() < 1e-10);

        // Perfect negative: all 6 pairs discordant → τ = -1.0
        let y_neg = [Some(4.0), Some(3.0), Some(2.0), Some(1.0)];
        assert!((kendall_correlation(&x, &y_neg).unwrap() + 1.0).abs() < 1e-10);

        // x=[1,2,3,4], y=[1,3,2,4]: C=5, D=1, n0=6 → τ = 4/6 = 2/3
        let y_partial = [Some(1.0), Some(3.0), Some(2.0), Some(4.0)];
        assert!((kendall_correlation(&x, &y_partial).unwrap() - 2.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_kendall_correlation_with_ties() {
        // x=[1,1,2,3]: n1=1 (one tied pair in x)
        let x = [Some(1.0), Some(1.0), Some(2.0), Some(3.0)];
        let y = [Some(1.0), Some(2.0), Some(3.0), Some(4.0)];
        // Pairs: (1,1)vs(1,2): tied_x; (1,1)vs(2,3): C; (1,1)vs(3,4): C;
        //        (1,2)vs(2,3): C; (1,2)vs(3,4): C; (2,3)vs(3,4): C
        // C=5, D=0, tied_x=1, tied_y=0, n0=6
        // τ = 5/sqrt(5*6) = 5/sqrt(30)
        let expected = 5.0f64 / 30.0f64.sqrt();
        assert!((kendall_correlation(&x, &y).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn test_kendall_correlation_with_none() {
        let x = [Some(1.0), None, Some(3.0)];
        let y = [Some(1.0), Some(2.0), Some(3.0)];
        assert_eq!(kendall_correlation(&x, &y), None);
    }

    #[test]
    fn test_kendall_correlation_empty() {
        assert_eq!(kendall_correlation(&[], &[]), None);
    }

    #[test]
    fn test_kendall_correlation_constant_series() {
        // x constant → tied_x = n0, denominator = 0 → None
        let x = [Some(2.0), Some(2.0), Some(2.0)];
        let y = [Some(1.0), Some(2.0), Some(3.0)];
        assert_eq!(kendall_correlation(&x, &y), None);
    }
}
