//! Per-tick resource generation.
//!
//! Verbatim port of
//! `legacy/core_split/packages/logic/src/mechanics/resourceGain.ts`. Lifted
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
//! Takes a `&GameState` (for direct `player.*` reads) plus a
//! [`ResourceGainPre`] bundle (for cross-mechanic pre-computed values that
//! aren't in any slice) and `dt`. Returns the post-tick player updates +
//! `G.*` cache values + events — the caller writes the player updates back
//! into `GameState` (matching the `(state, input) -> (state, events)`
//! convention used elsewhere in the crate).

use smallvec::SmallVec;
use synergismforkd_bignum::Decimal;

use crate::events::{AchievementGroup, CoreEvent};
use crate::state::GameState;

/// Cross-mechanic pre-computed values the aggregator needs that don't
/// live in any state slice. Built by the caller from earlier per-tick
/// outputs (`update_all_tick`, `calculate_tax`, `reset_currency`,
/// `compute_global_multipliers`).
#[derive(Debug, Clone, Copy)]
pub struct ResourceGainPre {
    /// `G.produceTotal`.
    pub produce_total: Decimal,
    /// `G.taxdivisor`.
    pub taxdivisor: Decimal,
    /// `G.taxdivisorcheck`.
    pub taxdivisorcheck: Decimal,
    /// `G.maxexponent`.
    pub maxexponent: f64,
    /// `G.prestigePointGain` (from `resetCurrency`).
    pub prestige_point_gain: Decimal,
    /// `G.transcendPointGain` (from `resetCurrency`).
    pub transcend_point_gain: Decimal,
    /// `G.reincarnationPointGain` (from `resetCurrency`).
    pub reincarnation_point_gain: Decimal,
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
    /// `G.globalCrystalMultiplier` (from `compute_global_multipliers`).
    pub global_crystal_multiplier: Decimal,
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
    /// `G.globalConstantMult`.
    pub global_constant_mult: Decimal,
    /// `G.challengeBaseRequirements[0..=4]` — log10 coin thresholds for c1-c5.
    pub challenge_base_requirements: [f64; 5],
}

impl Default for ResourceGainPre {
    /// Identity values — every multiplier is `1`, every producer rate is
    /// `0`, and the tax divisors collapse the coin gain to zero.
    fn default() -> Self {
        Self {
            produce_total: Decimal::zero(),
            taxdivisor: Decimal::one(),
            taxdivisorcheck: Decimal::one(),
            maxexponent: 1e308,
            prestige_point_gain: Decimal::zero(),
            transcend_point_gain: Decimal::zero(),
            reincarnation_point_gain: Decimal::zero(),
            first_produce_diamonds: 0.0,
            second_produce_diamonds: 0.0,
            third_produce_diamonds: 0.0,
            fourth_produce_diamonds: 0.0,
            fifth_produce_diamonds: 0.0,
            global_crystal_multiplier: Decimal::one(),
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
            first_produce_particles: 0.0,
            second_produce_particles: 0.0,
            third_produce_particles: 0.0,
            fourth_produce_particles: 0.0,
            fifth_produce_particles: 0.0,
            global_constant_mult: Decimal::one(),
            challenge_base_requirements: [10.0, 100.0, 1000.0, 10000.0, 100000.0],
        }
    }
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

/// Result of [`resource_gain`]. Every state field is the post-tick value —
/// caller writes them back into the corresponding [`GameState`] slice.
#[derive(Debug, Clone)]
pub struct ResourceGainResult {
    // ─── Coin counters ───────────────────────────────────────────────────
    /// New `player.coins`.
    pub coins: Decimal,
    /// New `player.coinsThisPrestige`.
    pub coins_this_prestige: Decimal,
    /// New `player.coinsThisTranscension`.
    pub coins_this_transcension: Decimal,
    /// New `player.coinsThisReincarnation`.
    pub coins_this_reincarnation: Decimal,
    /// New `player.coinsTotal`.
    pub coins_total: Decimal,
    // ─── Tier points ─────────────────────────────────────────────────────
    /// New `player.prestigePoints`.
    pub prestige_points: Decimal,
    /// New `player.transcendPoints`.
    pub transcend_points: Decimal,
    /// New `player.reincarnationPoints`.
    pub reincarnation_points: Decimal,
    // ─── Shards ──────────────────────────────────────────────────────────
    /// New `player.prestigeShards`.
    pub prestige_shards: Decimal,
    /// New `player.transcendShards`.
    pub transcend_shards: Decimal,
    /// New `player.reincarnationShards`.
    pub reincarnation_shards: Decimal,
    /// New `player.ascendShards`.
    pub ascend_shards: Decimal,
    // ─── Generated counters (first..fourth tier; fifth never updates) ────
    /// New `player.firstGeneratedDiamonds`.
    pub first_generated_diamonds: Decimal,
    /// New `player.secondGeneratedDiamonds`.
    pub second_generated_diamonds: Decimal,
    /// New `player.thirdGeneratedDiamonds`.
    pub third_generated_diamonds: Decimal,
    /// New `player.fourthGeneratedDiamonds`.
    pub fourth_generated_diamonds: Decimal,
    /// New `player.firstGeneratedMythos`.
    pub first_generated_mythos: Decimal,
    /// New `player.secondGeneratedMythos`.
    pub second_generated_mythos: Decimal,
    /// New `player.thirdGeneratedMythos`.
    pub third_generated_mythos: Decimal,
    /// New `player.fourthGeneratedMythos`.
    pub fourth_generated_mythos: Decimal,
    /// New `player.firstGeneratedParticles`.
    pub first_generated_particles: Decimal,
    /// New `player.secondGeneratedParticles`.
    pub second_generated_particles: Decimal,
    /// New `player.thirdGeneratedParticles`.
    pub third_generated_particles: Decimal,
    /// New `player.fourthGeneratedParticles`.
    pub fourth_generated_particles: Decimal,
    /// New `player.ascendBuilding1.generated`.
    pub ascend_building_1_generated: Decimal,
    /// New `player.ascendBuilding2.generated`.
    pub ascend_building_2_generated: Decimal,
    /// New `player.ascendBuilding3.generated`.
    pub ascend_building_3_generated: Decimal,
    /// New `player.ascendBuilding4.generated`.
    pub ascend_building_4_generated: Decimal,
    // ─── Challenge completions (auto-completion can increment these) ─────
    /// New `player.challengecompletions[1]`.
    pub c1_completions: f64,
    /// New `player.challengecompletions[2]`.
    pub c2_completions: f64,
    /// New `player.challengecompletions[3]`.
    pub c3_completions: f64,
    /// New `player.challengecompletions[4]`.
    pub c4_completions: f64,
    /// New `player.challengecompletions[5]`.
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
    /// Events for the UI tier to dispatch. `[CoreEvent; 8]` covers the
    /// typical worst case (1 achievement + up to 5 challenge auto-completions
    /// + headroom) without heap allocation.
    pub events: SmallVec<[CoreEvent; 8]>,
}

/// Per-tick resource generation. Pure given `state` + `pre` + `dt`; returns
/// the post-tick player + G slice plus an event list. The caller writes the
/// player-state fields in [`ResourceGainResult`] back into the corresponding
/// [`GameState`] slices.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn resource_gain(state: &GameState, pre: &ResourceGainPre, dt: f64) -> ResourceGainResult {
    let dt_scaled = dt / 0.025;
    let dt_scaled_dec = Decimal::from_finite(dt_scaled);
    let dt_dec = Decimal::from_finite(dt);
    let ten = Decimal::from_finite(10.0);
    let one = Decimal::one();
    let mut events: SmallVec<[CoreEvent; 8]> = SmallVec::new();

    // ─── Direct player-state reads ───────────────────────────────────────
    let upgrade = |i: usize| f64::from(state.upgrades.upgrades[i]);
    let research = |i: usize| state.researches.researches[i];
    let cube_upgrade = |i: usize| state.cube_upgrade_levels.cube_upgrades[i];

    // ─── Coin gain ───────────────────────────────────────────────────────
    let mut coins = state.upgrades.coins;
    let mut coins_this_prestige = state.coin_counters.coins_this_prestige;
    let mut coins_this_transcension = state.coin_counters.coins_this_transcension;
    let mut coins_this_reincarnation = state.coin_counters.coins_this_reincarnation;
    let mut coins_total = state.coin_counters.coins_total;
    if pre.produce_total >= Decimal::from_finite(0.001) {
        let cap_exponent = pre.maxexponent - pre.taxdivisorcheck.log(ten).to_number();
        let addcoin = (pre.produce_total / pre.taxdivisor)
            .min(ten.pow(Decimal::from_finite(cap_exponent)))
            * dt_scaled_dec;
        coins += addcoin;
        coins_this_prestige += addcoin;
        coins_this_transcension += addcoin;
        coins_this_reincarnation += addcoin;
        coins_total += addcoin;
    }

    // ─── Point gains ─────────────────────────────────────────────────────
    let mut prestige_points = state.upgrades.prestige_points;
    let mut transcend_points = state.upgrades.transcend_points;
    let mut reincarnation_points = state.upgrades.reincarnation_points;
    if (upgrade(93) - 1.0).abs() < f64::EPSILON && coins_this_prestige >= Decimal::from_finite(1e16)
    {
        prestige_points +=
            (pre.prestige_point_gain / Decimal::from_finite(4000.0) * dt_scaled_dec).floor();
    }
    if (upgrade(100) - 1.0).abs() < f64::EPSILON
        && coins_this_transcension >= Decimal::from_finite(1e100)
    {
        transcend_points +=
            (pre.transcend_point_gain / Decimal::from_finite(4000.0) * dt_scaled_dec).floor();
    }
    if cube_upgrade(28) > 0.0
        && state.reset_counters.transcend_shards >= Decimal::from_mantissa_exponent(1.0, 300.0)
    {
        reincarnation_points +=
            (pre.reincarnation_point_gain / Decimal::from_finite(4000.0) * dt_scaled_dec).floor();
    }

    // ─── Diamond cascade ─────────────────────────────────────────────────
    let diamonds = &state.diamond_producers;
    let gcm = pre.global_crystal_multiplier;
    let produce_first_diamonds = (diamonds.tiers[0].generated
        + Decimal::from_finite(diamonds.tiers[0].owned))
        * Decimal::from_finite(pre.first_produce_diamonds)
        * gcm;
    let produce_second_diamonds = (diamonds.tiers[1].generated
        + Decimal::from_finite(diamonds.tiers[1].owned))
        * Decimal::from_finite(pre.second_produce_diamonds)
        * gcm;
    let produce_third_diamonds = (diamonds.tiers[2].generated
        + Decimal::from_finite(diamonds.tiers[2].owned))
        * Decimal::from_finite(pre.third_produce_diamonds)
        * gcm;
    let produce_fourth_diamonds = (diamonds.tiers[3].generated
        + Decimal::from_finite(diamonds.tiers[3].owned))
        * Decimal::from_finite(pre.fourth_produce_diamonds)
        * gcm;
    let produce_fifth_diamonds = (diamonds.tiers[4].generated
        + Decimal::from_finite(diamonds.tiers[4].owned))
        * Decimal::from_finite(pre.fifth_produce_diamonds)
        * gcm;

    let fourth_generated_diamonds =
        diamonds.tiers[3].generated + produce_fifth_diamonds * dt_scaled_dec;
    let third_generated_diamonds =
        diamonds.tiers[2].generated + produce_fourth_diamonds * dt_scaled_dec;
    let second_generated_diamonds =
        diamonds.tiers[1].generated + produce_third_diamonds * dt_scaled_dec;
    let first_generated_diamonds =
        diamonds.tiers[0].generated + produce_second_diamonds * dt_scaled_dec;
    let produce_diamonds = produce_first_diamonds;

    let mut prestige_shards = state.reset_counters.prestige_shards;
    if state.challenges.current_transcension_challenge != 3
        && state.challenges.current_reincarnation_challenge != 10
    {
        prestige_shards += produce_diamonds * dt_scaled_dec;
    }

    // ─── Mythos cascade ──────────────────────────────────────────────────
    let mythos = &state.mythos_producers;
    let gmm = pre.global_mythos_multiplier;
    let produce_fifth_mythos = (mythos.tiers[4].generated
        + Decimal::from_finite(mythos.tiers[4].owned))
        * Decimal::from_finite(pre.fifth_produce_mythos)
        * gmm
        * pre.grandmaster_multiplier
        * pre.mythosupgrade_15;
    let produce_fourth_mythos = (mythos.tiers[3].generated
        + Decimal::from_finite(mythos.tiers[3].owned))
        * Decimal::from_finite(pre.fourth_produce_mythos)
        * gmm;
    let produce_third_mythos = (mythos.tiers[2].generated
        + Decimal::from_finite(mythos.tiers[2].owned))
        * Decimal::from_finite(pre.third_produce_mythos)
        * gmm
        * pre.mythosupgrade_14;
    let produce_second_mythos = (mythos.tiers[1].generated
        + Decimal::from_finite(mythos.tiers[1].owned))
        * Decimal::from_finite(pre.second_produce_mythos)
        * gmm;
    let produce_first_mythos = (mythos.tiers[0].generated
        + Decimal::from_finite(mythos.tiers[0].owned))
        * Decimal::from_finite(pre.first_produce_mythos)
        * gmm
        * pre.mythosupgrade_13;

    let fourth_generated_mythos = mythos.tiers[3].generated + produce_fifth_mythos * dt_scaled_dec;
    let third_generated_mythos = mythos.tiers[2].generated + produce_fourth_mythos * dt_scaled_dec;
    let second_generated_mythos = mythos.tiers[1].generated + produce_third_mythos * dt_scaled_dec;
    let first_generated_mythos = mythos.tiers[0].generated + produce_second_mythos * dt_scaled_dec;

    // produceMythos: recomputed after mutations using post-tick first_generated_mythos.
    let produce_mythos = (first_generated_mythos + Decimal::from_finite(mythos.tiers[0].owned))
        * Decimal::from_finite(pre.first_produce_mythos)
        * gmm
        * pre.mythosupgrade_13;
    let produce_per_second_mythos = produce_mythos * Decimal::from_finite(40.0);

    // ─── Particle cascade ────────────────────────────────────────────────
    let particles = &state.particle_buildings;
    let mut pm = Decimal::one();
    if upgrade(67) > 0.5 {
        let total_owned = particles.tiers[0].owned
            + particles.tiers[1].owned
            + particles.tiers[2].owned
            + particles.tiers[3].owned
            + particles.tiers[4].owned;
        pm *= Decimal::from_finite(1.03).pow(Decimal::from_finite(total_owned));
    }

    let produce_fifth_particles = (particles.tiers[4].generated
        + Decimal::from_finite(particles.tiers[4].owned))
        * Decimal::from_finite(pre.fifth_produce_particles);
    let produce_fourth_particles = (particles.tiers[3].generated
        + Decimal::from_finite(particles.tiers[3].owned))
        * Decimal::from_finite(pre.fourth_produce_particles);
    let produce_third_particles = (particles.tiers[2].generated
        + Decimal::from_finite(particles.tiers[2].owned))
        * Decimal::from_finite(pre.third_produce_particles);
    let produce_second_particles = (particles.tiers[1].generated
        + Decimal::from_finite(particles.tiers[1].owned))
        * Decimal::from_finite(pre.second_produce_particles);
    let produce_first_particles = (particles.tiers[0].generated
        + Decimal::from_finite(particles.tiers[0].owned))
        * Decimal::from_finite(pre.first_produce_particles)
        * pm;

    let fourth_generated_particles =
        particles.tiers[3].generated + produce_fifth_particles * dt_scaled_dec;
    let third_generated_particles =
        particles.tiers[2].generated + produce_fourth_particles * dt_scaled_dec;
    let second_generated_particles =
        particles.tiers[1].generated + produce_third_particles * dt_scaled_dec;
    let first_generated_particles =
        particles.tiers[0].generated + produce_second_particles * dt_scaled_dec;

    // produceParticles: recomputed after mutations using post-tick first_generated_particles.
    let produce_particles = (first_generated_particles
        + Decimal::from_finite(particles.tiers[0].owned))
        * Decimal::from_finite(pre.first_produce_particles)
        * pm;
    let produce_per_second_particles = produce_particles * Decimal::from_finite(40.0);

    // ─── Transcend / reincarnation shards ────────────────────────────────
    let mut transcend_shards = state.reset_counters.transcend_shards;
    if state.challenges.current_transcension_challenge != 3
        && state.challenges.current_reincarnation_challenge != 10
    {
        transcend_shards += produce_mythos * dt_scaled_dec;
    }
    let reincarnation_shards =
        state.reset_counters.reincarnation_shards + produce_particles * dt_scaled_dec;

    // ─── AscendBuildings cascade (raw `dt`, not `dt / 0.025`) ────────────
    let ascend = &state.tesseract_buildings;
    let ascend_owned = [
        ascend.ascend_building_1.owned,
        ascend.ascend_building_2.owned,
        ascend.ascend_building_3.owned,
        ascend.ascend_building_4.owned,
        ascend.ascend_building_5.owned,
    ];
    let mut ascend_generated = [
        ascend.ascend_building_1.generated,
        ascend.ascend_building_2.generated,
        ascend.ascend_building_3.generated,
        ascend.ascend_building_4.generated,
        ascend.ascend_building_5.generated,
    ];
    let mut ascend_prod = [Decimal::zero(); 5];
    for j in (0..5).rev() {
        ascend_prod[j] = (ascend_generated[j] + Decimal::from_finite(ascend_owned[j]))
            * Decimal::from_finite(0.05)
            * pre.global_constant_mult;
        if j != 0 {
            ascend_generated[j - 1] += ascend_prod[j] * dt_dec;
        }
    }
    let ascend_shards = state.campaigns.ascend_shards + ascend_prod[0] * dt_dec;

    // ─── awardAchievementGroup('constant') gate ──────────────────────────
    if state.reset_counters.ascension_count > 0.0 {
        events.push(CoreEvent::AchievementGroupAwarded {
            group: AchievementGroup::Constant,
        });
    }

    // ─── Challenge 1-5 auto-completion ───────────────────────────────────
    let mut c1_completions = state.challenges.challenge_completions[1];
    let mut c2_completions = state.challenges.challenge_completions[2];
    let mut c3_completions = state.challenges.challenge_completions[3];
    let mut c4_completions = state.challenges.challenge_completions[4];
    let mut c5_completions = state.challenges.challenge_completions[5];

    let auto_cap = |highest: f64, research_cap: f64| -> f64 {
        highest.min(25.0 + 5.0 * research_cap + 925.0 * research(105))
    };
    let threshold = |coef: f64, base: f64, completions: f64| -> Decimal {
        ten.pow(Decimal::from_finite(
            coef * base * (1.0 + completions).powf(2.0),
        ))
    };

    if research(71) > 0.5
        && c1_completions
            < auto_cap(
                state.challenges.highest_challenge_completions[1],
                research(66),
            )
        && coins >= threshold(1.25, pre.challenge_base_requirements[0], c1_completions)
    {
        c1_completions += 1.0;
        events.push(CoreEvent::ChallengeAutoCompleted {
            challenge_index: 1,
            new_completions: c1_completions,
        });
    }
    if research(72) > 0.5
        && c2_completions
            < auto_cap(
                state.challenges.highest_challenge_completions[2],
                research(67),
            )
        && coins >= threshold(1.6, pre.challenge_base_requirements[1], c2_completions)
    {
        c2_completions += 1.0;
        events.push(CoreEvent::ChallengeAutoCompleted {
            challenge_index: 2,
            new_completions: c2_completions,
        });
    }
    if research(73) > 0.5
        && c3_completions
            < auto_cap(
                state.challenges.highest_challenge_completions[3],
                research(68),
            )
        && coins >= threshold(1.7, pre.challenge_base_requirements[2], c3_completions)
    {
        c3_completions += 1.0;
        events.push(CoreEvent::ChallengeAutoCompleted {
            challenge_index: 3,
            new_completions: c3_completions,
        });
    }
    if research(74) > 0.5
        && c4_completions
            < auto_cap(
                state.challenges.highest_challenge_completions[4],
                research(69),
            )
        && coins >= threshold(1.45, pre.challenge_base_requirements[3], c4_completions)
    {
        c4_completions += 1.0;
        events.push(CoreEvent::ChallengeAutoCompleted {
            challenge_index: 4,
            new_completions: c4_completions,
        });
    }
    if research(75) > 0.5
        && c5_completions
            < auto_cap(
                state.challenges.highest_challenge_completions[5],
                research(70),
            )
        && coins >= threshold(2.0, pre.challenge_base_requirements[4], c5_completions)
    {
        c5_completions += 1.0;
        events.push(CoreEvent::ChallengeAutoCompleted {
            challenge_index: 5,
            new_completions: c5_completions,
        });
    }

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

    #[test]
    fn default_state_is_a_noop_for_resources() {
        let r = resource_gain(&GameState::default(), &ResourceGainPre::default(), 0.025);
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
        let state = GameState::default();
        let pre = ResourceGainPre {
            produce_total: Decimal::from_finite(100.0),
            ..ResourceGainPre::default()
        };
        // addcoin = min(100/1, 10^(1e308 - 0)) * (0.025/0.025) = 100.
        let r = resource_gain(&state, &pre, 0.025);
        assert_eq!(r.coins.to_number(), 100.0);
        assert_eq!(r.coins_this_prestige.to_number(), 100.0);
        assert_eq!(r.coins_this_transcension.to_number(), 100.0);
        assert_eq!(r.coins_this_reincarnation.to_number(), 100.0);
        assert_eq!(r.coins_total.to_number(), 100.0);
    }

    #[test]
    fn produce_total_below_threshold_is_skipped() {
        let state = GameState::default();
        let pre = ResourceGainPre {
            produce_total: Decimal::from_finite(1e-6),
            ..ResourceGainPre::default()
        };
        let r = resource_gain(&state, &pre, 0.025);
        assert_eq!(r.coins.to_number(), 0.0);
    }

    #[test]
    fn upgrade_93_drips_prestige_points_once_coins_cross_1e16() {
        let mut state = GameState::default();
        state.upgrades.upgrades[93] = 1;
        state.coin_counters.coins_this_prestige = Decimal::from_finite(1e16);
        let pre = ResourceGainPre {
            prestige_point_gain: Decimal::from_finite(8000.0),
            ..ResourceGainPre::default()
        };
        // floor(8000 / 4000 * 1) = floor(2) = 2.
        let r = resource_gain(&state, &pre, 0.025);
        assert_eq!(r.prestige_points.to_number(), 2.0);
    }

    #[test]
    fn upgrade_93_no_drip_below_threshold() {
        let mut state = GameState::default();
        state.upgrades.upgrades[93] = 1;
        state.coin_counters.coins_this_prestige = Decimal::from_finite(1e15);
        let pre = ResourceGainPre {
            prestige_point_gain: Decimal::from_finite(8000.0),
            ..ResourceGainPre::default()
        };
        let r = resource_gain(&state, &pre, 0.025);
        assert_eq!(r.prestige_points.to_number(), 0.0);
    }

    #[test]
    fn transcension_challenge_3_disables_shard_gains() {
        let mut state = GameState::default();
        state.challenges.current_transcension_challenge = 3;
        state.diamond_producers.tiers[0].generated = Decimal::from_finite(10.0);
        let pre = ResourceGainPre {
            first_produce_diamonds: 1.0,
            ..ResourceGainPre::default()
        };
        let r = resource_gain(&state, &pre, 0.025);
        assert_eq!(r.prestige_shards.to_number(), 0.0);
        assert_eq!(r.transcend_shards.to_number(), 0.0);
    }

    #[test]
    fn reincarnation_challenge_10_disables_shard_gains() {
        let mut state = GameState::default();
        state.challenges.current_reincarnation_challenge = 10;
        state.diamond_producers.tiers[0].generated = Decimal::from_finite(10.0);
        let pre = ResourceGainPre {
            first_produce_diamonds: 1.0,
            ..ResourceGainPre::default()
        };
        let r = resource_gain(&state, &pre, 0.025);
        assert_eq!(r.prestige_shards.to_number(), 0.0);
        assert_eq!(r.transcend_shards.to_number(), 0.0);
    }

    #[test]
    fn ascension_count_above_zero_emits_constant_group_event() {
        let mut state = GameState::default();
        state.reset_counters.ascension_count = 1.0;
        let r = resource_gain(&state, &ResourceGainPre::default(), 0.025);
        assert!(r.events.iter().any(|e| matches!(
            e,
            CoreEvent::AchievementGroupAwarded {
                group: AchievementGroup::Constant
            }
        )));
    }

    #[test]
    fn challenge_1_auto_completes_when_gates_open() {
        let mut state = GameState::default();
        state.researches.researches[71] = 1.0;
        state.challenges.highest_challenge_completions[1] = 5.0;
        state.upgrades.coins = Decimal::from_finite(1e100); // huge — easily clears threshold
        let r = resource_gain(&state, &ResourceGainPre::default(), 0.025);
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
        let mut state = GameState::default();
        state.researches.researches[71] = 1.0;
        state.challenges.highest_challenge_completions[1] = 5.0;
        state.challenges.challenge_completions[1] = 5.0;
        state.upgrades.coins = Decimal::from_finite(1e100);
        let r = resource_gain(&state, &ResourceGainPre::default(), 0.025);
        assert_eq!(r.c1_completions, 5.0);
    }

    #[test]
    fn diamond_cascade_propagates_one_step() {
        let mut state = GameState::default();
        state.diamond_producers.tiers[4].generated = Decimal::from_finite(10.0);
        let pre = ResourceGainPre {
            fifth_produce_diamonds: 1.0,
            ..ResourceGainPre::default()
        };
        let r = resource_gain(&state, &pre, 0.025);
        // produce_fifth_diamonds = (10 + 0) * 1 * 1 = 10.
        // fourth_generated += 10 * 1 = 10.
        assert_eq!(r.produce_fifth_diamonds.to_number(), 10.0);
        assert_eq!(r.fourth_generated_diamonds.to_number(), 10.0);
    }

    #[test]
    fn ascend_building_cascade_uses_raw_dt_not_scaled() {
        let mut state = GameState::default();
        state.tesseract_buildings.ascend_building_5.generated = Decimal::from_finite(20.0);
        // ascend_prod[4] = (20 + 0) * 0.05 * 1 = 1.
        // ascend_generated[3] += 1 * dt = 1 * 1 = 1 (raw dt, NOT dt/0.025).
        let r = resource_gain(&state, &ResourceGainPre::default(), 1.0);
        assert_eq!(r.ascend_building_production.fifth.to_number(), 1.0);
        assert_eq!(r.ascend_building_4_generated.to_number(), 1.0);
    }
}
