//! Golden-quark upgrade effective-level math.
//!
//! Verbatim port of
//! `legacy_core_split/packages/logic/src/mechanics/gqUpgradeLevels.ts`.
//! The UI tier keeps the GQ upgrade data table and the buy/UI flow;
//! this module owns the pure formulas that convert per-upgrade
//! snapshot + player-state inputs into the effective level used by
//! effect lookups, the level cap, and the polynomial bonus path
//! unlocked by `octeractImprovedFree`.
//!
//! Octeract upgrades have parallel helpers in
//! `octeract_upgrade_levels`. The two `freeLevelMultiplier`
//! implementations differ (GQ reads shop + `cube[75]`; Octeract
//! reads `cube[78]`).

/// Singularity counts that unlock an extra `+1` to a GQ upgrade's
/// level cap when the upgrade is flagged `canExceedCap`. Sorted
/// ascending; the loop walks and stops at the first unmet
/// threshold.
const OVERCLOCK_PERKS: &[f64] = &[
    50.0, 60.0, 75.0, 100.0, 125.0, 150.0, 175.0, 200.0, 225.0, 250.0,
];

/// GQ free-level multiplier. Sums the shop `freeUpgradeMult`
/// contribution with `0.3% × cubeUpgrades\[75\]`. Used by both the
/// free-level softcap and the polynomial-path bonus.
#[must_use]
pub fn gq_free_level_multiplier(shop_free_upgrade_mult: f64, cube_upgrade_75: f64) -> f64 {
    shop_free_upgrade_mult + 0.3 / 100.0 * cube_upgrade_75
}

/// Effective free levels for one GQ upgrade. Above the player's
/// purchased level, free levels accumulate at a square-root rate.
/// `min(level, base) + sqrt(max(0, base - level))`.
#[must_use]
pub fn gq_upgrade_free_level_softcap(free_level: f64, level: f64, free_level_mult: f64) -> f64 {
    let base_real_free_levels = free_level_mult * free_level;
    level.min(base_real_free_levels) + 0.0_f64.max(base_real_free_levels - level).sqrt()
}

/// Inputs to [`compute_gq_upgrade_max_level`].
#[derive(Debug, Clone, Copy)]
pub struct GqUpgradeMaxLevelInput {
    /// `goldenQuarkUpgrades[k].canExceedCap`. When `false`,
    /// returns `max_level` as-is.
    pub can_exceed_cap: bool,
    /// `goldenQuarkUpgrades[k].maxLevel` — base cap.
    pub max_level: f64,
    /// `player.highestSingularityCount`.
    pub highest_singularity_count: f64,
    /// `getOcteractUpgradeEffect('octeractSingUpgradeCap',
    /// 'goldenQuarkUpgradeCapIncrease')`. Added to the final cap
    /// for `canExceedCap` upgrades.
    pub octeract_sing_upgrade_cap_increase: f64,
}

/// Maximum level for a GQ upgrade. For `canExceedCap` upgrades,
/// walks the overclock-perks array adding `+1` per crossed
/// threshold, then adds the octeract-cap bonus. For
/// non-`canExceedCap` upgrades, returns `max_level` unchanged.
#[must_use]
pub fn compute_gq_upgrade_max_level(input: &GqUpgradeMaxLevelInput) -> f64 {
    if !input.can_exceed_cap {
        return input.max_level;
    }
    let mut cap = input.max_level;
    for &perk in OVERCLOCK_PERKS {
        if input.highest_singularity_count >= perk {
            cap += 1.0;
        } else {
            break;
        }
    }
    cap += input.octeract_sing_upgrade_cap_increase;
    cap
}

/// Inputs to [`actual_gq_upgrade_total_levels`].
#[derive(Debug, Clone, Copy)]
pub struct ActualGQUpgradeTotalLevelsInput {
    /// `goldenQuarkUpgrades[k].level`.
    pub level: f64,
    /// `goldenQuarkUpgrades[k].freeLevel`.
    pub free_level: f64,
    /// `goldenQuarkUpgrades[k].qualityOfLife`. QoL upgrades stay
    /// active inside the gating challenges.
    pub quality_of_life: bool,
    /// True when `upgradeKey === 'platonicDelta'`. The check is
    /// upgrade-key-specific in the UI tier; we accept it as a bool
    /// here so logic doesn't have to know the GQ upgrade key set.
    pub is_platonic_delta: bool,
    /// `player.singularityChallenges.noSingularityUpgrades.enabled`.
    pub in_no_singularity_upgrades: bool,
    /// `player.singularityChallenges.sadisticPrequel.enabled`.
    pub in_sadistic_prequel: bool,
    /// `player.singularityChallenges.limitedAscensions.enabled`.
    pub in_limited_ascensions: bool,
    /// `player.singularityChallenges.limitedTime.enabled`.
    pub in_limited_time: bool,
    /// GQ free-level multiplier — from [`gq_free_level_multiplier`].
    pub free_level_mult: f64,
    /// `getOcteractUpgradeEffect('octeractImprovedFree', 'unlocked')`.
    /// Gates the polynomial-bonus path entirely.
    pub improved_free_unlocked: bool,
    /// Sum of the four improved-free `freeLevelPower` /
    /// `freeLevelPowerIncrease` octeract-upgrade effects. The
    /// polynomial term is `(level × actualFreeLevels) ^ exponent`.
    pub improved_free_exponent: f64,
}

/// Effective total level for one GQ upgrade. Three gating layers:
/// 1. `noSingularityUpgrades` / `sadisticPrequel` AND not QoL → 0
/// 2. `platonicDelta` inside `limitedAscensions` / `limitedTime` /
///    `sadisticPrequel` → 0
/// 3. `max(linearLevels, polynomialLevels)` where polynomial only
///    contributes when `octeractImprovedFree` is unlocked
#[must_use]
pub fn actual_gq_upgrade_total_levels(input: &ActualGQUpgradeTotalLevelsInput) -> f64 {
    if (input.in_no_singularity_upgrades || input.in_sadistic_prequel) && !input.quality_of_life {
        return 0.0;
    }
    if (input.in_limited_ascensions || input.in_limited_time || input.in_sadistic_prequel)
        && input.is_platonic_delta
    {
        return 0.0;
    }

    let actual_free_levels =
        gq_upgrade_free_level_softcap(input.free_level, input.level, input.free_level_mult);
    let linear_levels = input.level + actual_free_levels;
    let polynomial_levels = if input.improved_free_unlocked {
        (input.level * actual_free_levels).powf(input.improved_free_exponent)
    } else {
        0.0
    };

    linear_levels.max(polynomial_levels)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn free_level_multiplier_sums_shop_and_cube75() {
        // shop=1.0, cube=100 → 1.0 + 0.003*100 = 1.3
        let result = gq_free_level_multiplier(1.0, 100.0);
        assert!((result - 1.3).abs() < 1e-12);
    }

    #[test]
    fn free_level_softcap_below_level_is_linear() {
        // free=10, level=20, mult=1 → base=10; min(20,10)=10; sqrt(0)=0 → 10
        let result = gq_upgrade_free_level_softcap(10.0, 20.0, 1.0);
        assert_eq!(result, 10.0);
    }

    #[test]
    fn free_level_softcap_above_level_uses_sqrt() {
        // free=100, level=10, mult=1 → base=100; min(10,100)=10; sqrt(90)≈9.487
        let result = gq_upgrade_free_level_softcap(100.0, 10.0, 1.0);
        let expected = 10.0 + 90.0_f64.sqrt();
        assert!((result - expected).abs() < 1e-12);
    }

    #[test]
    fn max_level_without_can_exceed_returns_base() {
        let result = compute_gq_upgrade_max_level(&GqUpgradeMaxLevelInput {
            can_exceed_cap: false,
            max_level: 10.0,
            highest_singularity_count: 1_000.0,
            octeract_sing_upgrade_cap_increase: 50.0,
        });
        assert_eq!(result, 10.0);
    }

    #[test]
    fn max_level_walks_overclock_perks() {
        // sing=80 → crosses 50, 60, 75 → +3
        let result = compute_gq_upgrade_max_level(&GqUpgradeMaxLevelInput {
            can_exceed_cap: true,
            max_level: 10.0,
            highest_singularity_count: 80.0,
            octeract_sing_upgrade_cap_increase: 0.0,
        });
        assert_eq!(result, 13.0);
    }

    #[test]
    fn max_level_stops_at_first_unmet() {
        // sing=70 → crosses 50, 60 (not 75) → +2
        let result = compute_gq_upgrade_max_level(&GqUpgradeMaxLevelInput {
            can_exceed_cap: true,
            max_level: 10.0,
            highest_singularity_count: 70.0,
            octeract_sing_upgrade_cap_increase: 0.0,
        });
        assert_eq!(result, 12.0);
    }

    #[test]
    fn max_level_includes_octeract_bonus() {
        let result = compute_gq_upgrade_max_level(&GqUpgradeMaxLevelInput {
            can_exceed_cap: true,
            max_level: 10.0,
            highest_singularity_count: 0.0,
            octeract_sing_upgrade_cap_increase: 5.0,
        });
        assert_eq!(result, 15.0);
    }

    #[test]
    fn total_levels_no_sing_upgrades_zeros_non_qol() {
        let result = actual_gq_upgrade_total_levels(&ActualGQUpgradeTotalLevelsInput {
            level: 100.0,
            free_level: 100.0,
            quality_of_life: false,
            is_platonic_delta: false,
            in_no_singularity_upgrades: true,
            in_sadistic_prequel: false,
            in_limited_ascensions: false,
            in_limited_time: false,
            free_level_mult: 1.0,
            improved_free_unlocked: false,
            improved_free_exponent: 0.0,
        });
        assert_eq!(result, 0.0);
    }

    #[test]
    fn total_levels_qol_survives_sing_upgrade_challenge() {
        let result = actual_gq_upgrade_total_levels(&ActualGQUpgradeTotalLevelsInput {
            level: 10.0,
            free_level: 0.0,
            quality_of_life: true,
            is_platonic_delta: false,
            in_no_singularity_upgrades: true,
            in_sadistic_prequel: false,
            in_limited_ascensions: false,
            in_limited_time: false,
            free_level_mult: 1.0,
            improved_free_unlocked: false,
            improved_free_exponent: 0.0,
        });
        assert_eq!(result, 10.0);
    }

    #[test]
    fn total_levels_platonic_delta_zeroed_in_limited_ascensions() {
        let result = actual_gq_upgrade_total_levels(&ActualGQUpgradeTotalLevelsInput {
            level: 100.0,
            free_level: 0.0,
            quality_of_life: true,
            is_platonic_delta: true,
            in_no_singularity_upgrades: false,
            in_sadistic_prequel: false,
            in_limited_ascensions: true,
            in_limited_time: false,
            free_level_mult: 1.0,
            improved_free_unlocked: false,
            improved_free_exponent: 0.0,
        });
        assert_eq!(result, 0.0);
    }

    #[test]
    fn total_levels_linear_path_no_improved_free() {
        // level=10, free=20, mult=1 → free softcap = min(10,20)+sqrt(10)
        // = 10 + 3.162... = 13.162
        // linear = 10 + 13.162 = 23.162
        let result = actual_gq_upgrade_total_levels(&ActualGQUpgradeTotalLevelsInput {
            level: 10.0,
            free_level: 20.0,
            quality_of_life: false,
            is_platonic_delta: false,
            in_no_singularity_upgrades: false,
            in_sadistic_prequel: false,
            in_limited_ascensions: false,
            in_limited_time: false,
            free_level_mult: 1.0,
            improved_free_unlocked: false,
            improved_free_exponent: 0.0,
        });
        let expected_softcap = 10.0 + 10.0_f64.sqrt();
        let expected_linear = 10.0 + expected_softcap;
        assert!((result - expected_linear).abs() < 1e-9);
    }
}
