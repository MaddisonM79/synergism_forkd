//! Per-rune-blessing effect formulas.
//!
//! Verbatim port of
//! `legacy_core_split/packages/logic/src/mechanics/runeBlessingEffects.ts`.
//! Pure 1-parameter functions extracted from the
//! `runeBlessings.<key>.effects` fields in the legacy
//! `packages/web_ui/src/RuneBlessings.ts`.
//!
//! All five blessings (speed, duplication, prism, thrift,
//! superiorIntellect) take a single `level` argument and return a struct
//! with one effect field. Four of them share the same `1 + level / 1e6`
//! shape; the fifth (SI) uses a logarithm.

/// Result of [`speed_rune_blessing_effects`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpeedRuneBlessingEffects {
    /// Global-speed multiplier.
    pub global_speed: f64,
}

/// `globalSpeed = 1 + level / 1e6`.
#[must_use]
pub fn speed_rune_blessing_effects(level: f64) -> SpeedRuneBlessingEffects {
    SpeedRuneBlessingEffects {
        global_speed: 1.0 + level / 1_000_000.0,
    }
}

/// Result of [`duplication_rune_blessing_effects`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DuplicationRuneBlessingEffects {
    /// Multiplier-boost multiplier.
    pub multiplier_boosts: f64,
}

/// `multiplierBoosts = 1 + level / 1e6`.
#[must_use]
pub fn duplication_rune_blessing_effects(level: f64) -> DuplicationRuneBlessingEffects {
    DuplicationRuneBlessingEffects {
        multiplier_boosts: 1.0 + level / 1_000_000.0,
    }
}

/// Result of [`prism_rune_blessing_effects`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PrismRuneBlessingEffects {
    /// Ant-sacrifice multiplier.
    pub ant_sacrifice_mult: f64,
}

/// `antSacrificeMult = 1 + level / 1e6`.
#[must_use]
pub fn prism_rune_blessing_effects(level: f64) -> PrismRuneBlessingEffects {
    PrismRuneBlessingEffects {
        ant_sacrifice_mult: 1.0 + level / 1_000_000.0,
    }
}

/// Result of [`thrift_rune_blessing_effects`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ThriftRuneBlessingEffects {
    /// Accelerator-boost cost-delay multiplier — pushes back the
    /// quadratic-growth threshold inside
    /// [`crate::mechanics::accelerator_boosts`].
    pub accel_boost_cost_delay: f64,
}

/// `accelBoostCostDelay = 1 + level / 1e6`.
#[must_use]
pub fn thrift_rune_blessing_effects(level: f64) -> ThriftRuneBlessingEffects {
    ThriftRuneBlessingEffects {
        accel_boost_cost_delay: 1.0 + level / 1_000_000.0,
    }
}

/// Result of [`superior_intellect_rune_blessing_effects`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SuperiorIntellectRuneBlessingEffects {
    /// Obtainium-to-ant-exponent contribution.
    pub obt_to_ant_exponent: f64,
}

/// `obtToAntExponent = ln(1 + level / 1e6)`. Note: natural log, not
/// log-base-10.
#[must_use]
pub fn superior_intellect_rune_blessing_effects(
    level: f64,
) -> SuperiorIntellectRuneBlessingEffects {
    SuperiorIntellectRuneBlessingEffects {
        obt_to_ant_exponent: (1.0 + level / 1_000_000.0).ln(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blessing_speed_zero_level_is_one() {
        assert_eq!(speed_rune_blessing_effects(0.0).global_speed, 1.0);
    }

    #[test]
    fn blessing_speed_one_million_level_is_two() {
        assert!((speed_rune_blessing_effects(1_000_000.0).global_speed - 2.0).abs() < 1e-12);
    }

    #[test]
    fn blessing_duplication_matches_shape() {
        let e = duplication_rune_blessing_effects(500_000.0);
        assert!((e.multiplier_boosts - 1.5).abs() < 1e-12);
    }

    #[test]
    fn blessing_prism_matches_shape() {
        let e = prism_rune_blessing_effects(500_000.0);
        assert!((e.ant_sacrifice_mult - 1.5).abs() < 1e-12);
    }

    #[test]
    fn blessing_thrift_matches_shape() {
        let e = thrift_rune_blessing_effects(500_000.0);
        assert!((e.accel_boost_cost_delay - 1.5).abs() < 1e-12);
    }

    #[test]
    fn blessing_si_uses_natural_log() {
        // ln(2) ≈ 0.6931
        let e = superior_intellect_rune_blessing_effects(1_000_000.0);
        assert!((e.obt_to_ant_exponent - std::f64::consts::LN_2).abs() < 1e-12);
    }
}
