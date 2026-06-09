//! Accelerator-boost cost formula.
//!
//! Verbatim port of `getAcceleratorBoostCost` from
//! `legacy/core_split/packages/logic/src/mechanics/acceleratorBoosts.ts`.
//!
//! Accelerator boosts are a separate ladder bought with `prestigePoints`
//! once `player.upgrades[46]` is owned (vs. the base-game accelerators bought
//! with coins). The cost climbs aggressively — 10^10 per level plus a
//! triangle-number kicker, and beyond `1000 * accel_boost_cost_delay` levels
//! the kicker grows quadratically.
//!
//! The accompanying buy loop still lives with the broader reset system in
//! the legacy `web_ui/Buy.ts` because it calls `reset('prestige')` inline;
//! that part migrates with the reset-system overhaul.

use smallvec::SmallVec;
use synergismforkd_bignum::Decimal;

use crate::events::CoreEvent;
use crate::math::smallest_inc::smallest_inc;
use crate::state::AcceleratorState;

const BUYMAX: f64 = 1e15;

/// Triangle number — closed form for 1 + 2 + … + n.
fn lin_sum(n: f64) -> f64 {
    n * (n + 1.0) / 2.0
}

/// Square-pyramidal number — closed form for 1² + 2² + … + n².
fn sqr_sum(n: f64) -> f64 {
    n * (n + 1.0) * (2.0 * n + 1.0) / 6.0
}

/// Input to [`get_accelerator_boost_cost`]. Mirrors the TS struct.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GetAcceleratorBoostCostInput {
    /// Cost-delay multiplier from the thrift rune blessing
    /// (`getRuneBlessingEffect('thrift').accelBoostCostDelay` in `web_ui`).
    /// Pushes back the level at which the quadratic-in-level growth kicks
    /// in: the threshold is `1000 * accel_boost_cost_delay`.
    pub accel_boost_cost_delay: f64,
}

/// Cost in `prestigePoints` of buying the `level`-th accelerator boost
/// (1-indexed; the formula internally decrements to 0-index).
pub fn get_accelerator_boost_cost(level: f64, input: GetAcceleratorBoostCostInput) -> Decimal {
    // Formula is 0-indexed; callers pass 1-indexed level.
    let level = level - 1.0;
    let base = Decimal::from_finite(1e3);
    let eff = input.accel_boost_cost_delay;

    let exp = if level > 1000.0 * eff {
        10.0 * level + lin_sum(level) + sqr_sum(level - 1000.0 * eff) / eff
    } else {
        10.0 * level + lin_sum(level)
    };
    let cost = base * Decimal::from_finite(10.0).pow(Decimal::from_finite(exp));

    if level > BUYMAX {
        let diminishing_exponent = 1.0 / 8.0;
        // Recurse with `BUYMAX + 1.0` since the TS function decrements before
        // checking, so passing `BUYMAX` makes the recursive call's `level`
        // equal to `BUYMAX - 1`, which is below the threshold.
        let quadrillion_cost = get_accelerator_boost_cost(BUYMAX + 1.0, input);
        let mut new_cost = quadrillion_cost.pow(Decimal::from_finite(
            (level / BUYMAX).powf(1.0 / diminishing_exponent),
        ));
        // Re-normalize after the in-place mantissa/exponent rewrite. Matches
        // the legacy break_infinity.js massaging — the goal is to push any
        // fractional-exponent value back into the mantissa so subsequent
        // arithmetic stays well-behaved.
        let new_extra = new_cost.exponent() - new_cost.exponent().floor();
        new_cost.set_exponent(new_cost.exponent().floor());
        new_cost.set_mantissa(new_cost.mantissa() * 10.0_f64.powf(new_extra));
        new_cost.normalize();
        return cost.max(new_cost);
    }
    cost
}

/// Input to [`buy_accelerator_boost_bulk`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BuyAcceleratorBoostInput {
    /// `getRuneBlessingEffect('thrift').accelBoostCostDelay`, captured once per
    /// buy (the thrift rune blessing pushes back the quadratic-cost threshold).
    pub accel_boost_cost_delay: f64,
}

/// Bulk accelerator-boost purchase — the `player.upgrades[46] >= 1` path of the
/// legacy `boostAccelerator` (`Buy.ts:386-457`). Spends `prestige_points`, with
/// no per-click cap and no prestige reset (that is the pre-upgrade path, which
/// stays in [`crate::tick`] because it calls `reset('prestige')`). Mirrors the
/// structure of [`buy_accelerator`](super::accelerators::buy_accelerator): a
/// high-end binary search past `BUYMAX`, otherwise a 4× bracket + stepdown
/// refine + forward walk. Sets the transcend / reincarnate no-accelerator flags
/// (a boost does **not** clear the prestige flag).
#[must_use]
pub fn buy_accelerator_boost_bulk(
    state: &mut AcceleratorState,
    prestige_points: &mut Decimal,
    input: BuyAcceleratorBoostInput,
) -> SmallVec<[CoreEvent; 4]> {
    let cost_input = GetAcceleratorBoostCostInput {
        accel_boost_cost_delay: input.accel_boost_cost_delay,
    };
    let mut events: SmallVec<[CoreEvent; 4]> = SmallVec::new();
    let starting_points = *prestige_points;
    let buy_start = state.accelerator_boost_bought;

    // High-end binary-search path (buyStart >= 1e15): snap to the largest
    // affordable count; the diminishing cost makes per-step accounting moot
    // (no points are subtracted here, matching the legacy source).
    if buy_start >= BUYMAX {
        let diminishing_exponent = 1.0_f64 / 8.0;
        let log10_resource = prestige_points.log10().to_number();
        let log10_quadrillion_cost = get_accelerator_boost_cost(BUYMAX, cost_input)
            .log10()
            .to_number();

        let mut hi = (BUYMAX
            * (1.0_f64).max((log10_resource / log10_quadrillion_cost).powf(diminishing_exponent)))
        .floor();
        let mut lo = BUYMAX;
        for _ in 0..128 {
            if hi - lo <= 0.5 {
                break;
            }
            let mid = (lo + (hi - lo) / 2.0).floor();
            if mid == lo || mid == hi {
                break;
            }
            if *prestige_points < get_accelerator_boost_cost(mid, cost_input) {
                hi = mid;
            } else {
                lo = mid;
            }
        }
        let buyable = lo;
        state.accelerator_boost_bought = buyable;
        state.accelerator_boost_cost = get_accelerator_boost_cost(buyable, cost_input);
        if state.accelerator_boost_bought > buy_start {
            events.push(CoreEvent::AcceleratorBoostsPurchased {
                before: buy_start,
                after: state.accelerator_boost_bought,
                spent: starting_points - *prestige_points,
            });
        }
        return events;
    }

    // Normal path: 4× bracket on buyInc, stepdown refine, then forward walk.
    let buydefault = buy_start + smallest_inc(buy_start);
    let mut buy_inc = 1.0_f64;
    let mut cost = get_accelerator_boost_cost(buy_start + buy_inc, cost_input);
    while *prestige_points >= cost {
        buy_inc *= 4.0;
        cost = get_accelerator_boost_cost(buy_start + buy_inc, cost_input);
    }
    let mut stepdown = (buy_inc / 8.0).floor();
    while stepdown >= smallest_inc(buy_inc) {
        if get_accelerator_boost_cost(buy_start + buy_inc - stepdown, cost_input)
            <= *prestige_points
        {
            stepdown = (stepdown / 2.0).floor();
        } else {
            buy_inc -= smallest_inc(buy_inc).max(stepdown);
        }
    }

    // Walk forward from ~6 below the bracket, paying `this_cost` (which starts
    // at the cost of the current level — the legacy first-step undercharge).
    let mut buy_from = (buy_start + buy_inc - 6.0 - smallest_inc(buy_inc)).max(buydefault);
    let mut this_cost = get_accelerator_boost_cost(buy_start, cost_input);
    while buy_from <= buy_start + buy_inc
        && *prestige_points >= get_accelerator_boost_cost(buy_from, cost_input)
    {
        *prestige_points -= this_cost;
        if buy_from >= BUYMAX {
            buy_from = BUYMAX;
        }
        state.accelerator_boost_bought = buy_from;
        buy_from += smallest_inc(buy_from);
        this_cost = get_accelerator_boost_cost(buy_from, cost_input);
        state.accelerator_boost_cost = this_cost;
        state.transcend_no_accelerator = false;
        state.reincarnate_no_accelerator = false;
        if buy_from >= BUYMAX {
            break;
        }
    }

    if state.accelerator_boost_bought > buy_start {
        events.push(CoreEvent::AcceleratorBoostsPurchased {
            before: buy_start,
            after: state.accelerator_boost_bought,
            spent: starting_points - *prestige_points,
        });
    }
    events
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(delay: f64) -> GetAcceleratorBoostCostInput {
        GetAcceleratorBoostCostInput {
            accel_boost_cost_delay: delay,
        }
    }

    #[test]
    fn level_one_returns_base_cost() {
        // level = 1 → decremented to 0 → 10^0 * 1e3 = 1e3
        let cost = get_accelerator_boost_cost(1.0, input(1.0));
        assert!((cost.to_number() - 1e3).abs() < 1e-6);
    }

    #[test]
    fn level_two_applies_first_exponent() {
        // level = 2 → decremented to 1 → exp = 10*1 + lin_sum(1) = 10 + 1 = 11
        // cost = 1e3 * 10^11 = 1e14
        let cost = get_accelerator_boost_cost(2.0, input(1.0));
        assert!((cost.to_number() - 1e14).abs() < 1e8);
    }

    #[test]
    fn cost_strictly_increases() {
        let inp = input(1.0);
        let a = get_accelerator_boost_cost(5.0, inp);
        let b = get_accelerator_boost_cost(6.0, inp);
        let c = get_accelerator_boost_cost(10.0, inp);
        assert!(a < b);
        assert!(b < c);
    }

    #[test]
    fn threshold_branch_kicks_in_past_1000_times_delay() {
        // With delay = 1.0, threshold is 1000. Compare level 1002 vs the
        // closed-form value computed without the sqr_sum term.
        let inp = input(1.0);
        let cost_at_1002 = get_accelerator_boost_cost(1002.0, inp);
        // Level 1002 → l = 1001 > 1000, so the sqr_sum branch fires. We
        // can't easily produce a closed-form check, but the cost must exceed
        // the no-sqr_sum branch's contribution at the same level. The
        // no-sqr_sum cost at l = 1001 is 1e3 * 10^(10*1001 + lin_sum(1001))
        // = 1e3 * 10^(10010 + 501501) ≈ 1e514. Anything beyond that means
        // the sqr_sum kicked in.
        let cost_at_1001 = get_accelerator_boost_cost(1001.0, inp);
        assert!(cost_at_1002 > cost_at_1001);
    }

    #[test]
    fn larger_cost_delay_pushes_threshold_higher() {
        // At level 1500: delay = 1.0 has crossed threshold (1000), delay = 2.0
        // has not (threshold = 2000). The pre-threshold cost should be less.
        let cost_d1 = get_accelerator_boost_cost(1500.0, input(1.0));
        let cost_d2 = get_accelerator_boost_cost(1500.0, input(2.0));
        assert!(cost_d2 < cost_d1);
    }

    // ─── buy_accelerator_boost_bulk ───────────────────────────────────────

    fn bulk_input() -> BuyAcceleratorBoostInput {
        BuyAcceleratorBoostInput {
            accel_boost_cost_delay: 1.0,
        }
    }

    #[test]
    fn bulk_buy_is_noop_when_broke() {
        let mut state = AcceleratorState::default();
        let mut points = Decimal::zero();
        let events = buy_accelerator_boost_bulk(&mut state, &mut points, bulk_input());
        assert_eq!(state.accelerator_boost_bought, 0.0);
        assert!(events.is_empty());
        assert!(state.transcend_no_accelerator); // flag untouched when nothing bought
    }

    #[test]
    fn bulk_buy_spends_prestige_points_and_buys() {
        let mut state = AcceleratorState::default();
        let mut points = Decimal::from_finite(1e30);
        let events = buy_accelerator_boost_bulk(&mut state, &mut points, bulk_input());
        assert!(state.accelerator_boost_bought > 0.0);
        assert!(points < Decimal::from_finite(1e30)); // spent something
        assert_eq!(events.len(), 1);
        // The boost clears the transcend / reincarnate flags (not prestige).
        assert!(!state.transcend_no_accelerator);
        assert!(!state.reincarnate_no_accelerator);
        assert!(state.prestige_no_accelerator);
    }
}
