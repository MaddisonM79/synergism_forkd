//! Ant-sacrifice orchestration — the effect side of the mechanic.
//!
//! Ports the legacy `sacrificeAnts()`
//! (Features/Ants/AntSacrifice/sacrifice.ts) and the per-tick `activateELO()`
//! (Rewards/ELO/RebornELO/lib/create-reborn.ts), which together restore the
//! offering / obtainium / talisman economy, raise the `immortalELO` high-water
//! mark, and populate the reborn-ELO leaderboards that drive the slice-3a
//! achievement progressive.
//!
//! The pure reward math lives in `mechanics::ant_sacrifice_reward_calc`,
//! `mechanics::ant_sacrifice_rewards`, and `mechanics::ant_reborn_elo`; this
//! module is the `&mut GameState` orchestrator that reduces the live StatLines
//! and applies the results, the way `tick::reset` consumes the reset
//! calculators (and reaches the private `super::compute_*` resource helpers).
//!
//! One piece is deliberately omitted:
//!
//! - **Lotus shortcut.** The `getLotusTimeExpiresAt()` branch that snaps
//!   `rebornELO = immortalELO` reads wall-clock time, forbidden in the logic
//!   crate. Omitted — the gradual activation path is always taken. (The quark
//!   award — `activateELO`'s closing `worlds.add(availableQuarksFromELO())` — is
//!   wired below; the `autoWarpCheck` / `dailyPowderResetUses` warp factor is an
//!   unported auto-feature and neutral-defaults to 1.)

use smallvec::{smallvec, SmallVec};
use synergismforkd_bignum::Decimal;

use crate::events::CoreEvent;
use crate::state::ants::{PlayerAntProducer, RebornELOEntry};
use crate::state::GameState;

// ─── Reward StatLine reducers (live, `&GameState`) ────────────────────────────

/// `calculateAntSacrificeMultiplier` (Calculate.ts:382) — the product of the
/// `antSacrificeRewardStats` StatLine times `calculateAntSacrificeCubeBlessing`.
///
/// Neutral-defaulted lines (faithful — identity at the current state):
/// `RuneBlessing` (prism rune-blessing `antSacrificeMult` — the rune-blessing
/// layer is unported; identity `1` at blessing level 0) and `Event` (UI-tier
/// event calendar → `1`). The trailing `calculateAntSacrificeCubeBlessing` reads
/// the live blessing cascade (`ant_sacrifice_cube_blessing`) now that `open()`
/// (P3.2) makes the cube-blessing levels accrue. (`compute_obtainium_gain`'s
/// ant-sacrifice obtainium source still neutral-defaults — see its docs.)
pub(super) fn compute_ant_sacrifice_multiplier(state: &GameState) -> Decimal {
    use crate::mechanics::achievement_rewards::sacrifice_mult;
    use crate::mechanics::ant_upgrades::ant_sacrifice_ant_upgrade_effect;
    use crate::mechanics::calculate::product_f64;
    use crate::mechanics::challenges::{calc_ecc, ChallengeType};

    // Ant upgrade slot: AntSacrifice (index 10).
    const ANT_UPGRADE_ANT_SACRIFICE: usize = 10;
    // `player.upgrades[40]` — accelerator-boost upgrade.
    const ACCELERATOR_BOOST_UPGRADE: usize = 40;

    let ach = &state.achievements.achievements;
    let researches = &state.researches.researches;
    let upgrades = &state.upgrades.upgrades;

    let stats = product_f64(&[
        sacrifice_mult(ach),
        ant_sacrifice_ant_upgrade_effect(super::true_ant_level(state, ANT_UPGRADE_ANT_SACRIFICE))
            .ant_sacrifice_multiplier,
        1.0 + researches[103] / 20.0,
        1.0 + researches[104] / 20.0,
        1.0, // RuneBlessing — prism rune-blessing antSacrificeMult (unported → 1)
        1.0 + (1.0 / 50.0)
            * calc_ecc(
                ChallengeType::Reincarnation,
                state.challenges.challenge_completions[10],
            ),
        1.0 + (1.0 / 50.0) * researches[122],
        1.0 + (3.0 / 100.0) * researches[133],
        1.0 + (2.0 / 100.0) * researches[163],
        1.0 + (1.0 / 100.0) * researches[193],
        1.0 + (1.0 / 4.0) * f64::from(upgrades[ACCELERATOR_BOOST_UPGRADE]),
        1.0, // Event — UI-tier event calendar → 1 + 0
    ]);

    // × calculateAntSacrificeCubeBlessing() — the live ant-sacrifice blessing
    // cascade (now that `open()` makes the cube-blessing levels accrue).
    Decimal::from_finite(stats) * ant_sacrifice_cube_blessing(state)
}

/// `rebornELOCreationSpeedMult` (RebornELO/lib/calculate.ts) — the product of
/// `rebornELOCreationSpeedMultStats`. Sets how fast `immortalELO` bleeds into
/// `rebornELO` per second in [`activate_elo`].
///
/// Neutral-defaulted lines (faithful — identity at the current state): only
/// `Exalt6` (singularity layer paused → `1`). The `MortuusTalisman` line reads
/// `mortuus_talisman_effects(talismanRarity[mortuus]).ant_bonus`
/// (`MORTUUS_INSCRIPT_VALUES[rarity]`, identity at rarity 0) and the
/// `CubeBlessing` line (`calculateAntELOCubeBlessing`) reads the live blessing
/// cascade via `ant_elo_cube_blessing` — both wired now that the underlying
/// state can accrue (talisman rarity / `open()` blessings).
fn compute_reborn_elo_creation_speed_mult(state: &GameState) -> f64 {
    use crate::mechanics::ant_reborn_elo::{
        reborn_elo_stage_modifiers, RebornELOStageModifiersInput,
    };
    use crate::mechanics::calculate::product_f64;
    use crate::mechanics::talisman_effects::mortuus_talisman_effects;
    use crate::state::talismans::TALISMAN_MORTUUS;

    // Ant producer slots: Queens .. HolySpirit.
    const QUEENS: usize = 4;
    const LORD_ROYALS: usize = 5;
    const ALMIGHTIES: usize = 6;
    const DISCIPLES: usize = 7;
    const HOLY_SPIRIT: usize = 8;
    // `player.upgrades[124]` — Coin upgrade 2x4.
    const COIN_UPGRADE_24: usize = 124;
    // `player.platonicUpgrades[12]`.
    const PLATONIC_UPGRADE_12: usize = 12;

    let researches = &state.researches.researches;
    let upgrades = &state.upgrades.upgrades;
    let producers = &state.ants.producers;
    let owned = |i: usize| producers[i].purchased > 0.0;

    let stage_mods = reborn_elo_stage_modifiers(&RebornELOStageModifiersInput {
        reborn_elo: state.ants.reborn_elo,
        sing_count: state.singularity.singularity_count,
    });

    product_f64(&[
        0.01, // Base
        super::compute_effective_ant_elo(state),
        1.0 + 0.1 * f64::from(upgrades[COIN_UPGRADE_24]),
        1.0 + researches[110] / 50.0,
        1.0 + researches[120] / 250.0,
        1.0 + researches[148] / 50.0,
        if owned(QUEENS) { 1.15 } else { 1.0 },
        if owned(LORD_ROYALS) { 1.25 } else { 1.0 },
        if owned(ALMIGHTIES) { 1.4 } else { 1.0 },
        if owned(DISCIPLES) { 2.0 } else { 1.0 },
        if owned(HOLY_SPIRIT) { 3.0 } else { 1.0 },
        stage_mods.reborn_speed_mult,
        // MortuusTalisman — getTalismanEffects('mortuus').antBonus =
        // MORTUUS_INSCRIPT_VALUES[talismanRarity[mortuus]] (1 at rarity 0).
        mortuus_talisman_effects(state.talismans.talisman_rarity[TALISMAN_MORTUUS] as i32)
            .ant_bonus,
        ant_elo_cube_blessing(state), // CubeBlessing — calculateAntELOCubeBlessing
        1.0 + state.cube_upgrade_levels.platonic_upgrades[PLATONIC_UPGRADE_12] / 10.0,
        1.0, // Exalt6 — singularity layer paused → 1
    ])
}

// ─── Live cube-blessing cascades (platonic → hypercube → tesseract → cube) ────

/// `calculateAntSacrificeCubeBlessing()` — the ant-sacrifice cube-blessing
/// cascade. Evaluates to `1` until cubes are opened (all blessing levels `0`);
/// P3.2's `open()` makes the levels accrue, so this reads the live state.
fn ant_sacrifice_cube_blessing(state: &GameState) -> Decimal {
    use crate::mechanics::cube_blessings::calculate_ant_sacrifice_cube_blessing;
    use crate::mechanics::hypercube_blessings::calculate_ant_sacrifice_hypercube_blessing;
    use crate::mechanics::platonic_blessings::calculate_hypercube_blessing_multiplier_platonic_blessing;
    use crate::mechanics::tesseract_blessings::calculate_ant_sacrifice_tesseract_blessing;

    // `player.cubeUpgrades[15]` — the ant-sacrifice cube-blessing DR increase.
    const CUBE_UPGRADE_ANT_SACRIFICE_BLESSING: usize = 15;

    let platonic =
        calculate_hypercube_blessing_multiplier_platonic_blessing(&state.platonic_blessings);
    let hypercube =
        calculate_ant_sacrifice_hypercube_blessing(&state.hypercube_blessings, platonic);
    let tesseract =
        calculate_ant_sacrifice_tesseract_blessing(&state.tesseract_blessings, hypercube);
    calculate_ant_sacrifice_cube_blessing(
        &state.cube_blessings,
        tesseract,
        state.cube_upgrade_levels.cube_upgrades[CUBE_UPGRADE_ANT_SACRIFICE_BLESSING],
    )
}

/// `calculateAntELOCubeBlessing()` — the ant-ELO cube-blessing cascade (its
/// hypercube layer takes no platonic amplifier). `1` until cubes are opened.
fn ant_elo_cube_blessing(state: &GameState) -> f64 {
    use crate::mechanics::cube_blessings::calculate_ant_elo_cube_blessing;
    use crate::mechanics::hypercube_blessings::calculate_ant_elo_hypercube_blessing;
    use crate::mechanics::tesseract_blessings::calculate_ant_elo_tesseract_blessing;

    // `player.cubeUpgrades[25]` — the ant-ELO cube-blessing exponent increase.
    const CUBE_UPGRADE_ANT_ELO_BLESSING: usize = 25;

    let hypercube = calculate_ant_elo_hypercube_blessing(&state.hypercube_blessings);
    let tesseract = calculate_ant_elo_tesseract_blessing(&state.tesseract_blessings, hypercube);
    calculate_ant_elo_cube_blessing(
        &state.cube_blessings,
        tesseract,
        state.cube_upgrade_levels.cube_upgrades[CUBE_UPGRADE_ANT_ELO_BLESSING],
    )
}

// ─── The sacrifice ────────────────────────────────────────────────────────────

/// Execute an ant sacrifice — the effect side of `AntSacrificeTriggered`.
///
/// Ports `sacrificeAnts()`: credits the immortal-ELO high-water gain,
/// offerings, obtainium (unless inside ascension challenge 14), and the seven
/// talisman craft items (gated on challenge-9 completions), then resets the
/// ants to crumbs. All rewards are computed from the pre-sacrifice state (as
/// `antSacrificeRewards()` does) before any mutation is applied. Returns the
/// [`CoreEvent::AntSacrificePerformed`] effect event.
///
/// The `sacMult` / `sacCount` achievement-group awards, the
/// `antSacrificeToReincarnation` reincarnation bump, and the `seeingRed`
/// mythical-fragment achievements are deferred to the achievement-awarding
/// workstream (P3.1) — see the inline TODOs.
pub(super) fn perform_ant_sacrifice(
    state: &mut GameState,
    reincarnation_point_gain: Decimal,
) -> SmallVec<[CoreEvent; 2]> {
    use crate::mechanics::achievement_awards;
    use crate::mechanics::ant_reborn_elo::{
        reborn_elo_stage_modifiers, RebornELOStageModifiersInput,
    };
    use crate::mechanics::ant_sacrifice_reward_calc::{
        calculate_ant_sacrifice_obtainium, calculate_ant_sacrifice_offering,
        calculate_immortal_elo_gain, AntSacrificeObtainiumInput, AntSacrificeOfferingInput,
        CalculateImmortalELOGainInput,
    };
    use crate::mechanics::ant_sacrifice_rewards::{
        calculate_ant_sacrifice_talisman_item, AntSacrificeTalismanItemInput, TalismanCraftItem,
    };
    use crate::mechanics::calculate::{calculate_offerings, CalculateOfferingsInput};

    // Ascension challenge 14 suppresses the obtainium credit (the reward is
    // still computed; only the credit to the balance is skipped).
    const ASCENSION_CHALLENGE_14: u32 = 14;

    // ── Compute all rewards from the pre-sacrifice state ─────────────────────
    let effective_elo = super::compute_effective_ant_elo(state);
    let ant_sac_mult = compute_ant_sacrifice_multiplier(state);
    let time_multiplier =
        super::offering_obtainium_time_multiplier(state, state.ants.ant_sacrifice_timer, true);
    let stage_mods = reborn_elo_stage_modifiers(&RebornELOStageModifiersInput {
        reborn_elo: state.ants.reborn_elo,
        sing_count: state.singularity.singularity_count,
    });
    let taxman_enabled = state.singularity.taxman_last_stand.enabled;
    let taxman_completions = state.singularity.taxman_last_stand.completions;

    let immortal_elo_gained = calculate_immortal_elo_gain(&CalculateImmortalELOGainInput {
        effective_elo,
        immortal_elo: state.ants.immortal_elo,
    });

    // offering_mult / obtainium_mult = calculateOfferings(false) /
    // calculateObtainium(false): the standard awards with the time multiplier
    // off (the ant-sacrifice time multiplier is applied separately below).
    let base_offerings = super::compute_base_offerings(state);
    let offering_mult = calculate_offerings(&CalculateOfferingsInput {
        base_offerings,
        time_multiplier: 1.0,
        offering_mult: super::compute_offering_mult(state, base_offerings),
        taxman_last_stand_enabled: taxman_enabled,
        taxman_last_stand_completions: taxman_completions,
        current_offerings: state.automation.offerings,
    });
    let offerings_gained = calculate_ant_sacrifice_offering(&AntSacrificeOfferingInput {
        ant_sac_mult,
        stage_mult: stage_mods.ant_sacrifice_offering_mult,
        time_multiplier,
        offering_mult,
        current_offerings: state.automation.offerings,
        taxman_last_stand_enabled: taxman_enabled,
        taxman_last_stand_completions: taxman_completions,
    });

    let obtainium_mult = super::compute_obtainium(
        state,
        super::compute_base_obtainium(state),
        reincarnation_point_gain,
        1.0,
    );
    let obtainium_reward = calculate_ant_sacrifice_obtainium(&AntSacrificeObtainiumInput {
        ant_sac_mult,
        stage_mult: stage_mods.ant_sacrifice_obtainium_mult,
        time_multiplier,
        obtainium_mult,
        current_obtainium: state.researches.obtainium,
        taxman_last_stand_enabled: taxman_enabled,
        taxman_last_stand_completions: taxman_completions,
    });
    let in_challenge_14 = state.challenges.current_ascension_challenge == ASCENSION_CHALLENGE_14;
    let obtainium_gained = if in_challenge_14 {
        Decimal::zero()
    } else {
        obtainium_reward
    };

    // Talisman craft items — gated on challenge-9 completions. Quantities are
    // `Decimal`; the talisman fragment/shard balances are `f64` in this slice,
    // so the credits round through `to_number()`.
    let talisman_gated = state.challenges.challenge_completions[9] > 0.0;
    let talisman_rewards: [f64; 7] = if talisman_gated {
        let reward_mult = ant_sac_mult * Decimal::from_finite(time_multiplier);
        let stage_mult = stage_mods.ant_sacrifice_talisman_fragment_mult;
        let qty = |item: TalismanCraftItem| {
            calculate_ant_sacrifice_talisman_item(&AntSacrificeTalismanItemInput {
                item,
                elo: effective_elo,
                reward_mult,
                stage_mult,
            })
            .to_number()
        };
        [
            qty(TalismanCraftItem::Shard),
            qty(TalismanCraftItem::CommonFragment),
            qty(TalismanCraftItem::UncommonFragment),
            qty(TalismanCraftItem::RareFragment),
            qty(TalismanCraftItem::EpicFragment),
            qty(TalismanCraftItem::LegendaryFragment),
            qty(TalismanCraftItem::MythicalFragment),
        ]
    } else {
        [0.0; 7]
    };

    // ── Apply mutations ──────────────────────────────────────────────────────
    state.ants.immortal_elo += immortal_elo_gained;
    state.automation.offerings += offerings_gained;
    if !in_challenge_14 {
        state.researches.obtainium += obtainium_reward;
    }
    if talisman_gated {
        let t = &mut state.talismans;
        t.talisman_shards += talisman_rewards[0];
        t.common_fragments += talisman_rewards[1];
        t.uncommon_fragments += talisman_rewards[2];
        t.rare_fragments += talisman_rewards[3];
        t.epic_fragments += talisman_rewards[4];
        t.legendary_fragments += talisman_rewards[5];
        t.mythical_fragments += talisman_rewards[6];
    }
    // awardAchievementGroup('sacMult') — reads the updated immortal ELO and the
    // still-owned producers, before the reset clears them.
    let immortal_elo = state.ants.immortal_elo;
    let producer_owned: [bool; 9] =
        std::array::from_fn(|i| state.ants.producers[i].purchased > 0.0);
    let awarded = achievement_awards::sac_mult_achievement_check(
        &mut state.achievements,
        immortal_elo,
        &producer_owned,
    );
    super::credit_achievement_quarks(state, awarded);

    reset_ants_for_sacrifice(state);

    // Trailing ungrouped seeingRed / seeingRedNoBlue checks (post-reset in
    // `sacrificeAnts`; the reset does not touch the mythical-fragment balance).
    let mythical_fragments = state.talismans.mythical_fragments;
    let awarded = achievement_awards::ant_sacrifice_fragment_achievement_check(
        &mut state.achievements,
        mythical_fragments,
        in_challenge_14,
    );
    super::credit_achievement_quarks(state, awarded);

    smallvec![CoreEvent::AntSacrificePerformed {
        offerings_gained,
        obtainium_gained,
        immortal_elo_gained,
    }]
}

/// `resetAnts(AntSacrificeTiers.sacrifice)` — the post-sacrifice reset (tier 0).
///
/// Resets crumbs, producers, masteries, the sacrifice-tier ant upgrades, and
/// `rebornELO` to their defaults; bumps the sacrifice id and count. `immortalELO`
/// is **not** reset at the sacrifice tier (it only clears at ascension), and the
/// reborn-ELO leaderboards persist (they clear at singularity / save-reset).
///
/// The `highestSingularityCount >= 10/15/20` crumb / producer / upgrade regrants
/// and the `sacCount` group award + `antSacrificeToReincarnation` bump are
/// omitted — the singularity layer is paused and the group awards belong to the
/// achievement-awarding workstream. All are inert at every non-singularity state.
fn reset_ants_for_sacrifice(state: &mut GameState) {
    // Ant upgrades whose `minimumResetTier` is `sacrifice` (0) — reset on every
    // sacrifice. Salvage(7) / Mortuus(11) / Mortuus2(13) / AscensionScore(14)
    // persist (higher reset tiers) and are intentionally absent.
    const SACRIFICE_TIER_UPGRADES: &[usize] = &[0, 1, 2, 3, 4, 5, 6, 8, 9, 10, 12];
    // `player.researches[116]` — 5x16, +1 sacrifice count per level.
    const RESEARCH_SAC_COUNT: usize = 116;

    // Crumbs → defaults (`defaultCrumbs` / `defaultCrumbsThisSacrifice` = 1).
    state.ants.crumbs = Decimal::one();
    state.ants.crumbs_this_sacrifice = Decimal::one();

    // Producers → empty; masteries → mastery 0 (highest_mastery preserved).
    for producer in &mut state.ants.producers {
        *producer = PlayerAntProducer::default();
    }
    for mastery in &mut state.ants.masteries {
        mastery.mastery = 0;
    }

    // Sacrifice-tier ant upgrades → 0.
    for &i in SACRIFICE_TIER_UPGRADES {
        state.ants.upgrades[i] = 0.0;
    }

    // rebornELO → 0; `activate_elo` re-accrues it from immortalELO each tick.
    state.ants.reborn_elo = 0.0;

    // Sacrifice bookkeeping. `currentSacrificeId` always increments (permanent
    // leaderboard tagging). `antSacrificeCount += 1 + researches[116]` — the
    // `antSacrificeCountMultiplier` achievement reward is unported (→ ×1).
    state.ants.current_sacrifice_id += 1;
    state.ants.ant_sacrifice_count += 1.0 + state.researches.researches[RESEARCH_SAC_COUNT];

    // awardAchievementGroup('sacCount') — on the updated sacrifice count.
    let sacrifice_count = state.ants.ant_sacrifice_count;
    let awarded = crate::mechanics::achievement_awards::sac_count_achievement_check(
        &mut state.achievements,
        sacrifice_count,
    );
    super::credit_achievement_quarks(state, awarded);

    // Timers reset.
    state.ants.ant_sacrifice_timer = 0.0;
    state.ants.ant_sacrifice_timer_real = 0.0;
}

// ─── Reborn-ELO activation + leaderboard ──────────────────────────────────────

/// `activateELO(dt)` (RebornELO/lib/create-reborn.ts) — bleed available
/// `immortalELO` into `rebornELO` at the creation-speed rate, then refresh the
/// reborn-ELO leaderboards.
///
/// Called each live tick from `phase_automation` (gated on `immortal_elo > 0`).
/// The Lotus wall-clock shortcut and the trailing quark award are omitted (see
/// the module header).
pub(super) fn activate_elo(state: &mut GameState, dt: f64) -> SmallVec<[CoreEvent; 1]> {
    use crate::mechanics::ant_reborn_elo::{
        calculate_available_reborn_elo, calculate_reborn_elo_thresholds,
        calculate_to_next_reborn_elo_threshold, AvailableRebornELOInput,
    };

    let to_activate = calculate_available_reborn_elo(&AvailableRebornELOInput {
        immortal_elo: state.ants.immortal_elo,
        reborn_elo: state.ants.reborn_elo,
    });
    if to_activate > 0.0 {
        let limit = state.ants.immortal_elo - state.ants.reborn_elo;
        let gain = limit.min(dt * compute_reborn_elo_creation_speed_mult(state));
        let mut stages = calculate_reborn_elo_thresholds(state.ants.reborn_elo);
        let mut elo_to_next =
            calculate_to_next_reborn_elo_threshold(state.ants.reborn_elo, Some(stages));
        let mut budget = gain;
        while budget >= elo_to_next {
            state.ants.reborn_elo += elo_to_next;
            budget -= elo_to_next;
            budget /= 1.02; // each threshold makes further ELO harder to gain
            stages += 1.0;
            elo_to_next =
                calculate_to_next_reborn_elo_threshold(state.ants.reborn_elo, Some(stages));
        }
        state.ants.reborn_elo += budget;
        state.ants.reborn_elo = state.ants.reborn_elo.min(state.ants.immortal_elo);
    }
    update_ant_leaderboards(state);

    // Quark award — `activateELO`'s closing `worlds.add(availableQuarksFromELO(),
    // false, true)` (useBonus false: the quark multiplier is already applied
    // inside `available_quarks_from_elo`; `addToQuarksThisSingularity` true).
    let quarks = available_quarks_from_elo(state);
    let mut events: SmallVec<[CoreEvent; 1]> = SmallVec::new();
    if quarks > 0.0 {
        state.quarks.worlds += Decimal::from_finite(quarks);
        state.golden_quarks.quarks_this_singularity += quarks;
        state.ants.quarks_gained_from_ants += quarks;
        events.push(CoreEvent::QuarksAwarded { quarks });
    }
    events
}

/// `availableQuarksFromELO` (QuarkCorner/lib/calculate-quarks.ts) — the
/// incremental quark gift from the reborn-ELO leaderboards. The daily
/// leaderboard's stages set the base quark count + per-stage multiplier; the
/// all-time leaderboard scales it (`quarks_from_elo_mult`); `applyBonus` (the
/// cached quark multiplier) multiplies; and the running
/// `quarks_gained_from_ants` total is subtracted so each tick only awards the
/// delta. The `autoWarpCheck ? 1 + dailyPowderResetUses : 1` factor
/// neutral-defaults to 1 (auto-warp is an unported auto-feature).
fn available_quarks_from_elo(state: &GameState) -> f64 {
    use crate::mechanics::ant_reborn_elo::{
        base_quarks_from_reborn_elo_stages, calculate_leaderboard_value,
        calculate_reborn_elo_thresholds, quarks_from_elo_mult,
    };

    let daily: Vec<f64> = state
        .ants
        .highest_reborn_elo_daily
        .iter()
        .map(|e| e.elo)
        .collect();
    let num_stages = calculate_reborn_elo_thresholds(calculate_leaderboard_value(&daily));
    let bq = base_quarks_from_reborn_elo_stages(num_stages);

    let ever: Vec<f64> = state
        .ants
        .highest_reborn_elo_ever
        .iter()
        .map(|e| e.elo)
        .collect();
    let ant_quark_mult = quarks_from_elo_mult(calculate_leaderboard_value(&ever)) * bq.stage_mult;

    // applyBonus(baseQuarks): the cached quark multiplier (a percent → ×).
    let applied = bq.base_quarks * (1.0 + state.quarks.quark_bonus / 100.0);
    (applied * ant_quark_mult - state.ants.quarks_gained_from_ants).max(0.0)
}

/// `updateAntLeaderboards` (QuarkCorner/lib/leaderboard-update.ts) — record the
/// current `rebornELO` (tagged by `currentSacrificeId`) into the daily and
/// all-time top-5 reborn-ELO leaderboards.
fn update_ant_leaderboards(state: &mut GameState) {
    let elo = state.ants.reborn_elo;
    let sacrifice_id = state.ants.current_sacrifice_id;
    update_single_leaderboard(&mut state.ants.highest_reborn_elo_daily, elo, sacrifice_id);
    update_single_leaderboard(&mut state.ants.highest_reborn_elo_ever, elo, sacrifice_id);
}

/// Insert/refresh one leaderboard: keep the top `LEADERBOARD_WEIGHTS.len()` (5)
/// entries sorted by ELO descending, deduping by `sacrifice_id` so a climbing
/// sacrifice updates its own entry in place. Verbatim port of
/// `updateSingleLeaderboard`.
fn update_single_leaderboard(
    leaderboard: &mut SmallVec<[RebornELOEntry; 5]>,
    elo: f64,
    sacrifice_id: u32,
) {
    use crate::mechanics::ant_reborn_elo::LEADERBOARD_WEIGHTS;
    let cap = LEADERBOARD_WEIGHTS.len();

    // Full and below the floor → nothing to do.
    if leaderboard.len() == cap {
        if let Some(last) = leaderboard.last() {
            if elo < last.elo {
                return;
            }
        }
    }

    if let Some(idx) = leaderboard
        .iter()
        .position(|e| e.sacrifice_id == sacrifice_id)
    {
        leaderboard[idx].elo = elo;
        if idx > 0 && leaderboard[idx].elo > leaderboard[idx - 1].elo {
            leaderboard.sort_by(|a, b| b.elo.total_cmp(&a.elo));
        }
    } else {
        leaderboard.push(RebornELOEntry { elo, sacrifice_id });
        leaderboard.sort_by(|a, b| b.elo.total_cmp(&a.elo));
    }

    if leaderboard.len() > cap {
        leaderboard.truncate(cap);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mechanics::ant_reborn_elo::calculate_leaderboard_value;

    #[test]
    fn sacrifice_credits_immortal_elo_offerings_and_resets_ants() {
        let mut state = GameState::default();
        // A non-zero sacrifice timer gives a non-zero time multiplier (the
        // threshold penalty is `(timer / 10)^2`, zero at timer 0).
        state.ants.ant_sacrifice_timer = 10.0;
        state.ants.crumbs = Decimal::from_finite(1e50);
        state.ants.crumbs_this_sacrifice = Decimal::from_finite(1e50);

        let events = perform_ant_sacrifice(&mut state, Decimal::zero());

        // Effective ELO at the default state is 1 (the `ants` level reward),
        // immortal ELO was 0 → gain 1.
        assert_eq!(state.ants.immortal_elo, 1.0);
        assert!(state.automation.offerings.to_number() > 0.0);
        // Ants reset to crumbs; bookkeeping advanced.
        assert_eq!(state.ants.crumbs.to_number(), 1.0);
        assert_eq!(state.ants.crumbs_this_sacrifice.to_number(), 1.0);
        assert_eq!(state.ants.reborn_elo, 0.0);
        assert_eq!(state.ants.ant_sacrifice_count, 1.0);
        // sacCount >= 1 achievement (#481, 3 points) fires on the first sacrifice.
        assert_eq!(state.achievements.achievement_points, 3.0);
        assert_ne!(state.achievements.achievements[481], 0);
        assert_eq!(state.ants.current_sacrifice_id, 1);
        assert_eq!(state.ants.ant_sacrifice_timer, 0.0);
        assert!(matches!(
            events.as_slice(),
            [CoreEvent::AntSacrificePerformed { .. }]
        ));
    }

    #[test]
    fn sacrifice_awards_sac_mult_when_elo_and_producer_met() {
        let mut state = GameState::default();
        state.ants.ant_sacrifice_timer = 10.0;
        // Pre-existing immortal ELO + an owned Breeders producer unlocks the
        // first sacMult tier (#176: immortalELO >= 50 && Breeders owned).
        state.ants.immortal_elo = 1_000.0;
        state.ants.producers[1].purchased = 5.0; // Breeders

        let _ = perform_ant_sacrifice(&mut state, Decimal::zero());

        // sacMult #176 (5 pts) + sacCount #481 (3 pts) = 8.
        assert_ne!(state.achievements.achievements[176], 0, "sacMult tier 1");
        assert_ne!(state.achievements.achievements[481], 0, "sacCount >= 1");
        assert_eq!(state.achievements.achievement_points, 8.0);
        // The producer was consumed by the post-sacrifice reset.
        assert_eq!(state.ants.producers[1].purchased, 0.0);
    }

    #[test]
    fn ant_sacrifice_multiplier_picks_up_live_cube_blessings() {
        let mut state = GameState::default();
        // No blessings opened → the cube-blessing factor is identity.
        assert_eq!(ant_sacrifice_cube_blessing(&state).to_number(), 1.0);
        let base = compute_ant_sacrifice_multiplier(&state).to_number();

        // Raising the ant-sacrifice cube blessing (as opening cubes would) lifts
        // both the cascade factor and the assembled multiplier above the base.
        state.cube_blessings.ant_sacrifice = 5_000.0;
        assert!(ant_sacrifice_cube_blessing(&state).to_number() > 1.0);
        assert!(compute_ant_sacrifice_multiplier(&state).to_number() > base);
    }

    #[test]
    fn reborn_elo_rate_picks_up_live_ant_elo_cube_blessing() {
        let mut state = GameState::default();
        assert_eq!(ant_elo_cube_blessing(&state), 1.0); // identity at level 0
        state.cube_blessings.ant_elo = 1e6;
        assert!(ant_elo_cube_blessing(&state) > 1.0);
    }

    #[test]
    fn reborn_elo_rate_picks_up_mortuus_talisman_rarity() {
        let mut state = GameState::default();
        let base = compute_reborn_elo_creation_speed_mult(&state);
        // Mortuus rarity 5 → MORTUUS_INSCRIPT_VALUES[5] = 1.3× the rate.
        state.talismans.talisman_rarity[5] = 5.0;
        assert!(compute_reborn_elo_creation_speed_mult(&state) > base);
    }

    #[test]
    fn ascension_challenge_14_suppresses_obtainium_credit() {
        let mut state = GameState::default();
        state.ants.ant_sacrifice_timer = 10.0;
        state.challenges.current_ascension_challenge = 14;
        let before = state.researches.obtainium;

        let events = perform_ant_sacrifice(&mut state, Decimal::zero());

        assert_eq!(state.researches.obtainium, before);
        let CoreEvent::AntSacrificePerformed {
            obtainium_gained, ..
        } = events[0]
        else {
            panic!("expected AntSacrificePerformed");
        };
        assert_eq!(obtainium_gained.to_number(), 0.0);
    }

    #[test]
    fn activate_elo_climbs_reborn_elo_and_populates_leaderboard() {
        let mut state = GameState::default();
        state.ants.immortal_elo = 1_000.0;
        assert!(state.ants.highest_reborn_elo_ever.is_empty());

        // A large dt lets the gradual activation reach the immortal-ELO cap.
        activate_elo(&mut state, 1e9);

        assert!(state.ants.reborn_elo > 0.0);
        assert!(state.ants.reborn_elo <= 1_000.0);
        assert!(!state.ants.highest_reborn_elo_ever.is_empty());
        let elos: Vec<f64> = state
            .ants
            .highest_reborn_elo_ever
            .iter()
            .map(|e| e.elo)
            .collect();
        // This is the value that unsticks the slice-3a reborn-ELO progressive.
        assert!(calculate_leaderboard_value(&elos) > 0.0);
    }

    #[test]
    fn activate_elo_noop_when_reborn_already_maxed() {
        let mut state = GameState::default();
        state.ants.immortal_elo = 500.0;
        state.ants.reborn_elo = 500.0; // already fully activated → no bleed
        activate_elo(&mut state, 1e9);
        assert_eq!(state.ants.reborn_elo, 500.0);
        // The leaderboard still records the (maxed) reborn ELO.
        assert_eq!(state.ants.highest_reborn_elo_ever[0].elo, 500.0);
    }

    #[test]
    fn activate_elo_awards_quarks_from_the_leaderboard() {
        let mut state = GameState::default();
        state.ants.immortal_elo = 200_000.0;

        let events = activate_elo(&mut state, 1e9);

        assert!(
            state.quarks.worlds.to_number() > 0.0,
            "the reborn-ELO leaderboard should award quarks"
        );
        assert!(state.ants.quarks_gained_from_ants > 0.0);
        assert!(events
            .iter()
            .any(|e| matches!(e, CoreEvent::QuarksAwarded { .. })));

        // The running `quarks_gained_from_ants` total dedups: re-activating at
        // the same leaderboard (dt 0 → no further climb) awards ~nothing more.
        let before = state.quarks.worlds.to_number();
        let _ = activate_elo(&mut state, 0.0);
        let delta = state.quarks.worlds.to_number() - before;
        assert!(delta < 1.0, "second activation re-awarded {delta} quarks");
    }

    #[test]
    fn leaderboard_dedups_by_sacrifice_id_and_caps_at_five() {
        let mut lb: SmallVec<[RebornELOEntry; 5]> = SmallVec::new();
        update_single_leaderboard(&mut lb, 100.0, 1);
        update_single_leaderboard(&mut lb, 200.0, 2);
        // Same sacrifice id climbing → updates in place, no new row.
        update_single_leaderboard(&mut lb, 150.0, 1);
        assert_eq!(lb.len(), 2);
        assert_eq!(lb[0].elo, 200.0); // sacrifice 2
        assert_eq!(lb[1].elo, 150.0); // sacrifice 1, updated

        // Eight distinct sacrifices total → capped at the top 5.
        for id in 3..=8 {
            update_single_leaderboard(&mut lb, f64::from(id) * 10.0, id);
        }
        assert_eq!(lb.len(), 5);
        // Sorted descending: the two big early entries survive on top.
        assert_eq!(lb[0].elo, 200.0);
        assert_eq!(lb[1].elo, 150.0);
    }

    #[test]
    fn leaderboard_below_floor_when_full_is_dropped() {
        let mut lb: SmallVec<[RebornELOEntry; 5]> = SmallVec::new();
        for id in 1..=5 {
            update_single_leaderboard(&mut lb, f64::from(id) * 100.0, id);
        }
        assert_eq!(lb.len(), 5);
        let floor = lb.last().unwrap().elo;
        // A new sacrifice below the current floor is rejected.
        update_single_leaderboard(&mut lb, floor - 1.0, 99);
        assert_eq!(lb.len(), 5);
        assert!(lb.iter().all(|e| e.sacrifice_id != 99));
    }

    // ── End-to-end wiring through `tack` (the orchestration blind spot) ──────

    #[test]
    fn tack_consumes_ant_sacrifice_trigger_and_executes_the_sacrifice() {
        use crate::tick::{tack, TackInput};

        let mut state = GameState::default();
        // Unlock ant sacrifice (achievement #173) and prime the auto-sacrifice
        // gate: enough crumbs, past the cooldown, auto-sacrifice enabled.
        state.achievements.achievements[173] = 1;
        state.ants.crumbs_this_sacrifice = Decimal::from_finite(1e40);
        state.ants.ant_sacrifice_timer_real = 0.1;
        state.ants.toggles.auto_sacrifice_enabled = true;

        let input = TackInput {
            dt: 0.025,
            ..TackInput::default()
        };
        let output = tack(&mut state, &input);

        // The emitted `AntSacrificeTriggered` intent is consumed and the
        // sacrifice executes — proving the consume-wiring (the dropped-event
        // class of bug) actually fires end to end.
        assert!(
            output
                .events
                .iter()
                .any(|e| matches!(e, CoreEvent::AntSacrificePerformed { .. })),
            "AntSacrificeTriggered should be consumed into an executed sacrifice"
        );
        assert!(state.ants.immortal_elo > 0.0);
        assert!(state.ants.ant_sacrifice_count >= 1.0);
        assert_eq!(state.ants.crumbs.to_number(), 1.0); // ants reset to crumbs
    }

    #[test]
    fn tack_reborn_elo_activation_lights_the_progressive() {
        use crate::tick::{tack, TackInput};

        let mut state = GameState::default();
        // Simulate a post-sacrifice state with a large activated reborn ELO.
        state.ants.immortal_elo = 200_000.0;
        state.ants.reborn_elo = 150_000.0;
        assert_eq!(state.achievements.achievement_points, 0.0);

        let input = TackInput {
            dt: 0.025,
            ..TackInput::default()
        };
        // Tick 1: `phase_automation`'s activation records the reborn ELO onto the
        // leaderboard (read by the *next* tick's `phase_global_state`).
        tack(&mut state, &input);
        assert!(!state.ants.highest_reborn_elo_ever.is_empty());
        // Tick 2: the slot-3 reborn-ELO progressive picks up the leaderboard.
        tack(&mut state, &input);

        assert!(
            state.achievements.achievement_points > 0.0,
            "the populated reborn-ELO leaderboard should light the slice-3a progressive"
        );
    }
}
