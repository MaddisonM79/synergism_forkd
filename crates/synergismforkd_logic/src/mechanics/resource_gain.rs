//! Per-tick resource generation.
//!
//! Verbatim port of
//! `legacy_core_split/packages/logic/src/mechanics/resourceGain.ts`. Lifted
//! from `packages/web_ui/src/Synergism.ts` (`resourceGain`), minus the
//! terminal challenge `resetCheck` dispatch which is async + modal-aware and
//! stays in the UI tier.
//!
//! Computes:
//!   1. Coin gain (5 coin counters when `produceTotal ≥ 0.001`).
//!   2. Per-tier point gains from upgrade-93 / upgrade-100 / cubeUpgrade-28.
//!   3. Four producer cascades (Diamonds, Mythos, Particles, AscendBuildings)
//!      — each computes its 5 `G.produce*Tier` fields then advances 4 generated
//!      counters from the next tier's production.
//!   4. Shard accumulation (prestige/transcend/reincarnation/ascend).
//!   5. `awardAchievementGroup('constant')` gate (`ascensionCount > 0`).
//!   6. Challenge 1-5 auto-completion (research-gated coin thresholds).
//!
//! Side effects surface as [`CoreEvent`]s:
//!   - `AchievementGroupAwarded { group: AchievementGroup::Constant }`
//!   - `ChallengeAutoCompleted { challenge_index, new_completions }` (one per
//!     c1-c5 increment)
//!
//! The caller orchestrates the pre-tick functions
//! (`calculate_total_accelerator_boost`, `update_all_tick`,
//! `update_all_multiplier`, `compute_global_multipliers`, `calculate_tax`,
//! `reset_currency`) before invoking this so all G inputs are fresh.

use crate::events::{AchievementGroup, CoreEvent};
use synergismforkd_bignum::Decimal;

/// Inputs to [`resource_gain`].
#[derive(Debug, Clone, Copy)]
pub struct ResourceGainInput {
    /// Tick delta in seconds (already scaled by `globalSpeedMult` by the
    /// caller).
    pub dt: f64,
    // ─── Coin gain inputs ────────────────────────────────────────────────
    /// `G.produceTotal`.
    pub produce_total: Decimal,
    /// `G.taxdivisor`.
    pub taxdivisor: Decimal,
    /// `G.taxdivisorcheck`.
    pub taxdivisorcheck: Decimal,
    /// `G.maxexponent`.
    pub maxexponent: f64,
    /// `player.coins`.
    pub coins: Decimal,
    /// `player.coinsThisPrestige`.
    pub coins_this_prestige: Decimal,
    /// `player.coinsThisTranscension`.
    pub coins_this_transcension: Decimal,
    /// `player.coinsThisReincarnation`.
    pub coins_this_reincarnation: Decimal,
    /// `player.coinsTotal`.
    pub coins_total: Decimal,
    // ─── Point gain inputs ───────────────────────────────────────────────
    /// `player.upgrades[93]` — when `=== 1` and `coinsThisPrestige >= 1e16`,
    /// drips `prestigePoints`.
    pub upgrade_93: f64,
    /// `player.upgrades[100]` — when `=== 1` and `coinsThisTranscension >= 1e100`,
    /// drips `transcendPoints`.
    pub upgrade_100: f64,
    /// `player.cubeUpgrades[28]` — when `> 0` and `transcendShards >= 1e300`,
    /// drips `reincarnationPoints`.
    pub cube_upgrade_28: f64,
    /// `player.prestigePoints`.
    pub prestige_points: Decimal,
    /// `player.transcendPoints`.
    pub transcend_points: Decimal,
    /// `player.reincarnationPoints`.
    pub reincarnation_points: Decimal,
    /// `G.prestigePointGain`.
    pub prestige_point_gain: Decimal,
    /// `G.transcendPointGain`.
    pub transcend_point_gain: Decimal,
    /// `G.reincarnationPointGain`.
    pub reincarnation_point_gain: Decimal,
    // ─── Diamond cascade ─────────────────────────────────────────────────
    /// `player.firstGeneratedDiamonds`.
    pub first_generated_diamonds: Decimal,
    /// `player.secondGeneratedDiamonds`.
    pub second_generated_diamonds: Decimal,
    /// `player.thirdGeneratedDiamonds`.
    pub third_generated_diamonds: Decimal,
    /// `player.fourthGeneratedDiamonds`.
    pub fourth_generated_diamonds: Decimal,
    /// `player.fifthGeneratedDiamonds`.
    pub fifth_generated_diamonds: Decimal,
    /// `player.firstOwnedDiamonds`.
    pub first_owned_diamonds: f64,
    /// `player.secondOwnedDiamonds`.
    pub second_owned_diamonds: f64,
    /// `player.thirdOwnedDiamonds`.
    pub third_owned_diamonds: f64,
    /// `player.fourthOwnedDiamonds`.
    pub fourth_owned_diamonds: f64,
    /// `player.fifthOwnedDiamonds`.
    pub fifth_owned_diamonds: f64,
    /// `G.produceFirstDiamonds` base factor (per-unit production rate).
    pub first_produce_diamonds: f64,
    /// `G.produceSecondDiamonds` base factor.
    pub second_produce_diamonds: f64,
    /// `G.produceThirdDiamonds` base factor.
    pub third_produce_diamonds: f64,
    /// `G.produceFourthDiamonds` base factor.
    pub fourth_produce_diamonds: f64,
    /// `G.produceFifthDiamonds` base factor.
    pub fifth_produce_diamonds: f64,
    /// `G.globalCrystalMultiplier`.
    pub global_crystal_multiplier: Decimal,
    // ─── Mythos cascade ──────────────────────────────────────────────────
    /// `player.firstGeneratedMythos`.
    pub first_generated_mythos: Decimal,
    /// `player.secondGeneratedMythos`.
    pub second_generated_mythos: Decimal,
    /// `player.thirdGeneratedMythos`.
    pub third_generated_mythos: Decimal,
    /// `player.fourthGeneratedMythos`.
    pub fourth_generated_mythos: Decimal,
    /// `player.fifthGeneratedMythos`.
    pub fifth_generated_mythos: Decimal,
    /// `player.firstOwnedMythos`.
    pub first_owned_mythos: f64,
    /// `player.secondOwnedMythos`.
    pub second_owned_mythos: f64,
    /// `player.thirdOwnedMythos`.
    pub third_owned_mythos: f64,
    /// `player.fourthOwnedMythos`.
    pub fourth_owned_mythos: f64,
    /// `player.fifthOwnedMythos`.
    pub fifth_owned_mythos: f64,
    /// `G.produceFirstMythos` base factor.
    pub first_produce_mythos: f64,
    /// `G.produceSecondMythos` base factor.
    pub second_produce_mythos: f64,
    /// `G.produceThirdMythos` base factor.
    pub third_produce_mythos: f64,
    /// `G.produceFourthMythos` base factor.
    pub fourth_produce_mythos: f64,
    /// `G.produceFifthMythos` base factor.
    pub fifth_produce_mythos: f64,
    /// `G.globalMythosMultiplier`.
    pub global_mythos_multiplier: Decimal,
    /// `G.grandmasterMultiplier` — only the fifth tier multiplies by this.
    pub grandmaster_multiplier: Decimal,
    /// `G.mythosupgrade13` — only the first tier multiplies by this.
    pub mythosupgrade_13: Decimal,
    /// `G.mythosupgrade14` — only the third tier multiplies by this.
    pub mythosupgrade_14: Decimal,
    /// `G.mythosupgrade15` — only the fifth tier multiplies by this.
    pub mythosupgrade_15: Decimal,
    // ─── Particle cascade ────────────────────────────────────────────────
    /// `player.firstGeneratedParticles`.
    pub first_generated_particles: Decimal,
    /// `player.secondGeneratedParticles`.
    pub second_generated_particles: Decimal,
    /// `player.thirdGeneratedParticles`.
    pub third_generated_particles: Decimal,
    /// `player.fourthGeneratedParticles`.
    pub fourth_generated_particles: Decimal,
    /// `player.fifthGeneratedParticles`.
    pub fifth_generated_particles: Decimal,
    /// `player.firstOwnedParticles`.
    pub first_owned_particles: f64,
    /// `player.secondOwnedParticles`.
    pub second_owned_particles: f64,
    /// `player.thirdOwnedParticles`.
    pub third_owned_particles: f64,
    /// `player.fourthOwnedParticles`.
    pub fourth_owned_particles: f64,
    /// `player.fifthOwnedParticles`.
    pub fifth_owned_particles: f64,
    /// `G.produceFirstParticles` base factor.
    pub first_produce_particles: f64,
    /// `G.produceSecondParticles` base factor.
    pub second_produce_particles: f64,
    /// `G.produceThirdParticles` base factor.
    pub third_produce_particles: f64,
    /// `G.produceFourthParticles` base factor.
    pub fourth_produce_particles: f64,
    /// `G.produceFifthParticles` base factor.
    pub fifth_produce_particles: f64,
    /// `player.upgrades[67]` — when `> 0.5`, applies `pm = 1.03 ^
    /// totalOwnedParticles` to the first tier.
    pub upgrade_67: f64,
    // ─── Shards ──────────────────────────────────────────────────────────
    /// `player.prestigeShards`.
    pub prestige_shards: Decimal,
    /// `player.transcendShards`.
    pub transcend_shards: Decimal,
    /// `player.reincarnationShards`.
    pub reincarnation_shards: Decimal,
    /// `player.ascendShards`.
    pub ascend_shards: Decimal,
    // ─── AscendBuildings ─────────────────────────────────────────────────
    /// `player.ascendBuilding1.generated`.
    pub ascend_building_1_generated: Decimal,
    /// `player.ascendBuilding2.generated`.
    pub ascend_building_2_generated: Decimal,
    /// `player.ascendBuilding3.generated`.
    pub ascend_building_3_generated: Decimal,
    /// `player.ascendBuilding4.generated`.
    pub ascend_building_4_generated: Decimal,
    /// `player.ascendBuilding5.generated`.
    pub ascend_building_5_generated: Decimal,
    /// `player.ascendBuilding1.owned`.
    pub ascend_building_1_owned: f64,
    /// `player.ascendBuilding2.owned`.
    pub ascend_building_2_owned: f64,
    /// `player.ascendBuilding3.owned`.
    pub ascend_building_3_owned: f64,
    /// `player.ascendBuilding4.owned`.
    pub ascend_building_4_owned: f64,
    /// `player.ascendBuilding5.owned`.
    pub ascend_building_5_owned: f64,
    /// `G.globalConstantMult`.
    pub global_constant_mult: Decimal,
    // ─── Achievement + challenge auto-completion inputs ──────────────────
    /// `player.ascensionCount` — gates the `awardAchievementGroup('constant')`
    /// event.
    pub ascension_count: f64,
    /// `player.currentChallenge.transcension` — disables `prestigeShards` /
    /// `transcendShards` gains when `=== 3`.
    pub transcension_challenge: u32,
    /// `player.currentChallenge.reincarnation` — also disables both shards
    /// branches when `=== 10`.
    pub reincarnation_challenge: u32,
    /// `player.researches[66]` — c1 highest-cap multiplier.
    pub research_66: f64,
    /// `player.researches[67]` — c2 highest-cap multiplier.
    pub research_67: f64,
    /// `player.researches[68]` — c3 highest-cap multiplier.
    pub research_68: f64,
    /// `player.researches[69]` — c4 highest-cap multiplier.
    pub research_69: f64,
    /// `player.researches[70]` — c5 highest-cap multiplier.
    pub research_70: f64,
    /// `player.researches[71]` — c1 auto-completion gate.
    pub research_71: f64,
    /// `player.researches[72]` — c2 auto-completion gate.
    pub research_72: f64,
    /// `player.researches[73]` — c3 auto-completion gate.
    pub research_73: f64,
    /// `player.researches[74]` — c4 auto-completion gate.
    pub research_74: f64,
    /// `player.researches[75]` — c5 auto-completion gate.
    pub research_75: f64,
    /// `player.researches[105]` — `+925 ×` to each c1-c5 highest cap.
    pub research_105: f64,
    /// `player.challengecompletions[1]`.
    pub c1_completions: f64,
    /// `player.challengecompletions[2]`.
    pub c2_completions: f64,
    /// `player.challengecompletions[3]`.
    pub c3_completions: f64,
    /// `player.challengecompletions[4]`.
    pub c4_completions: f64,
    /// `player.challengecompletions[5]`.
    pub c5_completions: f64,
    /// `player.highestchallengecompletions[1]`.
    pub highest_c1: f64,
    /// `player.highestchallengecompletions[2]`.
    pub highest_c2: f64,
    /// `player.highestchallengecompletions[3]`.
    pub highest_c3: f64,
    /// `player.highestchallengecompletions[4]`.
    pub highest_c4: f64,
    /// `player.highestchallengecompletions[5]`.
    pub highest_c5: f64,
    /// `G.challengeBaseRequirements[0..=4]` — log10 coin thresholds for c1-c5.
    pub challenge_base_requirements: [f64; 5],
}

/// `G.ascendBuildingProduction` — the five per-tick production values for the
/// AscendBuildings cascade. Surfaced as a sub-struct on
/// [`ResourceGainResult`] so the legacy `G.ascendBuildingProduction.{first,
/// second, third, fourth, fifth}` reads can stay structurally close to the
/// TS port.
#[derive(Debug, Clone, Copy)]
pub struct AscendBuildingProduction {
    /// `G.ascendBuildingProduction.first`.
    pub first: Decimal,
    /// `G.ascendBuildingProduction.second`.
    pub second: Decimal,
    /// `G.ascendBuildingProduction.third`.
    pub third: Decimal,
    /// `G.ascendBuildingProduction.fourth`.
    pub fourth: Decimal,
    /// `G.ascendBuildingProduction.fifth`.
    pub fifth: Decimal,
}

/// Result of [`resource_gain`]. Every field is the post-tick value, even if
/// unchanged.
#[derive(Debug, Clone)]
pub struct ResourceGainResult {
    // ─── Coin counters ───────────────────────────────────────────────────
    /// `player.coins`.
    pub coins: Decimal,
    /// `player.coinsThisPrestige`.
    pub coins_this_prestige: Decimal,
    /// `player.coinsThisTranscension`.
    pub coins_this_transcension: Decimal,
    /// `player.coinsThisReincarnation`.
    pub coins_this_reincarnation: Decimal,
    /// `player.coinsTotal`.
    pub coins_total: Decimal,
    // ─── Tier points ─────────────────────────────────────────────────────
    /// `player.prestigePoints`.
    pub prestige_points: Decimal,
    /// `player.transcendPoints`.
    pub transcend_points: Decimal,
    /// `player.reincarnationPoints`.
    pub reincarnation_points: Decimal,
    // ─── Shards ──────────────────────────────────────────────────────────
    /// `player.prestigeShards`.
    pub prestige_shards: Decimal,
    /// `player.transcendShards`.
    pub transcend_shards: Decimal,
    /// `player.reincarnationShards`.
    pub reincarnation_shards: Decimal,
    /// `player.ascendShards`.
    pub ascend_shards: Decimal,
    // ─── Generated counters (first..fourth tier; fifth never updates) ────
    /// `player.firstGeneratedDiamonds`.
    pub first_generated_diamonds: Decimal,
    /// `player.secondGeneratedDiamonds`.
    pub second_generated_diamonds: Decimal,
    /// `player.thirdGeneratedDiamonds`.
    pub third_generated_diamonds: Decimal,
    /// `player.fourthGeneratedDiamonds`.
    pub fourth_generated_diamonds: Decimal,
    /// `player.firstGeneratedMythos`.
    pub first_generated_mythos: Decimal,
    /// `player.secondGeneratedMythos`.
    pub second_generated_mythos: Decimal,
    /// `player.thirdGeneratedMythos`.
    pub third_generated_mythos: Decimal,
    /// `player.fourthGeneratedMythos`.
    pub fourth_generated_mythos: Decimal,
    /// `player.firstGeneratedParticles`.
    pub first_generated_particles: Decimal,
    /// `player.secondGeneratedParticles`.
    pub second_generated_particles: Decimal,
    /// `player.thirdGeneratedParticles`.
    pub third_generated_particles: Decimal,
    /// `player.fourthGeneratedParticles`.
    pub fourth_generated_particles: Decimal,
    /// `player.ascendBuilding1.generated`.
    pub ascend_building_1_generated: Decimal,
    /// `player.ascendBuilding2.generated`.
    pub ascend_building_2_generated: Decimal,
    /// `player.ascendBuilding3.generated`.
    pub ascend_building_3_generated: Decimal,
    /// `player.ascendBuilding4.generated`.
    pub ascend_building_4_generated: Decimal,
    // ─── Challenge completions (auto-completion can increment these) ─────
    /// `player.challengecompletions[1]`.
    pub c1_completions: f64,
    /// `player.challengecompletions[2]`.
    pub c2_completions: f64,
    /// `player.challengecompletions[3]`.
    pub c3_completions: f64,
    /// `player.challengecompletions[4]`.
    pub c4_completions: f64,
    /// `player.challengecompletions[5]`.
    pub c5_completions: f64,
    // ─── G cache updates ─────────────────────────────────────────────────
    /// `G.produceFirstDiamonds`.
    pub produce_first_diamonds: Decimal,
    /// `G.produceSecondDiamonds`.
    pub produce_second_diamonds: Decimal,
    /// `G.produceThirdDiamonds`.
    pub produce_third_diamonds: Decimal,
    /// `G.produceFourthDiamonds`.
    pub produce_fourth_diamonds: Decimal,
    /// `G.produceFifthDiamonds`.
    pub produce_fifth_diamonds: Decimal,
    /// `G.produceDiamonds` — equal to `produce_first_diamonds`.
    pub produce_diamonds: Decimal,
    /// `G.produceFirstMythos` — uses post-tick `firstGeneratedMythos`.
    pub produce_first_mythos: Decimal,
    /// `G.produceSecondMythos`.
    pub produce_second_mythos: Decimal,
    /// `G.produceThirdMythos`.
    pub produce_third_mythos: Decimal,
    /// `G.produceFourthMythos`.
    pub produce_fourth_mythos: Decimal,
    /// `G.produceFifthMythos`.
    pub produce_fifth_mythos: Decimal,
    /// `G.produceMythos` — recomputed after the mutation pass.
    pub produce_mythos: Decimal,
    /// `G.producePerSecondMythos` — `produce_mythos × 40`.
    pub produce_per_second_mythos: Decimal,
    /// `G.produceFirstParticles`.
    pub produce_first_particles: Decimal,
    /// `G.produceSecondParticles`.
    pub produce_second_particles: Decimal,
    /// `G.produceThirdParticles`.
    pub produce_third_particles: Decimal,
    /// `G.produceFourthParticles`.
    pub produce_fourth_particles: Decimal,
    /// `G.produceFifthParticles`.
    pub produce_fifth_particles: Decimal,
    /// `G.produceParticles` — recomputed after the mutation pass.
    pub produce_particles: Decimal,
    /// `G.producePerSecondParticles` — `produce_particles × 40`.
    pub produce_per_second_particles: Decimal,
    /// `G.ascendBuildingProduction`.
    pub ascend_building_production: AscendBuildingProduction,
    /// Events for the UI tier to dispatch.
    pub events: Vec<CoreEvent>,
}

/// Per-tick resource generation. Pure given the input bundle; returns the
/// full post-tick player + G slice plus an event list.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn resource_gain(input: &ResourceGainInput) -> ResourceGainResult {
    let dt = input.dt;
    let dt_scaled = dt / 0.025;
    let dt_scaled_dec = Decimal::from_finite(dt_scaled);
    let dt_dec = Decimal::from_finite(dt);
    let ten = Decimal::from_finite(10.0);
    let one = Decimal::one();
    let mut events: Vec<CoreEvent> = Vec::new();

    // ─── Coin gain ───────────────────────────────────────────────────────
    let mut coins = input.coins;
    let mut coins_this_prestige = input.coins_this_prestige;
    let mut coins_this_transcension = input.coins_this_transcension;
    let mut coins_this_reincarnation = input.coins_this_reincarnation;
    let mut coins_total = input.coins_total;
    if input.produce_total >= Decimal::from_finite(0.001) {
        let cap_exponent = input.maxexponent - input.taxdivisorcheck.log(ten).to_number();
        let addcoin = (input.produce_total / input.taxdivisor)
            .min(ten.pow(Decimal::from_finite(cap_exponent)))
            * dt_scaled_dec;
        coins += addcoin;
        coins_this_prestige += addcoin;
        coins_this_transcension += addcoin;
        coins_this_reincarnation += addcoin;
        coins_total += addcoin;
    }

    // ─── Point gains ─────────────────────────────────────────────────────
    let mut prestige_points = input.prestige_points;
    let mut transcend_points = input.transcend_points;
    let mut reincarnation_points = input.reincarnation_points;
    if (input.upgrade_93 - 1.0).abs() < f64::EPSILON
        && coins_this_prestige >= Decimal::from_finite(1e16)
    {
        prestige_points +=
            (input.prestige_point_gain / Decimal::from_finite(4000.0) * dt_scaled_dec).floor();
    }
    if (input.upgrade_100 - 1.0).abs() < f64::EPSILON
        && coins_this_transcension >= Decimal::from_finite(1e100)
    {
        transcend_points +=
            (input.transcend_point_gain / Decimal::from_finite(4000.0) * dt_scaled_dec).floor();
    }
    if input.cube_upgrade_28 > 0.0
        && input.transcend_shards >= Decimal::from_mantissa_exponent(1.0, 300.0)
    {
        reincarnation_points +=
            (input.reincarnation_point_gain / Decimal::from_finite(4000.0) * dt_scaled_dec).floor();
    }

    // ─── Diamond cascade ─────────────────────────────────────────────────
    let gcm = input.global_crystal_multiplier;
    let produce_first_diamonds = (input.first_generated_diamonds
        + Decimal::from_finite(input.first_owned_diamonds))
        * Decimal::from_finite(input.first_produce_diamonds)
        * gcm;
    let produce_second_diamonds = (input.second_generated_diamonds
        + Decimal::from_finite(input.second_owned_diamonds))
        * Decimal::from_finite(input.second_produce_diamonds)
        * gcm;
    let produce_third_diamonds = (input.third_generated_diamonds
        + Decimal::from_finite(input.third_owned_diamonds))
        * Decimal::from_finite(input.third_produce_diamonds)
        * gcm;
    let produce_fourth_diamonds = (input.fourth_generated_diamonds
        + Decimal::from_finite(input.fourth_owned_diamonds))
        * Decimal::from_finite(input.fourth_produce_diamonds)
        * gcm;
    let produce_fifth_diamonds = (input.fifth_generated_diamonds
        + Decimal::from_finite(input.fifth_owned_diamonds))
        * Decimal::from_finite(input.fifth_produce_diamonds)
        * gcm;

    let fourth_generated_diamonds =
        input.fourth_generated_diamonds + produce_fifth_diamonds * dt_scaled_dec;
    let third_generated_diamonds =
        input.third_generated_diamonds + produce_fourth_diamonds * dt_scaled_dec;
    let second_generated_diamonds =
        input.second_generated_diamonds + produce_third_diamonds * dt_scaled_dec;
    let first_generated_diamonds =
        input.first_generated_diamonds + produce_second_diamonds * dt_scaled_dec;
    let produce_diamonds = produce_first_diamonds;

    let mut prestige_shards = input.prestige_shards;
    if input.transcension_challenge != 3 && input.reincarnation_challenge != 10 {
        prestige_shards += produce_diamonds * dt_scaled_dec;
    }

    // ─── Mythos cascade ──────────────────────────────────────────────────
    let gmm = input.global_mythos_multiplier;
    let produce_fifth_mythos = (input.fifth_generated_mythos
        + Decimal::from_finite(input.fifth_owned_mythos))
        * Decimal::from_finite(input.fifth_produce_mythos)
        * gmm
        * input.grandmaster_multiplier
        * input.mythosupgrade_15;
    let produce_fourth_mythos = (input.fourth_generated_mythos
        + Decimal::from_finite(input.fourth_owned_mythos))
        * Decimal::from_finite(input.fourth_produce_mythos)
        * gmm;
    let produce_third_mythos = (input.third_generated_mythos
        + Decimal::from_finite(input.third_owned_mythos))
        * Decimal::from_finite(input.third_produce_mythos)
        * gmm
        * input.mythosupgrade_14;
    let produce_second_mythos = (input.second_generated_mythos
        + Decimal::from_finite(input.second_owned_mythos))
        * Decimal::from_finite(input.second_produce_mythos)
        * gmm;
    let produce_first_mythos = (input.first_generated_mythos
        + Decimal::from_finite(input.first_owned_mythos))
        * Decimal::from_finite(input.first_produce_mythos)
        * gmm
        * input.mythosupgrade_13;

    let fourth_generated_mythos =
        input.fourth_generated_mythos + produce_fifth_mythos * dt_scaled_dec;
    let third_generated_mythos =
        input.third_generated_mythos + produce_fourth_mythos * dt_scaled_dec;
    let second_generated_mythos =
        input.second_generated_mythos + produce_third_mythos * dt_scaled_dec;
    let first_generated_mythos =
        input.first_generated_mythos + produce_second_mythos * dt_scaled_dec;

    // produceMythos: recomputed after mutations using post-tick first_generated_mythos.
    let produce_mythos = (first_generated_mythos + Decimal::from_finite(input.first_owned_mythos))
        * Decimal::from_finite(input.first_produce_mythos)
        * gmm
        * input.mythosupgrade_13;
    let produce_per_second_mythos = produce_mythos * Decimal::from_finite(40.0);

    // ─── Particle cascade ────────────────────────────────────────────────
    let mut pm = Decimal::one();
    if input.upgrade_67 > 0.5 {
        let total_owned = input.first_owned_particles
            + input.second_owned_particles
            + input.third_owned_particles
            + input.fourth_owned_particles
            + input.fifth_owned_particles;
        pm *= Decimal::from_finite(1.03).pow(Decimal::from_finite(total_owned));
    }

    let produce_fifth_particles = (input.fifth_generated_particles
        + Decimal::from_finite(input.fifth_owned_particles))
        * Decimal::from_finite(input.fifth_produce_particles);
    let produce_fourth_particles = (input.fourth_generated_particles
        + Decimal::from_finite(input.fourth_owned_particles))
        * Decimal::from_finite(input.fourth_produce_particles);
    let produce_third_particles = (input.third_generated_particles
        + Decimal::from_finite(input.third_owned_particles))
        * Decimal::from_finite(input.third_produce_particles);
    let produce_second_particles = (input.second_generated_particles
        + Decimal::from_finite(input.second_owned_particles))
        * Decimal::from_finite(input.second_produce_particles);
    let produce_first_particles = (input.first_generated_particles
        + Decimal::from_finite(input.first_owned_particles))
        * Decimal::from_finite(input.first_produce_particles)
        * pm;

    let fourth_generated_particles =
        input.fourth_generated_particles + produce_fifth_particles * dt_scaled_dec;
    let third_generated_particles =
        input.third_generated_particles + produce_fourth_particles * dt_scaled_dec;
    let second_generated_particles =
        input.second_generated_particles + produce_third_particles * dt_scaled_dec;
    let first_generated_particles =
        input.first_generated_particles + produce_second_particles * dt_scaled_dec;

    // produceParticles: recomputed after mutations using post-tick first_generated_particles.
    let produce_particles = (first_generated_particles
        + Decimal::from_finite(input.first_owned_particles))
        * Decimal::from_finite(input.first_produce_particles)
        * pm;
    let produce_per_second_particles = produce_particles * Decimal::from_finite(40.0);

    // ─── Transcend / reincarnation shards ────────────────────────────────
    let mut transcend_shards = input.transcend_shards;
    if input.transcension_challenge != 3 && input.reincarnation_challenge != 10 {
        transcend_shards += produce_mythos * dt_scaled_dec;
    }
    let reincarnation_shards = input.reincarnation_shards + produce_particles * dt_scaled_dec;

    // ─── AscendBuildings cascade (raw `dt`, not `dt / 0.025`) ────────────
    let ascend_owned = [
        input.ascend_building_1_owned,
        input.ascend_building_2_owned,
        input.ascend_building_3_owned,
        input.ascend_building_4_owned,
        input.ascend_building_5_owned,
    ];
    let mut ascend_generated = [
        input.ascend_building_1_generated,
        input.ascend_building_2_generated,
        input.ascend_building_3_generated,
        input.ascend_building_4_generated,
        input.ascend_building_5_generated,
    ];
    let mut ascend_prod = [Decimal::zero(); 5];
    for j in (0..5).rev() {
        ascend_prod[j] = (ascend_generated[j] + Decimal::from_finite(ascend_owned[j]))
            * Decimal::from_finite(0.05)
            * input.global_constant_mult;
        if j != 0 {
            ascend_generated[j - 1] += ascend_prod[j] * dt_dec;
        }
    }
    let ascend_shards = input.ascend_shards + ascend_prod[0] * dt_dec;

    // ─── awardAchievementGroup('constant') gate ──────────────────────────
    if input.ascension_count > 0.0 {
        events.push(CoreEvent::AchievementGroupAwarded {
            group: AchievementGroup::Constant,
        });
    }

    // ─── Challenge 1-5 auto-completion ───────────────────────────────────
    let mut c1_completions = input.c1_completions;
    let mut c2_completions = input.c2_completions;
    let mut c3_completions = input.c3_completions;
    let mut c4_completions = input.c4_completions;
    let mut c5_completions = input.c5_completions;

    let auto_cap = |highest: f64, research: f64| -> f64 {
        highest.min(25.0 + 5.0 * research + 925.0 * input.research_105)
    };
    let threshold = |coef: f64, base: f64, completions: f64| -> Decimal {
        ten.pow(Decimal::from_finite(
            coef * base * (1.0 + completions).powf(2.0),
        ))
    };

    if input.research_71 > 0.5
        && c1_completions < auto_cap(input.highest_c1, input.research_66)
        && coins >= threshold(1.25, input.challenge_base_requirements[0], c1_completions)
    {
        c1_completions += 1.0;
        events.push(CoreEvent::ChallengeAutoCompleted {
            challenge_index: 1,
            new_completions: c1_completions,
        });
    }
    if input.research_72 > 0.5
        && c2_completions < auto_cap(input.highest_c2, input.research_67)
        && coins >= threshold(1.6, input.challenge_base_requirements[1], c2_completions)
    {
        c2_completions += 1.0;
        events.push(CoreEvent::ChallengeAutoCompleted {
            challenge_index: 2,
            new_completions: c2_completions,
        });
    }
    if input.research_73 > 0.5
        && c3_completions < auto_cap(input.highest_c3, input.research_68)
        && coins >= threshold(1.7, input.challenge_base_requirements[2], c3_completions)
    {
        c3_completions += 1.0;
        events.push(CoreEvent::ChallengeAutoCompleted {
            challenge_index: 3,
            new_completions: c3_completions,
        });
    }
    if input.research_74 > 0.5
        && c4_completions < auto_cap(input.highest_c4, input.research_69)
        && coins >= threshold(1.45, input.challenge_base_requirements[3], c4_completions)
    {
        c4_completions += 1.0;
        events.push(CoreEvent::ChallengeAutoCompleted {
            challenge_index: 4,
            new_completions: c4_completions,
        });
    }
    if input.research_75 > 0.5
        && c5_completions < auto_cap(input.highest_c5, input.research_70)
        && coins >= threshold(2.0, input.challenge_base_requirements[4], c5_completions)
    {
        c5_completions += 1.0;
        events.push(CoreEvent::ChallengeAutoCompleted {
            challenge_index: 5,
            new_completions: c5_completions,
        });
    }

    // Suppress an unused-variable warning — `one` is reserved for the
    // verbatim parity reads that match the TS body's `Decimal('1')` calls,
    // even though every site happens to inline a constant for now.
    let _ = one;

    ResourceGainResult {
        coins,
        coins_this_prestige,
        coins_this_transcension,
        coins_this_reincarnation,
        coins_total,
        prestige_points,
        transcend_points,
        reincarnation_points,
        prestige_shards,
        transcend_shards,
        reincarnation_shards,
        ascend_shards,
        first_generated_diamonds,
        second_generated_diamonds,
        third_generated_diamonds,
        fourth_generated_diamonds,
        first_generated_mythos,
        second_generated_mythos,
        third_generated_mythos,
        fourth_generated_mythos,
        first_generated_particles,
        second_generated_particles,
        third_generated_particles,
        fourth_generated_particles,
        ascend_building_1_generated: ascend_generated[0],
        ascend_building_2_generated: ascend_generated[1],
        ascend_building_3_generated: ascend_generated[2],
        ascend_building_4_generated: ascend_generated[3],
        c1_completions,
        c2_completions,
        c3_completions,
        c4_completions,
        c5_completions,
        produce_first_diamonds,
        produce_second_diamonds,
        produce_third_diamonds,
        produce_fourth_diamonds,
        produce_fifth_diamonds,
        produce_diamonds,
        produce_first_mythos,
        produce_second_mythos,
        produce_third_mythos,
        produce_fourth_mythos,
        produce_fifth_mythos,
        produce_mythos,
        produce_per_second_mythos,
        produce_first_particles,
        produce_second_particles,
        produce_third_particles,
        produce_fourth_particles,
        produce_fifth_particles,
        produce_particles,
        produce_per_second_particles,
        ascend_building_production: AscendBuildingProduction {
            first: ascend_prod[0],
            second: ascend_prod[1],
            third: ascend_prod[2],
            fourth: ascend_prod[3],
            fifth: ascend_prod[4],
        },
        events,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_input() -> ResourceGainInput {
        ResourceGainInput {
            dt: 0.025,
            produce_total: Decimal::zero(),
            taxdivisor: Decimal::one(),
            taxdivisorcheck: Decimal::one(),
            maxexponent: 1e308,
            coins: Decimal::zero(),
            coins_this_prestige: Decimal::zero(),
            coins_this_transcension: Decimal::zero(),
            coins_this_reincarnation: Decimal::zero(),
            coins_total: Decimal::zero(),
            upgrade_93: 0.0,
            upgrade_100: 0.0,
            cube_upgrade_28: 0.0,
            prestige_points: Decimal::zero(),
            transcend_points: Decimal::zero(),
            reincarnation_points: Decimal::zero(),
            prestige_point_gain: Decimal::zero(),
            transcend_point_gain: Decimal::zero(),
            reincarnation_point_gain: Decimal::zero(),
            first_generated_diamonds: Decimal::zero(),
            second_generated_diamonds: Decimal::zero(),
            third_generated_diamonds: Decimal::zero(),
            fourth_generated_diamonds: Decimal::zero(),
            fifth_generated_diamonds: Decimal::zero(),
            first_owned_diamonds: 0.0,
            second_owned_diamonds: 0.0,
            third_owned_diamonds: 0.0,
            fourth_owned_diamonds: 0.0,
            fifth_owned_diamonds: 0.0,
            first_produce_diamonds: 0.0,
            second_produce_diamonds: 0.0,
            third_produce_diamonds: 0.0,
            fourth_produce_diamonds: 0.0,
            fifth_produce_diamonds: 0.0,
            global_crystal_multiplier: Decimal::one(),
            first_generated_mythos: Decimal::zero(),
            second_generated_mythos: Decimal::zero(),
            third_generated_mythos: Decimal::zero(),
            fourth_generated_mythos: Decimal::zero(),
            fifth_generated_mythos: Decimal::zero(),
            first_owned_mythos: 0.0,
            second_owned_mythos: 0.0,
            third_owned_mythos: 0.0,
            fourth_owned_mythos: 0.0,
            fifth_owned_mythos: 0.0,
            first_produce_mythos: 0.0,
            second_produce_mythos: 0.0,
            third_produce_mythos: 0.0,
            fourth_produce_mythos: 0.0,
            fifth_produce_mythos: 0.0,
            global_mythos_multiplier: Decimal::one(),
            grandmaster_multiplier: Decimal::one(),
            mythosupgrade_13: Decimal::one(),
            mythosupgrade_14: Decimal::one(),
            mythosupgrade_15: Decimal::one(),
            first_generated_particles: Decimal::zero(),
            second_generated_particles: Decimal::zero(),
            third_generated_particles: Decimal::zero(),
            fourth_generated_particles: Decimal::zero(),
            fifth_generated_particles: Decimal::zero(),
            first_owned_particles: 0.0,
            second_owned_particles: 0.0,
            third_owned_particles: 0.0,
            fourth_owned_particles: 0.0,
            fifth_owned_particles: 0.0,
            first_produce_particles: 0.0,
            second_produce_particles: 0.0,
            third_produce_particles: 0.0,
            fourth_produce_particles: 0.0,
            fifth_produce_particles: 0.0,
            upgrade_67: 0.0,
            prestige_shards: Decimal::zero(),
            transcend_shards: Decimal::zero(),
            reincarnation_shards: Decimal::zero(),
            ascend_shards: Decimal::zero(),
            ascend_building_1_generated: Decimal::zero(),
            ascend_building_2_generated: Decimal::zero(),
            ascend_building_3_generated: Decimal::zero(),
            ascend_building_4_generated: Decimal::zero(),
            ascend_building_5_generated: Decimal::zero(),
            ascend_building_1_owned: 0.0,
            ascend_building_2_owned: 0.0,
            ascend_building_3_owned: 0.0,
            ascend_building_4_owned: 0.0,
            ascend_building_5_owned: 0.0,
            global_constant_mult: Decimal::one(),
            ascension_count: 0.0,
            transcension_challenge: 0,
            reincarnation_challenge: 0,
            research_66: 0.0,
            research_67: 0.0,
            research_68: 0.0,
            research_69: 0.0,
            research_70: 0.0,
            research_71: 0.0,
            research_72: 0.0,
            research_73: 0.0,
            research_74: 0.0,
            research_75: 0.0,
            research_105: 0.0,
            c1_completions: 0.0,
            c2_completions: 0.0,
            c3_completions: 0.0,
            c4_completions: 0.0,
            c5_completions: 0.0,
            highest_c1: 0.0,
            highest_c2: 0.0,
            highest_c3: 0.0,
            highest_c4: 0.0,
            highest_c5: 0.0,
            challenge_base_requirements: [10.0, 100.0, 1000.0, 10000.0, 100000.0],
        }
    }

    #[test]
    fn baseline_is_a_noop_for_resources() {
        let r = resource_gain(&baseline_input());
        assert_eq!(r.coins.to_number(), 0.0);
        assert_eq!(r.coins_total.to_number(), 0.0);
        assert_eq!(r.prestige_points.to_number(), 0.0);
        assert_eq!(r.transcend_points.to_number(), 0.0);
        assert_eq!(r.reincarnation_points.to_number(), 0.0);
        assert_eq!(r.prestige_shards.to_number(), 0.0);
        assert_eq!(r.transcend_shards.to_number(), 0.0);
        assert_eq!(r.reincarnation_shards.to_number(), 0.0);
        assert_eq!(r.ascend_shards.to_number(), 0.0);
        assert!(r.events.is_empty());
    }

    #[test]
    fn produce_total_above_threshold_adds_coins() {
        let mut input = baseline_input();
        input.produce_total = Decimal::from_finite(100.0); // > 0.001
                                                           // addcoin = min(100/1, 10^(1e308 - 0)) * (0.025/0.025) = 100
        let r = resource_gain(&input);
        assert_eq!(r.coins.to_number(), 100.0);
        assert_eq!(r.coins_this_prestige.to_number(), 100.0);
        assert_eq!(r.coins_this_transcension.to_number(), 100.0);
        assert_eq!(r.coins_this_reincarnation.to_number(), 100.0);
        assert_eq!(r.coins_total.to_number(), 100.0);
    }

    #[test]
    fn produce_total_below_threshold_is_skipped() {
        let mut input = baseline_input();
        input.produce_total = Decimal::from_finite(1e-6); // < 0.001
        let r = resource_gain(&input);
        assert_eq!(r.coins.to_number(), 0.0);
    }

    #[test]
    fn upgrade_93_drips_prestige_points_once_coins_cross_1e16() {
        let mut input = baseline_input();
        input.upgrade_93 = 1.0;
        input.coins_this_prestige = Decimal::from_finite(1e16);
        input.prestige_point_gain = Decimal::from_finite(8000.0);
        // floor(8000 / 4000 * 1) = floor(2) = 2
        let r = resource_gain(&input);
        assert_eq!(r.prestige_points.to_number(), 2.0);
    }

    #[test]
    fn upgrade_93_no_drip_below_threshold() {
        let mut input = baseline_input();
        input.upgrade_93 = 1.0;
        input.coins_this_prestige = Decimal::from_finite(1e15); // below 1e16
        input.prestige_point_gain = Decimal::from_finite(8000.0);
        let r = resource_gain(&input);
        assert_eq!(r.prestige_points.to_number(), 0.0);
    }

    #[test]
    fn transcension_challenge_3_disables_shard_gains() {
        let mut input = baseline_input();
        input.transcension_challenge = 3;
        input.first_generated_diamonds = Decimal::from_finite(10.0);
        input.first_produce_diamonds = 1.0;
        let r = resource_gain(&input);
        assert_eq!(r.prestige_shards.to_number(), 0.0);
        assert_eq!(r.transcend_shards.to_number(), 0.0);
        // reincarnation_shards is NOT gated by t-chal 3.
    }

    #[test]
    fn reincarnation_challenge_10_disables_shard_gains() {
        let mut input = baseline_input();
        input.reincarnation_challenge = 10;
        input.first_generated_diamonds = Decimal::from_finite(10.0);
        input.first_produce_diamonds = 1.0;
        let r = resource_gain(&input);
        assert_eq!(r.prestige_shards.to_number(), 0.0);
        assert_eq!(r.transcend_shards.to_number(), 0.0);
    }

    #[test]
    fn ascension_count_above_zero_emits_constant_group_event() {
        let mut input = baseline_input();
        input.ascension_count = 1.0;
        let r = resource_gain(&input);
        assert!(r.events.iter().any(|e| matches!(
            e,
            CoreEvent::AchievementGroupAwarded {
                group: AchievementGroup::Constant
            }
        )));
    }

    #[test]
    fn challenge_1_auto_completes_when_gates_open() {
        let mut input = baseline_input();
        input.research_71 = 1.0;
        input.highest_c1 = 5.0;
        input.coins = Decimal::from_finite(1e100); // huge — easily clears threshold
        let r = resource_gain(&input);
        assert_eq!(r.c1_completions, 1.0);
        assert!(r.events.iter().any(|e| matches!(
            e,
            CoreEvent::ChallengeAutoCompleted {
                challenge_index: 1,
                ..
            }
        )));
    }

    #[test]
    fn challenge_1_does_not_auto_complete_when_at_cap() {
        let mut input = baseline_input();
        input.research_71 = 1.0;
        input.highest_c1 = 5.0;
        input.c1_completions = 5.0; // already at highest
        input.coins = Decimal::from_finite(1e100);
        let r = resource_gain(&input);
        assert_eq!(r.c1_completions, 5.0);
    }

    #[test]
    fn diamond_cascade_propagates_one_step() {
        let mut input = baseline_input();
        // fifth tier generates → fourth gets fifth's production.
        input.fifth_generated_diamonds = Decimal::from_finite(10.0);
        input.fifth_produce_diamonds = 1.0;
        let r = resource_gain(&input);
        // produce_fifth_diamonds = (10 + 0) * 1 * 1 = 10
        // fourth_generated += 10 * 1 = 10
        assert_eq!(r.produce_fifth_diamonds.to_number(), 10.0);
        assert_eq!(r.fourth_generated_diamonds.to_number(), 10.0);
    }

    #[test]
    fn ascend_building_cascade_uses_raw_dt_not_scaled() {
        let mut input = baseline_input();
        input.dt = 1.0; // 1 second
        input.ascend_building_5_generated = Decimal::from_finite(20.0);
        // ascend_prod[4] = (20 + 0) * 0.05 * 1 = 1
        // ascend_generated[3] += 1 * dt = 1 * 1 = 1 (raw dt, NOT dt/0.025)
        let r = resource_gain(&input);
        assert_eq!(r.ascend_building_production.fifth.to_number(), 1.0);
        assert_eq!(r.ascend_building_4_generated.to_number(), 1.0);
    }
}
