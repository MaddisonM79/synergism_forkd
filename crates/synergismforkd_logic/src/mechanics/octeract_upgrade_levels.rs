//! Octeract upgrade effective-level math.
//!
//! Verbatim port of
//! `legacy/core_split/packages/logic/src/mechanics/octeractUpgradeLevels.ts`.
//! The UI tier owns the OcteractUpgrades data tables and the buy/UI
//! flow; this module owns the pure formulas that take a per-upgrade
//! `level` / `freeLevel` / `qualityOfLife` snapshot plus
//! player-state inputs and return the effective level used by
//! effect lookups and cost-to-next-level calculations.
//!
//! The same name `compute_free_level_multiplier` exists in
//! [`crate::mechanics::gq_upgrade_levels`] for golden-quark
//! upgrades, but the formula differs (GQ reads shop + `cube[75]`;
//! Octeract reads `cube[78]`).

/// Octeract free-level multiplier. `1 + 0.3% × cubeUpgrades[78]`.
#[must_use]
pub fn octeract_free_level_multiplier(cube_upgrade_78: f64) -> f64 {
    1.0 + 0.3 / 100.0 * cube_upgrade_78
}

/// Softcap on the effective free levels for one octeract upgrade.
/// `free_level × free_level_mult`.
#[must_use]
pub fn octeract_free_level_softcap(free_level: f64, free_level_mult: f64) -> f64 {
    free_level * free_level_mult
}

/// Inputs to [`actual_octeract_upgrade_total_levels`].
#[derive(Debug, Clone, Copy)]
pub struct ActualOcteractUpgradeTotalLevelsInput {
    /// `octeractUpgrades[k].level` — purchased level.
    pub level: f64,
    /// `octeractUpgrades[k].freeLevel` — accumulated free levels.
    pub free_level: f64,
    /// `octeractUpgrades[k].qualityOfLife`. When `false`, the
    /// upgrade is gated off inside noOcteracts / sadisticPrequel.
    pub quality_of_life: bool,
    /// `player.cubeUpgrades[78]`.
    pub cube_upgrade_78: f64,
    /// `player.singularityChallenges.noOcteracts.enabled`.
    pub in_no_octeracts: bool,
    /// `player.singularityChallenges.sadisticPrequel.enabled`.
    pub in_sadistic_prequel: bool,
}

/// Effective total level for one octeract upgrade.
///
/// - Returns `0` if the player is in `noOcteracts` or
///   `sadisticPrequel` AND the upgrade isn't `qualityOfLife`.
/// - Otherwise: when `level >= actual_free_levels`, returns
///   `actual_free_levels + level` (linear sum). When
///   `level < actual_free_levels`, returns
///   `2 × sqrt(actual_free_levels × level)` — a smoother softcap
///   that matches the linear formula at the boundary.
#[must_use]
pub fn actual_octeract_upgrade_total_levels(input: &ActualOcteractUpgradeTotalLevelsInput) -> f64 {
    if (input.in_no_octeracts || input.in_sadistic_prequel) && !input.quality_of_life {
        return 0.0;
    }

    let free_level_mult = octeract_free_level_multiplier(input.cube_upgrade_78);
    let actual_free_levels = octeract_free_level_softcap(input.free_level, free_level_mult);

    if input.level >= actual_free_levels {
        actual_free_levels + input.level
    } else {
        2.0 * (actual_free_levels * input.level).sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn free_level_mult_at_zero() {
        assert_eq!(octeract_free_level_multiplier(0.0), 1.0);
    }

    #[test]
    fn free_level_mult_includes_cube78_bonus() {
        // cube=100 → 1 + 0.003*100 = 1.3
        assert!((octeract_free_level_multiplier(100.0) - 1.3).abs() < 1e-12);
    }

    #[test]
    fn total_levels_gated_off_in_no_octeracts_for_non_qol() {
        let result = actual_octeract_upgrade_total_levels(&ActualOcteractUpgradeTotalLevelsInput {
            level: 100.0,
            free_level: 100.0,
            quality_of_life: false,
            cube_upgrade_78: 0.0,
            in_no_octeracts: true,
            in_sadistic_prequel: false,
        });
        assert_eq!(result, 0.0);
    }

    #[test]
    fn total_levels_qol_survives_no_octeracts() {
        let result = actual_octeract_upgrade_total_levels(&ActualOcteractUpgradeTotalLevelsInput {
            level: 10.0,
            free_level: 0.0,
            quality_of_life: true,
            cube_upgrade_78: 0.0,
            in_no_octeracts: true,
            in_sadistic_prequel: false,
        });
        assert_eq!(result, 10.0);
    }

    #[test]
    fn total_levels_linear_above_free() {
        // level=20, free=10, mult=1 → actual_free=10; linear=30
        let result = actual_octeract_upgrade_total_levels(&ActualOcteractUpgradeTotalLevelsInput {
            level: 20.0,
            free_level: 10.0,
            quality_of_life: false,
            cube_upgrade_78: 0.0,
            in_no_octeracts: false,
            in_sadistic_prequel: false,
        });
        assert_eq!(result, 30.0);
    }

    #[test]
    fn total_levels_sqrt_below_free() {
        // level=4, free=100, mult=1 → actual=100; 4<100 → 2*sqrt(100*4) = 2*20 = 40
        let result = actual_octeract_upgrade_total_levels(&ActualOcteractUpgradeTotalLevelsInput {
            level: 4.0,
            free_level: 100.0,
            quality_of_life: false,
            cube_upgrade_78: 0.0,
            in_no_octeracts: false,
            in_sadistic_prequel: false,
        });
        assert_eq!(result, 40.0);
    }

    #[test]
    fn total_levels_continuous_at_boundary() {
        // At level == actual_free, both formulas should agree
        let at = actual_octeract_upgrade_total_levels(&ActualOcteractUpgradeTotalLevelsInput {
            level: 100.0,
            free_level: 100.0,
            quality_of_life: false,
            cube_upgrade_78: 0.0,
            in_no_octeracts: false,
            in_sadistic_prequel: false,
        });
        // Linear branch: 100 + 100 = 200
        assert_eq!(at, 200.0);
    }
}
