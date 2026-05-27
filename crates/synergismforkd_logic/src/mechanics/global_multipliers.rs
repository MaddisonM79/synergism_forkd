//! Per-tick global multiplier aggregator.
//!
//! Verbatim port of
//! `legacy_core_split/packages/logic/src/mechanics/globalMultipliers.ts`.
//! Populates the 17 `G.*` multiplier fields the rest of the tick reads.
//!
//! Distinct from [`super::update_all_multiplier`], which computes the
//! `freeMultiplier` / `multiplierEffect` stack — the value of that
//! computation feeds back into this one via [`GlobalMultipliersPreEvaluated::multiplier_effect`].
//!
//! Takes a `&GameState` (for direct `player.*` reads) plus a
//! [`GlobalMultipliersPreEvaluated`] bundle (for cross-mechanic
//! pre-computed values that aren't in any slice). The legacy
//! `GlobalMultipliersInput` 80-field bag is gone — the per-tick
//! aggregator now reads state structurally.

use crate::mechanics::challenges::{calc_ecc, ChallengeType};
use crate::state::{GameState, RECESSION_INDEX};
use synergismforkd_bignum::Decimal;

/// Cross-mechanic pre-computed values the aggregator needs that don't
/// live in any state slice. Built by the caller from earlier per-tick
/// outputs (rune effects, ant effects, achievement rewards, shop
/// effects, the `G.*` cache).
#[derive(Debug, Clone, Copy)]
pub struct GlobalMultipliersPreEvaluated {
    /// `calculateCrystalCoinMultiplier()` — multiplied into `s`.
    pub crystal_mult: Decimal,
    /// `calculateBuildingPower()` — used in upgrade-55 `min(1000, buildingPower - 1)`.
    pub building_power: f64,
    /// `calculateBuildingPowerCoinMultiplier(buildingPower)` — multiplied into `s`,
    /// also feeds research-39/40.
    pub building_power_mult: Decimal,
    /// `calculateTotalCoinOwned()` — used in `first6CoinUp` + upgrade-20.
    pub total_coin_owned: f64,
    /// `getAntUpgradeEffect(AntUpgrades.Coins).coinMultiplier` — multiplied into
    /// `s`, also surfaced as `antMultiplier` output.
    pub ant_multiplier: Decimal,
    /// `crystalUpgrade3CrystalMultiplier()` — multiplied into `globalCrystalMultiplier`.
    pub crystal_upgrade_3_multiplier: Decimal,
    /// `+getAchievementReward('crystalMultiplier')` — multiplied into
    /// `globalCrystalMultiplier`.
    pub crystal_multiplier_achievement: f64,
    /// `+getAchievementReward('constUpgrade1Buff')` — added to `constUpgrade1`
    /// exponent base.
    pub const_upgrade_1_buff_achievement: f64,
    /// `+getAchievementReward('constUpgrade2Buff')` — bounded coefficient inside
    /// `constUpgrade2`.
    pub const_upgrade_2_buff_achievement: f64,
    /// `getRuneEffects('prism', 'productionLog10')` — exponent of 10 multiplied
    /// into `globalCrystalMultiplier`.
    pub prism_production_log10: f64,
    /// `getShopUpgradeEffects('constantEX', 'maxPercentIncrease')` — added into
    /// `constUpgrade2` bounded percentage.
    pub constant_ex_max_percent_increase: f64,
    /// `ascendBuildingDR()` — exponent applied to `constUpgrade2` contribution.
    pub ascend_building_dr_value: f64,
    /// `G.multiplierEffect` — multiplied into `s` (set by
    /// `update_all_multiplier` earlier this tick).
    pub multiplier_effect: Decimal,
    /// `G.acceleratorEffect` — multiplied into `s` (set by `update_all_tick`
    /// earlier this tick) + `mythosupgrade13`.
    pub accelerator_effect: Decimal,
    /// `G.totalMultiplier` — feeds upgrade-48
    /// (`× totalAccelerator / 1000 + 1`).
    pub total_multiplier: f64,
    /// `G.totalAccelerator` — feeds upgrade-48.
    pub total_accelerator: f64,
    /// `G.totalAcceleratorBoost` — exponent base for upgrade-51.
    pub total_accelerator_boost: f64,
    /// `G.challenge15Rewards.coinExponent.value` — exponent of
    /// `lol → globalCoinMultiplier`.
    pub challenge_15_coin_exponent: f64,
    /// `G.challenge15Rewards.exponent.value` — `(exponent - 1) × 1000` is bounded
    /// into `constUpgrade2` percent.
    pub challenge_15_exponent_value: f64,
    /// `G.challenge15Rewards.constantBonus.value` — multiplied into
    /// `globalConstantMult`.
    pub challenge_15_constant_bonus: f64,
    /// `G.recessionPower[player.corruptions.used.recession]` — exponent applied
    /// to `globalCoinMultiplier`.
    pub recession_power: f64,
}

impl Default for GlobalMultipliersPreEvaluated {
    /// Identity values — every multiplier/effect is the multiplicative
    /// or exponential identity, so passing this default yields the same
    /// result as having no cross-mechanic effects active.
    fn default() -> Self {
        Self {
            crystal_mult: Decimal::one(),
            building_power: 1.0,
            building_power_mult: Decimal::one(),
            total_coin_owned: 0.0,
            ant_multiplier: Decimal::one(),
            crystal_upgrade_3_multiplier: Decimal::one(),
            crystal_multiplier_achievement: 1.0,
            const_upgrade_1_buff_achievement: 0.0,
            const_upgrade_2_buff_achievement: 0.0,
            prism_production_log10: 0.0,
            constant_ex_max_percent_increase: 0.0,
            ascend_building_dr_value: 1.0,
            multiplier_effect: Decimal::one(),
            accelerator_effect: Decimal::one(),
            total_multiplier: 0.0,
            total_accelerator: 0.0,
            total_accelerator_boost: 0.0,
            challenge_15_coin_exponent: 1.0,
            challenge_15_exponent_value: 1.0,
            challenge_15_constant_bonus: 1.0,
            recession_power: 1.0,
        }
    }
}

/// Result of [`compute_global_multipliers`].
#[derive(Debug, Clone, Copy)]
pub struct GlobalMultipliersResult {
    /// `G.globalCoinMultiplier`.
    pub global_coin_multiplier: Decimal,
    /// `G.coinOneMulti`.
    pub coin_one_multi: Decimal,
    /// `G.coinTwoMulti`.
    pub coin_two_multi: Decimal,
    /// `G.coinThreeMulti`.
    pub coin_three_multi: Decimal,
    /// `G.coinFourMulti`.
    pub coin_four_multi: Decimal,
    /// `G.coinFiveMulti`.
    pub coin_five_multi: Decimal,
    /// `G.globalCrystalMultiplier`.
    pub global_crystal_multiplier: Decimal,
    /// `G.globalMythosMultiplier`.
    pub global_mythos_multiplier: Decimal,
    /// `G.grandmasterMultiplier`.
    pub grandmaster_multiplier: Decimal,
    /// `G.totalMythosOwned`.
    pub total_mythos_owned: f64,
    /// `G.mythosBuildingPower`.
    pub mythos_building_power: f64,
    /// `G.challengeThreeMultiplier`.
    pub challenge_three_multiplier: Decimal,
    /// `G.mythosupgrade13`.
    pub mythosupgrade_13: Decimal,
    /// `G.mythosupgrade14`.
    pub mythosupgrade_14: Decimal,
    /// `G.mythosupgrade15`.
    pub mythosupgrade_15: Decimal,
    /// `G.globalConstantMult`.
    pub global_constant_mult: Decimal,
    /// `G.antMultiplier` — pass-through of `pre.ant_multiplier`, surfaced
    /// for parity with the legacy implementation (the legacy code sets
    /// `G.antMultiplier` inside `multipliers()` so the shim must too).
    pub ant_multiplier: Decimal,
}

/// Per-tick global-multiplier aggregator. Direct transcription of the legacy
/// `multipliers()` body, sourcing direct `player.*` reads from `state` and
/// cross-mechanic pre-computed values from `pre`.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn compute_global_multipliers(
    state: &GameState,
    pre: &GlobalMultipliersPreEvaluated,
) -> GlobalMultipliersResult {
    let one = Decimal::one();
    let ten = Decimal::from_finite(10.0);

    // ─── Direct player-state reads ───────────────────────────────────────
    let upgrade = |i: usize| f64::from(state.upgrades.upgrades[i]);
    let research = |i: usize| state.researches.researches[i];
    let crystal_upgrade = |i: usize| state.crystal_upgrades.crystal_upgrades[i];
    let constant_upgrade = |i: usize| state.campaigns.constant_upgrades[i];
    let platonic_upgrade = |i: usize| state.cube_upgrade_levels.platonic_upgrades[i];
    let c_completions = |i: usize| state.challenges.challenge_completions[i];

    let coins = state.upgrades.coins;
    let prestige_points = state.upgrades.prestige_points;
    let transcend_points = state.upgrades.transcend_points;
    let reincarnation_points = state.upgrades.reincarnation_points;
    let transcend_shards = state.reset_counters.transcend_shards;
    let prestige_count = state.reset_counters.prestige_count;
    let transcend_count = state.reset_counters.transcend_count;
    let highest_singularity_count = state.singularity.highest_singularity_count;
    let golden_quarks = state.golden_quarks.golden_quarks.to_number();
    let overflux_powder = state.hepteracts.overflux_powder;
    let second_owned_coin = state.coin_producers.tiers[1].owned;
    let first_generated_mythos = state.mythos_producers.tiers[0].generated;
    let first_owned_mythos = state.mythos_producers.tiers[0].owned;
    let second_owned_mythos = state.mythos_producers.tiers[1].owned;
    let third_owned_mythos = state.mythos_producers.tiers[2].owned;
    let fourth_owned_mythos = state.mythos_producers.tiers[3].owned;
    let fifth_owned_mythos = state.mythos_producers.tiers[4].owned;
    let reincarnation_challenge = state.challenges.current_reincarnation_challenge;
    let ascension_challenge = state.challenges.current_ascension_challenge;
    let recession_corruption_level = state.corruptions.used.levels[RECESSION_INDEX];
    let achievement_points = state.achievements.achievement_points;

    // ─── Main `s` build-up ───────────────────────────────────────────────
    let mut s = Decimal::one();
    s *= pre.multiplier_effect;
    s *= pre.accelerator_effect;
    s *= pre.crystal_mult;
    s *= pre.building_power_mult;
    s *= pre.ant_multiplier;

    let first_6_coin_up = Decimal::from_finite(pre.total_coin_owned + 1.0)
        * Decimal::from_finite(1e30)
            .min(Decimal::from_finite(1.008).pow(Decimal::from_finite(pre.total_coin_owned)));

    if highest_singularity_count > 0.0 {
        let bonus = (golden_quarks + 1.0).powf(1.5) * (highest_singularity_count + 1.0).powf(2.0);
        s *= Decimal::from_finite(bonus);
    }
    if upgrade(6) > 0.5 {
        s *= first_6_coin_up;
    }
    if upgrade(12) > 0.5 {
        s *= Decimal::from_finite(1e4)
            .min(Decimal::from_finite(1.01).pow(Decimal::from_finite(prestige_count)));
    }
    if upgrade(20) > 0.5 {
        s *= Decimal::from_finite(pre.total_coin_owned / 4.0 + 1.0).pow(Decimal::from_finite(10.0));
    }
    if upgrade(41) > 0.5 {
        s *= Decimal::from_finite(1e30)
            .min((transcend_points + Decimal::from_finite(4.0)).pow(Decimal::from_finite(0.5)));
    }
    if upgrade(43) > 0.5 {
        s *= Decimal::from_finite(1e30)
            .min(Decimal::from_finite(1.01).pow(Decimal::from_finite(transcend_count)));
    }
    if upgrade(48) > 0.5 {
        s *= Decimal::from_finite((pre.total_multiplier * pre.total_accelerator) / 1000.0 + 1.0)
            .pow(Decimal::from_finite(8.0));
    }
    if reincarnation_challenge == 6 {
        s /= Decimal::from_finite(1e250);
    }
    if reincarnation_challenge == 7 {
        s /= Decimal::from_mantissa_exponent(1.0, 1250.0);
    }
    if reincarnation_challenge == 9 {
        s /= Decimal::from_mantissa_exponent(1.0, 2_000_000.0);
    }
    let c = s.pow(Decimal::from_finite(1.0 + 0.001 * research(17)));
    let mut lol = c.pow(Decimal::from_finite(1.0 + 0.025 * upgrade(123)));
    if ascension_challenge == 15 && platonic_upgrade(5) > 0.0 {
        lol = lol.pow(Decimal::from_finite(1.1));
    }
    if ascension_challenge == 15 && platonic_upgrade(14) > 0.0 {
        let log10_coins_plus_1 = (coins + one).log(ten).to_number();
        let exponent = 1.0
            + ((1.0 / 20.0) * f64::from(recession_corruption_level) * log10_coins_plus_1)
                / (1e7 + log10_coins_plus_1);
        lol = lol.pow(Decimal::from_finite(exponent));
    }
    if ascension_challenge == 15 && platonic_upgrade(15) > 0.0 {
        lol = lol.pow(Decimal::from_finite(1.1));
    }
    lol = lol.pow(Decimal::from_finite(pre.challenge_15_coin_exponent));
    let mut global_coin_multiplier = lol;
    global_coin_multiplier = global_coin_multiplier.pow(Decimal::from_finite(pre.recession_power));

    // ─── Per-coin-tier multipliers ───────────────────────────────────────
    let mut coin_one_multi = Decimal::one();
    if upgrade(1) > 0.5 {
        coin_one_multi *= first_6_coin_up;
    }
    if upgrade(10) > 0.5 {
        coin_one_multi *= Decimal::from_finite(2.0)
            .pow(Decimal::from_finite(50.0_f64.min(second_owned_coin / 15.0)));
    }
    if upgrade(56) > 0.5 {
        coin_one_multi *= Decimal::from_mantissa_exponent(1.0, 5000.0);
    }

    let mut coin_two_multi = Decimal::one();
    if upgrade(2) > 0.5 {
        coin_two_multi *= first_6_coin_up;
    }
    if upgrade(13) > 0.5 {
        let inner = (first_generated_mythos + Decimal::from_finite(first_owned_mythos) + one)
            .pow(Decimal::from_finite(4.0 / 3.0))
            * Decimal::from_finite(1e22);
        coin_two_multi *= Decimal::from_finite(1e50).min(inner);
    }
    if upgrade(19) > 0.5 {
        coin_two_multi *=
            Decimal::from_finite(1e200).min(transcend_points * Decimal::from_finite(1e30) + one);
    }
    if upgrade(57) > 0.5 {
        coin_two_multi *= Decimal::from_mantissa_exponent(1.0, 7500.0);
    }

    let mut coin_three_multi = Decimal::one();
    if upgrade(3) > 0.5 {
        coin_three_multi *= first_6_coin_up;
    }
    if upgrade(18) > 0.5 {
        coin_three_multi *= Decimal::from_finite(1e125).min(transcend_shards + one);
    }
    if upgrade(58) > 0.5 {
        coin_three_multi *= Decimal::from_mantissa_exponent(1.0, 15_000.0);
    }

    let mut coin_four_multi = Decimal::one();
    if upgrade(4) > 0.5 {
        coin_four_multi *= first_6_coin_up;
    }
    if upgrade(17) > 0.5 {
        coin_four_multi *= Decimal::from_finite(1e100);
    }
    if upgrade(59) > 0.5 {
        coin_four_multi *= Decimal::from_mantissa_exponent(1.0, 25_000.0);
    }

    let mut coin_five_multi = Decimal::one();
    if upgrade(5) > 0.5 {
        coin_five_multi *= first_6_coin_up;
    }
    if upgrade(60) > 0.5 {
        coin_five_multi *= Decimal::from_mantissa_exponent(1.0, 35_000.0);
    }

    // ─── Crystal multiplier ──────────────────────────────────────────────
    let mut global_crystal_multiplier = Decimal::one();
    global_crystal_multiplier *= Decimal::from_finite(pre.crystal_multiplier_achievement);
    global_crystal_multiplier *= ten.pow(Decimal::from_finite(pre.prism_production_log10));
    if upgrade(36) > 0.5 {
        global_crystal_multiplier *= Decimal::from_mantissa_exponent(1.0, 5000.0)
            .min(prestige_points.pow(Decimal::from_finite(1.0 / 500.0)));
    }
    if upgrade(63) > 0.5 {
        global_crystal_multiplier *= Decimal::from_mantissa_exponent(1.0, 6000.0)
            .min((reincarnation_points + one).pow(Decimal::from_finite(6.0)));
    }
    if research(39) > 0.5 {
        global_crystal_multiplier *= pre
            .building_power_mult
            .pow(Decimal::from_finite(1.0 / 50.0));
    }
    global_crystal_multiplier *= Decimal::from_finite(1.0 + 0.01 * crystal_upgrade(0))
        .pow(Decimal::from_finite(achievement_points));
    let log10_coins_plus_1 = (coins + one).log(ten).to_number();
    global_crystal_multiplier *=
        Decimal::from_finite(1.0 + crystal_upgrade(1) * log10_coins_plus_1 / 100.0).pow(
            Decimal::from_finite(2.0 + (crystal_upgrade(1) + 1.0).log2()),
        );
    global_crystal_multiplier *= pre.crystal_upgrade_3_multiplier;
    global_crystal_multiplier *=
        Decimal::from_finite(1.0 + 0.05 * crystal_upgrade(4)).pow(Decimal::from_finite(
            c_completions(1)
                + c_completions(2)
                + c_completions(3)
                + c_completions(4)
                + c_completions(5),
        ));
    global_crystal_multiplier *= ten.pow(Decimal::from_finite(calc_ecc(
        ChallengeType::Transcend,
        c_completions(5),
    )));
    global_crystal_multiplier *= Decimal::from_finite(1e4).pow(Decimal::from_finite(
        research(5) * (1.0 + 0.5 * calc_ecc(ChallengeType::Ascension, c_completions(14))),
    ));
    global_crystal_multiplier *= Decimal::from_finite(2.5).pow(Decimal::from_finite(research(26)));
    global_crystal_multiplier *= Decimal::from_finite(2.5).pow(Decimal::from_finite(research(27)));

    // ─── Mythos multiplier ───────────────────────────────────────────────
    let mut global_mythos_multiplier = Decimal::one();
    if upgrade(37) > 0.5 {
        let log10_pp_plus_10 = (prestige_points + ten).log(ten);
        global_mythos_multiplier *= log10_pp_plus_10.pow(Decimal::from_finite(2.0));
    }
    if upgrade(42) > 0.5 {
        let inner = (prestige_points + one).pow(Decimal::from_finite(1.0 / 50.0))
            / Decimal::from_finite(2.5)
            + one;
        global_mythos_multiplier *= Decimal::from_finite(1e50).min(inner);
    }
    if upgrade(47) > 0.5 {
        global_mythos_multiplier *= Decimal::from_finite(1.01)
            .pow(Decimal::from_finite(achievement_points))
            * Decimal::from_finite(achievement_points / 5.0 + 1.0);
    }
    if upgrade(51) > 0.5 {
        global_mythos_multiplier *=
            Decimal::from_finite(pre.total_accelerator_boost).pow(Decimal::from_finite(2.0));
    }
    if upgrade(52) > 0.5 {
        global_mythos_multiplier *= global_mythos_multiplier.pow(Decimal::from_finite(0.025));
    }
    if upgrade(64) > 0.5 {
        global_mythos_multiplier *= (reincarnation_points + one).pow(Decimal::from_finite(2.0));
    }
    if research(40) > 0.5 {
        global_mythos_multiplier *= pre
            .building_power_mult
            .pow(Decimal::from_finite(1.0 / 250.0));
    }

    // ─── Grandmaster + challenge-3 + mythos-upgrade multipliers ─────────
    let mut grandmaster_multiplier = Decimal::one();
    let total_mythos_owned = first_owned_mythos
        + second_owned_mythos
        + third_owned_mythos
        + fourth_owned_mythos
        + fifth_owned_mythos;

    let mythos_building_power = 1.0 + calc_ecc(ChallengeType::Transcend, c_completions(3)) / 200.0;
    let challenge_three_multiplier =
        Decimal::from_finite(mythos_building_power).pow(Decimal::from_finite(total_mythos_owned));

    grandmaster_multiplier *= challenge_three_multiplier;

    let mut mythosupgrade_13 = Decimal::one();
    let mut mythosupgrade_14 = Decimal::one();
    let mut mythosupgrade_15 = Decimal::one();
    if (upgrade(53) - 1.0).abs() < f64::EPSILON {
        mythosupgrade_13 *= Decimal::from_mantissa_exponent(1.0, 1250.0).min(
            pre.accelerator_effect
                .pow(Decimal::from_finite(1.0 / 125.0)),
        );
    }
    if (upgrade(54) - 1.0).abs() < f64::EPSILON {
        mythosupgrade_14 *= Decimal::from_mantissa_exponent(1.0, 2000.0)
            .min(pre.multiplier_effect.pow(Decimal::from_finite(1.0 / 180.0)));
    }
    if (upgrade(55) - 1.0).abs() < f64::EPSILON {
        mythosupgrade_15 *= Decimal::from_mantissa_exponent(1.0, 1000.0).pow(Decimal::from_finite(
            1000.0_f64.min(pre.building_power - 1.0),
        ));
    }

    // ─── Constant (ascension) multiplier ────────────────────────────────
    let mut global_constant_mult = Decimal::one();
    global_constant_mult *= Decimal::from_finite(
        1.05 + pre.const_upgrade_1_buff_achievement + 0.001 * platonic_upgrade(18),
    )
    .pow(Decimal::from_finite(constant_upgrade(1)));
    let constant_upgrade_2_percent_cap = 100.0
        + 1000.0 * pre.const_upgrade_2_buff_achievement
        + 10.0 * pre.constant_ex_max_percent_increase
        + 1000.0 * (pre.challenge_15_exponent_value - 1.0)
        + 3.0 * platonic_upgrade(18);
    global_constant_mult *=
        Decimal::from_finite(1.0 + 0.001 * constant_upgrade_2_percent_cap.min(constant_upgrade(2)))
            .pow(Decimal::from_finite(pre.ascend_building_dr_value));
    global_constant_mult *= Decimal::from_finite(1.0 + (2.0 / 100.0) * research(139));
    global_constant_mult *= Decimal::from_finite(1.0 + (3.0 / 100.0) * research(154));
    global_constant_mult *= Decimal::from_finite(1.0 + (5.0 / 100.0) * research(184));
    global_constant_mult *= Decimal::from_finite(1.0 + (10.0 / 100.0) * research(199));
    global_constant_mult *= Decimal::from_finite(pre.challenge_15_constant_bonus);
    if platonic_upgrade(5) > 0.0 {
        global_constant_mult *= Decimal::from_finite(2.0);
    }
    if platonic_upgrade(10) > 0.0 {
        global_constant_mult *= Decimal::from_finite(10.0);
    }
    if platonic_upgrade(15) > 0.0 {
        global_constant_mult *= Decimal::from_finite(1e250);
    }
    global_constant_mult *= Decimal::from_finite(overflux_powder + 1.0)
        .pow(Decimal::from_finite(10.0 * platonic_upgrade(16)));

    GlobalMultipliersResult {
        global_coin_multiplier,
        coin_one_multi,
        coin_two_multi,
        coin_three_multi,
        coin_four_multi,
        coin_five_multi,
        global_crystal_multiplier,
        global_mythos_multiplier,
        grandmaster_multiplier,
        total_mythos_owned,
        mythos_building_power,
        challenge_three_multiplier,
        mythosupgrade_13,
        mythosupgrade_14,
        mythosupgrade_15,
        global_constant_mult,
        ant_multiplier: pre.ant_multiplier,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_state_produces_identity_multipliers() {
        let result = compute_global_multipliers(
            &GameState::default(),
            &GlobalMultipliersPreEvaluated::default(),
        );
        // With every flag off and unit pre-evaluated values, every multiplier
        // should collapse to one.
        assert_eq!(result.coin_one_multi.to_number(), 1.0);
        assert_eq!(result.coin_two_multi.to_number(), 1.0);
        assert_eq!(result.coin_three_multi.to_number(), 1.0);
        assert_eq!(result.coin_four_multi.to_number(), 1.0);
        assert_eq!(result.coin_five_multi.to_number(), 1.0);
        // `s` starts at 1, gets multiplied by 1's; globalCoinMultiplier = s^1.
        assert_eq!(result.global_coin_multiplier.to_number(), 1.0);
        assert_eq!(result.total_mythos_owned, 0.0);
        assert_eq!(result.mythos_building_power, 1.0);
        // antMultiplier pass-through.
        assert_eq!(result.ant_multiplier.to_number(), 1.0);
    }

    #[test]
    fn upgrade_56_multiplies_coin_one_by_1e5000() {
        let mut state = GameState::default();
        state.upgrades.upgrades[56] = 1;
        let result = compute_global_multipliers(&state, &GlobalMultipliersPreEvaluated::default());
        assert!(result.coin_one_multi.exponent() >= 4999.0);
        assert!(result.coin_one_multi.exponent() <= 5001.0);
    }

    #[test]
    fn upgrade_60_multiplies_coin_five_by_1e35000() {
        let mut state = GameState::default();
        state.upgrades.upgrades[60] = 1;
        let result = compute_global_multipliers(&state, &GlobalMultipliersPreEvaluated::default());
        assert!(result.coin_five_multi.exponent() >= 34_999.0);
        assert!(result.coin_five_multi.exponent() <= 35_001.0);
    }

    #[test]
    fn highest_singularity_count_adds_bonus_to_s() {
        let mut state = GameState::default();
        state.singularity.highest_singularity_count = 1.0;
        // bonus = (0+1)^1.5 * (1+1)^2 = 1 * 4 = 4; globalCoinMultiplier = s = 4.
        let result = compute_global_multipliers(&state, &GlobalMultipliersPreEvaluated::default());
        assert!((result.global_coin_multiplier.to_number() - 4.0).abs() < 1e-9);
    }

    #[test]
    fn reincarnation_challenge_6_divides_s_by_1e250() {
        let mut state = GameState::default();
        state.challenges.current_reincarnation_challenge = 6;
        let result = compute_global_multipliers(&state, &GlobalMultipliersPreEvaluated::default());
        assert!((result.global_coin_multiplier.exponent() - (-250.0)).abs() < 1e-6);
    }

    #[test]
    fn reincarnation_challenge_9_divides_s_by_1e2000000() {
        let mut state = GameState::default();
        state.challenges.current_reincarnation_challenge = 9;
        let result = compute_global_multipliers(&state, &GlobalMultipliersPreEvaluated::default());
        assert!((result.global_coin_multiplier.exponent() - (-2_000_000.0)).abs() < 1.0);
    }

    #[test]
    fn total_mythos_owned_sums_five_fields() {
        let mut state = GameState::default();
        state.mythos_producers.tiers[0].owned = 1.0;
        state.mythos_producers.tiers[1].owned = 2.0;
        state.mythos_producers.tiers[2].owned = 3.0;
        state.mythos_producers.tiers[3].owned = 4.0;
        state.mythos_producers.tiers[4].owned = 5.0;
        let result = compute_global_multipliers(&state, &GlobalMultipliersPreEvaluated::default());
        assert_eq!(result.total_mythos_owned, 15.0);
    }

    #[test]
    fn c3_completions_feed_mythos_building_power() {
        let mut state = GameState::default();
        // calc_ecc(transcend, c3=0) = 0 → mythos_building_power = 1.0.
        let r0 = compute_global_multipliers(&state, &GlobalMultipliersPreEvaluated::default());
        assert_eq!(r0.mythos_building_power, 1.0);
        // CalcECC ramps with completions; the value here just needs to be > 1.
        state.challenges.challenge_completions[3] = 50.0;
        let r1 = compute_global_multipliers(&state, &GlobalMultipliersPreEvaluated::default());
        assert!(r1.mythos_building_power > 1.0);
    }

    #[test]
    fn platonic_upgrade_5_doubles_global_constant_mult() {
        let state = GameState::default();
        let r0 = compute_global_multipliers(&state, &GlobalMultipliersPreEvaluated::default());
        let mut state2 = GameState::default();
        state2.cube_upgrade_levels.platonic_upgrades[5] = 1.0;
        let r1 = compute_global_multipliers(&state2, &GlobalMultipliersPreEvaluated::default());
        let r0_v = r0.global_constant_mult.to_number();
        let r1_v = r1.global_constant_mult.to_number();
        assert!((r1_v - 2.0 * r0_v).abs() < 1e-9);
    }

    #[test]
    fn platonic_upgrade_10_multiplies_global_constant_mult_by_10() {
        let state = GameState::default();
        let r0 = compute_global_multipliers(&state, &GlobalMultipliersPreEvaluated::default());
        let mut state2 = GameState::default();
        state2.cube_upgrade_levels.platonic_upgrades[10] = 1.0;
        let r1 = compute_global_multipliers(&state2, &GlobalMultipliersPreEvaluated::default());
        let r0_v = r0.global_constant_mult.to_number();
        let r1_v = r1.global_constant_mult.to_number();
        assert!((r1_v - 10.0 * r0_v).abs() < 1e-9);
    }
}
