//! Per-tick prestige / transcend / reincarnation point-gain calculator.
//!
//! Verbatim port of `resetCurrency` from
//! `legacy/core_split/packages/logic/src/mechanics/resetCurrency.ts` (in
//! turn lifted from the legacy `packages/web_ui/src/Synergism.ts`).
//!
//! Three independent point formulas with shared challenge overrides:
//! - Transcension Challenge 5 forces `prestige_pow` to `0.01`.
//! - Reincarnation Challenge 10 forces `prestige_pow` to `1e-4` and
//!   `transcend_pow` to `0.001`.
//! - Ascension Challenge 12 zeroes `reincarnation_point_gain`.
//!
//! The caller pre-evaluates `CalcECC('transcend', completions[5])` as
//! `ecc5`, the deflation-table lookup as `deflation_multiplier`, the
//! particleGain achievement reward as `particle_gain_reward`, and passes
//! the `G.acceleratorEffect` `Decimal` through directly.

use synergismforkd_bignum::Decimal;

/// Inputs to [`reset_currency`]. Mirrors `ResetCurrencyInput`.
#[derive(Debug, Clone, Copy)]
pub struct ResetCurrencyInput {
    /// Pre-evaluated `CalcECC('transcend', player.challengecompletions[5])`
    /// — contributes `+ ecc5 / 100` to the base `prestige_pow` of `0.5`.
    pub ecc5: f64,
    /// `player.currentChallenge.transcension` — when `== 5`, forces
    /// `prestige_pow` to `0.01` and disables the upgrade-16 multiplier.
    /// `0` means no transcension challenge active.
    pub transcension_challenge: u32,
    /// `player.currentChallenge.reincarnation` — when `== 10`, forces
    /// `prestige_pow` to `1e-4`, `transcend_pow` to `0.001`, and disables
    /// both upgrade multipliers. When non-zero, also re-exponents
    /// `reincarnation_point_gain` by `0.01`. `0` means no reincarnation
    /// challenge active.
    pub reincarnation_challenge: u32,
    /// `player.currentChallenge.ascension` — when `== 12`, zeroes
    /// `reincarnation_point_gain` after all other calculations. `0` means
    /// no ascension challenge active.
    pub ascension_challenge: u32,
    /// Pre-evaluated
    /// `G.deflationMultiplier[player.corruptions.used.deflation]` —
    /// multiplies `prestige_pow`, and used as the upgrade-16
    /// `acceleratorEffect` exponent scaler
    /// (`1/3 * deflation_multiplier`).
    pub deflation_multiplier: f64,
    /// `player.coinsThisPrestige` — base for the prestige-point formula
    /// `floor((coinsThisPrestige / 1e12) ^ prestige_pow)`.
    pub coins_this_prestige: Decimal,
    /// `player.coinsThisTranscension` — base for the transcend-point
    /// formula `floor((coinsThisTranscension / 1e100) ^ transcend_pow)`.
    pub coins_this_transcension: Decimal,
    /// `player.transcendShards` — base for the reincarnation-point formula
    /// `floor((transcendShards / 1e300) ^ 0.01)`.
    pub transcend_shards: Decimal,
    /// `player.upgrades[16]` — when `> 0.5` and outside t-chal 5 / r-chal
    /// 10, multiplies `prestige_point_gain` by
    /// `min(10^1e33, acceleratorEffect ^ (deflation_multiplier / 3))`.
    pub upgrade_16: f64,
    /// `player.upgrades[44]` — when `> 0.5` and outside t-chal 5 / r-chal
    /// 10, multiplies `transcend_point_gain` by
    /// `min(1e6, 1.01 ^ transcend_count)`.
    pub upgrade_44: f64,
    /// `player.upgrades[65]` — when `> 0.5`, multiplies
    /// `reincarnation_point_gain` by `5`.
    pub upgrade_65: f64,
    /// `player.transcendCount` — exponent base for the upgrade-44
    /// multiplier.
    pub transcend_count: f64,
    /// `G.acceleratorEffect` — Decimal base for the upgrade-16 prestige
    /// multiplier.
    pub accelerator_effect: Decimal,
    /// Pre-evaluated `+getAchievementReward('particleGain')` — multiplied
    /// into `reincarnation_point_gain` unconditionally.
    pub particle_gain_reward: f64,
}

/// Result of [`reset_currency`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ResetCurrencyResult {
    /// Per-tick prestige-point gain (Diamonds prestige currency).
    pub prestige_point_gain: Decimal,
    /// Per-tick transcend-point gain (Mythos prestige currency).
    pub transcend_point_gain: Decimal,
    /// Per-tick reincarnation-point gain (Particles prestige currency).
    pub reincarnation_point_gain: Decimal,
}

/// Compute the three per-tick reset-currency point-gain values. Pure: no
/// mutation of the input; returns a fresh result the caller writes back
/// to `G.prestigePointGain` / `G.transcendPointGain` /
/// `G.reincarnationPointGain`.
#[must_use]
pub fn reset_currency(input: &ResetCurrencyInput) -> ResetCurrencyResult {
    let mut prestige_pow = 0.5 + input.ecc5 / 100.0;
    let mut transcend_pow = 0.03;

    if input.transcension_challenge == 5 {
        prestige_pow = 0.01;
    }
    if input.reincarnation_challenge == 10 {
        prestige_pow = 1e-4;
        transcend_pow = 0.001;
    }
    prestige_pow *= input.deflation_multiplier;

    let mut prestige_point_gain = (input.coins_this_prestige / Decimal::from_finite(1e12))
        .pow(Decimal::from_finite(prestige_pow))
        .floor();
    if input.upgrade_16 > 0.5
        && input.transcension_challenge != 5
        && input.reincarnation_challenge != 10
    {
        let cap = Decimal::from_finite(10.0).pow(Decimal::from_finite(1e33));
        let scaled = input
            .accelerator_effect
            .pow(Decimal::from_finite(input.deflation_multiplier / 3.0));
        prestige_point_gain *= cap.min(scaled);
    }

    let mut transcend_point_gain = (input.coins_this_transcension / Decimal::from_finite(1e100))
        .pow(Decimal::from_finite(transcend_pow))
        .floor();
    if input.upgrade_44 > 0.5
        && input.transcension_challenge != 5
        && input.reincarnation_challenge != 10
    {
        let cap = Decimal::from_finite(1e6);
        let scaled = Decimal::from_finite(1.01).pow(Decimal::from_finite(input.transcend_count));
        transcend_point_gain *= cap.min(scaled);
    }

    let mut reincarnation_point_gain = (input.transcend_shards / Decimal::from_finite(1e300))
        .pow(Decimal::from_finite(0.01))
        .floor();
    if input.reincarnation_challenge != 0 {
        reincarnation_point_gain = reincarnation_point_gain.pow(Decimal::from_finite(0.01));
    }
    reincarnation_point_gain *= Decimal::from_finite(input.particle_gain_reward);
    if input.upgrade_65 > 0.5 {
        reincarnation_point_gain *= Decimal::from_finite(5.0);
    }
    if input.ascension_challenge == 12 {
        reincarnation_point_gain = Decimal::zero();
    }

    ResetCurrencyResult {
        prestige_point_gain,
        transcend_point_gain,
        reincarnation_point_gain,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline() -> ResetCurrencyInput {
        ResetCurrencyInput {
            ecc5: 0.0,
            transcension_challenge: 0,
            reincarnation_challenge: 0,
            ascension_challenge: 0,
            deflation_multiplier: 1.0,
            coins_this_prestige: Decimal::zero(),
            coins_this_transcension: Decimal::zero(),
            transcend_shards: Decimal::zero(),
            upgrade_16: 0.0,
            upgrade_44: 0.0,
            upgrade_65: 0.0,
            transcend_count: 0.0,
            accelerator_effect: Decimal::one(),
            particle_gain_reward: 1.0,
        }
    }

    #[test]
    fn no_coins_means_no_gain() {
        let result = reset_currency(&baseline());
        assert_eq!(result.prestige_point_gain, Decimal::zero());
        assert_eq!(result.transcend_point_gain, Decimal::zero());
        assert_eq!(result.reincarnation_point_gain, Decimal::zero());
    }

    #[test]
    fn prestige_formula_matches_floor_of_pow() {
        // coins_this_prestige = 1e16, prestige_pow = 0.5 → (1e16/1e12)^0.5
        // = (1e4)^0.5 = 100. floor(100) = 100.
        let input = ResetCurrencyInput {
            coins_this_prestige: Decimal::from_finite(1e16),
            ..baseline()
        };
        let result = reset_currency(&input);
        assert!((result.prestige_point_gain.to_number() - 100.0).abs() < 1e-9);
    }

    #[test]
    fn ecc5_boosts_prestige_pow() {
        // ecc5 = 100 → prestige_pow = 0.5 + 1.0 = 1.5
        // coins = 1e14 → (1e14 / 1e12)^1.5 = 100^1.5 = 1000
        let input = ResetCurrencyInput {
            ecc5: 100.0,
            coins_this_prestige: Decimal::from_finite(1e14),
            ..baseline()
        };
        let result = reset_currency(&input);
        assert!((result.prestige_point_gain.to_number() - 1000.0).abs() < 1e-6);
    }

    #[test]
    fn transcension_challenge_5_forces_low_prestige_pow() {
        // prestige_pow = 0.01 → (1e16 / 1e12)^0.01 = 1e4^0.01 ≈ 1.0964
        // floor = 1
        let input = ResetCurrencyInput {
            transcension_challenge: 5,
            coins_this_prestige: Decimal::from_finite(1e16),
            ..baseline()
        };
        let result = reset_currency(&input);
        assert_eq!(result.prestige_point_gain.to_number(), 1.0);
    }

    #[test]
    fn reincarnation_challenge_10_forces_tiny_pows() {
        // Both prestige_pow and transcend_pow drop. With 1e16 coins,
        // prestige_pow = 1e-4 → (1e4)^1e-4 ≈ 1.00092 → floor 1.
        let input = ResetCurrencyInput {
            reincarnation_challenge: 10,
            coins_this_prestige: Decimal::from_finite(1e16),
            coins_this_transcension: Decimal::from_finite(1e104),
            ..baseline()
        };
        let result = reset_currency(&input);
        assert_eq!(result.prestige_point_gain.to_number(), 1.0);
        // transcend_pow = 0.001 → (1e4)^0.001 ≈ 1.00925 → floor 1.
        assert_eq!(result.transcend_point_gain.to_number(), 1.0);
    }

    #[test]
    fn deflation_multiplier_scales_prestige_pow() {
        // Doubling deflation_multiplier doubles prestige_pow.
        let baseline_input = ResetCurrencyInput {
            coins_this_prestige: Decimal::from_finite(1e16),
            ..baseline()
        };
        let doubled = ResetCurrencyInput {
            deflation_multiplier: 2.0,
            ..baseline_input
        };
        let baseline_result = reset_currency(&baseline_input);
        let doubled_result = reset_currency(&doubled);
        // baseline: pow = 0.5 → 100
        // doubled:  pow = 1.0 → 10_000
        assert!((baseline_result.prestige_point_gain.to_number() - 100.0).abs() < 1e-9);
        assert!((doubled_result.prestige_point_gain.to_number() - 10_000.0).abs() < 1e-6);
    }

    #[test]
    fn upgrade_16_multiplies_when_outside_disabling_challenges() {
        let outside = ResetCurrencyInput {
            upgrade_16: 1.0,
            accelerator_effect: Decimal::from_finite(1e10),
            coins_this_prestige: Decimal::from_finite(1e16),
            ..baseline()
        };
        // acceleratorEffect ^ (1/3) = 1e10^(1/3) ≈ 2154.43 (capped well below 10^1e33)
        // Base gain = 100; with upgrade: ≈ 215443.
        let result = reset_currency(&outside);
        assert!(result.prestige_point_gain.to_number() > 200_000.0);
        assert!(result.prestige_point_gain.to_number() < 300_000.0);

        // Same scenario but inside C5 — multiplier disabled.
        let inside_c5 = ResetCurrencyInput {
            transcension_challenge: 5,
            ..outside
        };
        let result_c5 = reset_currency(&inside_c5);
        // C5 forces prestige_pow to 0.01 → tiny gain, upgrade not applied.
        assert_eq!(result_c5.prestige_point_gain.to_number(), 1.0);
    }

    #[test]
    fn upgrade_44_caps_at_1e6_when_outside_disabling_challenges() {
        // transcend_count = 10000 → 1.01^10000 ≈ 1.6e43; capped to 1e6.
        let input = ResetCurrencyInput {
            upgrade_44: 1.0,
            transcend_count: 10_000.0,
            coins_this_transcension: Decimal::from_finite(1e104),
            ..baseline()
        };
        let result = reset_currency(&input);
        // Base = (1e4)^0.03 ≈ 1.318 → floor 1. With cap: 1 * 1e6 = 1e6.
        assert!((result.transcend_point_gain.to_number() - 1e6).abs() < 1.0);
    }

    #[test]
    fn upgrade_65_multiplies_reincarnation_by_five() {
        let with_65 = ResetCurrencyInput {
            upgrade_65: 1.0,
            transcend_shards: Decimal::from_mantissa_exponent(1.0, 310.0),
            particle_gain_reward: 1.0,
            ..baseline()
        };
        let without_65 = ResetCurrencyInput {
            upgrade_65: 0.0,
            ..with_65
        };
        let with_result = reset_currency(&with_65);
        let without_result = reset_currency(&without_65);
        let ratio = with_result.reincarnation_point_gain.to_number()
            / without_result.reincarnation_point_gain.to_number();
        assert!((ratio - 5.0).abs() < 1e-9);
    }

    #[test]
    fn ascension_12_zeroes_reincarnation_gain() {
        let input = ResetCurrencyInput {
            ascension_challenge: 12,
            transcend_shards: Decimal::from_mantissa_exponent(1.0, 310.0),
            upgrade_65: 1.0,
            particle_gain_reward: 1e10,
            ..baseline()
        };
        let result = reset_currency(&input);
        assert_eq!(result.reincarnation_point_gain, Decimal::zero());
    }

    #[test]
    fn reincarnation_challenge_active_re_exponents_gain() {
        // With reincarnation_challenge != 0, the base gain is re-exponented
        // by 0.01 before particle_gain_reward / upgrade_65 amplifiers.
        // Use 1e500 so the base gain is 100 (floor((1e500/1e300)^0.01) =
        // floor(100) = 100) — that way the 0.01-re-exponent has bite.
        let plain = ResetCurrencyInput {
            transcend_shards: Decimal::from_mantissa_exponent(1.0, 500.0),
            ..baseline()
        };
        let in_challenge = ResetCurrencyInput {
            reincarnation_challenge: 1,
            ..plain
        };
        let plain_result = reset_currency(&plain);
        let challenge_result = reset_currency(&in_challenge);
        // plain: floor((1e500/1e300)^0.01) = floor(100) = 100
        // challenge: 100^0.01 ≈ 1.047 → much smaller.
        assert!((plain_result.reincarnation_point_gain.to_number() - 100.0).abs() < 1e-9);
        assert!(challenge_result.reincarnation_point_gain < plain_result.reincarnation_point_gain);
    }

    #[test]
    fn particle_gain_reward_multiplies_reincarnation() {
        let with_reward = ResetCurrencyInput {
            transcend_shards: Decimal::from_mantissa_exponent(1.0, 310.0),
            particle_gain_reward: 7.0,
            ..baseline()
        };
        let without_reward = ResetCurrencyInput {
            particle_gain_reward: 1.0,
            ..with_reward
        };
        let with_result = reset_currency(&with_reward);
        let without_result = reset_currency(&without_reward);
        let ratio = with_result.reincarnation_point_gain.to_number()
            / without_result.reincarnation_point_gain.to_number();
        assert!((ratio - 7.0).abs() < 1e-9);
    }
}
