//! Challenge-15 reward values — the numeric subset of the legacy
//! `c15RewardFormulae` (`Variables.ts`) gated by `c15RewardUpdate`
//! (`Statistics.ts`). Each reward is its `baseValue` (1) until
//! `player.challenge15Exponent` reaches the reward's `requirement`,
//! after which it follows the per-reward scaling formula of the
//! exponent. Below the requirement the reward is the multiplicative
//! identity (1), matching the bundles' previous forwarded defaults.
//!
//! Ported subset: the rewards consumed by the Phase-2 aggregator
//! `*Pre` bundles (`coinExponent`, `exponent`, `constantBonus`,
//! `accelerator`, `multiplier`). Other c15 rewards (`taxes`, `runeExp`,
//! `antSpeed`, `globalSpeed`, …) land with the chunks that consume them.

use crate::math::sigmoid::calculate_sigmoid;

/// `challenge15Rewards.coinExponent.value` — exponent applied to the
/// global coin multiplier. Requirement `3000`.
#[must_use]
pub fn coin_exponent(exponent: f64) -> f64 {
    if exponent >= 3_000.0 {
        1.0 + (1.0 / 150.0) * (exponent / 750.0).log2()
    } else {
        1.0
    }
}

/// `challenge15Rewards.accelerator.value`. Requirement `10000`.
#[must_use]
pub fn accelerator(exponent: f64) -> f64 {
    if exponent >= 10_000.0 {
        1.0 + (1.0 / 20.0) * (exponent / 2_500.0).log2()
    } else {
        1.0
    }
}

/// `challenge15Rewards.multiplier.value` — identical formula to
/// [`accelerator`]. Requirement `10000`.
#[must_use]
pub fn multiplier(exponent: f64) -> f64 {
    if exponent >= 10_000.0 {
        1.0 + (1.0 / 20.0) * (exponent / 2_500.0).log2()
    } else {
        1.0
    }
}

/// `challenge15Rewards.constantBonus.value` — multiplied into
/// `globalConstantMult`. Requirement `1e8`.
#[must_use]
pub fn constant_bonus(exponent: f64) -> f64 {
    if exponent >= 1e8 {
        1.0 + (1.0 / 5.0) * (exponent / 1e8).powf(2.0 / 3.0)
    } else {
        1.0
    }
}

/// `challenge15Rewards.exponent.value` — sigmoid bonus feeding
/// `constUpgrade2`. Requirement `2e16`.
#[must_use]
pub fn exponent_reward(exponent: f64) -> f64 {
    if exponent >= 2e16 {
        calculate_sigmoid(1.05, exponent, 1e18)
    } else {
        1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn below_requirement_is_identity() {
        assert_eq!(coin_exponent(0.0), 1.0);
        assert_eq!(accelerator(0.0), 1.0);
        assert_eq!(multiplier(0.0), 1.0);
        assert_eq!(constant_bonus(0.0), 1.0);
        assert_eq!(exponent_reward(0.0), 1.0);
    }

    #[test]
    fn coin_exponent_scales_above_requirement() {
        // e = 3000 → 1 + (1/150)*log2(3000/750) = 1 + (1/150)*log2(4) = 1 + 2/150
        assert!((coin_exponent(3_000.0) - (1.0 + 2.0 / 150.0)).abs() < 1e-12);
    }

    #[test]
    fn accelerator_and_multiplier_match_and_scale() {
        // e = 10000 → 1 + (1/20)*log2(4) = 1.1
        let v = accelerator(10_000.0);
        assert!((v - 1.1).abs() < 1e-12);
        assert_eq!(multiplier(10_000.0), v);
    }

    #[test]
    fn constant_bonus_scales_above_requirement() {
        // e = 1e8 → 1 + (1/5)*(1)^(2/3) = 1.2
        assert!((constant_bonus(1e8) - 1.2).abs() < 1e-12);
    }
}
