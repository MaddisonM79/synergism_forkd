//! Hepteract effect formulas — convert a hepteract balance into a
//! multiplier or stat.
//!
//! Verbatim port of
//! `legacy/core_split/packages/logic/src/mechanics/cubes/hepteracts.ts`.
//! 8 hepteract types; 7 of them produce their effect directly from
//! `hept` alone. The 8th (`quark`) raises its base to an exponent that
//! combines a fixed DR with a `DR_INCREASE` term that sums
//! contributions from several external effect sources. The shim
//! recomputes that exponent on each call and passes it in.

/// Result of [`chronos_hepteract_effects`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChronosHepteractEffects {
    /// Ascension-speed multiplier.
    pub ascension_speed: f64,
}

/// `ascensionSpeed = 1 + 6 * hept / 10_000`.
#[must_use]
pub fn chronos_hepteract_effects(hept: f64) -> ChronosHepteractEffects {
    ChronosHepteractEffects {
        ascension_speed: 1.0 + 6.0 * hept / 10_000.0,
    }
}

/// Result of [`hyperrealism_hepteract_effects`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HyperrealismHepteractEffects {
    /// Hypercube multiplier.
    pub hypercube_multiplier: f64,
}

/// `hypercubeMultiplier = 1 + 6 * hept / 10_000`.
#[must_use]
pub fn hyperrealism_hepteract_effects(hept: f64) -> HyperrealismHepteractEffects {
    HyperrealismHepteractEffects {
        hypercube_multiplier: 1.0 + 6.0 * hept / 10_000.0,
    }
}

/// Result of [`quark_hepteract_effects`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QuarkHepteractEffects {
    /// Quark multiplier.
    pub quark_multiplier: f64,
}

/// Quark hepteract — the only one whose exponent isn't a constant. It
/// sums a fixed DR with contributions from singularity / octeract /
/// shop upgrades. Callers precompute that sum and pass it in via
/// `dr_exponent` (= `DR + DR_INCREASE()`).
#[must_use]
pub fn quark_hepteract_effects(hept: f64, dr_exponent: f64) -> QuarkHepteractEffects {
    QuarkHepteractEffects {
        quark_multiplier: (1.0 + 0.2 * (1.0 + hept / 500.0).log2()).powf(dr_exponent),
    }
}

/// Result of [`challenge_hepteract_effects`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChallengeHepteractEffects {
    /// C15 score multiplier.
    pub c15_score_multiplier: f64,
}

/// `c15ScoreMultiplier = 1 + 5 * hept / 10_000`.
#[must_use]
pub fn challenge_hepteract_effects(hept: f64) -> ChallengeHepteractEffects {
    ChallengeHepteractEffects {
        c15_score_multiplier: 1.0 + 5.0 * hept / 10_000.0,
    }
}

/// Result of [`abyss_hepteract_effects`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AbyssHepteractEffects {
    /// Salvage additive contribution.
    pub salvage: f64,
}

/// `salvage = 0.1 * floor(10 * log2(max(1, hept * 2)))`. The `max(1, …)`
/// guard avoids `log2(0)` producing `-Infinity` at `hept = 0`.
#[must_use]
pub fn abyss_hepteract_effects(hept: f64) -> AbyssHepteractEffects {
    AbyssHepteractEffects {
        salvage: 0.1 * (10.0 * 1.0_f64.max(hept * 2.0).log2()).floor(),
    }
}

/// Result of [`accelerator_hepteract_effects`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AcceleratorHepteractEffects {
    /// Additive accelerator count.
    pub accelerators: f64,
    /// Accelerator multiplier.
    pub accelerator_multiplier: f64,
}

/// `accelerators = 2000 * hept`,
/// `acceleratorMultiplier = 1 + 3 * hept / 10_000`.
#[must_use]
pub fn accelerator_hepteract_effects(hept: f64) -> AcceleratorHepteractEffects {
    AcceleratorHepteractEffects {
        accelerators: 2_000.0 * hept,
        accelerator_multiplier: 1.0 + 3.0 * hept / 10_000.0,
    }
}

/// Result of [`accelerator_boost_hepteract_effects`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AcceleratorBoostHepteractEffects {
    /// Accelerator-boost multiplier.
    pub accelerator_boost_multiplier: f64,
}

/// `acceleratorBoostMultiplier = 1 + hept / 1_000`.
#[must_use]
pub fn accelerator_boost_hepteract_effects(hept: f64) -> AcceleratorBoostHepteractEffects {
    AcceleratorBoostHepteractEffects {
        accelerator_boost_multiplier: 1.0 + hept / 1_000.0,
    }
}

/// Result of [`multiplier_hepteract_effects`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MultiplierHepteractEffects {
    /// Additive multiplier count.
    pub multiplier: f64,
    /// Multiplier-multiplier (yes, the legacy name is recursive).
    pub multiplier_multiplier: f64,
}

/// `multiplier = 1000 * hept`,
/// `multiplierMultiplier = 1 + 3 * hept / 10_000`.
#[must_use]
pub fn multiplier_hepteract_effects(hept: f64) -> MultiplierHepteractEffects {
    MultiplierHepteractEffects {
        multiplier: 1_000.0 * hept,
        multiplier_multiplier: 1.0 + 3.0 * hept / 10_000.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chronos_zero_hept_is_one() {
        assert_eq!(chronos_hepteract_effects(0.0).ascension_speed, 1.0);
    }

    #[test]
    fn chronos_10k_hept_is_seven() {
        // 1 + 6 * 10000 / 10000 = 7
        assert_eq!(chronos_hepteract_effects(10_000.0).ascension_speed, 7.0);
    }

    #[test]
    fn quark_zero_hept_is_one_pow_dr_exponent() {
        // (1 + 0.2 * log2(1))^anything = 1
        assert_eq!(quark_hepteract_effects(0.0, 1.0).quark_multiplier, 1.0);
        assert_eq!(quark_hepteract_effects(0.0, 5.0).quark_multiplier, 1.0);
    }

    #[test]
    fn quark_exponent_amplifies_result() {
        // Higher dr_exponent → larger multiplier for the same hept
        let low = quark_hepteract_effects(1_000.0, 1.0).quark_multiplier;
        let high = quark_hepteract_effects(1_000.0, 2.0).quark_multiplier;
        assert!(high > low);
    }

    #[test]
    fn abyss_zero_hept_uses_log_guard() {
        // max(1, 0*2=0) = 1, log2(1) = 0, floor(0) = 0
        assert_eq!(abyss_hepteract_effects(0.0).salvage, 0.0);
    }

    #[test]
    fn abyss_above_one_hept_is_positive() {
        // 0.1 * floor(10 * log2(2)) = 0.1 * 10 = 1
        assert_eq!(abyss_hepteract_effects(1.0).salvage, 1.0);
    }

    #[test]
    fn accelerator_zero_hept() {
        let e = accelerator_hepteract_effects(0.0);
        assert_eq!(e.accelerators, 0.0);
        assert_eq!(e.accelerator_multiplier, 1.0);
    }

    #[test]
    fn accelerator_one_hept() {
        let e = accelerator_hepteract_effects(1.0);
        assert_eq!(e.accelerators, 2_000.0);
        assert!((e.accelerator_multiplier - 1.0003).abs() < 1e-9);
    }

    #[test]
    fn multiplier_one_hept() {
        let e = multiplier_hepteract_effects(1.0);
        assert_eq!(e.multiplier, 1_000.0);
        assert!((e.multiplier_multiplier - 1.0003).abs() < 1e-9);
    }
}
