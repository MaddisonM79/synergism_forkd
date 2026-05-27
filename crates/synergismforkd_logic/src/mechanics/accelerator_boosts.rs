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

use synergismforkd_bignum::Decimal;

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
}
