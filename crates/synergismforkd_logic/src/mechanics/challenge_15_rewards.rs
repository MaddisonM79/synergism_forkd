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
//! `accelerator`, `multiplier`), the tax-phase reward (`taxes`), the
//! speed rewards (`globalSpeed`, `ascensionSpeed`) that feed the
//! global / ascension speed StatLine products, the ascension-score
//! reward (`score`), the obtainium reward (`obtainium`) feeding the
//! obtainium DR-ignore StatLine product, and the `antSpeed` reward
//! feeding the ant-speed StatLine product. Other c15 rewards
//! (`runeExp`, `blessingBonus`, …) land with the chunks that consume
//! them.

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

/// `challenge15Rewards.taxes.value` — multiplies the coin-production tax
/// exponent (so a value `< 1` *reduces* tax). Requirement `5000`,
/// `baseValue` `1`. Legacy formula `0.98 ^ log2(e / 1250)`.
#[must_use]
pub fn taxes(exponent: f64) -> f64 {
    if exponent >= 5_000.0 {
        0.98_f64.powf((exponent / 1_250.0).log2())
    } else {
        1.0
    }
}

/// `challenge15Rewards.obtainium.value` — multiplied into the obtainium
/// DR-ignore (`allObtainiumIgnoreDRStats`) StatLine product. Requirement
/// `7500`, `baseValue` `1`. Legacy formula `1 + (1/4)·(e / 7.5e3)^0.6`.
#[must_use]
pub fn obtainium(exponent: f64) -> f64 {
    if exponent >= 7_500.0 {
        1.0 + (1.0 / 4.0) * (exponent / 7.5e3).powf(0.6)
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

/// `challenge15Rewards.globalSpeed.value` — multiplied into the global
/// speed StatLine product. Requirement `1e7`. Legacy formula
/// `1 + (1/20) * log2(e / 2.5e6)`.
#[must_use]
pub fn global_speed(exponent: f64) -> f64 {
    if exponent >= 1e7 {
        1.0 + (1.0 / 20.0) * (exponent / 2.5e6).log2()
    } else {
        1.0
    }
}

/// `challenge15Rewards.ascensionSpeed.value` — multiplied into the
/// ascension speed StatLine product. Requirement `1.5e18`. Legacy
/// formula `1 + 5/100 + 2 * log2(e / 1.5e18) / 100`.
#[must_use]
pub fn ascension_speed(exponent: f64) -> f64 {
    if exponent >= 1.5e18 {
        1.0 + 5.0 / 100.0 + 2.0 * (exponent / 1.5e18).log2() / 100.0
    } else {
        1.0
    }
}

/// `challenge15Rewards.score.value` — multiplied into the ascension-score
/// bonus multiplier (`compute_ascension_score_bonus_multiplier`, ultimately
/// the octeract cube StatLine via `calculateAscensionScore`). Requirement
/// `1e10`. Legacy two-branch formula: at/above `1e20`,
/// `1 + (1/4)·(e/1e10)^(1/8)·(1e10)^(1/8)`; between `1e10` and `1e20`,
/// `1 + (1/4)·(e/1e10)^(1/4)`.
#[must_use]
pub fn score(exponent: f64) -> f64 {
    if exponent < 1e10 {
        return 1.0;
    }
    if exponent >= 1e20 {
        1.0 + (1.0 / 4.0) * (exponent / 1e10).powf(1.0 / 8.0) * 1e10_f64.powf(1.0 / 8.0)
    } else {
        1.0 + (1.0 / 4.0) * (exponent / 1e10).powf(1.0 / 4.0)
    }
}

/// `challenge15Rewards.antSpeed.value` — multiplied into the ant-speed
/// StatLine product (`antSpeedStats`). Requirement `2e5`, `baseValue` `1`.
/// Legacy formula `(1 + log2(e / 2e5))^4`.
#[must_use]
pub fn ant_speed(exponent: f64) -> f64 {
    if exponent >= 2e5 {
        (1.0 + (exponent / 2e5).log2()).powi(4)
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
        assert_eq!(taxes(0.0), 1.0);
        assert_eq!(obtainium(0.0), 1.0);
        assert_eq!(accelerator(0.0), 1.0);
        assert_eq!(multiplier(0.0), 1.0);
        assert_eq!(constant_bonus(0.0), 1.0);
        assert_eq!(exponent_reward(0.0), 1.0);
        assert_eq!(global_speed(0.0), 1.0);
        assert_eq!(ascension_speed(0.0), 1.0);
        assert_eq!(score(0.0), 1.0);
        assert_eq!(ant_speed(0.0), 1.0);
        // Just under the taxes requirement → still identity.
        assert_eq!(taxes(4_999.0), 1.0);
        // Just under the obtainium requirement → still identity.
        assert_eq!(obtainium(7_499.0), 1.0);
        // Just under the speed requirements → still identity.
        assert_eq!(global_speed(9.9e6), 1.0);
        assert_eq!(ascension_speed(1.4e18), 1.0);
        // Just under the score requirement → still identity.
        assert_eq!(score(9.9e9), 1.0);
        // Just under the ant-speed requirement → still identity.
        assert_eq!(ant_speed(1.99e5), 1.0);
    }

    #[test]
    fn ant_speed_scales_above_requirement() {
        // e = 2e5 → (1 + log2(1))^4 = 1.
        assert!((ant_speed(2e5) - 1.0).abs() < 1e-12);
        // e = 4e5 → (1 + log2(2))^4 = 2^4 = 16.
        assert!((ant_speed(4e5) - 16.0).abs() < 1e-9);
    }

    #[test]
    fn score_scales_above_requirement_with_branch_at_1e20() {
        // At exactly 1e10: 1 + (1/4)·1^(1/4) = 1.25.
        assert!((score(1e10) - 1.25).abs() < 1e-12);
        // Monotonic increasing in the 1/4 branch.
        assert!(score(1e15) > score(1e10));
        // The 1/8 branch (≥ 1e20) is continuous with the 1/4 branch at 1e20:
        // (e/1e10)^(1/4) = (e/1e10)^(1/8)·(1e10)^(1/8) when e = 1e20.
        let lo = 1.0 + (1.0 / 4.0) * (1e20_f64 / 1e10).powf(1.0 / 4.0);
        assert!((score(1e20) - lo).abs() < 1e-6);
    }

    #[test]
    fn obtainium_scales_above_requirement() {
        // e = 7500 → 1 + (1/4)·(7500/7500)^0.6 = 1.25.
        assert!((obtainium(7_500.0) - 1.25).abs() < 1e-12);
        // Monotonic increasing above the requirement.
        assert!(obtainium(7.5e4) > obtainium(7_500.0));
    }

    #[test]
    fn taxes_scales_below_one_above_requirement() {
        // e = 5000 → 0.98 ^ log2(5000/1250) = 0.98 ^ log2(4) = 0.98^2.
        let expected = 0.98_f64.powi(2);
        assert!((taxes(5_000.0) - expected).abs() < 1e-12);
        // Reward only ever reduces the exponent above the requirement.
        assert!(taxes(5_000.0) < 1.0);
        assert!(taxes(20_000.0) < taxes(5_000.0));
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

    #[test]
    fn speed_rewards_scale_above_requirement() {
        // global: e = 1e7 → 1 + (1/20)*log2(1e7/2.5e6) = 1 + (1/20)*log2(4) = 1.1
        assert!((global_speed(1e7) - 1.1).abs() < 1e-12);
        // ascension: e = 1.5e18 → 1.05 + 0.02*log2(1) = 1.05
        assert!((ascension_speed(1.5e18) - 1.05).abs() < 1e-12);
        // ascension: e = 3e18 → 1.05 + 0.02*log2(2) = 1.07
        assert!((ascension_speed(3e18) - 1.07).abs() < 1e-12);
    }
}
