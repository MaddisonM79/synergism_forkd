//! The `updateAll` autobuyer driver — Phase 5 of the tick.
//!
//! Ports the autobuyer families of `updateAll` (`Synergism.ts:3997-4291`):
//! the upgrade-tab autobuyer plus the building / prestige / transcension /
//! reincarnation / ascension / talisman / ant autobuyers. Each family gates
//! on the persisted toggle + the per-family unlock (and milestone where the
//! legacy does), then dispatches the already-ported buy primitive via
//! [`super::dispatch_buy`] — those primitives are autobuyer-aware (cap at 500,
//! no-op + emit nothing when unaffordable or maxed), so the affordability
//! pre-checks the legacy does are redundant and omitted.
//!
//! Because `player.toggles[1..=26]` all default `false` and `shoptoggles`'
//! per-tier unlock upgrades are unowned at default, the whole driver is inert
//! on a fresh save.
//!
//! **All 13 `updateAll` autobuyer families are wired** — including the formerly
//! deferred three: talisman (Family 11, `buyTalismanLevelToRarityIncrease`),
//! tesseract (Family 12, AMOUNT mode + `calculate_tess_buildings_in_budget`),
//! and ant-upgrades (Family 13, per-upgrade achievement / research / milestone
//! gates). The PERCENTAGE-mode tesseract path (`autoBuyTesseracts`, on-ascension)
//! is a separate call site, not part of this `updateAll` driver.

use crate::events::{ProducerType, UpgradeTier};
use crate::mechanics::accelerators::BuyAcceleratorInput;
use crate::mechanics::ant_masteries::{
    can_buy_ant_mastery, BuyAntMasteryInput, CanBuyAntMasteryInput, MAX_ANT_MASTERY_LEVEL,
};
use crate::mechanics::ant_producers::BuyAntProducerInput;
use crate::mechanics::ant_upgrades::{building_cost_scale_ant_upgrade_effect, BuyAntUpgradeInput};
use crate::mechanics::auto_upgrades::{
    buy_generator, click_upgrades, diamond_upgrade_reward, ClickUpgradesUnlocks,
    DIAMOND_UPGRADE_18_ACHIEVEMENT, DIAMOND_UPGRADE_19_ACHIEVEMENT, DIAMOND_UPGRADE_20_ACHIEVEMENT,
    UPGRADE_COSTS,
};
use crate::mechanics::calculate::{get_reduction_value, ReductionValueInput};
use crate::mechanics::challenges::{calc_ecc, ChallengeType};
use crate::mechanics::constant_upgrades::BuyConstantUpgradeInput;
use crate::mechanics::crystal_upgrades::BuyCrystalUpgradesInput;
use crate::mechanics::level_milestones::{get_level_milestone, LevelMilestoneKey};
use crate::mechanics::multipliers::BuyMultiplierInput;
use crate::mechanics::particle_buildings::BuyParticleBuildingInput;
use crate::mechanics::producers::{BuyMaxInput, GetProducerCostInput};
use crate::mechanics::rune_effects::{
    prism_rune_effects, thrift_rune_effects, PrismRuneKey, ThriftRuneKey,
};
use crate::mechanics::upgrades::{buy_upgrades, BuyUpgradeInput};
use crate::state::runes::{RUNE_PRISM, RUNE_THRIFT};
use crate::state::{BuyAmount, GameState};
use synergismforkd_bignum::Decimal;

use super::{dispatch_buy, AutomationPre, BuyRequest, TickOutput};

/// Run the full `updateAll` autobuyer pass. Mirrors the call order of
/// `updateAll`: upgrades, then the building / prestige / transcension /
/// reincarnation tabs, then ascension (constant + tesseract), talismans and
/// ants. `pre.prestige_point_gain` feeds the accelerator-boost path.
pub(crate) fn run_auto_buy(state: &mut GameState, pre: &AutomationPre, output: &mut TickOutput) {
    let ppg = pre.prestige_point_gain;

    // ── Family 1: autoUpgrades ───────────────────────────────────────
    auto_upgrades(state, output);

    // ── Family 2: coin producers 1..=5 ───────────────────────────────
    // toggles[t] && upgrades[80 + t] === 1.
    for t in 1..=5u8 {
        if state.automation.toggles[t as usize] && state.upgrades.upgrades[80 + t as usize] == 1 {
            let req = BuyRequest::ProducerMax(BuyMaxInput {
                index: t,
                producer_type: ProducerType::Coin,
                cost_input: producer_cost_input(state),
            });
            output.events.extend(dispatch_buy(state, &req, ppg));
        }
    }

    // ── Family 3: accelerator ────────────────────────────────────────
    // toggles[6] && upgrades[86] === 1.
    if state.automation.toggles[6] && state.upgrades.upgrades[86] == 1 {
        let req = BuyRequest::Accelerator(accelerator_input(state));
        output.events.extend(dispatch_buy(state, &req, ppg));
    }

    // ── Family 4: multiplier ─────────────────────────────────────────
    // toggles[7] && upgrades[87] === 1.
    if state.automation.toggles[7] && state.upgrades.upgrades[87] == 1 {
        let req = BuyRequest::Multiplier(multiplier_input(state));
        output.events.extend(dispatch_buy(state, &req, ppg));
    }

    // ── Family 5: accelerator boost ──────────────────────────────────
    // toggles[8] && upgrades[88] === 1 && upgrades[46] === 1.
    if state.automation.toggles[8]
        && state.upgrades.upgrades[88] == 1
        && state.upgrades.upgrades[46] == 1
    {
        output
            .events
            .extend(dispatch_buy(state, &BuyRequest::AcceleratorBoost, ppg));
    }

    // ── Family 6: diamond producers 1..=5 ────────────────────────────
    // toggles[9 + t] && getLevelMilestone(tierNCrystalAutobuy) === 1.
    let level = state.level.level;
    for t in 1..=5u8 {
        if state.automation.toggles[9 + t as usize] && crystal_autobuy_unlocked(level, t) {
            let req = BuyRequest::ProducerMax(BuyMaxInput {
                index: t,
                producer_type: ProducerType::Diamonds,
                cost_input: producer_cost_input(state),
            });
            output.events.extend(dispatch_buy(state, &req, ppg));
        }
    }

    // ── Family 7: crystal upgrades 1..=5 ─────────────────────────────
    // getLevelMilestone(tierNCrystalAutobuy) === 1 (no toggle).
    for t in 1..=5u8 {
        if crystal_autobuy_unlocked(level, t) {
            let req = BuyRequest::CrystalUpgrade(crystal_input(state, t));
            output.events.extend(dispatch_buy(state, &req, ppg));
        }
    }

    // ── Family 8: mythos producers 1..=5 ─────────────────────────────
    // toggles[15 + t] && upgrades[93 + t] === 1.
    for t in 1..=5u8 {
        if state.automation.toggles[15 + t as usize]
            && state.upgrades.upgrades[93 + t as usize] == 1
        {
            let req = BuyRequest::ProducerMax(BuyMaxInput {
                index: t,
                producer_type: ProducerType::Mythos,
                cost_input: producer_cost_input(state),
            });
            output.events.extend(dispatch_buy(state, &req, ppg));
        }
    }

    // ── Family 9: particle buildings 1..=5 ───────────────────────────
    // toggles[21 + t] && cubeUpgrades[7] > 0.
    let cube_upgrade_7 = state.cube_upgrade_levels.cube_upgrades[7];
    for t in 1..=5u8 {
        if state.automation.toggles[21 + t as usize] && cube_upgrade_7 > 0.0 {
            let req = BuyRequest::ParticleBuilding(BuyParticleBuildingInput {
                index: t,
                in_ascension_challenge_15: state.challenges.current_ascension_challenge == 15,
                autobuyer: true,
                particlebuyamount: BuyAmount::One,
            });
            output.events.extend(dispatch_buy(state, &req, ppg));
        }
    }

    // ── Family 10: constant upgrades 1..=10 ──────────────────────────
    // researches[175] > 0 → free buys (the primitive checks affordability).
    if state.researches.researches[175] > 0.0 {
        for i in 1..=10usize {
            let req = BuyRequest::ConstantUpgrade(BuyConstantUpgradeInput { index: i });
            output.events.extend(dispatch_buy(state, &req, ppg));
        }
    }

    // ── Family 11: talismans (buyTalismanLevelToRarityIncrease) ──────
    talisman_autobuyer(state, output, ppg);

    // ── Family 12: tesseract buildings (AMOUNT mode) ─────────────────
    tesseract_autobuyer(state, output, ppg);

    // ── Family 13 (producers + masteries): ant autobuyers ────────────
    // Gated on the ant autobuy toggles + getAchievementReward('antAutobuyers').
    // The ant-UPGRADE autobuyer is deferred (see the module docs).
    let tiers_unlocked = ant_autobuyer_tiers_unlocked(&state.achievements.achievements);
    if state.ants.toggles.autobuy_producers {
        let max = state.ants.toggles.max_buy_producers;
        for ant in (0..=8u8).rev() {
            if i32::from(ant) <= tiers_unlocked {
                let req = BuyRequest::AntProducer(BuyAntProducerInput { index: ant, max });
                output.events.extend(dispatch_buy(state, &req, ppg));
            }
        }
    }
    if state.ants.toggles.autobuy_masteries {
        for ant in (0..=8u8).rev() {
            if i32::from(ant) > tiers_unlocked {
                continue;
            }
            // while canBuyAntMastery && mastery < highestMastery (autobuy only
            // rebuilds the high-water mark lost on reincarnation).
            loop {
                let mastery = state.ants.masteries[ant as usize].mastery;
                let highest = state.ants.masteries[ant as usize].highest_mastery;
                if mastery >= highest {
                    break;
                }
                let check = CanBuyAntMasteryInput {
                    producer: ant,
                    mastery_level: mastery,
                    max_level: MAX_ANT_MASTERY_LEVEL,
                    current_elo: state.ants.reborn_elo,
                    current_particles: state.upgrades.reincarnation_points,
                };
                if !can_buy_ant_mastery(&check) {
                    break;
                }
                let req = BuyRequest::AntMastery(BuyAntMasteryInput { producer: ant });
                let events = dispatch_buy(state, &req, ppg);
                if events.is_empty() {
                    break;
                }
                output.events.extend(events);
            }
        }
    }

    // ── Family 13 (upgrades): ant-upgrade autobuyer ──────────────────
    // autobuyAntUpgrades — each of the 16 ant upgrades gates on its own
    // `autobuy()` predicate (per-upgrade achievement reward / research /
    // level milestone). `max_buy_upgrades` chooses single-vs-max.
    if state.ants.toggles.autobuy_upgrades {
        let max = state.ants.toggles.max_buy_upgrades;
        for upgrade in 0..16u8 {
            if ant_upgrade_autobuy_unlocked(state, upgrade) {
                let req = BuyRequest::AntUpgrade(BuyAntUpgradeInput {
                    index: upgrade,
                    max,
                });
                output.events.extend(dispatch_buy(state, &req, ppg));
            }
        }
    }
}

/// Per-ant-upgrade autobuy achievement index for upgrades 0-10 (the
/// `getAchievementReward('<name>Autobuy')` gate, `AntUpgrades/data/data.ts`):
/// AntSpeed/Coins→176, Taxes→177, AcceleratorBoosts/Multipliers→178,
/// Offerings→179, BuildingCostScale→482, Salvage→174, FreeRunes→484,
/// Obtainium→137, AntSacrifice→486 (verified against the antSacrificeUnlock=173
/// anchor). Upgrades 11-15 use research / level-milestone gates instead.
const ANT_UPGRADE_AUTOBUY_ACHIEVEMENTS: [usize; 11] =
    [176, 176, 177, 178, 178, 179, 482, 174, 484, 137, 486];

/// `antUpgradeData[upgrade].autobuy()` — whether ant upgrade `upgrade` (0-15)
/// may autobuy. 0-10: the per-upgrade achievement reward; 11 (Mortuus):
/// `researches[145] > 0`; 12-15 (AntELO/WowCubes/AscensionScore/Mortuus2): a
/// level milestone.
fn ant_upgrade_autobuy_unlocked(state: &GameState, upgrade: u8) -> bool {
    match upgrade {
        0..=10 => {
            let idx = ANT_UPGRADE_AUTOBUY_ACHIEVEMENTS[upgrade as usize];
            state
                .achievements
                .achievements
                .get(idx)
                .is_some_and(|&v| v != 0)
        }
        11 => state.researches.researches[145] > 0.0,
        12 => get_level_milestone(LevelMilestoneKey::AntSpeed2Autobuyer, state.level.level) > 0.5,
        13 => get_level_milestone(LevelMilestoneKey::WowCubesAutobuyer, state.level.level) > 0.5,
        14 => {
            get_level_milestone(
                LevelMilestoneKey::AscensionScoreAutobuyer,
                state.level.level,
            ) > 0.5
        }
        15 => get_level_milestone(LevelMilestoneKey::Mortuus2Autobuyer, state.level.level) > 0.5,
        _ => false,
    }
}

/// Achievement indices granting `antAutobuyers` (each reward `1`).
/// `getAchievementReward('antAutobuyers')` sums them, so the count owned is the
/// reward. Calibrated against `legacy/original/src/Achievements.ts`.
const ANT_AUTOBUYER_ACHIEVEMENTS: [usize; 9] = [173, 176, 177, 178, 179, 180, 181, 182, 349];

/// `+getAchievementReward('antAutobuyers') - 1` — the highest ant-producer tier
/// (0-indexed) whose producer / mastery autobuyer is unlocked; `-1` = none.
fn ant_autobuyer_tiers_unlocked(achievements: &[u8]) -> i32 {
    let count = ANT_AUTOBUYER_ACHIEVEMENTS
        .iter()
        .filter(|&&idx| achievements.get(idx).is_some_and(|&v| v != 0))
        .count();
    count as i32 - 1
}

/// `getLevelMilestone('tierNCrystalAutobuy') === 1` for crystal-autobuy tier
/// `tier` (`1..=5`) at player `level`. Gates the diamond-producer and
/// crystal-upgrade autobuyers.
fn crystal_autobuy_unlocked(level: f64, tier: u8) -> bool {
    let key = match tier {
        1 => LevelMilestoneKey::Tier1CrystalAutobuy,
        2 => LevelMilestoneKey::Tier2CrystalAutobuy,
        3 => LevelMilestoneKey::Tier3CrystalAutobuy,
        4 => LevelMilestoneKey::Tier4CrystalAutobuy,
        _ => LevelMilestoneKey::Tier5CrystalAutobuy,
    };
    get_level_milestone(key, level) > 0.5
}

/// `G.crystalUpgradesCost[0..5]` (`Variables.ts:36`) — `log10` base cost.
const CRYSTAL_UPGRADES_COST: [f64; 5] = [6.0, 15.0, 20.0, 40.0, 100.0];
/// `G.crystalUpgradeCostIncrement[0..5]` (`Variables.ts:37`).
const CRYSTAL_UPGRADE_COST_INCREMENT: [f64; 5] = [8.0, 15.0, 20.0, 40.0, 100.0];

/// `buyCrystalUpgrades(tier, true)` input. The prism cost-divisor uses the
/// effective prism rune level (`getRuneEffects('prism', 'costDivisorLog10')`).
fn crystal_input(state: &GameState, tier: u8) -> BuyCrystalUpgradesInput {
    let prism_level = super::first_five_effective_rune_level(state, RUNE_PRISM);
    let idx = usize::from(tier - 1);
    BuyCrystalUpgradesInput {
        i: tier,
        auto: true,
        prism_cost_divisor_log10: prism_rune_effects(prism_level, PrismRuneKey::CostDivisorLog10),
        crystal_upgrades_cost: CRYSTAL_UPGRADES_COST[idx],
        crystal_upgrade_cost_increment: CRYSTAL_UPGRADE_COST_INCREMENT[idx],
        upgrade_73: f64::from(state.upgrades.upgrades[73]),
        in_any_reincarnation_challenge: state.challenges.current_reincarnation_challenge != 0,
    }
}

/// `autoUpgrades()` (`Automation.ts:50`). The buy primitives check owned +
/// affordability internally, so we gate only on the per-block unlock + shop
/// toggle and dispatch.
fn auto_upgrades(state: &mut GameState, output: &mut TickOutput) {
    // Read everything the buys gate on before taking the &mut upgrade borrow.
    let shop = state.automation.shop_toggles;
    let cube_upgrade_8 = state.cube_upgrade_levels.cube_upgrades[8];
    let cube_upgrade_19 = state.cube_upgrade_levels.cube_upgrades[19];
    let highest_singularity = state.singularity.highest_singularity_count;
    let unlocks = ClickUpgradesUnlocks {
        prestige: state.reset_counters.prestige_unlocked,
        transcend: state.reset_counters.transcend_unlocked,
        reincarnate: state.reset_counters.reincarnate_unlocked,
    };
    let diamond_18 = diamond_upgrade_reward(
        &state.achievements.achievements,
        DIAMOND_UPGRADE_18_ACHIEVEMENT,
    );
    let diamond_19 = diamond_upgrade_reward(
        &state.achievements.achievements,
        DIAMOND_UPGRADE_19_ACHIEVEMENT,
    );
    let diamond_20 = diamond_upgrade_reward(
        &state.achievements.achievements,
        DIAMOND_UPGRADE_20_ACHIEVEMENT,
    );

    let up = &mut state.upgrades;
    let mk = |tier: UpgradeTier, i: usize| BuyUpgradeInput {
        tier,
        pos: i as u32,
        cost_exponent: UPGRADE_COSTS[i],
        requirement_exists: true,
    };

    // Generators 101..=120 (upgrades[90] && shoptoggles.generators).
    if up.upgrades[90] > 0 && shop.generators {
        for i in 1..=20usize {
            output.events.extend(buy_generator(up, i));
        }
    }
    // Coin upgrades 1..=20, then 121..=125 (cubeUpgrades[19] > 0).
    if up.upgrades[91] > 0 && shop.coin {
        for i in 1..=20usize {
            output
                .events
                .extend(buy_upgrades(up, mk(UpgradeTier::Coin, i)));
        }
        if cube_upgrade_19 > 0.0 {
            for i in 121..=125usize {
                output
                    .events
                    .extend(buy_upgrades(up, mk(UpgradeTier::Coin, i)));
            }
        }
    }
    // Prestige upgrades 21..=37, then 38/39/40 (diamondUpgrade-gated).
    if up.upgrades[92] > 0 && shop.prestige {
        for i in 21..=37usize {
            output
                .events
                .extend(buy_upgrades(up, mk(UpgradeTier::Prestige, i)));
        }
        if diamond_18 {
            output
                .events
                .extend(buy_upgrades(up, mk(UpgradeTier::Prestige, 38)));
        }
        if diamond_19 {
            output
                .events
                .extend(buy_upgrades(up, mk(UpgradeTier::Prestige, 39)));
        }
        if diamond_20 {
            output
                .events
                .extend(buy_upgrades(up, mk(UpgradeTier::Prestige, 40)));
        }
    }
    // Transcend upgrades 41..=60.
    if up.upgrades[99] > 0 && shop.transcend {
        for i in 41..=60usize {
            output
                .events
                .extend(buy_upgrades(up, mk(UpgradeTier::Transcend, i)));
        }
    }
    // Reincarnation upgrades 61..=80 (cubeUpgrades[8] > 0).
    if cube_upgrade_8 > 0.0 && shop.reincarnate {
        for i in 61..=80usize {
            output
                .events
                .extend(buy_upgrades(up, mk(UpgradeTier::Reincarnation, i)));
        }
    }
    // Singularity-25: click upgrades 81..=100.
    if highest_singularity >= 25.0 {
        for i in 81..=100usize {
            output.events.extend(click_upgrades(up, unlocks, i));
        }
    }
}

/// The challenge-state flags + reduction value the producer cost solver reads.
/// `cost_divisor` is the legacy `r` (`getReductionValue()`), **not**
/// `G.costDivisor` (which is always `1` and lives in the solver).
///
/// Public (re-exported from the crate root): the UI builds its manual
/// `BuyRequest::ProducerMax` inputs through this same function so manual
/// and autobuyer cost math can never drift.
#[must_use]
pub fn producer_cost_input(state: &GameState) -> GetProducerCostInput {
    GetProducerCostInput {
        cost_divisor: reduction_value(state),
        in_transcension_challenge_4: state.challenges.current_transcension_challenge == 4,
        in_reincarnation_challenge_8: state.challenges.current_reincarnation_challenge == 8,
        in_reincarnation_challenge_10: state.challenges.current_reincarnation_challenge == 10,
        challengecompletions_4: state.challenges.challenge_completions[4],
        challengecompletions_8: state.challenges.challenge_completions[8],
    }
}

/// `getReductionValue()` (`Buy.ts:16`) — the producer-cost reduction `r`.
///
/// Public (re-exported from the crate root) for the UI's manual
/// `BuyRequest::Producer` inputs; see [`producer_cost_input`].
#[must_use]
pub fn reduction_value(state: &GameState) -> f64 {
    let researches_sum: f64 = (56..=60).map(|i| state.researches.researches[i]).sum();
    let ant_building_cost_scale =
        building_cost_scale_ant_upgrade_effect(state.ants.upgrades[6]).building_cost_scale;
    let thrift_level = super::first_five_effective_rune_level(state, RUNE_THRIFT);
    get_reduction_value(&ReductionValueInput {
        thrift_cost_delay: thrift_rune_effects(thrift_level, ThriftRuneKey::CostDelay),
        researches_sum,
        challenge_completions_4: state.challenges.challenge_completions[4],
        ant_building_cost_scale,
    })
}

/// `buyAccelerator(true)` input. The accelerator cost uses `G.costDivisor`
/// (always `1`), not the reduction value.
fn accelerator_input(state: &GameState) -> BuyAcceleratorInput {
    BuyAcceleratorInput {
        autobuyer: true,
        coinbuyamount: BuyAmount::One,
        cost_divisor: 1.0,
        transcend_ecc: calc_ecc(
            ChallengeType::Transcend,
            state.challenges.challenge_completions[4],
        ),
        in_transcension_challenge_4: state.challenges.current_transcension_challenge == 4,
        in_reincarnation_challenge_8: state.challenges.current_reincarnation_challenge == 8,
    }
}

/// `buyMultiplier(true)` input — same shape as [`accelerator_input`].
fn multiplier_input(state: &GameState) -> BuyMultiplierInput {
    BuyMultiplierInput {
        autobuyer: true,
        coinbuyamount: BuyAmount::One,
        cost_divisor: 1.0,
        transcend_ecc: calc_ecc(
            ChallengeType::Transcend,
            state.challenges.challenge_completions[4],
        ),
        in_transcension_challenge_4: state.challenges.current_transcension_challenge == 4,
        in_reincarnation_challenge_8: state.challenges.current_reincarnation_challenge == 8,
    }
}

/// `buyTalismanLevelToRarityIncrease(t, true)` for every talisman — updateAll's
/// talisman autobuyer (`Synergism.ts:4286`). Gated on `auto_fortify_toggle` +
/// (`researches[130] > 0 || researches[135] > 0`). For each talisman it buys
/// single levels toward the next rarity tier
/// ([`levels_until_talisman_rarity_increase`]) while affordable — `dispatch_buy`
/// returns no events when a level is unaffordable or the cap binds, ending the
/// loop. Inert at default (`auto_fortify_toggle` false).
fn talisman_autobuyer(state: &mut GameState, output: &mut TickOutput, ppg: Decimal) {
    use crate::mechanics::talisman_costs::talisman_costs_for_level;
    use crate::mechanics::talisman_levels::{
        levels_until_talisman_rarity_increase, BuyTalismanLevelInput,
        LevelsUntilTalismanRarityIncreaseInput, TALISMAN_MAX_LEVELS,
    };

    if !state.automation.auto_fortify_toggle {
        return;
    }
    if !(state.researches.researches[130] > 0.0 || state.researches.researches[135] > 0.0) {
        return;
    }
    let universal = universal_talisman_level_cap_increase(state);
    for (t, &max_level) in TALISMAN_MAX_LEVELS.iter().enumerate() {
        let level_cap = max_level + talisman_level_cap_increase(state, t, universal);
        let levels_to_buy =
            levels_until_talisman_rarity_increase(&LevelsUntilTalismanRarityIncreaseInput {
                level: state.talismans.talisman_levels[t],
                max_level,
                current_rarity: state.talismans.talisman_rarity[t],
                level_cap,
            });
        if levels_to_buy <= 0.0 {
            continue;
        }
        // Bound by levelsToBuy; the dispatch ends the loop early (no events)
        // once a level is unaffordable or `level_cap` binds.
        for _ in 0..(levels_to_buy as u64) {
            let level = state.talismans.talisman_levels[t];
            let req = BuyRequest::TalismanLevel(BuyTalismanLevelInput {
                index: t,
                costs: talisman_costs_for_level(t, level),
                level_cap,
            });
            let events = dispatch_buy(state, &req, ppg);
            if events.is_empty() {
                break;
            }
            output.events.extend(events);
        }
    }
}

/// `universalTalismanMaxLevelIncreasers()` (`Talismans.ts`) — the talisman
/// level-cap bonus shared by most talismans. `0` at the default state.
/// (`taxmanLastStand` talismanFreeLevel is deferred — a singularity challenge,
/// inert until entered.)
fn universal_talisman_level_cap_increase(state: &GameState) -> f64 {
    use crate::mechanics::octeracts::{
        octeract_talisman_level_cap_1_effect, octeract_talisman_level_cap_2_effect,
        octeract_talisman_level_cap_3_effect, octeract_talisman_level_cap_4_effect,
    };
    use crate::state::octeract_upgrades::{
        OCTERACT_TALISMAN_LEVEL_CAP_1, OCTERACT_TALISMAN_LEVEL_CAP_2,
        OCTERACT_TALISMAN_LEVEL_CAP_3, OCTERACT_TALISMAN_LEVEL_CAP_4,
    };
    let oct = |i: usize| {
        state.octeract_upgrades.upgrades[i].level + state.octeract_upgrades.upgrades[i].free_level
    };
    6.0 * calc_ecc(
        ChallengeType::Ascension,
        state.challenges.challenge_completions[13],
    ) + (state.researches.researches[200] / 400.0).floor()
        + octeract_talisman_level_cap_1_effect(oct(OCTERACT_TALISMAN_LEVEL_CAP_1))
        + octeract_talisman_level_cap_2_effect(oct(OCTERACT_TALISMAN_LEVEL_CAP_2))
        + octeract_talisman_level_cap_3_effect(oct(OCTERACT_TALISMAN_LEVEL_CAP_3))
        + octeract_talisman_level_cap_4_effect(oct(OCTERACT_TALISMAN_LEVEL_CAP_4))
}

/// Per-talisman `levelCapIncrease()` (`Talismans.ts`): cookieGrandma `+54` /
/// horseShoe `+88` (constants), achievement = the `achievementTalismanEnhancement`
/// level milestone, everything else the universal increaser. The
/// metaphysics/plastic/mortuus custom extras are deferred (0/small at the
/// reachable state).
fn talisman_level_cap_increase(state: &GameState, talisman: usize, universal: f64) -> f64 {
    use crate::state::{TALISMAN_ACHIEVEMENT, TALISMAN_COOKIE_GRANDMA, TALISMAN_HORSE_SHOE};
    match talisman {
        TALISMAN_COOKIE_GRANDMA => 54.0,
        TALISMAN_HORSE_SHOE => 88.0,
        TALISMAN_ACHIEVEMENT => get_level_milestone(
            LevelMilestoneKey::AchievementTalismanEnhancement,
            state.level.level,
        ),
        _ => universal,
    }
}

/// AMOUNT-mode tesseract-building autobuyer (`updateAll`, `Synergism.ts:4256`).
/// Gated on `researches[190] > 0 && tesseract_auto_buyer_toggle && ascension_reset_mode == Amount`.
/// Solves the cheapest-first distribution of `wow_tesseracts − reserve` across the
/// auto-buy-enabled tiers ([`calculate_tess_buildings_in_budget`]) and dispatches
/// the deltas highest-tier-first. The PERCENTAGE mode (on-ascension
/// `autoBuyTesseracts`) is a separate call site and not driven here.
fn tesseract_autobuyer(state: &mut GameState, output: &mut TickOutput, ppg: Decimal) {
    use crate::mechanics::tesseract_buildings::{
        calculate_tess_buildings_in_budget, BuyTesseractBuildingInput,
    };
    use crate::state::automation::AutoAscensionMode;

    if state.researches.researches[190] <= 0.0
        || !state.automation.tesseract_auto_buyer_toggle
        || state.automation.ascension_reset_mode != AutoAscensionMode::Amount
    {
        return;
    }
    let owned: [Option<f64>; 5] = std::array::from_fn(|i| {
        if state.automation.auto_tesseracts[i + 1] {
            Some(state.tesseract_buildings.building((i + 1) as u8).owned)
        } else {
            None
        }
    });
    let budget =
        state.tesseract_buildings.wow_tesseracts - state.automation.tesseract_auto_buyer_amount;
    let buy_to = calculate_tess_buildings_in_budget(owned, budget);
    // Highest tier to lowest (matches the legacy order — guards float fuzz).
    for i in (0..5).rev() {
        if let (Some(from), Some(to)) = (owned[i], buy_to[i]) {
            if to > from {
                let req = BuyRequest::TesseractBuilding(BuyTesseractBuildingInput {
                    index: (i + 1) as u8,
                    amount: to - from,
                });
                output.events.extend(dispatch_buy(state, &req, ppg));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn talisman_autobuyer_levels_toward_rarity_target() {
        let mut state = GameState::default();
        state.automation.auto_fortify_toggle = true;
        state.researches.researches[130] = 1.0; // gate
                                                // Exemption (idx 0) at rarity 1 (recompute runs earlier in the real
                                                // tick; set directly since we exercise the autobuyer in isolation).
        state.talismans.talisman_rarity[0] = 1.0;
        state.talismans.talisman_shards = 1e9;
        state.talismans.common_fragments = 1e9;
        let mut output = TickOutput::default();
        talisman_autobuyer(&mut state, &mut output, Decimal::zero());
        // rarity 1, maxLevel 180 ⇒ buys toward ceil(180/6)=30 levels.
        assert!(state.talismans.talisman_levels[0] > 0.0);
        assert!(!output.events.is_empty());
    }

    #[test]
    fn talisman_autobuyer_inert_without_toggle() {
        let mut state = GameState::default();
        state.researches.researches[130] = 1.0;
        state.talismans.talisman_rarity[0] = 1.0;
        state.talismans.talisman_shards = 1e9;
        let mut output = TickOutput::default();
        talisman_autobuyer(&mut state, &mut output, Decimal::zero());
        assert_eq!(state.talismans.talisman_levels[0], 0.0);
        assert!(output.events.is_empty());
    }

    #[test]
    fn ant_upgrade_autobuy_gate_maps_correctly() {
        let mut state = GameState::default();
        assert!(!ant_upgrade_autobuy_unlocked(&state, 0));
        // Achievement 176 (inceptus + fortunae) unlocks AntSpeed (0) + Coins (1).
        state.achievements.achievements[176] = 1;
        assert!(ant_upgrade_autobuy_unlocked(&state, 0));
        assert!(ant_upgrade_autobuy_unlocked(&state, 1));
        assert!(!ant_upgrade_autobuy_unlocked(&state, 2)); // Taxes needs 177
                                                           // Mortuus (11) gates on research[145], not an achievement.
        assert!(!ant_upgrade_autobuy_unlocked(&state, 11));
        state.researches.researches[145] = 1.0;
        assert!(ant_upgrade_autobuy_unlocked(&state, 11));
    }

    #[test]
    fn ant_upgrade_autobuyer_buys_unlocked_upgrade() {
        let mut state = GameState::default();
        state.ants.toggles.autobuy_upgrades = true;
        state.achievements.achievements[176] = 1; // unlock AntSpeed (0)
        state.ants.crumbs = Decimal::from_mantissa_exponent(1.0, 20.0); // ample
        let mut output = TickOutput::default();
        run_auto_buy(&mut state, &AutomationPre::default(), &mut output);
        assert!(state.ants.upgrades[0] > 0.0);
    }

    #[test]
    fn tesseract_autobuyer_buys_within_budget() {
        let mut state = GameState::default();
        state.researches.researches[190] = 1.0;
        state.automation.tesseract_auto_buyer_toggle = true; // ascension mode defaults Amount
        state.automation.auto_tesseracts[1] = true; // enable tier 1
        state.tesseract_buildings.wow_tesseracts = 1000.0;
        let mut output = TickOutput::default();
        tesseract_autobuyer(&mut state, &mut output, Decimal::zero());
        assert!(state.tesseract_buildings.building(1).owned > 0.0);
        assert!(!output.events.is_empty());
    }

    #[test]
    fn tesseract_autobuyer_inert_without_toggle() {
        let mut state = GameState::default();
        state.researches.researches[190] = 1.0;
        state.automation.auto_tesseracts[1] = true;
        state.tesseract_buildings.wow_tesseracts = 1000.0;
        let mut output = TickOutput::default();
        tesseract_autobuyer(&mut state, &mut output, Decimal::zero());
        assert_eq!(state.tesseract_buildings.building(1).owned, 0.0);
        assert!(output.events.is_empty());
    }
}
