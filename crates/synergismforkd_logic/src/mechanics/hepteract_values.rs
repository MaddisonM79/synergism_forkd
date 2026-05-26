//! Hepteract effective-value and cap helpers.
//!
//! Verbatim port of
//! `legacy_core_split/packages/logic/src/mechanics/hepteractValues.ts`
//! (lifted from the legacy `packages/web_ui/src/Hepteracts.ts`).
//!
//! - [`hepteract_effective`]: applies the diminishing-returns formula
//!   past `LIMIT`. Special-cased for `quark` (which uses a custom
//!   non-polynomial formula owned by the UI tier — here we just pass
//!   `BAL` through when the caller flags it as the quark hept).
//! - [`hepteract_cap`]: `BASE_CAP * 2^TIMES_CAP_EXTENDED` — the
//!   player's expanded cap before any Exalt 3 doubling.
//! - [`hepteract_final_cap`]:
//!   `hepteract_cap * (limitedAscensions Exalt 3 reward active ? 2 :
//!   1)` — the post-Exalt cap actually used by the UI.

/// Inputs to [`hepteract_effective`].
#[derive(Debug, Clone, Copy)]
pub struct HepteractEffectiveInput {
    /// `hepteracts[k].BAL` — raw accumulated value.
    pub raw_amount: f64,
    /// `hepteracts[k].LIMIT` — threshold past which DR applies.
    pub limit: f64,
    /// `hepteracts[k].DR + hepteracts[k].DR_INCREASE()` — combined
    /// diminishing-returns exponent. The UI tier evaluates
    /// `DR_INCREASE` since it can depend on upgrade state; this module
    /// gets the resolved scalar.
    pub dr_exponent: f64,
    /// `true` when this is the `quark` hept. Quark hept has a custom
    /// non-polynomial formula owned by the UI tier; logic just passes
    /// `BAL` through.
    pub is_quark: bool,
}

/// Effective hepteract value with diminishing returns past `LIMIT`.
///
/// - **quark**: just returns `raw_amount` (the UI tier owns the custom
///   formula).
/// - **`raw_amount ≤ LIMIT`**: linear, returns `raw_amount`.
/// - **`raw_amount > LIMIT`**:
///   `LIMIT * (raw_amount / LIMIT) ^ dr_exponent` — the value past
///   `LIMIT` is softened by the DR exponent (`dr_exponent < 1` for
///   most hepts, so growth past `LIMIT` is sub-linear).
#[must_use]
pub fn hepteract_effective(input: &HepteractEffectiveInput) -> f64 {
    if input.is_quark {
        return input.raw_amount;
    }
    let mut effective_value = input.raw_amount.min(input.limit);
    if input.raw_amount > input.limit {
        effective_value *= (input.raw_amount / input.limit).powf(input.dr_exponent);
    }
    effective_value
}

/// Player's expanded hepteract cap before the Exalt 3 doubling.
/// `BASE_CAP * 2^TIMES_CAP_EXTENDED` — each expansion doubles the cap.
#[must_use]
pub fn hepteract_cap(base_cap: f64, times_cap_extended: f64) -> f64 {
    2.0_f64.powf(times_cap_extended) * base_cap
}

/// The cap actually used by the UI — multiplies [`hepteract_cap`] by 2
/// if the limitedAscensions (Exalt 3) `hepteractCap` reward is active,
/// else 1.
#[must_use]
pub fn hepteract_final_cap(
    base_cap: f64,
    times_cap_extended: f64,
    exalt_3_hepteract_cap: bool,
) -> f64 {
    let special_multiplier = if exalt_3_hepteract_cap { 2.0 } else { 1.0 };
    hepteract_cap(base_cap, times_cap_extended) * special_multiplier
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hepteract_effective_quark_passes_through() {
        let input = HepteractEffectiveInput {
            raw_amount: 1e18,
            limit: 100.0,
            dr_exponent: 0.5,
            is_quark: true,
        };
        assert_eq!(hepteract_effective(&input), 1e18);
    }

    #[test]
    fn hepteract_effective_below_limit_is_linear() {
        let input = HepteractEffectiveInput {
            raw_amount: 50.0,
            limit: 100.0,
            dr_exponent: 0.5,
            is_quark: false,
        };
        assert_eq!(hepteract_effective(&input), 50.0);
    }

    #[test]
    fn hepteract_effective_above_limit_uses_dr() {
        // raw = 400, limit = 100, dr = 0.5 → 100 * (400/100)^0.5 = 200
        let input = HepteractEffectiveInput {
            raw_amount: 400.0,
            limit: 100.0,
            dr_exponent: 0.5,
            is_quark: false,
        };
        assert!((hepteract_effective(&input) - 200.0).abs() < 1e-9);
    }

    #[test]
    fn hepteract_effective_at_exact_limit() {
        // raw = limit → returns limit (no DR branch)
        let input = HepteractEffectiveInput {
            raw_amount: 100.0,
            limit: 100.0,
            dr_exponent: 0.5,
            is_quark: false,
        };
        assert_eq!(hepteract_effective(&input), 100.0);
    }

    #[test]
    fn hepteract_cap_doubles_per_extension() {
        assert_eq!(hepteract_cap(1_000.0, 0.0), 1_000.0);
        assert_eq!(hepteract_cap(1_000.0, 1.0), 2_000.0);
        assert_eq!(hepteract_cap(1_000.0, 5.0), 32_000.0);
    }

    #[test]
    fn hepteract_final_cap_exalt_3_doubles() {
        let without = hepteract_final_cap(1_000.0, 3.0, false);
        let with_exalt = hepteract_final_cap(1_000.0, 3.0, true);
        // 1000 * 2^3 = 8000; with exalt → 16000
        assert_eq!(without, 8_000.0);
        assert_eq!(with_exalt, 16_000.0);
    }
}
