//! Crystal-upgrade purchases.
//!
//! Verbatim port of `buyCrystalUpgrades` from
//! `legacy/core_split/packages/logic/src/mechanics/crystalUpgrades.ts`.
//!
//! Crystal upgrades are a separate upgrade ladder bought with prestige
//! shards. Unlike the discrete `buy_upgrades` bitmap, each crystal upgrade
//! has an integer level that grows whenever the player has shards to
//! spare. The buy formula computes the maximum affordable level
//! analytically (no loop), then sets the level if it exceeds the current
//! one and deducts the cost (manual path only — the autobuyer is granted
//! free levels to dodge a late-game precision issue).

use smallvec::SmallVec;
use synergismforkd_bignum::Decimal;

use crate::events::CoreEvent;
use crate::state::CrystalUpgradesState;

/// Inputs to [`buy_crystal_upgrades`]. Mirrors `BuyCrystalUpgradesInput`.
#[derive(Debug, Clone, Copy)]
pub struct BuyCrystalUpgradesInput {
    /// 1-based crystal-upgrade index.
    pub i: u8,
    /// True when the autobuyer is driving — skips the
    /// `prestige_shards` deduction.
    pub auto: bool,
    /// Prism rune cost-divisor (`log10`) —
    /// `getRuneEffects('prism', 'costDivisorLog10')`.
    pub prism_cost_divisor_log10: f64,
    /// Base cost (`log10`) for this upgrade — `G.crystalUpgradesCost[i-1]`.
    pub crystal_upgrades_cost: f64,
    /// Cost growth (`log10`) per level —
    /// `G.crystalUpgradeCostIncrement[i-1]`.
    pub crystal_upgrade_cost_increment: f64,
    /// `player.upgrades[73]` — gates the +10 bonus when also in a
    /// reincarnation challenge.
    pub upgrade_73: f64,
    /// `player.currentChallenge.reincarnation !== 0` — gates the +10
    /// bonus.
    pub in_any_reincarnation_challenge: bool,
}

/// Closed-form solve for "max level affordable with current shards". The
/// cumulative cost to reach level `n` is roughly
///
/// ```text
/// crystal_upgrades_cost
///   + crystal_upgrade_cost_increment * (n * (n - 1) / 2)
/// ```
///
/// in `log10`. Invert to find `n` given `log10(prestige_shards + 1)`.
fn calculate_crystal_buy(
    prestige_shards: Decimal,
    prism_cost_divisor_log10: f64,
    crystal_upgrades_cost: f64,
    crystal_upgrade_cost_increment: f64,
) -> f64 {
    let exponent = (prestige_shards + Decimal::one()).log10().to_number();
    let radicand = (2.0 * (exponent + prism_cost_divisor_log10 - crystal_upgrades_cost)
        / crystal_upgrade_cost_increment
        + 1.0 / 4.0)
        .max(0.0);
    (radicand.powf(1.0 / 2.0) + 1.0 / 2.0).floor()
}

/// Buy as many crystal-upgrade levels as affordable. Mirrors the closed-form
/// max-affordable solve in the legacy TS — no loop, no per-level fee.
#[must_use]
pub fn buy_crystal_upgrades(
    state: &mut CrystalUpgradesState,
    input: BuyCrystalUpgradesInput,
) -> SmallVec<[CoreEvent; 4]> {
    let mut events: SmallVec<[CoreEvent; 4]> = SmallVec::new();
    let u = usize::from(input.i - 1);

    // Bonus levels: +10 when player owns upgrade 73 AND is currently
    // inside any reincarnation challenge. The bonus levels do NOT
    // contribute to the cost.
    let mut c = 0.0_f64;
    if input.upgrade_73 > 0.5 && input.in_any_reincarnation_challenge {
        c += 10.0;
    }

    let to_buy = calculate_crystal_buy(
        state.prestige_shards,
        input.prism_cost_divisor_log10,
        input.crystal_upgrades_cost,
        input.crystal_upgrade_cost_increment,
    );

    let before = state.crystal_upgrades[u];
    let target = to_buy + c;

    if target > before {
        // The TS source applied a `100 / 100 *` factor here as a legacy
        // no-op multiplier; the operation is mathematically equivalent
        // to a plain assignment, so it's dropped (clippy's `eq_op` would
        // refuse to compile the literal otherwise).
        state.crystal_upgrades[u] = target;
        let starting_shards = state.prestige_shards;
        if to_buy > 0.0 && !input.auto {
            let cost_log10 = input.crystal_upgrades_cost - input.prism_cost_divisor_log10
                + input.crystal_upgrade_cost_increment
                    * (1.0 / 2.0 * (to_buy - 1.0 / 2.0).powi(2) - 1.0 / 8.0);
            let cost = Decimal::from_finite(10.0).pow(Decimal::from_finite(cost_log10));
            state.prestige_shards = (state.prestige_shards - cost).max(Decimal::zero());
        }
        events.push(CoreEvent::CrystalUpgradePurchased {
            i: input.i,
            before,
            after: state.crystal_upgrades[u],
            spent: starting_shards - state.prestige_shards,
        });
    }

    events
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::CRYSTAL_UPGRADES_DEFAULT_LEN;

    fn baseline_input() -> BuyCrystalUpgradesInput {
        BuyCrystalUpgradesInput {
            i: 1,
            auto: false,
            prism_cost_divisor_log10: 0.0,
            crystal_upgrades_cost: 4.0, // base cost 10^4
            crystal_upgrade_cost_increment: 2.0,
            upgrade_73: 0.0,
            in_any_reincarnation_challenge: false,
        }
    }

    fn baseline_state() -> CrystalUpgradesState {
        CrystalUpgradesState {
            prestige_shards: Decimal::zero(),
            crystal_upgrades: [0.0; CRYSTAL_UPGRADES_DEFAULT_LEN],
        }
    }

    #[test]
    fn no_shards_buys_nothing() {
        let mut state = baseline_state();
        let events = buy_crystal_upgrades(&mut state, baseline_input());
        assert_eq!(state.crystal_upgrades[0], 0.0);
        assert!(events.is_empty());
    }

    #[test]
    fn enough_shards_advances_level() {
        let mut state = CrystalUpgradesState {
            prestige_shards: Decimal::from_finite(1e8),
            ..baseline_state()
        };
        let baseline_shards = state.prestige_shards;
        let events = buy_crystal_upgrades(&mut state, baseline_input());
        assert!(state.crystal_upgrades[0] > 0.0);
        assert!(state.prestige_shards < baseline_shards);
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn autobuyer_grants_free_levels() {
        // Autobuyer skips the deduction → shards unchanged but level
        // advances.
        let mut state = CrystalUpgradesState {
            prestige_shards: Decimal::from_finite(1e8),
            ..baseline_state()
        };
        let baseline_shards = state.prestige_shards;
        let auto = BuyCrystalUpgradesInput {
            auto: true,
            ..baseline_input()
        };
        let events = buy_crystal_upgrades(&mut state, auto);
        assert!(state.crystal_upgrades[0] > 0.0);
        assert_eq!(state.prestige_shards, baseline_shards);
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn upgrade_73_in_reincarnation_grants_plus_ten_bonus() {
        // No shards → calculate_crystal_buy returns 0; but the +10 bonus
        // still applies if upgrade_73 owned and in reincarnation challenge.
        let mut state = baseline_state();
        let baseline_shards = state.prestige_shards;
        let with_bonus = BuyCrystalUpgradesInput {
            upgrade_73: 1.0,
            in_any_reincarnation_challenge: true,
            ..baseline_input()
        };
        let events = buy_crystal_upgrades(&mut state, with_bonus);
        assert_eq!(state.crystal_upgrades[0], 10.0);
        // No shards were deducted (to_buy == 0 guard).
        assert_eq!(state.prestige_shards, baseline_shards);
        // Event is emitted because target > before.
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn upgrade_73_only_grants_bonus_in_reincarnation_challenge() {
        let mut state = baseline_state();
        let outside = BuyCrystalUpgradesInput {
            upgrade_73: 1.0,
            in_any_reincarnation_challenge: false,
            ..baseline_input()
        };
        let events = buy_crystal_upgrades(&mut state, outside);
        // Without the challenge flag, no bonus.
        assert_eq!(state.crystal_upgrades[0], 0.0);
        assert!(events.is_empty());
    }

    #[test]
    fn already_at_target_is_noop() {
        // Pre-set the level to the calculated buy value; the function
        // should not change state or emit an event.
        let mut state = CrystalUpgradesState {
            prestige_shards: Decimal::from_finite(1e8),
            crystal_upgrades: [1_000_000.0; CRYSTAL_UPGRADES_DEFAULT_LEN], // far above any reasonable buy
        };
        let baseline_shards = state.prestige_shards;
        let events = buy_crystal_upgrades(&mut state, baseline_input());
        assert_eq!(state.crystal_upgrades[0], 1_000_000.0);
        assert_eq!(state.prestige_shards, baseline_shards);
        assert!(events.is_empty());
    }

    #[test]
    fn event_spent_matches_resource_delta() {
        let mut state = CrystalUpgradesState {
            prestige_shards: Decimal::from_finite(1e8),
            ..baseline_state()
        };
        let baseline_shards = state.prestige_shards;
        let events = buy_crystal_upgrades(&mut state, baseline_input());
        let spent = baseline_shards - state.prestige_shards;
        assert_eq!(events.len(), 1);
        match &events[0] {
            CoreEvent::CrystalUpgradePurchased {
                i,
                before,
                after,
                spent: ev_spent,
            } => {
                assert_eq!(*i, 1);
                assert_eq!(*before, 0.0);
                assert_eq!(*after, state.crystal_upgrades[0]);
                assert_eq!(*ev_spent, spent);
            }
            other => panic!("expected CrystalUpgradePurchased, got {other:?}"),
        }
    }

    #[test]
    fn prism_divisor_reduces_cost_and_increases_buyable() {
        let small_state = CrystalUpgradesState {
            prestige_shards: Decimal::from_finite(1e8),
            ..baseline_state()
        };
        let plain = baseline_input();
        let with_prism = BuyCrystalUpgradesInput {
            prism_cost_divisor_log10: 3.0, // divides effective cost by 1000
            ..plain
        };
        let mut plain_state = small_state.clone();
        let _ = buy_crystal_upgrades(&mut plain_state, plain);
        let mut prism_state = small_state.clone();
        let _ = buy_crystal_upgrades(&mut prism_state, with_prism);
        // With prism divisor, the player should reach a higher level.
        assert!(prism_state.crystal_upgrades[0] > plain_state.crystal_upgrades[0]);
    }

    #[test]
    fn shards_clamped_at_zero_if_deduction_exceeds_balance() {
        // The closed-form solve picks a level matching the resources; the
        // deduction formula can produce a value slightly different
        // (rounded down). The .max(0) clamp guarantees non-negative shards
        // even if the deduction overshoots.
        let mut state = CrystalUpgradesState {
            prestige_shards: Decimal::from_finite(1e6),
            ..baseline_state()
        };
        let _ = buy_crystal_upgrades(&mut state, baseline_input());
        assert!(state.prestige_shards >= Decimal::zero());
    }
}
