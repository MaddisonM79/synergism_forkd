//! Per-rune-spirit effect formulas.
//!
//! Verbatim port of
//! `legacy_core_split/packages/logic/src/mechanics/runeSpiritEffects.ts`.
//! Pure 1-parameter functions extracted from the
//! `runeSpirits.<key>.effects` fields in the legacy
//! `packages/web_ui/src/RuneSpirits.ts`.
//!
//! All five spirits (speed, duplication, prism, thrift,
//! superiorIntellect) take a single `level` argument and return a
//! struct with one effect field. Four of them share the same
//! `1 + level / 1e9` shape. Prism is the odd one out — `level / 1e9`
//! with no `1 +` prefix, because `crystal_caps` is an additive cap
//! bonus rather than a multiplier.

/// Result of [`speed_rune_spirit_effects`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpeedRuneSpiritEffects {
    /// Global-speed multiplier.
    pub global_speed: f64,
}

/// `globalSpeed = 1 + level / 1e9`.
#[must_use]
pub fn speed_rune_spirit_effects(level: f64) -> SpeedRuneSpiritEffects {
    SpeedRuneSpiritEffects {
        global_speed: 1.0 + level / 1_000_000_000.0,
    }
}

/// Result of [`duplication_rune_spirit_effects`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DuplicationRuneSpiritEffects {
    /// Wow-cubes multiplier.
    pub wow_cubes: f64,
}

/// `wowCubes = 1 + level / 1e9`.
#[must_use]
pub fn duplication_rune_spirit_effects(level: f64) -> DuplicationRuneSpiritEffects {
    DuplicationRuneSpiritEffects {
        wow_cubes: 1.0 + level / 1_000_000_000.0,
    }
}

/// Result of [`prism_rune_spirit_effects`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PrismRuneSpiritEffects {
    /// Additive crystal-cap bonus (not a multiplier — no `1 +` prefix).
    pub crystal_caps: f64,
}

/// `crystalCaps = level / 1e9` (additive — no `1 +` prefix).
#[must_use]
pub fn prism_rune_spirit_effects(level: f64) -> PrismRuneSpiritEffects {
    PrismRuneSpiritEffects {
        crystal_caps: level / 1_000_000_000.0,
    }
}

/// Result of [`thrift_rune_spirit_effects`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ThriftRuneSpiritEffects {
    /// Offerings multiplier.
    pub offerings: f64,
}

/// `offerings = 1 + level / 1e9`.
#[must_use]
pub fn thrift_rune_spirit_effects(level: f64) -> ThriftRuneSpiritEffects {
    ThriftRuneSpiritEffects {
        offerings: 1.0 + level / 1_000_000_000.0,
    }
}

/// Result of [`superior_intellect_rune_spirit_effects`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SuperiorIntellectRuneSpiritEffects {
    /// Obtainium multiplier.
    pub obtainium: f64,
}

/// `obtainium = 1 + level / 1e9`.
#[must_use]
pub fn superior_intellect_rune_spirit_effects(level: f64) -> SuperiorIntellectRuneSpiritEffects {
    SuperiorIntellectRuneSpiritEffects {
        obtainium: 1.0 + level / 1_000_000_000.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spirit_speed_zero_is_one() {
        assert_eq!(speed_rune_spirit_effects(0.0).global_speed, 1.0);
    }

    #[test]
    fn spirit_speed_one_billion_is_two() {
        assert!((speed_rune_spirit_effects(1e9).global_speed - 2.0).abs() < 1e-12);
    }

    #[test]
    fn spirit_prism_has_no_one_prefix() {
        // Distinctive shape — additive cap, not multiplier.
        assert_eq!(prism_rune_spirit_effects(0.0).crystal_caps, 0.0);
        assert!((prism_rune_spirit_effects(1e9).crystal_caps - 1.0).abs() < 1e-12);
    }

    #[test]
    fn spirit_duplication_matches_shape() {
        assert!((duplication_rune_spirit_effects(500_000_000.0).wow_cubes - 1.5).abs() < 1e-12);
    }

    #[test]
    fn spirit_thrift_matches_shape() {
        assert!((thrift_rune_spirit_effects(500_000_000.0).offerings - 1.5).abs() < 1e-12);
    }

    #[test]
    fn spirit_si_matches_shape() {
        assert!(
            (superior_intellect_rune_spirit_effects(500_000_000.0).obtainium - 1.5).abs() < 1e-12
        );
    }
}
