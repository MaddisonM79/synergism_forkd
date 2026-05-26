//! Sigmoid-style curve helpers.
//!
//! Verbatim port of `legacy_core_split/packages/logic/src/math/sigmoid.ts`.
//! Generic 1-line formulas that return 1 at zero progress and asymptote
//! toward `constant` as progress grows. Used by cube/quark multiplier code
//! in the legacy `web_ui`.

/// Doubling-half sigmoid:
///   `1 + (constant - 1) * (1 - 2^(-factor / divisor))`
/// Returns 1 when `factor = 0` and asymptotes to `constant` as `factor → ∞`.
/// `divisor` controls how quickly the curve saturates.
pub fn calculate_sigmoid(constant: f64, factor: f64, divisor: f64) -> f64 {
    1.0 + (constant - 1.0) * (1.0 - 2.0_f64.powf(-factor / divisor))
}

/// Natural-exponential sigmoid:
///   `1 + (constant - 1) * (1 - e^(-coefficient))`
/// Same shape as [`calculate_sigmoid`] but uses `e` as the base —
/// `coefficient` is the natural-log progress rather than a halving count.
pub fn calculate_sigmoid_exponential(constant: f64, coefficient: f64) -> f64 {
    1.0 + (constant - 1.0) * (1.0 - (-coefficient).exp())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sigmoid_is_one_at_zero_progress() {
        assert_eq!(calculate_sigmoid(2.0, 0.0, 1.0), 1.0);
        assert_eq!(calculate_sigmoid(100.0, 0.0, 1e6), 1.0);
        assert_eq!(calculate_sigmoid(0.5, 0.0, 1.0), 1.0);
    }

    #[test]
    fn sigmoid_asymptotes_to_constant() {
        // factor / divisor very large → 2^(-very-large) ≈ 0 → result ≈ constant.
        let result = calculate_sigmoid(10.0, 1e6, 1.0);
        assert!((result - 10.0).abs() < 1e-9);
    }

    #[test]
    fn sigmoid_inverts_when_constant_below_one() {
        // constant < 1: curve falls from 1 toward `constant`.
        let result = calculate_sigmoid(0.5, 1e6, 1.0);
        assert!((result - 0.5).abs() < 1e-9);
    }

    #[test]
    fn sigmoid_exponential_is_one_at_zero_coefficient() {
        assert_eq!(calculate_sigmoid_exponential(2.0, 0.0), 1.0);
        assert_eq!(calculate_sigmoid_exponential(1e6, 0.0), 1.0);
    }

    #[test]
    fn sigmoid_exponential_asymptotes_to_constant() {
        let result = calculate_sigmoid_exponential(20.0, 100.0);
        assert!((result - 20.0).abs() < 1e-9);
    }

    #[test]
    fn sigmoid_exponential_matches_closed_form_at_unit_coefficient() {
        // 1 + (c - 1) * (1 - 1/e)
        let expected = 1.0 + (5.0 - 1.0) * (1.0 - (-1.0_f64).exp());
        assert_eq!(calculate_sigmoid_exponential(5.0, 1.0), expected);
    }
}
