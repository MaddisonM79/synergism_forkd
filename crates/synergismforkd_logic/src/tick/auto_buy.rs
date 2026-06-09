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

use crate::events::{ProducerType, UpgradeTier};
use crate::mechanics::accelerators::BuyAcceleratorInput;
use crate::mechanics::ant_upgrades::building_cost_scale_ant_upgrade_effect;
use crate::mechanics::auto_upgrades::{
    buy_generator, click_upgrades, diamond_upgrade_reward, ClickUpgradesUnlocks,
    DIAMOND_UPGRADE_18_ACHIEVEMENT, DIAMOND_UPGRADE_19_ACHIEVEMENT, DIAMOND_UPGRADE_20_ACHIEVEMENT,
    UPGRADE_COSTS,
};
use crate::mechanics::calculate::{get_reduction_value, ReductionValueInput};
use crate::mechanics::challenges::{calc_ecc, ChallengeType};
use crate::mechanics::multipliers::BuyMultiplierInput;
use crate::mechanics::producers::{BuyMaxInput, GetProducerCostInput};
use crate::mechanics::upgrades::{buy_upgrades, BuyUpgradeInput};
use crate::state::{BuyAmount, GameState};

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
/// `G.costDivisor` (which is always `1` and lives in the solver). The thrift
/// rune `costDelay` term is neutral-defaulted to `0` (it needs the effective
/// rune-level pipeline); the researches[56..=60], transcend-ECC, and
/// ant-building-cost-scale terms are computed — faithful at default and
/// through mid-game.
fn producer_cost_input(state: &GameState) -> GetProducerCostInput {
    GetProducerCostInput {
        cost_divisor: reduction_value(state),
        in_transcension_challenge_4: state.challenges.current_transcension_challenge == 4,
        in_reincarnation_challenge_8: state.challenges.current_reincarnation_challenge == 8,
        in_reincarnation_challenge_10: state.challenges.current_reincarnation_challenge == 10,
        challengecompletions_4: state.challenges.challenge_completions[4],
        challengecompletions_8: state.challenges.challenge_completions[8],
    }
}

/// `getReductionValue()` (`Buy.ts:16`) minus the thrift-rune term (neutral 0).
fn reduction_value(state: &GameState) -> f64 {
    let researches_sum: f64 = (56..=60).map(|i| state.researches.researches[i]).sum();
    let ant_building_cost_scale =
        building_cost_scale_ant_upgrade_effect(state.ants.upgrades[6]).building_cost_scale;
    get_reduction_value(&ReductionValueInput {
        thrift_cost_delay: 0.0,
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
