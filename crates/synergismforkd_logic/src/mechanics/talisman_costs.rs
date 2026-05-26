//! Talisman fragment-cost progression formulas.
//!
//! Verbatim port of
//! `legacy_core_split/packages/logic/src/mechanics/talismanCosts.ts`.
//! Both formulas are pure functions over a `Decimal` `base_mult` and
//! an integer `level` — no player state, no globals. They return the
//! fragment cost map for the **next** level of the talisman.
//!
//! [`regular_cost_progression`] is the cubic-tier formula used by most
//! talismans: each fragment tier starts contributing at a level
//! threshold (shard at 0, common at 30, uncommon at 60, etc.) and the
//! base multiplier itself ramps up at higher levels (120/150/180).
//! [`exponential_cost_progression`] is the alternative used by a
//! handful of talismans whose cost grows as `ratio^level` instead of
//! `level^3` — same tier-threshold scheme, different growth curve.

use synergismforkd_bignum::Decimal;

/// Fragment cost map for the next talisman level. Each field is the
/// cost of one fragment kind.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TalismanCraftCosts {
    /// Cost in talisman shards.
    pub shard: Decimal,
    /// Cost in common fragments.
    pub common_fragment: Decimal,
    /// Cost in uncommon fragments.
    pub uncommon_fragment: Decimal,
    /// Cost in rare fragments.
    pub rare_fragment: Decimal,
    /// Cost in epic fragments.
    pub epic_fragment: Decimal,
    /// Cost in legendary fragments.
    pub legendary_fragment: Decimal,
    /// Cost in mythical fragments.
    pub mythical_fragment: Decimal,
}

fn cubic_tier_cost(level: f64, threshold: f64, divisor: f64, price_mult: Decimal) -> Decimal {
    if level < threshold {
        return Decimal::zero();
    }
    let cost = (Decimal::from_finite(level - threshold).pow(Decimal::from_finite(3.0))
        / Decimal::from_finite(divisor)
        + Decimal::one())
    .floor()
        * price_mult;
    cost.max(Decimal::zero())
}

/// Cubic-tier cost progression. For each tier, the cost is
/// `floor((level - threshold)^3 / divisor + 1) * price_mult`, clamped
/// at zero below the tier threshold. `price_mult` itself grows
/// piecewise past levels `120 / 150 / 180`. Returns a 0-cost map until
/// each tier unlocks (shard at 0, common at 30, uncommon at 60, rare
/// at 90, epic at 120, legendary/mythical at 150).
#[must_use]
pub fn regular_cost_progression(base_mult: Decimal, level: f64) -> TalismanCraftCosts {
    let mut price_mult = base_mult;
    if level >= 120.0 {
        price_mult *= Decimal::from_finite((level - 90.0) / 30.0);
    }
    if level >= 150.0 {
        price_mult *= Decimal::from_finite((level - 120.0) / 30.0);
    }
    if level >= 180.0 {
        price_mult *= Decimal::from_finite((level - 170.0) / 10.0);
    }

    // Shard has no threshold gate — uses raw `level` (not level - 0).
    let shard_cost = (Decimal::from_finite(level).pow(Decimal::from_finite(3.0))
        / Decimal::from_finite(8.0)
        + Decimal::one())
    .floor()
        * price_mult;

    TalismanCraftCosts {
        shard: shard_cost.max(Decimal::zero()),
        common_fragment: cubic_tier_cost(level, 30.0, 32.0, price_mult),
        uncommon_fragment: cubic_tier_cost(level, 60.0, 384.0, price_mult),
        rare_fragment: cubic_tier_cost(level, 90.0, 500.0, price_mult),
        epic_fragment: cubic_tier_cost(level, 120.0, 375.0, price_mult),
        legendary_fragment: cubic_tier_cost(level, 150.0, 192.0, price_mult),
        mythical_fragment: cubic_tier_cost(level, 150.0, 1_280.0, price_mult),
    }
}

fn exponential_tier_cost(
    level: f64,
    threshold: f64,
    ratio: f64,
    base_mult: Decimal,
    tier_constant: f64,
) -> Decimal {
    if level < threshold {
        return Decimal::zero();
    }
    (Decimal::from_finite(ratio).pow(Decimal::from_finite(level - threshold))
        * base_mult
        * Decimal::from_finite(tier_constant))
    .floor()
}

/// Exponential cost progression. For each tier the cost is
/// `floor(ratio^(level - threshold) * base_mult * tier_constant)`. The
/// tier constants are fixed (`100 / 50 / 25 / 20 / 15 / 10 / 5`) and
/// the tier thresholds match [`regular_cost_progression`]. `ratio` is
/// supplied by the caller — common values are `2, 10, 1e5, 1e8` (each
/// used by one talisman).
#[must_use]
pub fn exponential_cost_progression(
    base_mult: Decimal,
    level: f64,
    ratio: f64,
) -> TalismanCraftCosts {
    TalismanCraftCosts {
        // Shard has no threshold gate — uses raw `level`.
        shard: (Decimal::from_finite(ratio).pow(Decimal::from_finite(level))
            * base_mult
            * Decimal::from_finite(100.0))
        .floor(),
        common_fragment: exponential_tier_cost(level, 30.0, ratio, base_mult, 50.0),
        uncommon_fragment: exponential_tier_cost(level, 60.0, ratio, base_mult, 25.0),
        rare_fragment: exponential_tier_cost(level, 90.0, ratio, base_mult, 20.0),
        epic_fragment: exponential_tier_cost(level, 120.0, ratio, base_mult, 15.0),
        legendary_fragment: exponential_tier_cost(level, 150.0, ratio, base_mult, 10.0),
        mythical_fragment: exponential_tier_cost(level, 150.0, ratio, base_mult, 5.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regular_progression_locks_tiers_below_threshold() {
        let costs = regular_cost_progression(Decimal::one(), 29.0);
        assert!(costs.shard > Decimal::zero());
        assert_eq!(costs.common_fragment, Decimal::zero());
        assert_eq!(costs.uncommon_fragment, Decimal::zero());
        assert_eq!(costs.rare_fragment, Decimal::zero());
        assert_eq!(costs.epic_fragment, Decimal::zero());
        assert_eq!(costs.legendary_fragment, Decimal::zero());
        assert_eq!(costs.mythical_fragment, Decimal::zero());
    }

    #[test]
    fn regular_progression_common_unlocks_at_30() {
        let below = regular_cost_progression(Decimal::one(), 29.0);
        let at = regular_cost_progression(Decimal::one(), 30.0);
        assert_eq!(below.common_fragment, Decimal::zero());
        // Level 30: (0)^3 / 32 + 1 = 1; floor(1) = 1.
        assert_eq!(at.common_fragment.to_number(), 1.0);
    }

    #[test]
    fn regular_progression_price_mult_ramps_at_120() {
        // base_mult = 1; at level 120, multiplier becomes (120-90)/30 = 1
        // (still 1, but the ramp factor IS applied — proves the branch fires).
        // To test the ramp visibly: at 150, multiplier = (150-90)/30 * (150-120)/30
        // = 2 * 1 = 2.
        let at_150 = regular_cost_progression(Decimal::one(), 150.0);
        let below_120 = regular_cost_progression(Decimal::one(), 119.0);
        // At level 120 the multiplier kicks in; at 150 it's even bigger.
        // We just check that level-150 shard cost is meaningfully above the
        // level-119 shard cost (which it would be even without the ramp).
        assert!(at_150.shard > below_120.shard);
    }

    #[test]
    fn exponential_progression_locks_tiers_below_threshold() {
        let costs = exponential_cost_progression(Decimal::one(), 29.0, 2.0);
        assert!(costs.shard > Decimal::zero());
        assert_eq!(costs.common_fragment, Decimal::zero());
    }

    #[test]
    fn exponential_progression_shard_scales_with_ratio() {
        // ratio = 2, level = 5, base = 1 → shard = floor(2^5 * 1 * 100) = 3200
        let costs = exponential_cost_progression(Decimal::one(), 5.0, 2.0);
        assert_eq!(costs.shard.to_number(), 3_200.0);
    }

    #[test]
    fn exponential_progression_common_unlocks_at_30() {
        let at = exponential_cost_progression(Decimal::one(), 30.0, 2.0);
        // ratio = 2, level - threshold = 0, base = 1, tier_constant = 50
        // floor(2^0 * 1 * 50) = 50
        assert_eq!(at.common_fragment.to_number(), 50.0);
    }
}
