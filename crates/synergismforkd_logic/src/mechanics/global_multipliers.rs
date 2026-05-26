//! Per-tick global multiplier aggregator.
//!
//! Verbatim port of
//! `legacy_core_split/packages/logic/src/mechanics/globalMultipliers.ts`.
//! Populates the 17 `G.*` multiplier fields the rest of the tick reads.
//! Composes ~30 pre-evaluated calls plus 50+ `player.*` / `G.*` reads —
//! the UI tier pre-evaluates everything and passes plain values in.
//!
//! Distinct from [`super::update_all_multiplier`], which computes the
//! `freeMultiplier` / `multiplierEffect` stack — the value of that
//! computation feeds back into this one via `multiplier_effect`.

use crate::mechanics::challenges::{calc_ecc, ChallengeType};
use synergismforkd_bignum::Decimal;

/// Inputs to [`compute_global_multipliers`].
#[derive(Debug, Clone, Copy)]
pub struct GlobalMultipliersInput {
    // ─── Direct player state — upgrades 1-60 (coin tier perks) ──────────
    /// `player.upgrades[1]` — gates `coinOneMulti × first6CoinUp`.
    pub upgrade_1: f64,
    /// `player.upgrades[2]` — gates `coinTwoMulti × first6CoinUp`.
    pub upgrade_2: f64,
    /// `player.upgrades[3]` — gates `coinThreeMulti × first6CoinUp`.
    pub upgrade_3: f64,
    /// `player.upgrades[4]` — gates `coinFourMulti × first6CoinUp`.
    pub upgrade_4: f64,
    /// `player.upgrades[5]` — gates `coinFiveMulti × first6CoinUp`.
    pub upgrade_5: f64,
    /// `player.upgrades[6]` — adds `first6CoinUp` to the main `s` multiplier.
    pub upgrade_6: f64,
    /// `player.upgrades[10]` — `coinOneMulti × 2 ^ min(50, secondOwnedCoin / 15)`.
    pub upgrade_10: f64,
    /// `player.upgrades[12]` — `s × min(1e4, 1.01^prestigeCount)`.
    pub upgrade_12: f64,
    /// `player.upgrades[13]` — `coinTwoMulti × min(1e50, (firstGenMythos + firstOwnMythos + 1)^(4/3) × 1e22)`.
    pub upgrade_13: f64,
    /// `player.upgrades[17]` — `coinFourMulti × 1e100`.
    pub upgrade_17: f64,
    /// `player.upgrades[18]` — `coinThreeMulti × min(1e125, transcendShards + 1)`.
    pub upgrade_18: f64,
    /// `player.upgrades[19]` — `coinTwoMulti × min(1e200, transcendPoints × 1e30 + 1)`.
    pub upgrade_19: f64,
    /// `player.upgrades[20]` — `s × (totalCoinOwned / 4 + 1)^10`.
    pub upgrade_20: f64,
    /// `player.upgrades[41]` — `s × min(1e30, (transcendPoints + 4)^0.5)`.
    pub upgrade_41: f64,
    /// `player.upgrades[43]` — `s × min(1e30, 1.01^transcendCount)`.
    pub upgrade_43: f64,
    /// `player.upgrades[48]` — `s × ((totalMultiplier × totalAccelerator) / 1000 + 1)^8`.
    pub upgrade_48: f64,
    /// `player.upgrades[56]` — `coinOneMulti × 1e5000`.
    pub upgrade_56: f64,
    /// `player.upgrades[57]` — `coinTwoMulti × 1e7500`.
    pub upgrade_57: f64,
    /// `player.upgrades[58]` — `coinThreeMulti × 1e15000`.
    pub upgrade_58: f64,
    /// `player.upgrades[59]` — `coinFourMulti × 1e25000`.
    pub upgrade_59: f64,
    /// `player.upgrades[60]` — `coinFiveMulti × 1e35000`.
    pub upgrade_60: f64,
    // ─── upgrades 36-67 (crystal / mythos tier) ──────────────────────────
    /// `player.upgrades[36]` — `globalCrystalMultiplier × min(1e5000, prestigePoints^(1/500))`.
    pub upgrade_36: f64,
    /// `player.upgrades[37]` — `globalMythosMultiplier × log10(prestigePoints + 10)^2`.
    pub upgrade_37: f64,
    /// `player.upgrades[42]` — `globalMythosMultiplier × min(1e50, (prestigePoints + 1)^(1/50) / 2.5 + 1)`.
    pub upgrade_42: f64,
    /// `player.upgrades[47]` — `globalMythosMultiplier × 1.01^aP × (aP / 5 + 1)`.
    pub upgrade_47: f64,
    /// `player.upgrades[51]` — `globalMythosMultiplier × totalAcceleratorBoost^2`.
    pub upgrade_51: f64,
    /// `player.upgrades[52]` — `globalMythosMultiplier × itself^0.025`.
    pub upgrade_52: f64,
    /// `player.upgrades[53]` — `mythosupgrade13 × min(1e1250, acceleratorEffect^(1/125))`.
    pub upgrade_53: f64,
    /// `player.upgrades[54]` — `mythosupgrade14 × min(1e2000, multiplierEffect^(1/180))`.
    pub upgrade_54: f64,
    /// `player.upgrades[55]` — `mythosupgrade15 × 1e1000 ^ min(1000, buildingPower - 1)`.
    pub upgrade_55: f64,
    /// `player.upgrades[63]` — `globalCrystalMultiplier × min(1e6000, (reincarnationPoints + 1)^6)`.
    pub upgrade_63: f64,
    /// `player.upgrades[64]` — `globalMythosMultiplier × (reincarnationPoints + 1)^2`.
    pub upgrade_64: f64,
    /// `player.upgrades[123]` — additional `1 + 0.025n` exponent on `c`.
    pub upgrade_123: f64,
    // ─── Researches ──────────────────────────────────────────────────────
    /// `player.researches[5]` — `globalCrystalMultiplier × 1e4 ^ (n × (1 + ECC(asc, c14) / 2))`.
    pub research_5: f64,
    /// `player.researches[17]` — `1 + 0.001n` exponent on `s` for `c`.
    pub research_17: f64,
    /// `player.researches[26]` — `globalCrystalMultiplier × 2.5^n`.
    pub research_26: f64,
    /// `player.researches[27]` — `globalCrystalMultiplier × 2.5^n`.
    pub research_27: f64,
    /// `player.researches[39]` — `globalCrystalMultiplier × buildingPowerMult^(1/50)`.
    pub research_39: f64,
    /// `player.researches[40]` — `globalMythosMultiplier × buildingPowerMult^(1/250)`.
    pub research_40: f64,
    /// `player.researches[139]` — `globalConstantMult × (1 + 0.02n)`.
    pub research_139: f64,
    /// `player.researches[154]` — `globalConstantMult × (1 + 0.03n)`.
    pub research_154: f64,
    /// `player.researches[184]` — `globalConstantMult × (1 + 0.05n)`.
    pub research_184: f64,
    /// `player.researches[199]` — `globalConstantMult × (1 + 0.10n)`.
    pub research_199: f64,
    // ─── Crystal / constant / platonic upgrades ──────────────────────────
    /// `player.crystalUpgrades[0]` — exponent base for the achievement-points-fueled bonus.
    pub crystal_upgrade_0: f64,
    /// `player.crystalUpgrades[1]` — feeds the `log10(coins + 1)` bonus + its `log2` exponent.
    pub crystal_upgrade_1: f64,
    /// `player.crystalUpgrades[4]` — exponent base for the `c1-c5` completion sum bonus.
    pub crystal_upgrade_4: f64,
    /// `player.constantUpgrades[1]` — `globalConstantMult ^ (1.05 + constUpgrade1Buff + 0.001 × plat18)`.
    pub constant_upgrade_1: f64,
    /// `player.constantUpgrades[2]` — bounded percentage bump fed into `globalConstantMult`.
    pub constant_upgrade_2: f64,
    /// `player.platonicUpgrades[5]` — `× 2` when `> 0`.
    pub platonic_upgrade_5: f64,
    /// `player.platonicUpgrades[10]` — `× 10` when `> 0`.
    pub platonic_upgrade_10: f64,
    /// `player.platonicUpgrades[14]` — a-chal 15 reincarnation-corrupted log10-coin exponent term.
    pub platonic_upgrade_14: f64,
    /// `player.platonicUpgrades[15]` — a-chal 15: `^1.1` on `lol` + `globalConstantMult × 1e250` when `> 0`.
    pub platonic_upgrade_15: f64,
    /// `player.platonicUpgrades[16]` — `(overfluxPowder + 1) ^ (10 × plat16)` multiplied into `globalConstantMult`.
    pub platonic_upgrade_16: f64,
    /// `player.platonicUpgrades[18]` — adds to `constUpgrade1` base + bounded contribution to `constUpgrade2` percent.
    pub platonic_upgrade_18: f64,
    // ─── Resources / counters ────────────────────────────────────────────
    /// `player.coins` — log10 base for `crystalUpgrade1` bonus + plat14 r-corruption term.
    pub coins: Decimal,
    /// `player.prestigePoints` — feeds upgrade-36, upgrade-37, upgrade-42 multipliers.
    pub prestige_points: Decimal,
    /// `player.transcendPoints` — feeds upgrade-19, upgrade-41 multipliers.
    pub transcend_points: Decimal,
    /// `player.reincarnationPoints` — feeds upgrade-63, upgrade-64 multipliers.
    pub reincarnation_points: Decimal,
    /// `player.transcendShards` — feeds upgrade-18 `coinThree` multiplier.
    pub transcend_shards: Decimal,
    /// `player.prestigeCount` — exponent for upgrade-12.
    pub prestige_count: f64,
    /// `player.transcendCount` — exponent for upgrade-43.
    pub transcend_count: f64,
    /// `player.highestSingularityCount` — gates the singularity `s` multiplier when `> 0`.
    pub highest_singularity_count: f64,
    /// `player.goldenQuarks` — base for `(goldenQuarks + 1)^1.5` singularity bonus.
    pub golden_quarks: f64,
    /// `player.overfluxPowder` — base for plat16 `globalConstantMult` bonus.
    pub overflux_powder: f64,
    // ─── Coin tier owned counts (for derived totals) ─────────────────────
    /// `player.secondOwnedCoin` — used in upgrade-10 (`2 ^ min(50, n / 15)`).
    pub second_owned_coin: f64,
    /// `player.firstGeneratedMythos` — feeds upgrade-13.
    pub first_generated_mythos: Decimal,
    /// `player.firstOwnedMythos` — feeds upgrade-13 + `globalMythosOwned`.
    pub first_owned_mythos: f64,
    /// `player.secondOwnedMythos` — feeds `totalMythosOwned`.
    pub second_owned_mythos: f64,
    /// `player.thirdOwnedMythos` — feeds `totalMythosOwned`.
    pub third_owned_mythos: f64,
    /// `player.fourthOwnedMythos` — feeds `totalMythosOwned`.
    pub fourth_owned_mythos: f64,
    /// `player.fifthOwnedMythos` — feeds `totalMythosOwned`.
    pub fifth_owned_mythos: f64,
    // ─── Challenge state ─────────────────────────────────────────────────
    /// `player.challengecompletions[1]` — sum feeds `crystalUpgrade4` exponent.
    pub c1_completions: f64,
    /// `player.challengecompletions[2]` — sum feeds `crystalUpgrade4` exponent.
    pub c2_completions: f64,
    /// `player.challengecompletions[3]` — feeds `mythosBuildingPower` (`+CalcECC(transcend, n)/200`).
    pub c3_completions: f64,
    /// `player.challengecompletions[4]` — sum feeds `crystalUpgrade4` exponent.
    pub c4_completions: f64,
    /// `player.challengecompletions[5]` — sum feeds `crystalUpgrade4` exponent + `globalCrystalMultiplier × 10^CalcECC(transcend, c5)`.
    pub c5_completions: f64,
    /// `player.challengecompletions[14]` — fed through `CalcECC` for `ecc14a`.
    pub c14_completions: f64,
    /// `player.currentChallenge.reincarnation` — r-chal 6/7/9 each divide `s` by a constant.
    pub reincarnation_challenge: u32,
    /// `player.currentChallenge.ascension` — gates `platonicUpgrade` 5/14/15 `lol` exponent terms.
    pub ascension_challenge: u32,
    /// `player.corruptions.used.recession` — feeds `recessionPower` lookup + plat14 exponent.
    pub recession_corruption_level: u32,
    // ─── Pre-evaluated values (already-migrated callers) ─────────────────
    /// `calculateCrystalCoinMultiplier()` — multiplied into `s`.
    pub crystal_mult: Decimal,
    /// `calculateBuildingPower()` — used in upgrade-55 `min(1000, buildingPower - 1)`.
    pub building_power: f64,
    /// `calculateBuildingPowerCoinMultiplier(buildingPower)` — multiplied into `s`, also feeds research-39/40.
    pub building_power_mult: Decimal,
    /// `calculateTotalCoinOwned()` — used in `first6CoinUp` + upgrade-20.
    pub total_coin_owned: f64,
    /// `getAntUpgradeEffect(AntUpgrades.Coins).coinMultiplier` — multiplied into `s`, also surfaced as `antMultiplier` output.
    pub ant_multiplier: Decimal,
    /// `crystalUpgrade3CrystalMultiplier()` — multiplied into `globalCrystalMultiplier`.
    pub crystal_upgrade_3_multiplier: Decimal,
    /// Module-level `achievementPoints` from `Achievements.ts` — feeds `crystalUpgrade0` exponent + upgrade-47.
    pub achievement_points: f64,
    /// `+getAchievementReward('crystalMultiplier')` — multiplied into `globalCrystalMultiplier`.
    pub crystal_multiplier_achievement: f64,
    /// `+getAchievementReward('constUpgrade1Buff')` — added to `constUpgrade1` exponent base.
    pub const_upgrade_1_buff_achievement: f64,
    /// `+getAchievementReward('constUpgrade2Buff')` — bounded coefficient inside `constUpgrade2`.
    pub const_upgrade_2_buff_achievement: f64,
    /// `getRuneEffects('prism', 'productionLog10')` — exponent of 10 multiplied into `globalCrystalMultiplier`.
    pub prism_production_log10: f64,
    /// `getShopUpgradeEffects('constantEX', 'maxPercentIncrease')` — added into `constUpgrade2` bounded percentage.
    pub constant_ex_max_percent_increase: f64,
    /// `ascendBuildingDR()` — exponent applied to `constUpgrade2` contribution.
    pub ascend_building_dr_value: f64,
    // ─── G inputs (pre-extracted by web_ui) ──────────────────────────────
    /// `G.multiplierEffect` — multiplied into `s` (set by `updateAllMultiplier` earlier this tick).
    pub multiplier_effect: Decimal,
    /// `G.acceleratorEffect` — multiplied into `s` (set by `updateAllTick` earlier this tick) + `mythosupgrade13`.
    pub accelerator_effect: Decimal,
    /// `G.totalMultiplier` — feeds upgrade-48 (`× totalAccelerator / 1000 + 1`).
    pub total_multiplier: f64,
    /// `G.totalAccelerator` — feeds upgrade-48.
    pub total_accelerator: f64,
    /// `G.totalAcceleratorBoost` — exponent base for upgrade-51.
    pub total_accelerator_boost: f64,
    /// `G.challenge15Rewards.coinExponent.value` — exponent of `lol → globalCoinMultiplier`.
    pub challenge_15_coin_exponent: f64,
    /// `G.challenge15Rewards.exponent.value` — `(exponent - 1) × 1000` is bounded into `constUpgrade2` percent.
    pub challenge_15_exponent_value: f64,
    /// `G.challenge15Rewards.constantBonus.value` — multiplied into `globalConstantMult`.
    pub challenge_15_constant_bonus: f64,
    /// `G.recessionPower[player.corruptions.used.recession]` — exponent applied to `globalCoinMultiplier`.
    pub recession_power: f64,
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
    /// `G.antMultiplier` — pass-through of `input.ant_multiplier`, surfaced
    /// for parity with the legacy implementation (the legacy code sets
    /// `G.antMultiplier` inside `multipliers()` so the shim must too).
    pub ant_multiplier: Decimal,
}

/// Per-tick global-multiplier aggregator. Direct transcription of the legacy
/// `multipliers()` body with input/output substitution — preserves the exact
/// computation order so parity holds across every code path.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn compute_global_multipliers(input: &GlobalMultipliersInput) -> GlobalMultipliersResult {
    let one = Decimal::one();
    let ten = Decimal::from_finite(10.0);

    let mut s = Decimal::one();
    s *= input.multiplier_effect;
    s *= input.accelerator_effect;
    s *= input.crystal_mult;
    s *= input.building_power_mult;
    s *= input.ant_multiplier;

    let first_6_coin_up = Decimal::from_finite(input.total_coin_owned + 1.0)
        * Decimal::from_finite(1e30)
            .min(Decimal::from_finite(1.008).pow(Decimal::from_finite(input.total_coin_owned)));

    if input.highest_singularity_count > 0.0 {
        let bonus = (input.golden_quarks + 1.0).powf(1.5)
            * (input.highest_singularity_count + 1.0).powf(2.0);
        s *= Decimal::from_finite(bonus);
    }
    if input.upgrade_6 > 0.5 {
        s *= first_6_coin_up;
    }
    if input.upgrade_12 > 0.5 {
        s *= Decimal::from_finite(1e4)
            .min(Decimal::from_finite(1.01).pow(Decimal::from_finite(input.prestige_count)));
    }
    if input.upgrade_20 > 0.5 {
        s *= Decimal::from_finite(input.total_coin_owned / 4.0 + 1.0)
            .pow(Decimal::from_finite(10.0));
    }
    if input.upgrade_41 > 0.5 {
        s *= Decimal::from_finite(1e30).min(
            (input.transcend_points + Decimal::from_finite(4.0)).pow(Decimal::from_finite(0.5)),
        );
    }
    if input.upgrade_43 > 0.5 {
        s *= Decimal::from_finite(1e30)
            .min(Decimal::from_finite(1.01).pow(Decimal::from_finite(input.transcend_count)));
    }
    if input.upgrade_48 > 0.5 {
        s *=
            Decimal::from_finite((input.total_multiplier * input.total_accelerator) / 1000.0 + 1.0)
                .pow(Decimal::from_finite(8.0));
    }
    if input.reincarnation_challenge == 6 {
        s /= Decimal::from_finite(1e250);
    }
    if input.reincarnation_challenge == 7 {
        s /= Decimal::from_mantissa_exponent(1.0, 1250.0);
    }
    if input.reincarnation_challenge == 9 {
        s /= Decimal::from_mantissa_exponent(1.0, 2_000_000.0);
    }
    let c = s.pow(Decimal::from_finite(1.0 + 0.001 * input.research_17));
    let mut lol = c.pow(Decimal::from_finite(1.0 + 0.025 * input.upgrade_123));
    if input.ascension_challenge == 15 && input.platonic_upgrade_5 > 0.0 {
        lol = lol.pow(Decimal::from_finite(1.1));
    }
    if input.ascension_challenge == 15 && input.platonic_upgrade_14 > 0.0 {
        let log10_coins_plus_1 = (input.coins + one).log(ten).to_number();
        let exponent = 1.0
            + ((1.0 / 20.0) * f64::from(input.recession_corruption_level) * log10_coins_plus_1)
                / (1e7 + log10_coins_plus_1);
        lol = lol.pow(Decimal::from_finite(exponent));
    }
    if input.ascension_challenge == 15 && input.platonic_upgrade_15 > 0.0 {
        lol = lol.pow(Decimal::from_finite(1.1));
    }
    lol = lol.pow(Decimal::from_finite(input.challenge_15_coin_exponent));
    let mut global_coin_multiplier = lol;
    global_coin_multiplier =
        global_coin_multiplier.pow(Decimal::from_finite(input.recession_power));

    let mut coin_one_multi = Decimal::one();
    if input.upgrade_1 > 0.5 {
        coin_one_multi *= first_6_coin_up;
    }
    if input.upgrade_10 > 0.5 {
        coin_one_multi *= Decimal::from_finite(2.0).pow(Decimal::from_finite(
            50.0_f64.min(input.second_owned_coin / 15.0),
        ));
    }
    if input.upgrade_56 > 0.5 {
        coin_one_multi *= Decimal::from_mantissa_exponent(1.0, 5000.0);
    }

    let mut coin_two_multi = Decimal::one();
    if input.upgrade_2 > 0.5 {
        coin_two_multi *= first_6_coin_up;
    }
    if input.upgrade_13 > 0.5 {
        let inner =
            (input.first_generated_mythos + Decimal::from_finite(input.first_owned_mythos) + one)
                .pow(Decimal::from_finite(4.0 / 3.0))
                * Decimal::from_finite(1e22);
        coin_two_multi *= Decimal::from_finite(1e50).min(inner);
    }
    if input.upgrade_19 > 0.5 {
        coin_two_multi *= Decimal::from_finite(1e200)
            .min(input.transcend_points * Decimal::from_finite(1e30) + one);
    }
    if input.upgrade_57 > 0.5 {
        coin_two_multi *= Decimal::from_mantissa_exponent(1.0, 7500.0);
    }

    let mut coin_three_multi = Decimal::one();
    if input.upgrade_3 > 0.5 {
        coin_three_multi *= first_6_coin_up;
    }
    if input.upgrade_18 > 0.5 {
        coin_three_multi *= Decimal::from_finite(1e125).min(input.transcend_shards + one);
    }
    if input.upgrade_58 > 0.5 {
        coin_three_multi *= Decimal::from_mantissa_exponent(1.0, 15_000.0);
    }

    let mut coin_four_multi = Decimal::one();
    if input.upgrade_4 > 0.5 {
        coin_four_multi *= first_6_coin_up;
    }
    if input.upgrade_17 > 0.5 {
        coin_four_multi *= Decimal::from_finite(1e100);
    }
    if input.upgrade_59 > 0.5 {
        coin_four_multi *= Decimal::from_mantissa_exponent(1.0, 25_000.0);
    }

    let mut coin_five_multi = Decimal::one();
    if input.upgrade_5 > 0.5 {
        coin_five_multi *= first_6_coin_up;
    }
    if input.upgrade_60 > 0.5 {
        coin_five_multi *= Decimal::from_mantissa_exponent(1.0, 35_000.0);
    }

    let mut global_crystal_multiplier = Decimal::one();
    global_crystal_multiplier *= Decimal::from_finite(input.crystal_multiplier_achievement);
    global_crystal_multiplier *= ten.pow(Decimal::from_finite(input.prism_production_log10));
    if input.upgrade_36 > 0.5 {
        global_crystal_multiplier *= Decimal::from_mantissa_exponent(1.0, 5000.0)
            .min(input.prestige_points.pow(Decimal::from_finite(1.0 / 500.0)));
    }
    if input.upgrade_63 > 0.5 {
        global_crystal_multiplier *= Decimal::from_mantissa_exponent(1.0, 6000.0)
            .min((input.reincarnation_points + one).pow(Decimal::from_finite(6.0)));
    }
    if input.research_39 > 0.5 {
        global_crystal_multiplier *= input
            .building_power_mult
            .pow(Decimal::from_finite(1.0 / 50.0));
    }
    global_crystal_multiplier *= Decimal::from_finite(1.0 + 0.01 * input.crystal_upgrade_0)
        .pow(Decimal::from_finite(input.achievement_points));
    let log10_coins_plus_1 = (input.coins + one).log(ten).to_number();
    global_crystal_multiplier *=
        Decimal::from_finite(1.0 + input.crystal_upgrade_1 * log10_coins_plus_1 / 100.0).pow(
            Decimal::from_finite(2.0 + (input.crystal_upgrade_1 + 1.0).log2()),
        );
    global_crystal_multiplier *= input.crystal_upgrade_3_multiplier;
    global_crystal_multiplier *=
        Decimal::from_finite(1.0 + 0.05 * input.crystal_upgrade_4).pow(Decimal::from_finite(
            input.c1_completions
                + input.c2_completions
                + input.c3_completions
                + input.c4_completions
                + input.c5_completions,
        ));
    global_crystal_multiplier *= ten.pow(Decimal::from_finite(calc_ecc(
        ChallengeType::Transcend,
        input.c5_completions,
    )));
    global_crystal_multiplier *= Decimal::from_finite(1e4).pow(Decimal::from_finite(
        input.research_5 * (1.0 + 0.5 * calc_ecc(ChallengeType::Ascension, input.c14_completions)),
    ));
    global_crystal_multiplier *=
        Decimal::from_finite(2.5).pow(Decimal::from_finite(input.research_26));
    global_crystal_multiplier *=
        Decimal::from_finite(2.5).pow(Decimal::from_finite(input.research_27));

    let mut global_mythos_multiplier = Decimal::one();
    if input.upgrade_37 > 0.5 {
        let log10_pp_plus_10 = (input.prestige_points + ten).log(ten);
        global_mythos_multiplier *= log10_pp_plus_10.pow(Decimal::from_finite(2.0));
    }
    if input.upgrade_42 > 0.5 {
        let inner = (input.prestige_points + one).pow(Decimal::from_finite(1.0 / 50.0))
            / Decimal::from_finite(2.5)
            + one;
        global_mythos_multiplier *= Decimal::from_finite(1e50).min(inner);
    }
    if input.upgrade_47 > 0.5 {
        global_mythos_multiplier *= Decimal::from_finite(1.01)
            .pow(Decimal::from_finite(input.achievement_points))
            * Decimal::from_finite(input.achievement_points / 5.0 + 1.0);
    }
    if input.upgrade_51 > 0.5 {
        global_mythos_multiplier *=
            Decimal::from_finite(input.total_accelerator_boost).pow(Decimal::from_finite(2.0));
    }
    if input.upgrade_52 > 0.5 {
        global_mythos_multiplier *= global_mythos_multiplier.pow(Decimal::from_finite(0.025));
    }
    if input.upgrade_64 > 0.5 {
        global_mythos_multiplier *=
            (input.reincarnation_points + one).pow(Decimal::from_finite(2.0));
    }
    if input.research_40 > 0.5 {
        global_mythos_multiplier *= input
            .building_power_mult
            .pow(Decimal::from_finite(1.0 / 250.0));
    }

    let mut grandmaster_multiplier = Decimal::one();
    let total_mythos_owned = input.first_owned_mythos
        + input.second_owned_mythos
        + input.third_owned_mythos
        + input.fourth_owned_mythos
        + input.fifth_owned_mythos;

    let mythos_building_power =
        1.0 + calc_ecc(ChallengeType::Transcend, input.c3_completions) / 200.0;
    let challenge_three_multiplier =
        Decimal::from_finite(mythos_building_power).pow(Decimal::from_finite(total_mythos_owned));

    grandmaster_multiplier *= challenge_three_multiplier;

    let mut mythosupgrade_13 = Decimal::one();
    let mut mythosupgrade_14 = Decimal::one();
    let mut mythosupgrade_15 = Decimal::one();
    if (input.upgrade_53 - 1.0).abs() < f64::EPSILON {
        mythosupgrade_13 *= Decimal::from_mantissa_exponent(1.0, 1250.0).min(
            input
                .accelerator_effect
                .pow(Decimal::from_finite(1.0 / 125.0)),
        );
    }
    if (input.upgrade_54 - 1.0).abs() < f64::EPSILON {
        mythosupgrade_14 *= Decimal::from_mantissa_exponent(1.0, 2000.0).min(
            input
                .multiplier_effect
                .pow(Decimal::from_finite(1.0 / 180.0)),
        );
    }
    if (input.upgrade_55 - 1.0).abs() < f64::EPSILON {
        mythosupgrade_15 *= Decimal::from_mantissa_exponent(1.0, 1000.0).pow(Decimal::from_finite(
            1000.0_f64.min(input.building_power - 1.0),
        ));
    }

    let mut global_constant_mult = Decimal::one();
    global_constant_mult *= Decimal::from_finite(
        1.05 + input.const_upgrade_1_buff_achievement + 0.001 * input.platonic_upgrade_18,
    )
    .pow(Decimal::from_finite(input.constant_upgrade_1));
    let constant_upgrade_2_percent_cap = 100.0
        + 1000.0 * input.const_upgrade_2_buff_achievement
        + 10.0 * input.constant_ex_max_percent_increase
        + 1000.0 * (input.challenge_15_exponent_value - 1.0)
        + 3.0 * input.platonic_upgrade_18;
    global_constant_mult *= Decimal::from_finite(
        1.0 + 0.001 * constant_upgrade_2_percent_cap.min(input.constant_upgrade_2),
    )
    .pow(Decimal::from_finite(input.ascend_building_dr_value));
    global_constant_mult *= Decimal::from_finite(1.0 + (2.0 / 100.0) * input.research_139);
    global_constant_mult *= Decimal::from_finite(1.0 + (3.0 / 100.0) * input.research_154);
    global_constant_mult *= Decimal::from_finite(1.0 + (5.0 / 100.0) * input.research_184);
    global_constant_mult *= Decimal::from_finite(1.0 + (10.0 / 100.0) * input.research_199);
    global_constant_mult *= Decimal::from_finite(input.challenge_15_constant_bonus);
    if input.platonic_upgrade_5 > 0.0 {
        global_constant_mult *= Decimal::from_finite(2.0);
    }
    if input.platonic_upgrade_10 > 0.0 {
        global_constant_mult *= Decimal::from_finite(10.0);
    }
    if input.platonic_upgrade_15 > 0.0 {
        global_constant_mult *= Decimal::from_finite(1e250);
    }
    global_constant_mult *= Decimal::from_finite(input.overflux_powder + 1.0)
        .pow(Decimal::from_finite(10.0 * input.platonic_upgrade_16));

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
        ant_multiplier: input.ant_multiplier,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_input() -> GlobalMultipliersInput {
        GlobalMultipliersInput {
            // upgrades 1-60
            upgrade_1: 0.0,
            upgrade_2: 0.0,
            upgrade_3: 0.0,
            upgrade_4: 0.0,
            upgrade_5: 0.0,
            upgrade_6: 0.0,
            upgrade_10: 0.0,
            upgrade_12: 0.0,
            upgrade_13: 0.0,
            upgrade_17: 0.0,
            upgrade_18: 0.0,
            upgrade_19: 0.0,
            upgrade_20: 0.0,
            upgrade_41: 0.0,
            upgrade_43: 0.0,
            upgrade_48: 0.0,
            upgrade_56: 0.0,
            upgrade_57: 0.0,
            upgrade_58: 0.0,
            upgrade_59: 0.0,
            upgrade_60: 0.0,
            // upgrades 36-67
            upgrade_36: 0.0,
            upgrade_37: 0.0,
            upgrade_42: 0.0,
            upgrade_47: 0.0,
            upgrade_51: 0.0,
            upgrade_52: 0.0,
            upgrade_53: 0.0,
            upgrade_54: 0.0,
            upgrade_55: 0.0,
            upgrade_63: 0.0,
            upgrade_64: 0.0,
            upgrade_123: 0.0,
            // researches
            research_5: 0.0,
            research_17: 0.0,
            research_26: 0.0,
            research_27: 0.0,
            research_39: 0.0,
            research_40: 0.0,
            research_139: 0.0,
            research_154: 0.0,
            research_184: 0.0,
            research_199: 0.0,
            // crystal / constant / platonic
            crystal_upgrade_0: 0.0,
            crystal_upgrade_1: 0.0,
            crystal_upgrade_4: 0.0,
            constant_upgrade_1: 0.0,
            constant_upgrade_2: 0.0,
            platonic_upgrade_5: 0.0,
            platonic_upgrade_10: 0.0,
            platonic_upgrade_14: 0.0,
            platonic_upgrade_15: 0.0,
            platonic_upgrade_16: 0.0,
            platonic_upgrade_18: 0.0,
            // resources / counters
            coins: Decimal::zero(),
            prestige_points: Decimal::zero(),
            transcend_points: Decimal::zero(),
            reincarnation_points: Decimal::zero(),
            transcend_shards: Decimal::zero(),
            prestige_count: 0.0,
            transcend_count: 0.0,
            highest_singularity_count: 0.0,
            golden_quarks: 0.0,
            overflux_powder: 0.0,
            // coin / mythos owned
            second_owned_coin: 0.0,
            first_generated_mythos: Decimal::zero(),
            first_owned_mythos: 0.0,
            second_owned_mythos: 0.0,
            third_owned_mythos: 0.0,
            fourth_owned_mythos: 0.0,
            fifth_owned_mythos: 0.0,
            // challenge state
            c1_completions: 0.0,
            c2_completions: 0.0,
            c3_completions: 0.0,
            c4_completions: 0.0,
            c5_completions: 0.0,
            c14_completions: 0.0,
            reincarnation_challenge: 0,
            ascension_challenge: 0,
            recession_corruption_level: 0,
            // pre-evaluated
            crystal_mult: Decimal::one(),
            building_power: 1.0,
            building_power_mult: Decimal::one(),
            total_coin_owned: 0.0,
            ant_multiplier: Decimal::one(),
            crystal_upgrade_3_multiplier: Decimal::one(),
            achievement_points: 0.0,
            crystal_multiplier_achievement: 1.0,
            const_upgrade_1_buff_achievement: 0.0,
            const_upgrade_2_buff_achievement: 0.0,
            prism_production_log10: 0.0,
            constant_ex_max_percent_increase: 0.0,
            ascend_building_dr_value: 1.0,
            // G inputs
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

    #[test]
    fn baseline_input_produces_identity_multipliers() {
        let result = compute_global_multipliers(&baseline_input());
        // With all flags off and unit pre-evaluated values, every multiplier
        // should collapse to one.
        assert_eq!(result.coin_one_multi.to_number(), 1.0);
        assert_eq!(result.coin_two_multi.to_number(), 1.0);
        assert_eq!(result.coin_three_multi.to_number(), 1.0);
        assert_eq!(result.coin_four_multi.to_number(), 1.0);
        assert_eq!(result.coin_five_multi.to_number(), 1.0);
        // s starts at 1 and is multiplied by 1's; globalCoinMultiplier = s^1.
        assert_eq!(result.global_coin_multiplier.to_number(), 1.0);
        assert_eq!(result.total_mythos_owned, 0.0);
        assert_eq!(result.mythos_building_power, 1.0);
        // antMultiplier pass-through.
        assert_eq!(result.ant_multiplier.to_number(), 1.0);
    }

    #[test]
    fn upgrade_56_multiplies_coin_one_by_1e5000() {
        let mut input = baseline_input();
        input.upgrade_56 = 1.0;
        let result = compute_global_multipliers(&input);
        // 1e5000 is the only contributor → coin_one_multi has exponent ≈ 5000.
        assert!(result.coin_one_multi.exponent() >= 4999.0);
        assert!(result.coin_one_multi.exponent() <= 5001.0);
    }

    #[test]
    fn upgrade_60_multiplies_coin_five_by_1e35000() {
        let mut input = baseline_input();
        input.upgrade_60 = 1.0;
        let result = compute_global_multipliers(&input);
        assert!(result.coin_five_multi.exponent() >= 34_999.0);
        assert!(result.coin_five_multi.exponent() <= 35_001.0);
    }

    #[test]
    fn highest_singularity_count_adds_bonus_to_s() {
        let mut input = baseline_input();
        input.highest_singularity_count = 1.0;
        input.golden_quarks = 0.0;
        // bonus = (0+1)^1.5 * (1+1)^2 = 1 * 4 = 4
        let result = compute_global_multipliers(&input);
        // global_coin_multiplier = (s^(1+0.001*0))^(1+0.025*0)^... = s = 4
        assert!((result.global_coin_multiplier.to_number() - 4.0).abs() < 1e-9);
    }

    #[test]
    fn reincarnation_challenge_6_divides_s_by_1e250() {
        let mut input = baseline_input();
        input.reincarnation_challenge = 6;
        let result = compute_global_multipliers(&input);
        // s = 1 / 1e250 → globalCoinMultiplier ≈ 1e-250.
        assert!((result.global_coin_multiplier.exponent() - (-250.0)).abs() < 1e-6);
    }

    #[test]
    fn reincarnation_challenge_9_divides_s_by_1e2000000() {
        let mut input = baseline_input();
        input.reincarnation_challenge = 9;
        let result = compute_global_multipliers(&input);
        assert!((result.global_coin_multiplier.exponent() - (-2_000_000.0)).abs() < 1.0);
    }

    #[test]
    fn total_mythos_owned_sums_five_fields() {
        let mut input = baseline_input();
        input.first_owned_mythos = 1.0;
        input.second_owned_mythos = 2.0;
        input.third_owned_mythos = 3.0;
        input.fourth_owned_mythos = 4.0;
        input.fifth_owned_mythos = 5.0;
        let result = compute_global_multipliers(&input);
        assert_eq!(result.total_mythos_owned, 15.0);
    }

    #[test]
    fn c3_completions_feed_mythos_building_power() {
        let mut input = baseline_input();
        // CalcECC(transcend, c3=0) = 0 → mythos_building_power = 1.0
        let r0 = compute_global_multipliers(&input);
        assert_eq!(r0.mythos_building_power, 1.0);
        // CalcECC ramps with completions; the value here just needs to be > 1.
        input.c3_completions = 50.0;
        let r1 = compute_global_multipliers(&input);
        assert!(r1.mythos_building_power > 1.0);
    }

    #[test]
    fn platonic_upgrade_5_doubles_global_constant_mult() {
        let mut input = baseline_input();
        let r0 = compute_global_multipliers(&input);
        input.platonic_upgrade_5 = 1.0;
        let r1 = compute_global_multipliers(&input);
        let r0_v = r0.global_constant_mult.to_number();
        let r1_v = r1.global_constant_mult.to_number();
        assert!((r1_v - 2.0 * r0_v).abs() < 1e-9);
    }

    #[test]
    fn platonic_upgrade_10_multiplies_global_constant_mult_by_10() {
        let mut input = baseline_input();
        let r0 = compute_global_multipliers(&input);
        input.platonic_upgrade_10 = 1.0;
        let r1 = compute_global_multipliers(&input);
        let r0_v = r0.global_constant_mult.to_number();
        let r1_v = r1.global_constant_mult.to_number();
        assert!((r1_v - 10.0 * r0_v).abs() < 1e-9);
    }
}
