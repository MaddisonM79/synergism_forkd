//! Per-reward level scaling formulas.
//!
//! Verbatim port of
//! `legacy_core_split/packages/logic/src/mechanics/levelRewards.ts`
//! (lifted from the legacy `packages/web_ui/src/Levels.ts`). Each
//! reward is a pure `(level: f64) -> f64` paired with the
//! `min_level` at which it activates and the `default_value`
//! returned below that.

/// Key for [`get_level_reward`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LevelRewardKey {
    /// Stepwise additive salvage (from level 0).
    Salvage,
    /// Quark multiplier (from level 20).
    Quarks,
    /// Offering multiplier (from level 0).
    Offerings,
    /// Obtainium multiplier (from level 15).
    Obtainium,
    /// Ant ELO bonus (from level 60).
    Ants,
    /// Wow-cubes multiplier (from level 70).
    WowCubes,
    /// Wow-tesseracts multiplier (from level 90).
    WowTesseracts,
    /// Wow-hypercubes multiplier (from level 110).
    WowHyperCubes,
    /// Wow-platonic-cubes multiplier (from level 140).
    WowPlatonicCubes,
    /// Wow-hepteract-cubes multiplier (from level 170).
    WowHepteractCubes,
    /// Wow-octeracts multiplier (from level 210).
    WowOcteracts,
    /// Ambrosia luck (from level 230).
    AmbrosiaLuck,
    /// Red-ambrosia luck (from level 260).
    RedAmbrosiaLuck,
}

// ─── Per-reward effect formulas ────────────────────────────────────────────

/// Salvage grows by `1/level` for the first 100 levels, then `2/level`
/// for the next 100, then `3/level`, etc. Stepwise loop that
/// subtracts a 100-level block per pass, bumping the per-level rate
/// by 1.
#[must_use]
pub fn salvage_effect(level: f64) -> f64 {
    let mut salvage = 0.0_f64;
    let mut salvage_per_level = 1.0_f64;
    let mut remaining_levels = level;
    while remaining_levels >= 100.0 {
        salvage += salvage_per_level * 100.0;
        remaining_levels -= 100.0;
        salvage_per_level += 1.0;
    }
    salvage += salvage_per_level * remaining_levels;
    salvage
}

/// Quark multiplier — `1.01^floor(level / 20)`. Steps up every 20
/// levels.
#[must_use]
pub fn quarks_effect(level: f64) -> f64 {
    1.01_f64.powf((level / 20.0).floor())
}

/// Offering multiplier — `1.01^level * 1.02^max(0, level - 100)`.
#[must_use]
pub fn offerings_effect(level: f64) -> f64 {
    1.01_f64.powf(level) * 1.02_f64.powf((level - 100.0).max(0.0))
}

/// Obtainium multiplier —
/// `1.01^(level - 15) * 1.02^max(0, level - 100)`.
#[must_use]
pub fn obtainium_effect(level: f64) -> f64 {
    1.01_f64.powf(level - 15.0) * 1.02_f64.powf((level - 100.0).max(0.0))
}

/// Ant ELO bonus. Three-band linear formula:
/// - Levels 60..=130: `+25` per level (capped at 71 levels)
/// - Levels 100..=200: `+50` per level (capped at 100 levels)
/// - Levels 200+: `+100` per level
///
/// The three contributions sum together; the visible behavior is the
/// `25 / 50 / 100` step at each band.
#[must_use]
pub fn ants_effect(level: f64) -> f64 {
    let first_100_levels = 71.0_f64.min(level - 59.0) * 25.0;
    let next_100_levels = 0.0_f64.max(100.0_f64.min(level - 100.0)) * 50.0;
    let remaining_levels = 0.0_f64.max(level - 200.0) * 100.0;
    first_100_levels + next_100_levels + remaining_levels
}

/// Shape shared by the six wow-cube rewards:
///
/// ```text
/// (1 + (level - linear_offset) / 20) * 1.07 ^ (floor(level / 10) - tenth_offset)
/// ```
///
/// Each cube tier has different offsets and `min_level` — see the
/// per-cube exports below.
fn wow_cube_shaped_effect(level: f64, linear_offset: f64, tenth_offset: f64) -> f64 {
    (1.0 + (level - linear_offset) / 20.0) * 1.07_f64.powf((level / 10.0).floor() - tenth_offset)
}

/// Wow-cubes multiplier.
#[must_use]
pub fn wow_cubes_effect(level: f64) -> f64 {
    wow_cube_shaped_effect(level, 60.0, 6.0)
}

/// Wow-tesseracts multiplier.
#[must_use]
pub fn wow_tesseracts_effect(level: f64) -> f64 {
    wow_cube_shaped_effect(level, 80.0, 8.0)
}

/// Wow-hypercubes multiplier.
#[must_use]
pub fn wow_hyper_cubes_effect(level: f64) -> f64 {
    wow_cube_shaped_effect(level, 100.0, 10.0)
}

/// Wow-platonic-cubes multiplier.
#[must_use]
pub fn wow_platonic_cubes_effect(level: f64) -> f64 {
    wow_cube_shaped_effect(level, 120.0, 12.0)
}

/// Wow-hepteract-cubes multiplier.
#[must_use]
pub fn wow_hepteract_cubes_effect(level: f64) -> f64 {
    wow_cube_shaped_effect(level, 150.0, 15.0)
}

/// Octeract multiplier — uses `1.02` base (not `1.07`) and a
/// per-level (not per-tenth) exponent.
#[must_use]
pub fn wow_octeracts_effect(level: f64) -> f64 {
    (1.0 + (level - 209.0) / 20.0) * 1.02_f64.powf(level - 209.0)
}

/// Flat `4` ambrosia luck per level past 229.
#[must_use]
pub fn ambrosia_luck_effect(level: f64) -> f64 {
    4.0 * (level - 229.0)
}

/// Flat `1` red-ambrosia luck per level past 259.
#[must_use]
pub fn red_ambrosia_luck_effect(level: f64) -> f64 {
    level - 259.0
}

// ─── Dispatcher ────────────────────────────────────────────────────────────

/// Returns the active reward value for a given achievement level.
/// Below the reward's `min_level`, returns the `default_value`;
/// otherwise invokes the reward's effect.
#[must_use]
pub fn get_level_reward(reward: LevelRewardKey, level: f64) -> f64 {
    use LevelRewardKey as K;
    match reward {
        K::Salvage => salvage_effect(level),
        K::Quarks => {
            if level >= 20.0 {
                quarks_effect(level)
            } else {
                1.0
            }
        }
        K::Offerings => offerings_effect(level),
        K::Obtainium => {
            if level >= 15.0 {
                obtainium_effect(level)
            } else {
                1.0
            }
        }
        K::Ants => {
            if level >= 60.0 {
                ants_effect(level)
            } else {
                1.0
            }
        }
        K::WowCubes => {
            if level >= 70.0 {
                wow_cubes_effect(level)
            } else {
                1.0
            }
        }
        K::WowTesseracts => {
            if level >= 90.0 {
                wow_tesseracts_effect(level)
            } else {
                1.0
            }
        }
        K::WowHyperCubes => {
            if level >= 110.0 {
                wow_hyper_cubes_effect(level)
            } else {
                1.0
            }
        }
        K::WowPlatonicCubes => {
            if level >= 140.0 {
                wow_platonic_cubes_effect(level)
            } else {
                1.0
            }
        }
        K::WowHepteractCubes => {
            if level >= 170.0 {
                wow_hepteract_cubes_effect(level)
            } else {
                1.0
            }
        }
        K::WowOcteracts => {
            if level >= 210.0 {
                wow_octeracts_effect(level)
            } else {
                1.0
            }
        }
        K::AmbrosiaLuck => {
            if level >= 230.0 {
                ambrosia_luck_effect(level)
            } else {
                0.0
            }
        }
        K::RedAmbrosiaLuck => {
            if level >= 260.0 {
                red_ambrosia_luck_effect(level)
            } else {
                0.0
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn salvage_at_50_is_50() {
        // 50 levels at 1/level → 50
        assert_eq!(salvage_effect(50.0), 50.0);
    }

    #[test]
    fn salvage_at_150_is_200() {
        // First 100 at 1/level = 100; next 50 at 2/level = 100; total 200
        assert_eq!(salvage_effect(150.0), 200.0);
    }

    #[test]
    fn salvage_at_300_is_600() {
        // 100*1 + 100*2 + 100*3 = 600
        assert_eq!(salvage_effect(300.0), 600.0);
    }

    #[test]
    fn quarks_steps_every_20_levels() {
        assert_eq!(quarks_effect(19.0), 1.0); // floor(19/20) = 0
                                              // 1.01^1 = 1.01
        assert!((quarks_effect(20.0) - 1.01).abs() < 1e-12);
        // 1.01^2 = 1.0201
        assert!((quarks_effect(40.0) - 1.0201).abs() < 1e-12);
    }

    #[test]
    fn offerings_below_100_is_simple_compound() {
        // 1.01^50 * 1.02^0
        assert!((offerings_effect(50.0) - 1.01_f64.powi(50)).abs() < 1e-12);
    }

    #[test]
    fn ants_at_120_first_band_only() {
        // first_100 = min(71, 120-59) * 25 = 61 * 25 = 1525
        // next_100 = max(0, min(100, 120-100)) * 50 = 20 * 50 = 1000
        // remaining = max(0, 120-200) * 100 = 0
        // total = 2525
        assert_eq!(ants_effect(120.0), 2_525.0);
    }

    #[test]
    fn get_level_reward_dispatches_with_gates() {
        assert_eq!(get_level_reward(LevelRewardKey::Quarks, 10.0), 1.0);
        assert!(get_level_reward(LevelRewardKey::Quarks, 30.0) > 1.0);
        assert_eq!(get_level_reward(LevelRewardKey::AmbrosiaLuck, 100.0), 0.0);
        assert_eq!(get_level_reward(LevelRewardKey::AmbrosiaLuck, 230.0), 4.0);
    }
}
