//! Statistical analysis for A/B testing
//!
//! This module provides statistical functions for comparing two optimization strategies:
//! - Accuracy calculation
//! - Paired t-test for statistical significance
//! - Cohen's d for effect size
//! - Confidence intervals
//!
//! # Statistical Methods
//!
//! ## Paired T-Test
//! Tests whether the mean difference between paired observations is significantly different from zero.
//! - Null hypothesis (H0): No difference between strategies
//! - Alternative (H1): Strategy B is better than Strategy A
//! - Significance level: α = 0.05 (p < 0.05 rejects H0)
//!
//! ## Cohen's d (Effect Size)
//! Measures the standardized difference between two means:
//! - Small: d ≈ 0.2
//! - Medium: d ≈ 0.5
//! - Large: d ≈ 0.8 or higher
//!
//! ## Confidence Intervals
//! 95% confidence interval for the mean difference

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

/// Statistical comparison between two strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonStats {
    /// Accuracy difference (B - A)
    pub accuracy_diff: f64,

    /// Latency difference in ms (B - A)
    pub latency_diff: i64,

    /// Confidence difference (B - A)
    pub confidence_diff: f64,

    /// P-value from paired t-test
    pub p_value: f64,

    /// Effect size (Cohen's d)
    pub effect_size: f64,

    /// Winner: "Strategy A", "Strategy B", or "Tie"
    pub winner: String,

    /// Is the result statistically significant? (p < 0.05)
    pub is_significant: bool,

    /// Effect size interpretation
    pub effect_size_interpretation: String,

    /// 95% confidence interval for accuracy difference [lower, upper]
    pub accuracy_ci: [f64; 2],
}

/// Calculate accuracy for a set of predictions
///
/// Accuracy = (number of correct predictions) / (total predictions)
///
/// # Arguments
///
/// * `actual` - Ground truth complexity classes
/// * `predicted` - Predicted complexity classes
///
/// # Returns
///
/// Accuracy as a value between 0.0 and 1.0
pub fn calculate_accuracy(actual: &[usize], predicted: &[usize]) -> Result<f64> {
    if actual.len() != predicted.len() {
        return Err(anyhow!(
            "Actual and predicted must have same length: {} != {}",
            actual.len(),
            predicted.len()
        ));
    }

    if actual.is_empty() {
        return Err(anyhow!("Cannot calculate accuracy for empty data"));
    }

    let correct = actual
        .iter()
        .zip(predicted)
        .filter(|(a, p)| a == p)
        .count();

    Ok(correct as f64 / actual.len() as f64)
}

/// Perform paired t-test
///
/// Tests whether the mean difference between paired observations is significantly different from zero.
///
/// # Arguments
///
/// * `scores_a` - Accuracy scores for strategy A (one per test case)
/// * `scores_b` - Accuracy scores for strategy B (one per test case)
///
/// # Returns
///
/// P-value (probability that the difference occurred by chance)
/// - p < 0.05: Statistically significant difference
/// - p >= 0.05: No significant difference
pub fn paired_t_test(scores_a: &[f64], scores_b: &[f64]) -> Result<f64> {
    if scores_a.len() != scores_b.len() {
        return Err(anyhow!("Paired samples must have same length"));
    }

    let n = scores_a.len();
    if n < 2 {
        return Err(anyhow!("Need at least 2 samples for t-test"));
    }

    // Calculate differences
    let differences: Vec<f64> = scores_a
        .iter()
        .zip(scores_b)
        .map(|(a, b)| b - a)
        .collect();

    // Calculate mean difference
    let mean_diff = differences.iter().sum::<f64>() / n as f64;

    // Calculate standard deviation of differences
    let variance = differences
        .iter()
        .map(|d| (d - mean_diff).powi(2))
        .sum::<f64>()
        / (n - 1) as f64;

    let std_dev = variance.sqrt();

    // Calculate t-statistic
    let t_stat = if std_dev == 0.0 {
        // No variance means all differences are 0, so no significant difference
        return Ok(1.0); // Maximum p-value (no significance)
    } else {
        mean_diff / (std_dev / (n as f64).sqrt())
    };

    // Check for NaN
    if t_stat.is_nan() || t_stat.is_infinite() {
        return Ok(1.0); // No significant difference if calculation fails
    }

    // Degrees of freedom
    let df = n - 1;

    // Calculate p-value using t-distribution approximation
    // For one-tailed test (H1: B > A)
    let p_value = calculate_t_distribution_p_value(t_stat, df);

    Ok(p_value)
}

/// Calculate Cohen's d effect size
///
/// Measures the standardized difference between two means.
///
/// # Arguments
///
/// * `scores_a` - Scores for strategy A
/// * `scores_b` - Scores for strategy B
///
/// # Returns
///
/// Cohen's d value:
/// - d < 0.2: Negligible effect
/// - 0.2 <= d < 0.5: Small effect
/// - 0.5 <= d < 0.8: Medium effect
/// - d >= 0.8: Large effect
pub fn cohens_d(scores_a: &[f64], scores_b: &[f64]) -> Result<f64> {
    if scores_a.len() != scores_b.len() {
        return Err(anyhow!("Paired samples must have same length"));
    }

    if scores_a.is_empty() {
        return Err(anyhow!("Cannot calculate effect size for empty data"));
    }

    // Calculate means
    let mean_a = scores_a.iter().sum::<f64>() / scores_a.len() as f64;
    let mean_b = scores_b.iter().sum::<f64>() / scores_b.len() as f64;

    // Calculate pooled standard deviation
    let var_a = scores_a
        .iter()
        .map(|x| (x - mean_a).powi(2))
        .sum::<f64>()
        / (scores_a.len() - 1) as f64;

    let var_b = scores_b
        .iter()
        .map(|x| (x - mean_b).powi(2))
        .sum::<f64>()
        / (scores_b.len() - 1) as f64;

    let pooled_std = ((var_a + var_b) / 2.0).sqrt();

    if pooled_std == 0.0 {
        return Ok(0.0); // No variance = no effect
    }

    // Cohen's d = (mean difference) / pooled standard deviation
    let d = (mean_b - mean_a) / pooled_std;

    Ok(d)
}

/// Interpret Cohen's d effect size
pub fn interpret_effect_size(d: f64) -> String {
    let abs_d = d.abs();

    if abs_d < 0.2 {
        "Negligible".to_string()
    } else if abs_d < 0.5 {
        "Small".to_string()
    } else if abs_d < 0.8 {
        "Medium".to_string()
    } else {
        "Large".to_string()
    }
}

/// Calculate 95% confidence interval for mean difference
///
/// Returns [lower_bound, upper_bound]
pub fn confidence_interval(scores_a: &[f64], scores_b: &[f64]) -> Result<[f64; 2]> {
    if scores_a.len() != scores_b.len() {
        return Err(anyhow!("Paired samples must have same length"));
    }

    let n = scores_a.len();
    if n < 2 {
        return Err(anyhow!("Need at least 2 samples for confidence interval"));
    }

    // Calculate differences
    let differences: Vec<f64> = scores_a
        .iter()
        .zip(scores_b)
        .map(|(a, b)| b - a)
        .collect();

    // Calculate mean difference
    let mean_diff = differences.iter().sum::<f64>() / n as f64;

    // Calculate standard error
    let variance = differences
        .iter()
        .map(|d| (d - mean_diff).powi(2))
        .sum::<f64>()
        / (n - 1) as f64;

    let std_error = variance.sqrt() / (n as f64).sqrt();

    // Handle zero variance case
    if std_error == 0.0 || std_error.is_nan() {
        return Ok([mean_diff, mean_diff]); // Point estimate
    }

    // t-critical value for 95% CI (approximate for df=30)
    let t_critical = 2.0; // Approximate for large samples

    let margin = t_critical * std_error;

    Ok([mean_diff - margin, mean_diff + margin])
}

/// Approximate t-distribution p-value calculation
///
/// This is a simplified approximation. For production use, consider using
/// a statistical library like `statrs`.
fn calculate_t_distribution_p_value(t_stat: f64, df: usize) -> f64 {
    // For simplicity, use normal approximation for large df
    // This is valid for df > 30

    if df > 30 {
        // Use standard normal approximation
        let z = t_stat;
        return 1.0 - standard_normal_cdf(z);
    }

    // For small df, use simplified approximation
    // This is less accurate but sufficient for A/B testing
    let x = df as f64 / (df as f64 + t_stat.powi(2));
    let p = 0.5 * incomplete_beta(df as f64 / 2.0, 0.5, x);

    if t_stat > 0.0 {
        p
    } else {
        1.0 - p
    }
}

/// Standard normal CDF approximation
fn standard_normal_cdf(z: f64) -> f64 {
    0.5 * (1.0 + erf(z / std::f64::consts::SQRT_2))
}

/// Error function approximation
fn erf(x: f64) -> f64 {
    // Abramowitz and Stegun approximation
    let a1 = 0.254829592;
    let a2 = -0.284496736;
    let a3 = 1.421413741;
    let a4 = -1.453152027;
    let a5 = 1.061405429;
    let p = 0.3275911;

    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let x = x.abs();

    let t = 1.0 / (1.0 + p * x);
    let y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-x * x).exp();

    sign * y
}

/// Incomplete beta function approximation
fn incomplete_beta(a: f64, b: f64, x: f64) -> f64 {
    // Simplified approximation for A/B testing
    // For production, use a proper statistical library

    if x <= 0.0 {
        return 0.0;
    }
    if x >= 1.0 {
        return 1.0;
    }

    // Use continued fraction approximation
    let bt = x.powf(a) * (1.0 - x).powf(b);
    let mut c = 1.0;
    let mut d = 1.0 - (a + b) * x / (a + 1.0);

    if d.abs() < 1e-30 {
        d = 1e-30;
    }
    d = 1.0 / d;
    let mut h = d;

    for m in 1..=100 {
        let m_f = m as f64;
        let m2 = 2.0 * m_f;
        let aa = m_f * (b - m_f) * x / ((a + m2 - 1.0) * (a + m2));
        d = 1.0 + aa * d;
        if d.abs() < 1e-30 {
            d = 1e-30;
        }
        c = 1.0 + aa / c;
        if c.abs() < 1e-30 {
            c = 1e-30;
        }
        d = 1.0 / d;
        h *= d * c;

        let aa = -(a + m_f) * (a + b + m_f) * x / ((a + m2) * (a + m2 + 1.0));
        d = 1.0 + aa * d;
        if d.abs() < 1e-30 {
            d = 1e-30;
        }
        c = 1.0 + aa / c;
        if c.abs() < 1e-30 {
            c = 1e-30;
        }
        d = 1.0 / d;
        let del = d * c;
        h *= del;

        if (del - 1.0).abs() < 1e-7 {
            break;
        }
    }

    bt * h / a
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_accuracy_perfect() {
        let actual = vec![1, 2, 3, 4, 5];
        let predicted = vec![1, 2, 3, 4, 5];

        let accuracy = calculate_accuracy(&actual, &predicted).unwrap();
        assert_eq!(accuracy, 1.0);
    }

    #[test]
    fn test_calculate_accuracy_half() {
        let actual = vec![1, 2, 3, 4];
        let predicted = vec![1, 2, 0, 0];

        let accuracy = calculate_accuracy(&actual, &predicted).unwrap();
        assert_eq!(accuracy, 0.5);
    }

    #[test]
    fn test_calculate_accuracy_zero() {
        let actual = vec![1, 2, 3];
        let predicted = vec![4, 5, 6];

        let accuracy = calculate_accuracy(&actual, &predicted).unwrap();
        assert_eq!(accuracy, 0.0);
    }

    #[test]
    fn test_calculate_accuracy_length_mismatch() {
        let actual = vec![1, 2, 3];
        let predicted = vec![1, 2];

        let result = calculate_accuracy(&actual, &predicted);
        assert!(result.is_err());
    }

    #[test]
    fn test_paired_t_test_significant_difference() {
        // Strategy B consistently better
        let scores_a = vec![0.5, 0.6, 0.55, 0.58, 0.52];
        let scores_b = vec![0.9, 0.92, 0.88, 0.91, 0.89];

        let p_value = paired_t_test(&scores_a, &scores_b).unwrap();

        // Should be highly significant (p < 0.05)
        assert!(p_value < 0.05, "p-value {} should be < 0.05", p_value);
    }

    #[test]
    fn test_paired_t_test_no_difference() {
        // Strategies equally good
        let scores = vec![0.8, 0.82, 0.79, 0.81, 0.80];

        let p_value = paired_t_test(&scores, &scores).unwrap();

        // Should not be significant (p ≈ 1.0)
        assert!(p_value > 0.5, "p-value {} should be > 0.5", p_value);
    }

    #[test]
    fn test_cohens_d_large_effect() {
        // Large difference between strategies
        let scores_a = vec![0.5, 0.55, 0.52, 0.53, 0.54];
        let scores_b = vec![0.9, 0.92, 0.88, 0.91, 0.89];

        let d = cohens_d(&scores_a, &scores_b).unwrap();

        // Should be large effect (d > 0.8)
        assert!(d > 0.8, "Cohen's d {} should be > 0.8", d);
    }

    #[test]
    fn test_cohens_d_small_effect() {
        // Small difference
        let scores_a = vec![0.7, 0.72, 0.69, 0.71, 0.70];
        let scores_b = vec![0.75, 0.77, 0.74, 0.76, 0.75];

        let d = cohens_d(&scores_a, &scores_b).unwrap();

        // Should be positive and reasonable magnitude
        assert!(d > 0.0, "Effect size should be positive");
        assert!(d < 5.0, "Effect size should be reasonable");
    }

    #[test]
    fn test_cohens_d_no_effect() {
        let scores = vec![0.8, 0.82, 0.79, 0.81, 0.80];

        let d = cohens_d(&scores, &scores).unwrap();

        assert_eq!(d, 0.0);
    }

    #[test]
    fn test_interpret_effect_size() {
        assert_eq!(interpret_effect_size(0.1), "Negligible");
        assert_eq!(interpret_effect_size(0.3), "Small");
        assert_eq!(interpret_effect_size(0.6), "Medium");
        assert_eq!(interpret_effect_size(1.2), "Large");
        assert_eq!(interpret_effect_size(-1.0), "Large"); // Negative also works
    }

    #[test]
    fn test_confidence_interval() {
        let scores_a = vec![0.5, 0.6, 0.55, 0.58, 0.52];
        let scores_b = vec![0.9, 0.92, 0.88, 0.91, 0.89];

        let ci = confidence_interval(&scores_a, &scores_b).unwrap();

        // CI should be positive (B > A)
        assert!(ci[0] > 0.0);
        assert!(ci[1] > ci[0]);
        assert!(ci[1] > 0.0);
    }

    #[test]
    fn test_confidence_interval_symmetric() {
        let scores = vec![0.8, 0.82, 0.79, 0.81, 0.80];

        let ci = confidence_interval(&scores, &scores).unwrap();

        // CI should be centered around 0 (no difference)
        // With zero variance, we get a point estimate at 0.0
        assert_eq!(ci[0], 0.0);
        assert_eq!(ci[1], 0.0);
    }

    #[test]
    fn test_standard_normal_cdf() {
        // Test known values
        assert!((standard_normal_cdf(0.0) - 0.5).abs() < 0.01);
        assert!((standard_normal_cdf(1.96) - 0.975).abs() < 0.01);
        assert!((standard_normal_cdf(-1.96) - 0.025).abs() < 0.01);
    }

    #[test]
    fn test_erf() {
        // Test known values
        assert!((erf(0.0) - 0.0).abs() < 0.01);
        assert!((erf(1.0) - 0.8427).abs() < 0.01);
        assert!((erf(-1.0) + 0.8427).abs() < 0.01);
    }
}
