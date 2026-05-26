//! Rune level-bonus + OOM-increase aggregators.
//!
//! Verbatim port of
//! `legacy_core_split/packages/logic/src/mechanics/runeLevelBonuses.ts`.
//! Each formula sums a small fixed set of player/upgrade contributions.
//! The legacy UI pre-extracts each contribution from `player.*` /
//! `getTalismanEffects(...)` / `getAmbrosiaUpgradeEffects(...)` /
//! `getLevelMilestone(...)` / `CalcECC(...)` and passes the bundled
//! inputs.
//!
//! Functions migrated:
//! - [`first_five_free_levels`] — small constant + cap
//! - [`bonus_rune_levels_speed`] / `_duplication` / `_infinite_ascent` —
//!   non-trivial coin-based or singularity-perk math; the four other
//!   `bonusRuneLevels*` are 1-line pass-throughs (just talisman bonus)
//!   and stay in the UI tier
//! - [`speed_rune_oom_increase`] / `_duplication` / `_prism` / `_thrift`
//!   / `_si` — the five "real" OOM aggregators. The
//!   infinite-ascent/antiquities/horse-shoe `OOMIncrease` are 1-line
//!   pass-throughs and stay in the UI tier
//!
//! All formulas are pure `f64` math. Where the legacy uses
//! `Decimal.log(x, base)`, the caller pre-evaluates the log to an
//! `f64`.

// ─── firstFiveFreeLevels ───────────────────────────────────────────────────

/// Inputs to [`first_five_free_levels`].
#[derive(Debug, Clone, Copy)]
pub struct FirstFiveFreeLevelsInput {
    /// `getAntUpgradeEffect(AntUpgrades.FreeRunes).freeRuneLevel`.
    pub free_runes_ant_upgrade: f64,
    /// `player.constantUpgrades[7]` — capped at 1000, `×7`.
    pub constant_upgrade_7: f64,
}

/// Free levels granted to the first five runes
/// (`free_runes_ant_upgrade + 7 * min(constant_upgrade_7, 1000)`).
#[must_use]
pub fn first_five_free_levels(input: &FirstFiveFreeLevelsInput) -> f64 {
    input.free_runes_ant_upgrade + 7.0 * input.constant_upgrade_7.min(1000.0)
}

// ─── bonusRuneLevels (non-trivial: speed, duplication, infinite_ascent) ───

/// Inputs to [`bonus_rune_levels_speed`].
#[derive(Debug, Clone, Copy)]
pub struct BonusRuneLevelsSpeedInput {
    /// `getRuneBonusFromAllTalismans('speed')`.
    pub talisman_bonus: f64,
    /// `player.upgrades[27]` — the upgrade level, multiplied by the
    /// coin-log split.
    pub upgrade_27: f64,
    /// `Math.floor(Decimal.log(player.coins.add(1), 1e10))` —
    /// pre-evaluated.
    pub coin_log_1e10_floor: f64,
    /// `Math.floor(Decimal.log(player.coins.add(1), 1e50))` —
    /// pre-evaluated.
    pub coin_log_1e50_floor: f64,
    /// `player.upgrades[29]` — coin-count-based bonus.
    pub upgrade_29: f64,
    /// `firstOwnedCoin + ... + fifthOwnedCoin`.
    pub total_owned_coins_first_five: f64,
}

/// Bonus speed-rune levels —
/// `talisman_bonus + upgrade_27_term + upgrade_29_term`. See
/// [`BonusRuneLevelsSpeedInput`] for term definitions.
#[must_use]
pub fn bonus_rune_levels_speed(input: &BonusRuneLevelsSpeedInput) -> f64 {
    // upgrade_27 contribution: caps the 1e10-log at 50, then ADDS a
    // second 1e50-log term that's offset by -10 and clamped to [0, 50].
    // The two terms are summed before multiplying by upgrade_27.
    let upgrade_27_term = input.upgrade_27
        * (50.0_f64.min(input.coin_log_1e10_floor)
            + 0.0_f64.max(50.0_f64.min(input.coin_log_1e50_floor - 10.0)));
    let upgrade_29_term = input.upgrade_29
        * 100.0_f64
            .min(input.total_owned_coins_first_five / 400.0)
            .floor();
    input.talisman_bonus + upgrade_27_term + upgrade_29_term
}

/// Inputs to [`bonus_rune_levels_duplication`].
#[derive(Debug, Clone, Copy)]
pub struct BonusRuneLevelsDuplicationInput {
    /// `getRuneBonusFromAllTalismans('duplication')`.
    pub talisman_bonus: f64,
    /// `player.upgrades[28]` — coin-count-based bonus.
    pub upgrade_28: f64,
    /// `firstOwnedCoin + ... + fifthOwnedCoin`.
    pub total_owned_coins_first_five: f64,
    /// `player.upgrades[30]` — coin-log-based bonus.
    pub upgrade_30: f64,
    /// `Math.floor(Decimal.log(player.coins.add(1), 1e30))`.
    pub coin_log_1e30_floor: f64,
    /// `Math.floor(Decimal.log(player.coins.add(1), 1e300))`.
    pub coin_log_1e300_floor: f64,
}

/// Bonus duplication-rune levels — analogous shape to
/// [`bonus_rune_levels_speed`] but with different coin-log
/// thresholds.
#[must_use]
pub fn bonus_rune_levels_duplication(input: &BonusRuneLevelsDuplicationInput) -> f64 {
    let upgrade_28_term =
        input.upgrade_28 * 100.0_f64.min((input.total_owned_coins_first_five / 400.0).floor());
    // upgrade_30: sum of two log-caps (different bases), each ceiling 50.
    let upgrade_30_term = input.upgrade_30
        * (50.0_f64.min(input.coin_log_1e30_floor) + 50.0_f64.min(input.coin_log_1e300_floor));
    input.talisman_bonus + upgrade_28_term + upgrade_30_term
}

/// Inputs to [`bonus_rune_levels_infinite_ascent`].
#[derive(Debug, Clone, Copy)]
pub struct BonusRuneLevelsInfiniteAscentInput {
    /// `PCoinUpgradeEffects.INSTANT_UNLOCK_2 ? 6 : 0` — pre-evaluated
    /// to the number.
    pub instant_unlock_2_bonus: f64,
    /// `player.cubeUpgrades[73]`.
    pub cube_upgrade_73: f64,
    /// `player.campaigns.bonusRune6`.
    pub campaign_bonus_rune_6: f64,
    /// `getRuneBonusFromAllTalismans('infiniteAscent')`.
    pub talisman_bonus: f64,
    /// `getRuneEffects('finiteDescent', 'infiniteAscentFreeLevel')`.
    pub finite_descent_bonus: f64,
}

/// Bonus infinite-ascent rune levels — simple sum of five
/// contributions.
#[must_use]
pub fn bonus_rune_levels_infinite_ascent(input: &BonusRuneLevelsInfiniteAscentInput) -> f64 {
    input.instant_unlock_2_bonus
        + input.cube_upgrade_73
        + input.campaign_bonus_rune_6
        + input.talisman_bonus
        + input.finite_descent_bonus
}

// ─── runeOOMIncrease (speed, duplication, prism, thrift, SI) ──────────────
//
// All five share a common ascension-challenge term:
//   `CalcECC('ascension', c11) + 1.5 * CalcECC('ascension', c14)`
// plus per-rune research/cube/talisman/ambrosia/milestone contributions.
// The legacy UI passes the already-evaluated CalcECC + ambrosia +
// milestone values.

/// Inputs to [`speed_rune_oom_increase`].
#[derive(Debug, Clone, Copy)]
pub struct SpeedRuneOOMIncreaseInput {
    /// `player.upgrades[66]` — multiplied by 2.
    pub upgrade_66: f64,
    /// `player.researches[78]`.
    pub research_78: f64,
    /// `player.researches[111]`.
    pub research_111: f64,
    /// `CalcECC('ascension', player.challengecompletions[11])`.
    pub c11_ascension_ecc: f64,
    /// `CalcECC('ascension', player.challengecompletions[14])` —
    /// multiplied by 1.5.
    pub c14_ascension_ecc: f64,
    /// `player.cubeUpgrades[16]`.
    pub cube_upgrade_16: f64,
    /// `getTalismanEffects('chronos').speedOOMBonus`.
    pub chronos_speed_oom_bonus: f64,
    /// `getAmbrosiaUpgradeEffects('ambrosiaRuneOOMBonus', 'runeOOMBonus')`.
    pub ambrosia_rune_oom_bonus: f64,
    /// `getLevelMilestone('speedRune')`.
    pub speed_rune_level_milestone: f64,
}

/// Speed-rune OOM increase — sum of nine contributions.
#[must_use]
pub fn speed_rune_oom_increase(input: &SpeedRuneOOMIncreaseInput) -> f64 {
    input.upgrade_66 * 2.0
        + input.research_78
        + input.research_111
        + input.c11_ascension_ecc
        + 1.5 * input.c14_ascension_ecc
        + input.cube_upgrade_16
        + input.chronos_speed_oom_bonus
        + input.ambrosia_rune_oom_bonus
        + input.speed_rune_level_milestone
}

/// Inputs to [`duplication_rune_oom_increase`].
#[derive(Debug, Clone, Copy)]
pub struct DuplicationRuneOOMIncreaseInput {
    /// `CalcECC('transcend', player.challengecompletions[1])` —
    /// multiplied by 0.75.
    pub c1_transcend_ecc: f64,
    /// `player.upgrades[66]` — multiplied by 2.
    pub upgrade_66: f64,
    /// `player.researches[90]`.
    pub research_90: f64,
    /// `player.researches[112]`.
    pub research_112: f64,
    /// `CalcECC('ascension', player.challengecompletions[11])`.
    pub c11_ascension_ecc: f64,
    /// `CalcECC('ascension', player.challengecompletions[14])` —
    /// multiplied by 1.5.
    pub c14_ascension_ecc: f64,
    /// `getTalismanEffects('exemption').duplicationOOMBonus`.
    pub exemption_duplication_oom_bonus: f64,
    /// `getAmbrosiaUpgradeEffects('ambrosiaRuneOOMBonus', 'runeOOMBonus')`.
    pub ambrosia_rune_oom_bonus: f64,
    /// `getLevelMilestone('duplicationRune')`.
    pub duplication_rune_level_milestone: f64,
}

/// Duplication-rune OOM increase — sum of nine contributions, with C1
/// transcend ECC weighted at 0.75.
#[must_use]
pub fn duplication_rune_oom_increase(input: &DuplicationRuneOOMIncreaseInput) -> f64 {
    0.75 * input.c1_transcend_ecc
        + input.upgrade_66 * 2.0
        + input.research_90
        + input.research_112
        + input.c11_ascension_ecc
        + 1.5 * input.c14_ascension_ecc
        + input.exemption_duplication_oom_bonus
        + input.ambrosia_rune_oom_bonus
        + input.duplication_rune_level_milestone
}

/// Inputs to [`prism_rune_oom_increase`].
#[derive(Debug, Clone, Copy)]
pub struct PrismRuneOOMIncreaseInput {
    /// `player.upgrades[66]` — multiplied by 2.
    pub upgrade_66: f64,
    /// `player.researches[79]`.
    pub research_79: f64,
    /// `player.researches[113]`.
    pub research_113: f64,
    /// `CalcECC('ascension', player.challengecompletions[11])`.
    pub c11_ascension_ecc: f64,
    /// `CalcECC('ascension', player.challengecompletions[14])` —
    /// multiplied by 1.5.
    pub c14_ascension_ecc: f64,
    /// `player.cubeUpgrades[16]`.
    pub cube_upgrade_16: f64,
    /// `getTalismanEffects('mortuus').prismOOMBonus`.
    pub mortuus_prism_oom_bonus: f64,
    /// `getAmbrosiaUpgradeEffects('ambrosiaRuneOOMBonus', 'runeOOMBonus')`.
    pub ambrosia_rune_oom_bonus: f64,
    /// `getLevelMilestone('prismRune')`.
    pub prism_rune_level_milestone: f64,
}

/// Prism-rune OOM increase — sum of nine contributions.
#[must_use]
pub fn prism_rune_oom_increase(input: &PrismRuneOOMIncreaseInput) -> f64 {
    input.upgrade_66 * 2.0
        + input.research_79
        + input.research_113
        + input.c11_ascension_ecc
        + 1.5 * input.c14_ascension_ecc
        + input.cube_upgrade_16
        + input.mortuus_prism_oom_bonus
        + input.ambrosia_rune_oom_bonus
        + input.prism_rune_level_milestone
}

/// Inputs to [`thrift_rune_oom_increase`].
#[derive(Debug, Clone, Copy)]
pub struct ThriftRuneOOMIncreaseInput {
    /// `player.upgrades[66]` — multiplied by 2.
    pub upgrade_66: f64,
    /// `player.researches[77]`.
    pub research_77: f64,
    /// `player.researches[114]`.
    pub research_114: f64,
    /// `CalcECC('ascension', player.challengecompletions[11])`.
    pub c11_ascension_ecc: f64,
    /// `CalcECC('ascension', player.challengecompletions[14])` —
    /// multiplied by 1.5.
    pub c14_ascension_ecc: f64,
    /// `player.cubeUpgrades[37]`.
    pub cube_upgrade_37: f64,
    /// `getTalismanEffects('midas').thriftOOMBonus`.
    pub midas_thrift_oom_bonus: f64,
    /// `getAmbrosiaUpgradeEffects('ambrosiaRuneOOMBonus', 'runeOOMBonus')`.
    pub ambrosia_rune_oom_bonus: f64,
    /// `getLevelMilestone('thriftRune')`.
    pub thrift_rune_level_milestone: f64,
}

/// Thrift-rune OOM increase — sum of nine contributions.
#[must_use]
pub fn thrift_rune_oom_increase(input: &ThriftRuneOOMIncreaseInput) -> f64 {
    input.upgrade_66 * 2.0
        + input.research_77
        + input.research_114
        + input.c11_ascension_ecc
        + 1.5 * input.c14_ascension_ecc
        + input.cube_upgrade_37
        + input.midas_thrift_oom_bonus
        + input.ambrosia_rune_oom_bonus
        + input.thrift_rune_level_milestone
}

/// Inputs to [`superior_intellect_rune_oom_increase`].
#[derive(Debug, Clone, Copy)]
pub struct SuperiorIntellectRuneOOMIncreaseInput {
    /// `player.upgrades[66]` — multiplied by 2.
    pub upgrade_66: f64,
    /// `player.researches[115]`.
    pub research_115: f64,
    /// `CalcECC('ascension', player.challengecompletions[11])`.
    pub c11_ascension_ecc: f64,
    /// `CalcECC('ascension', player.challengecompletions[14])` —
    /// multiplied by 1.5.
    pub c14_ascension_ecc: f64,
    /// `player.cubeUpgrades[37]`.
    pub cube_upgrade_37: f64,
    /// `getTalismanEffects('polymath').SIOOMBonus`.
    pub polymath_si_oom_bonus: f64,
    /// `getAmbrosiaUpgradeEffects('ambrosiaRuneOOMBonus', 'runeOOMBonus')`.
    pub ambrosia_rune_oom_bonus: f64,
    /// `getLevelMilestone('SIRune')`.
    pub si_rune_level_milestone: f64,
}

/// SI-rune OOM increase — sum of eight contributions (one fewer than
/// the others — no research-78 / -90 / -79 / -77 sibling).
#[must_use]
pub fn superior_intellect_rune_oom_increase(input: &SuperiorIntellectRuneOOMIncreaseInput) -> f64 {
    input.upgrade_66 * 2.0
        + input.research_115
        + input.c11_ascension_ecc
        + 1.5 * input.c14_ascension_ecc
        + input.cube_upgrade_37
        + input.polymath_si_oom_bonus
        + input.ambrosia_rune_oom_bonus
        + input.si_rune_level_milestone
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─── firstFiveFreeLevels ───────────────────────────────────────────────

    #[test]
    fn first_five_free_levels_caps_constant_upgrade_7() {
        let input = FirstFiveFreeLevelsInput {
            free_runes_ant_upgrade: 0.0,
            constant_upgrade_7: 5000.0,
        };
        // Capped at 1000, then × 7 = 7000.
        assert_eq!(first_five_free_levels(&input), 7000.0);
    }

    #[test]
    fn first_five_free_levels_sums_both_contributions() {
        let input = FirstFiveFreeLevelsInput {
            free_runes_ant_upgrade: 10.0,
            constant_upgrade_7: 5.0,
        };
        // 10 + 7 * 5 = 45
        assert_eq!(first_five_free_levels(&input), 45.0);
    }

    // ─── bonusRuneLevelsSpeed ──────────────────────────────────────────────

    #[test]
    fn bonus_rune_levels_speed_caps_logs_at_50() {
        let input = BonusRuneLevelsSpeedInput {
            talisman_bonus: 0.0,
            upgrade_27: 1.0,
            coin_log_1e10_floor: 100.0, // capped to 50
            coin_log_1e50_floor: 100.0, // 100-10=90 → capped to 50
            upgrade_29: 0.0,
            total_owned_coins_first_five: 0.0,
        };
        // 1 * (50 + 50) = 100
        assert_eq!(bonus_rune_levels_speed(&input), 100.0);
    }

    #[test]
    fn bonus_rune_levels_speed_upgrade_29_caps_total_owned() {
        // upgrade_29 * floor(min(100, owned/400)) — with owned = 1e6 →
        // 1e6/400 = 2500 → capped at 100 → floor 100.
        let input = BonusRuneLevelsSpeedInput {
            talisman_bonus: 0.0,
            upgrade_27: 0.0,
            coin_log_1e10_floor: 0.0,
            coin_log_1e50_floor: 0.0,
            upgrade_29: 1.0,
            total_owned_coins_first_five: 1_000_000.0,
        };
        assert_eq!(bonus_rune_levels_speed(&input), 100.0);
    }

    // ─── bonusRuneLevelsDuplication ────────────────────────────────────────

    #[test]
    fn bonus_rune_levels_duplication_sums_correctly() {
        let input = BonusRuneLevelsDuplicationInput {
            talisman_bonus: 7.0,
            upgrade_28: 1.0,
            total_owned_coins_first_five: 4000.0, // 4000/400 = 10 (no cap)
            upgrade_30: 1.0,
            coin_log_1e30_floor: 25.0,   // below 50 cap
            coin_log_1e300_floor: 100.0, // capped to 50
        };
        // 7 + 1*10 + 1*(25+50) = 92
        assert_eq!(bonus_rune_levels_duplication(&input), 92.0);
    }

    // ─── bonusRuneLevelsInfiniteAscent ─────────────────────────────────────

    #[test]
    fn bonus_rune_levels_infinite_ascent_is_simple_sum() {
        let input = BonusRuneLevelsInfiniteAscentInput {
            instant_unlock_2_bonus: 6.0,
            cube_upgrade_73: 2.0,
            campaign_bonus_rune_6: 3.0,
            talisman_bonus: 4.0,
            finite_descent_bonus: 1.0,
        };
        assert_eq!(bonus_rune_levels_infinite_ascent(&input), 16.0);
    }

    // ─── speedRuneOOMIncrease ──────────────────────────────────────────────

    #[test]
    fn speed_rune_oom_increase_zero_inputs_is_zero() {
        let input = SpeedRuneOOMIncreaseInput {
            upgrade_66: 0.0,
            research_78: 0.0,
            research_111: 0.0,
            c11_ascension_ecc: 0.0,
            c14_ascension_ecc: 0.0,
            cube_upgrade_16: 0.0,
            chronos_speed_oom_bonus: 0.0,
            ambrosia_rune_oom_bonus: 0.0,
            speed_rune_level_milestone: 0.0,
        };
        assert_eq!(speed_rune_oom_increase(&input), 0.0);
    }

    #[test]
    fn speed_rune_oom_increase_c14_weighted_by_1_5() {
        let input = SpeedRuneOOMIncreaseInput {
            upgrade_66: 0.0,
            research_78: 0.0,
            research_111: 0.0,
            c11_ascension_ecc: 0.0,
            c14_ascension_ecc: 4.0, // × 1.5 = 6
            cube_upgrade_16: 0.0,
            chronos_speed_oom_bonus: 0.0,
            ambrosia_rune_oom_bonus: 0.0,
            speed_rune_level_milestone: 0.0,
        };
        assert_eq!(speed_rune_oom_increase(&input), 6.0);
    }

    #[test]
    fn duplication_rune_oom_increase_c1_transcend_weighted_by_0_75() {
        let input = DuplicationRuneOOMIncreaseInput {
            c1_transcend_ecc: 4.0, // × 0.75 = 3
            upgrade_66: 0.0,
            research_90: 0.0,
            research_112: 0.0,
            c11_ascension_ecc: 0.0,
            c14_ascension_ecc: 0.0,
            exemption_duplication_oom_bonus: 0.0,
            ambrosia_rune_oom_bonus: 0.0,
            duplication_rune_level_milestone: 0.0,
        };
        assert_eq!(duplication_rune_oom_increase(&input), 3.0);
    }

    #[test]
    fn si_rune_oom_increase_omits_research_pair() {
        // SI lacks a sibling research_78/-90/-79/-77; this test just
        // confirms the sum has 8 fields rather than 9.
        let input = SuperiorIntellectRuneOOMIncreaseInput {
            upgrade_66: 1.0,
            research_115: 2.0,
            c11_ascension_ecc: 3.0,
            c14_ascension_ecc: 4.0,
            cube_upgrade_37: 5.0,
            polymath_si_oom_bonus: 6.0,
            ambrosia_rune_oom_bonus: 7.0,
            si_rune_level_milestone: 8.0,
        };
        // 2 + 2 + 3 + 6 + 5 + 6 + 7 + 8 = 39
        assert_eq!(superior_intellect_rune_oom_increase(&input), 39.0);
    }
}
