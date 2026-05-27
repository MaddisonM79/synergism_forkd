//! Per-talisman effect formulas.
//!
//! Verbatim port of
//! `legacy/core_split/packages/logic/src/mechanics/talismanEffects.ts`.
//! Pure 1-arg functions extracted from the
//! `talismans.<key>.effects` fields in the legacy
//! `packages/web_ui/src/Talismans.ts`. Each talisman has a rarity tier
//! `n` (0..=10) that indexes into a small per-talisman lookup array;
//! for `n >= 6` (the "signature" tier) most also unlock an additional
//! effect.
//!
//! The 11 inscript-value arrays move here as module-level constants
//! since they're pure data.

const EXEMPTION_INSCRIPT_VALUES: [f64; 11] = [
    0.0, -0.2, -0.3, -0.4, -0.45, -0.5, -0.55, -0.6, -0.61, -0.62, -0.65,
];
const CHRONOS_INSCRIPT_VALUES: [f64; 11] = [
    1.0, 1.04, 1.08, 1.12, 1.16, 1.20, 1.25, 1.30, 1.325, 1.35, 1.4,
];
const MIDAS_INSCRIPT_VALUES: [f64; 11] = [
    1.0, 1.04, 1.08, 1.12, 1.16, 1.20, 1.25, 1.30, 1.325, 1.35, 1.40,
];
const METAPHYSICS_INSCRIPT_VALUES: [f64; 11] =
    [1.0, 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 1.7, 1.8, 1.9, 2.0];
const POLYMATH_INSCRIPT_VALUES: [f64; 11] = [
    1.0, 1.04, 1.08, 1.12, 1.16, 1.20, 1.25, 1.30, 1.325, 1.35, 1.40,
];
const MORTUUS_INSCRIPT_VALUES: [f64; 11] =
    [1.0, 1.05, 1.1, 1.15, 1.2, 1.3, 1.4, 1.5, 1.65, 1.8, 2.0];
const PLASTIC_INSCRIPT_VALUES: [f64; 11] = [
    1.0, 1.005, 1.01, 1.015, 1.02, 1.025, 1.03, 1.04, 1.045, 1.05, 1.0666,
];
const WOW_SQUARE_INSCRIPT_VALUES: [f64; 11] = [
    1.0, 1.025, 1.05, 1.075, 1.1, 1.125, 1.15, 1.2, 1.225, 1.25, 1.30,
];
const ACHIEVEMENT_EFFECT_INSCRIPT_VALUES: [f64; 11] = [
    0.0, 0.001, 0.002, 0.003, 0.004, 0.006, 0.008, 0.01, 0.015, 0.02, 0.03,
];
const COOKIE_GRANDMA_INSCRIPT_VALUES: [f64; 11] = [
    0.0, 0.01, 0.02, 0.03, 0.04, 0.05, 0.06, 0.07, 0.08, 0.09, 0.10,
];
const HORSE_SHOE_INSCRIPT_VALUES: [f64; 11] = [
    0.0, 0.001, 0.002, 0.003, 0.004, 0.005, 0.007, 0.01, 0.012, 0.015, 0.02,
];

/// Look up `array[n]` returning `fallback` for out-of-range indices —
/// mirrors the legacy `array[n] ?? fallback` shape.
fn lookup_or(array: &[f64; 11], n: i32, fallback: f64) -> f64 {
    if (0..=10).contains(&n) {
        array[n as usize]
    } else {
        fallback
    }
}

/// Result of [`exemption_talisman_effects`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExemptionTalismanEffects {
    /// Tax-reduction contribution (negative value scales the
    /// `exemptionTalismanTaxReduction` slot in `calculate_tax`).
    pub tax_reduction: f64,
    /// Duplication-rune OOM bonus at signature tier.
    pub duplication_oom_bonus: f64,
}

#[must_use]
pub fn exemption_talisman_effects(n: i32) -> ExemptionTalismanEffects {
    ExemptionTalismanEffects {
        tax_reduction: lookup_or(&EXEMPTION_INSCRIPT_VALUES, n, 0.0),
        duplication_oom_bonus: if n >= 6 { 12.0 } else { 0.0 },
    }
}

/// Result of [`chronos_talisman_effects`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChronosTalismanEffects {
    /// Global-speed multiplier.
    pub global_speed: f64,
    /// Speed-rune OOM bonus at signature tier.
    pub speed_oom_bonus: f64,
}

#[must_use]
pub fn chronos_talisman_effects(n: i32) -> ChronosTalismanEffects {
    ChronosTalismanEffects {
        global_speed: lookup_or(&CHRONOS_INSCRIPT_VALUES, n, 1.0),
        speed_oom_bonus: if n >= 6 { 12.0 } else { 0.0 },
    }
}

/// Result of [`midas_talisman_effects`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MidasTalismanEffects {
    /// Cube blessing-bonus multiplier.
    pub blessing_bonus: f64,
    /// Thrift-rune OOM bonus at signature tier.
    pub thrift_oom_bonus: f64,
}

#[must_use]
pub fn midas_talisman_effects(n: i32) -> MidasTalismanEffects {
    MidasTalismanEffects {
        blessing_bonus: lookup_or(&MIDAS_INSCRIPT_VALUES, n, 1.0),
        thrift_oom_bonus: if n >= 6 { 12.0 } else { 0.0 },
    }
}

/// Result of [`metaphysics_talisman_effects`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MetaphysicsTalismanEffects {
    /// Talisman effect multiplier.
    pub talisman_effect: f64,
    /// Extra talisman effect at signature tier (`1.07`, else `1`).
    pub extra_talisman_effect: f64,
}

#[must_use]
pub fn metaphysics_talisman_effects(n: i32) -> MetaphysicsTalismanEffects {
    MetaphysicsTalismanEffects {
        talisman_effect: lookup_or(&METAPHYSICS_INSCRIPT_VALUES, n, 1.0),
        extra_talisman_effect: if n >= 6 { 1.07 } else { 1.0 },
    }
}

/// Result of [`polymath_talisman_effects`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PolymathTalismanEffects {
    /// Ascension-speed bonus.
    pub ascension_speed_bonus: f64,
    /// SI-rune OOM bonus at signature tier.
    pub si_oom_bonus: f64,
}

#[must_use]
pub fn polymath_talisman_effects(n: i32) -> PolymathTalismanEffects {
    PolymathTalismanEffects {
        ascension_speed_bonus: lookup_or(&POLYMATH_INSCRIPT_VALUES, n, 1.0),
        si_oom_bonus: if n >= 6 { 12.0 } else { 0.0 },
    }
}

/// Result of [`mortuus_talisman_effects`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MortuusTalismanEffects {
    /// Ant-bonus multiplier.
    pub ant_bonus: f64,
    /// Prism-rune OOM bonus at signature tier.
    pub prism_oom_bonus: f64,
}

#[must_use]
pub fn mortuus_talisman_effects(n: i32) -> MortuusTalismanEffects {
    MortuusTalismanEffects {
        ant_bonus: lookup_or(&MORTUUS_INSCRIPT_VALUES, n, 1.0),
        prism_oom_bonus: if n >= 6 { 12.0 } else { 0.0 },
    }
}

/// Result of [`plastic_talisman_effects`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlasticTalismanEffects {
    /// Quark bonus multiplier.
    pub quark_bonus: f64,
}

#[must_use]
pub fn plastic_talisman_effects(n: i32) -> PlasticTalismanEffects {
    PlasticTalismanEffects {
        quark_bonus: lookup_or(&PLASTIC_INSCRIPT_VALUES, n, 1.0),
    }
}

/// Result of [`wow_square_talisman_effects`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WowSquareTalismanEffects {
    /// Even-dimension bonus multiplier.
    pub even_dim_bonus: f64,
    /// Odd-dimension bonus at signature tier.
    pub odd_dim_bonus: f64,
}

#[must_use]
pub fn wow_square_talisman_effects(n: i32) -> WowSquareTalismanEffects {
    WowSquareTalismanEffects {
        even_dim_bonus: lookup_or(&WOW_SQUARE_INSCRIPT_VALUES, n, 1.0),
        odd_dim_bonus: if n >= 6 { 1.20 } else { 1.0 },
    }
}

/// Result of [`achievement_talisman_effects`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AchievementTalismanEffects {
    /// Positive-salvage multiplier (additive).
    pub positive_salvage_mult: f64,
    /// Negative-salvage multiplier at signature tier (`-0.02`, else
    /// `0`).
    pub negative_salvage_mult: f64,
}

#[must_use]
pub fn achievement_talisman_effects(n: i32) -> AchievementTalismanEffects {
    AchievementTalismanEffects {
        positive_salvage_mult: lookup_or(&ACHIEVEMENT_EFFECT_INSCRIPT_VALUES, n, 1.0),
        negative_salvage_mult: if n >= 6 { -0.02 } else { 0.0 },
    }
}

/// Result of [`cookie_grandma_talisman_effects`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CookieGrandmaTalismanEffects {
    /// Free corruption-level contribution.
    pub free_corruption_level: f64,
    /// `cookieSix` unlocked at signature tier.
    pub cookie_six: bool,
}

#[must_use]
pub fn cookie_grandma_talisman_effects(n: i32) -> CookieGrandmaTalismanEffects {
    CookieGrandmaTalismanEffects {
        free_corruption_level: lookup_or(&COOKIE_GRANDMA_INSCRIPT_VALUES, n, 0.0),
        cookie_six: n >= 6,
    }
}

/// Result of [`horse_shoe_talisman_effects`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HorseShoeTalismanEffects {
    /// Luck percentage (additive).
    pub luck_percentage: f64,
    /// Red-luck contribution at signature tier (`40`, else `0`).
    pub red_luck: f64,
}

#[must_use]
pub fn horse_shoe_talisman_effects(n: i32) -> HorseShoeTalismanEffects {
    HorseShoeTalismanEffects {
        luck_percentage: lookup_or(&HORSE_SHOE_INSCRIPT_VALUES, n, 0.0),
        red_luck: if n >= 6 { 40.0 } else { 0.0 },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exemption_tier_0_is_zero_no_bonus() {
        let e = exemption_talisman_effects(0);
        assert_eq!(e.tax_reduction, 0.0);
        assert_eq!(e.duplication_oom_bonus, 0.0);
    }

    #[test]
    fn exemption_tier_5_no_signature_bonus() {
        let e = exemption_talisman_effects(5);
        assert_eq!(e.tax_reduction, -0.5);
        assert_eq!(e.duplication_oom_bonus, 0.0);
    }

    #[test]
    fn exemption_tier_6_unlocks_signature_bonus() {
        let e = exemption_talisman_effects(6);
        assert_eq!(e.tax_reduction, -0.55);
        assert_eq!(e.duplication_oom_bonus, 12.0);
    }

    #[test]
    fn exemption_out_of_range_returns_fallback() {
        let e = exemption_talisman_effects(11);
        assert_eq!(e.tax_reduction, 0.0); // fallback
        assert!(e.duplication_oom_bonus > 0.0); // n >= 6 is still true
    }

    #[test]
    fn exemption_negative_index_returns_fallback() {
        let e = exemption_talisman_effects(-1);
        assert_eq!(e.tax_reduction, 0.0);
        assert_eq!(e.duplication_oom_bonus, 0.0);
    }

    #[test]
    fn chronos_tier_5_global_speed_is_1_20() {
        let e = chronos_talisman_effects(5);
        assert_eq!(e.global_speed, 1.20);
    }

    #[test]
    fn cookie_grandma_signature_flag() {
        assert!(!cookie_grandma_talisman_effects(5).cookie_six);
        assert!(cookie_grandma_talisman_effects(6).cookie_six);
        assert!(cookie_grandma_talisman_effects(10).cookie_six);
    }

    #[test]
    fn plastic_tier_10_max_quark_bonus() {
        assert_eq!(plastic_talisman_effects(10).quark_bonus, 1.0666);
    }

    #[test]
    fn metaphysics_signature_extra_effect() {
        // n=5 → 1, n=6 → 1.07
        assert_eq!(metaphysics_talisman_effects(5).extra_talisman_effect, 1.0);
        assert_eq!(metaphysics_talisman_effects(6).extra_talisman_effect, 1.07);
    }

    #[test]
    fn achievement_negative_salvage_only_at_signature() {
        assert_eq!(achievement_talisman_effects(5).negative_salvage_mult, 0.0);
        assert_eq!(achievement_talisman_effects(6).negative_salvage_mult, -0.02);
    }

    #[test]
    fn horse_shoe_signature_red_luck() {
        assert_eq!(horse_shoe_talisman_effects(5).red_luck, 0.0);
        assert_eq!(horse_shoe_talisman_effects(6).red_luck, 40.0);
    }
}
