//! Multiplier cost formula and purchase loop.
//!
//! Verbatim port of `getCostMultiplier` + `buyMultiplier` from
//! `legacy_core_split/packages/logic/src/mechanics/multipliers.ts`. The shape
//! mirrors [`crate::mechanics::accelerators`] — same two-path buy loop and
//! flag-flip rules, different cost-curve constants.

use smallvec::SmallVec;
use synergismforkd_bignum::Decimal;

use crate::events::CoreEvent;
use crate::math::smallest_inc::smallest_inc;
use crate::state::{BuyAmount, MultiplierState};

const BUYMAX: f64 = 1e15;

/// Input to [`get_cost_multiplier`]. Mirrors `GetCostMultiplierInput` in the
/// TS source — the `player.*` / `G.*` reads hoisted into an explicit
/// parameter.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GetCostMultiplierInput {
    /// `G.costDivisor` at call time.
    pub cost_divisor: f64,
    /// `CalcECC('transcend', player.challengecompletions[4])` — Eternal
    /// Challenge transcend completions.
    pub transcend_ecc: f64,
    /// `player.currentChallenge.transcension === 4`
    pub in_transcension_challenge_4: bool,
    /// `player.currentChallenge.reincarnation === 8`
    pub in_reincarnation_challenge_8: bool,
}

/// Cost in coins of buying the `buying_to`-th multiplier (1-indexed; the
/// formula internally decrements to 0-index, matching the TS source).
pub fn get_cost_multiplier(buying_to: f64, input: GetCostMultiplierInput) -> Decimal {
    let buying_to = buying_to - 1.0;

    let original_cost = 1e4;
    let mut cost = Decimal::from_finite(original_cost);
    cost *= Decimal::from_finite(10.0).pow(Decimal::from_finite(buying_to / input.cost_divisor));

    let transcend_break = 2.0 * input.transcend_ecc;
    if buying_to > 75.0 + transcend_break {
        let num = buying_to - 75.0 - transcend_break;
        let factorial_bit = Decimal::from_finite(num).factorial();
        let pow_bit = Decimal::from_finite(10.0).pow(Decimal::from_finite(num));
        cost *= factorial_bit * pow_bit;
    }

    if buying_to > 2000.0 + transcend_break {
        let sum_num = buying_to - 2000.0 - transcend_break;
        let sum_bit = sum_num * (sum_num + 1.0) / 2.0;
        cost *= Decimal::from_finite(2.0).pow(Decimal::from_finite(sum_bit));
    }

    if input.in_transcension_challenge_4 {
        let sum_bit = buying_to * (buying_to + 1.0) / 2.0;
        cost *= Decimal::from_finite(10.0).pow(Decimal::from_finite(sum_bit));
    }

    if input.in_reincarnation_challenge_8 {
        let sum_bit = buying_to * (buying_to + 1.0) / 2.0;
        cost *= Decimal::from_finite(1e50).pow(Decimal::from_finite(sum_bit));
    }

    if buying_to > BUYMAX {
        let diminishing_exponent = 1.0 / 8.0;
        // See accelerators.rs for the BUYMAX + 1.0 rationale.
        let quadrillion_cost = get_cost_multiplier(BUYMAX + 1.0, input);
        let mut new_cost = quadrillion_cost.pow(Decimal::from_finite(
            (buying_to / BUYMAX).powf(1.0 / diminishing_exponent),
        ));
        let new_extra = new_cost.exponent() - new_cost.exponent().floor();
        new_cost.set_exponent(new_cost.exponent().floor());
        new_cost.set_mantissa(new_cost.mantissa() * 10.0_f64.powf(new_extra));
        new_cost.normalize();
        return cost.max(new_cost);
    }
    cost
}

/// Input to [`buy_multiplier`]. Mirrors `BuyMultiplierInput`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BuyMultiplierInput {
    /// True when the autobuyer is driving — bypasses the per-click cap.
    pub autobuyer: bool,
    /// Per-click purchase cap selected in the UI.
    pub coinbuyamount: BuyAmount,
    /// `G.costDivisor` at call time.
    pub cost_divisor: f64,
    /// `CalcECC('transcend', player.challengecompletions[4])`.
    pub transcend_ecc: f64,
    /// `player.currentChallenge.transcension === 4`
    pub in_transcension_challenge_4: bool,
    /// `player.currentChallenge.reincarnation === 8`
    pub in_reincarnation_challenge_8: bool,
}

impl BuyMultiplierInput {
    fn cost_input(self) -> GetCostMultiplierInput {
        GetCostMultiplierInput {
            cost_divisor: self.cost_divisor,
            transcend_ecc: self.transcend_ecc,
            in_transcension_challenge_4: self.in_transcension_challenge_4,
            in_reincarnation_challenge_8: self.in_reincarnation_challenge_8,
        }
    }
}

/// Buy as many multipliers as possible given the current coin balance and
/// per-click cap. Verbatim port of `buyMultiplier`. See the
/// [`crate::mechanics::accelerators::buy_accelerator`] docs for the two-path
/// algorithm — this is a parallel implementation with different field names.
#[must_use]
pub fn buy_multiplier(
    state: &mut MultiplierState,
    coins: &mut Decimal,
    input: BuyMultiplierInput,
) -> SmallVec<[CoreEvent; 4]> {
    let cost_input = input.cost_input();
    let mut events: SmallVec<[CoreEvent; 4]> = SmallVec::new();
    let starting_coins = *coins;
    let buy_start = state.multiplier_bought;

    // High-end binary search path.
    if buy_start >= BUYMAX {
        let diminishing_exponent = 1.0_f64 / 8.0;
        let log10_resource = coins.log10().to_number();
        let log10_quadrillion_cost = get_cost_multiplier(BUYMAX, cost_input).log10().to_number();

        let mut hi = (BUYMAX
            * (1.0_f64).max((log10_resource / log10_quadrillion_cost).powf(diminishing_exponent)))
        .floor();
        let mut lo = BUYMAX;
        while hi - lo > 0.5 {
            let mid = (lo + (hi - lo) / 2.0).floor();
            if mid == lo || mid == hi {
                break;
            }
            if *coins < get_cost_multiplier(mid, cost_input) {
                hi = mid;
            } else {
                lo = mid;
            }
        }
        let buyable = lo;
        state.multiplier_bought = buyable;
        state.multiplier_cost = get_cost_multiplier(buyable, cost_input);
        if state.multiplier_bought > 0.0 {
            state.prestige_no_multiplier = false;
            state.transcend_no_multiplier = false;
            state.reincarnate_no_multiplier = false;
        }
        if state.multiplier_bought > buy_start {
            events.push(CoreEvent::MultipliersPurchased {
                before: buy_start,
                after: state.multiplier_bought,
                spent: starting_coins - *coins,
            });
        }
        return events;
    }

    // Normal path: bracket with 4× doubling, refine with stepdown, walk forward.
    let buydefault = buy_start + smallest_inc(buy_start);
    let mut buy_to = buydefault;

    let mut cash_to_buy = get_cost_multiplier(buy_to, cost_input);
    while *coins >= cash_to_buy {
        buy_to *= 4.0;
        cash_to_buy = get_cost_multiplier(buy_to, cost_input);
    }
    let mut stepdown = (buy_to / 8.0).floor();
    while stepdown >= smallest_inc(buy_to) {
        if get_cost_multiplier(buy_to - stepdown, cost_input) <= *coins {
            stepdown = (stepdown / 2.0).floor();
        } else {
            buy_to -= smallest_inc(buy_to).max(stepdown);
        }
    }

    if !input.autobuyer {
        let cap_to = state.multiplier_bought + input.coinbuyamount.as_f64();
        if cap_to < buy_to {
            buy_to = cap_to;
        }
    }

    let mut buy_from = (buy_to - 6.0 - smallest_inc(buy_to)).max(buydefault);
    let mut this_cost = get_cost_multiplier(buy_from, cost_input);
    while buy_from <= buy_to && *coins >= this_cost {
        if buy_from >= BUYMAX {
            buy_from = BUYMAX;
        }
        *coins -= this_cost;
        state.multiplier_bought = buy_from;
        buy_from += smallest_inc(buy_from);
        this_cost = get_cost_multiplier(buy_from, cost_input);
        state.multiplier_cost = this_cost;
        if buy_from >= BUYMAX {
            break;
        }
    }

    if state.multiplier_bought > 0.0 {
        state.prestige_no_multiplier = false;
        state.transcend_no_multiplier = false;
        state.reincarnate_no_multiplier = false;
    }

    if state.multiplier_bought > buy_start {
        events.push(CoreEvent::MultipliersPurchased {
            before: buy_start,
            after: state.multiplier_bought,
            spent: starting_coins - *coins,
        });
    }

    events
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline() -> GetCostMultiplierInput {
        GetCostMultiplierInput {
            cost_divisor: 1.0,
            transcend_ecc: 0.0,
            in_transcension_challenge_4: false,
            in_reincarnation_challenge_8: false,
        }
    }

    // ─── get_cost_multiplier ──────────────────────────────────────────────

    #[test]
    fn first_multiplier_is_original_cost() {
        // buying_to = 1 → 0 → cost = 1e4 * 10^0 = 1e4
        let cost = get_cost_multiplier(1.0, baseline());
        assert!((cost.to_number() - 1e4).abs() < 1e-4);
    }

    #[test]
    fn second_multiplier_applies_first_factor() {
        // buying_to = 2 → 1 → cost = 1e4 * 10^1 = 1e5
        let cost = get_cost_multiplier(2.0, baseline());
        assert!((cost.to_number() - 1e5).abs() < 1.0);
    }

    #[test]
    fn cost_divisor_reduces_growth() {
        // cost_divisor = 2 → exponent halved → cost = 1e4 * 10^0.5
        let input = GetCostMultiplierInput {
            cost_divisor: 2.0,
            ..baseline()
        };
        let cost = get_cost_multiplier(2.0, input);
        let expected = 1e4 * 10.0_f64.sqrt();
        assert!((cost.to_number() - expected).abs() / expected < 1e-9);
    }

    #[test]
    fn cost_strictly_increases_below_threshold() {
        let inp = baseline();
        let a = get_cost_multiplier(5.0, inp);
        let b = get_cost_multiplier(6.0, inp);
        let c = get_cost_multiplier(50.0, inp);
        assert!(a < b);
        assert!(b < c);
    }

    #[test]
    fn factorial_branch_kicks_in_past_75() {
        // Threshold is 75 (vs accelerators' 125). At buying_to = 77 (l = 76)
        // the factorial term fires.
        let inp = baseline();
        let cost_76 = get_cost_multiplier(76.0, inp);
        let cost_77 = get_cost_multiplier(77.0, inp);
        assert!(cost_77 > cost_76);
    }

    #[test]
    fn transcend_challenge_4_multiplies_cost() {
        let challenged = GetCostMultiplierInput {
            in_transcension_challenge_4: true,
            ..baseline()
        };
        let a = get_cost_multiplier(3.0, baseline());
        let b = get_cost_multiplier(3.0, challenged);
        assert!(b > a);
    }

    #[test]
    fn reincarnation_challenge_8_multiplies_more_than_t4() {
        let in_t4 = GetCostMultiplierInput {
            in_transcension_challenge_4: true,
            ..baseline()
        };
        let in_r8 = GetCostMultiplierInput {
            in_reincarnation_challenge_8: true,
            ..baseline()
        };
        let t4_cost = get_cost_multiplier(3.0, in_t4);
        let r8_cost = get_cost_multiplier(3.0, in_r8);
        assert!(r8_cost > t4_cost);
    }

    #[test]
    fn transcend_ecc_pushes_factorial_threshold() {
        // With transcend_ecc = 1, transcend_break = 2 (vs accelerators' 5),
        // threshold becomes 77. At buying_to = 78 (l = 77), without ECC the
        // factorial branch fires; with ECC it doesn't.
        let no_ecc = baseline();
        let with_ecc = GetCostMultiplierInput {
            transcend_ecc: 1.0,
            ..baseline()
        };
        let cost_no_ecc = get_cost_multiplier(78.0, no_ecc);
        let cost_with_ecc = get_cost_multiplier(78.0, with_ecc);
        assert!(cost_no_ecc > cost_with_ecc);
    }

    // ─── buy_multiplier ───────────────────────────────────────────────────

    fn empty_state() -> MultiplierState {
        MultiplierState {
            multiplier_bought: 0.0,
            multiplier_cost: get_cost_multiplier(1.0, baseline()),
            prestige_no_multiplier: true,
            transcend_no_multiplier: true,
            reincarnate_no_multiplier: true,
        }
    }

    fn buy_input() -> BuyMultiplierInput {
        BuyMultiplierInput {
            autobuyer: false,
            coinbuyamount: BuyAmount::HundredThousand,
            cost_divisor: 1.0,
            transcend_ecc: 0.0,
            in_transcension_challenge_4: false,
            in_reincarnation_challenge_8: false,
        }
    }

    #[test]
    fn buy_is_noop_with_zero_coins() {
        let mut state = empty_state();
        let mut coins = Decimal::zero();
        let events = buy_multiplier(&mut state, &mut coins, buy_input());
        assert_eq!(state.multiplier_bought, 0.0);
        assert_eq!(coins, Decimal::zero());
        assert!(events.is_empty());
        assert!(state.prestige_no_multiplier);
        assert!(state.transcend_no_multiplier);
        assert!(state.reincarnate_no_multiplier);
    }

    #[test]
    fn buy_purchases_at_least_one_when_affordable() {
        // First multiplier costs 1e4 coins. Give the player 1e5.
        let mut state = empty_state();
        let mut coins = Decimal::from_finite(1e5);
        let baseline_coins = coins;
        let events = buy_multiplier(&mut state, &mut coins, buy_input());
        assert!(state.multiplier_bought > 0.0);
        assert!(coins < baseline_coins);
        assert_eq!(events.len(), 1);
        assert!(!state.prestige_no_multiplier);
        assert!(!state.transcend_no_multiplier);
        assert!(!state.reincarnate_no_multiplier);
    }

    #[test]
    fn buy_event_spent_matches_resource_delta() {
        let mut state = empty_state();
        let mut coins = Decimal::from_finite(1e8);
        let baseline_coins = coins;
        let events = buy_multiplier(&mut state, &mut coins, buy_input());
        let spent = baseline_coins - coins;
        assert_eq!(events.len(), 1);
        match &events[0] {
            CoreEvent::MultipliersPurchased {
                before,
                after,
                spent: ev_spent,
            } => {
                assert_eq!(*before, 0.0);
                assert_eq!(*after, state.multiplier_bought);
                assert_eq!(*ev_spent, spent);
            }
            other => panic!("expected MultipliersPurchased, got {other:?}"),
        }
    }

    #[test]
    fn per_click_cap_limits_purchases() {
        let mut state = empty_state();
        let mut coins = Decimal::from_finite(1e20);
        let capped = BuyMultiplierInput {
            coinbuyamount: BuyAmount::One,
            ..buy_input()
        };
        let _ = buy_multiplier(&mut state, &mut coins, capped);
        assert_eq!(state.multiplier_bought, 1.0);
    }

    #[test]
    fn autobuyer_ignores_per_click_cap() {
        let mut state = empty_state();
        let mut coins = Decimal::from_finite(1e20);
        let auto = BuyMultiplierInput {
            autobuyer: true,
            coinbuyamount: BuyAmount::One,
            ..buy_input()
        };
        let _ = buy_multiplier(&mut state, &mut coins, auto);
        assert!(state.multiplier_bought > 1.0);
    }
}
