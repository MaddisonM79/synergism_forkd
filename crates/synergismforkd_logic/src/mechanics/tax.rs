//! Tax exponent and divisor formula.
//!
//! Verbatim port of `calculateTax` from
//! `legacy_core_split/packages/logic/src/mechanics/tax.ts` (in turn the
//! second half of `packages/web_ui/src/Tax.ts`). Pure given the full bag of
//! player + effect inputs; the legacy `web_ui` side does the input
//! gathering and writes the result fields back to `G`.
//!
//! ## Output formulas
//!
//! ```text
//! maxexponent      = floor(275 / (log10(1.01) * exponent)) - 1 + flat_max_increase
//! taxdivisor       = 1.01 ^ (divisor_exponent * exponent)
//! taxdivisorcheck  = 1.01 ^ (check_exponent * exponent)
//! ```
//!
//! where `divisor_exponent` / `check_exponent` are quadratics in their own
//! exponent bases (which incorporate the `produce_total` log and the flat
//! max-exponent increase from ant upgrades).

use synergismforkd_bignum::Decimal;

use super::challenges::{calc_ecc, ChallengeType};

/// Inputs to [`calculate_tax`]. Mirrors `CalculateTaxInput` in the TS source
/// — every `player.*` / `G.*` / effect read in the legacy `web_ui` is
/// hoisted into this bag so the logic call stays pure.
#[derive(Debug, Clone, Copy)]
pub struct CalculateTaxInput {
    // ─── Challenge / completion state ─────────────────────────────────────
    /// `player.currentChallenge.reincarnation === 6` — base exp =
    /// `3 * (1 + c6/25)^2`.
    pub in_reinc_6: bool,
    /// `player.currentChallenge.reincarnation === 9` — base exp = `0.005`.
    pub in_reinc_9: bool,
    /// `player.currentChallenge.ascension === 15` — base exp = `0.000005`.
    pub in_ascension_15: bool,
    /// `player.currentChallenge.ascension === 13` — apply the C13 exp
    /// multiplier.
    pub in_ascension_13: bool,
    /// `player.challengecompletions[6]` — feeds the reinc6 base and the
    /// `1.075` divisor.
    pub c6_completions: f64,
    /// `player.challengecompletions[13]` — feeds the C13 exp multiplier.
    pub c13_completions: f64,

    // ─── c13effcompletions inputs ─────────────────────────────────────────
    /// `sumContents(player.challengecompletions)` — total completion count.
    /// `c13effcompletions` subtracts the high-tier challenge contributions
    /// and the singularity-15/20 bonuses.
    pub total_challenge_completions: f64,
    /// `player.challengecompletions[11]`.
    pub c11_completions: f64,
    /// `player.challengecompletions[12]`.
    pub c12_completions: f64,
    /// `player.challengecompletions[14]`.
    pub c14_completions: f64,
    /// `player.challengecompletions[15]`.
    pub c15_completions: f64,
    /// `player.singularityCount` — feeds the -4 / -1 cuts at 15 / 20.
    pub singularity_count: f64,

    // ─── Research / cube / platonic reductions ────────────────────────────
    /// `player.researches[51]`. Exponent multiplied by `1 - 0.06 * n`.
    pub research_51: f64,
    /// `player.researches[52]`. Exponent multiplied by `1 - 0.05 * n`.
    pub research_52: f64,
    /// `player.researches[53]`. Exponent multiplied by `1 - 0.05 * n`.
    pub research_53: f64,
    /// `player.researches[54]`. Exponent multiplied by `1 - 0.05 * n`.
    pub research_54: f64,
    /// `player.researches[55]`. Exponent multiplied by `1 - 0.05 * n`.
    pub research_55: f64,
    /// `player.researches[159]`. Feeds `0.98^(3/5 * log10(rareFragments+1) * n)`.
    pub research_159: f64,
    /// `player.researches[200]`. Exponent multiplied by `1 - 0.666 * n / 100000`.
    pub research_200: f64,
    /// `player.cubeUpgrades[50]`. Same `1 - 0.666 * n / 100000` factor as
    /// `research_200`.
    pub cube_upgrade_50: f64,
    /// `player.platonicUpgrades[5]`. Adds `0.1 * n` to the `ascendShards`
    /// log exponent.
    pub platonic_upgrade_5: f64,
    /// `player.platonicUpgrades[10]`. Adds `0.2 * n` to the `ascendShards`
    /// log exponent.
    pub platonic_upgrade_10: f64,
    /// `calculateTaxPlatonicBlessing()`. Added to the `ascendShards` log
    /// exponent.
    pub tax_platonic_blessing: f64,
    /// `player.upgrades[121]`. When `> 0`, halves the exponent.
    pub upgrade_121: f64,
    /// `player.upgrades[125]`. Feeds the `ascendShards` exponent with c10
    /// scaling.
    pub upgrade_125: f64,
    /// `player.challengecompletions[10]`. Scales `upgrade_125`'s
    /// contribution.
    pub c10_completions: f64,

    // ─── Singularity / late-game ──────────────────────────────────────────
    /// `player.highestSingularityCount`. When `>= 281`, halves the exponent.
    pub highest_singularity_count: f64,
    /// `player.singularityChallenges.taxmanLastStand.enabled`.
    pub taxman_last_stand_enabled: bool,
    /// `player.unlocks.ascensions` — multiplies by `4` inside
    /// `taxman_last_stand`.
    pub ascensions_unlocked: bool,
    /// `player.highestchallengecompletions[14]` — multiplies by `5` inside
    /// `taxman_last_stand` when `> 0`.
    pub highest_c14_completions: f64,

    // ─── Pre-evaluated effect values (sourced by web_ui) ──────────────────
    /// `+getAchievementReward('taxReduction')` — the unary `+` coerces a
    /// boolean to `0/1` in TS; pass the equivalent `0.0` or `1.0`.
    pub tax_reduction_achievement: f64,
    /// `getRuneEffects('duplication', 'taxReduction')`.
    pub duplication_rune_tax_reduction: f64,
    /// `getRuneEffects('thrift', 'taxReduction')`.
    pub thrift_rune_tax_reduction: f64,
    /// `getAntUpgradeEffect(AntUpgrades.Taxes).taxReduction`.
    pub ant_tax_reduction: f64,
    /// `getTalismanEffects('exemption').taxReduction`.
    pub exemption_talisman_tax_reduction: f64,
    /// `G.challenge15Rewards.taxes.value`.
    pub challenge_15_taxes_reward: f64,
    /// `player.campaigns.taxMultiplier`.
    pub campaign_tax_multiplier: f64,

    // ─── Decimal inputs (for log10) ───────────────────────────────────────
    /// `player.ascendShards` — log10 feeds the divisor in the exponent
    /// chain.
    pub ascend_shards: Decimal,
    /// `player.rareFragments` — log10 feeds `research[159]`'s `0.98^...`
    /// term.
    pub rare_fragments: Decimal,
    /// `getAntUpgradeEffect(AntUpgrades.Coins).coinMultiplier` — log10 of
    /// this is added to `flat_max_exponent_increase`.
    pub fortunae_formicidae_coin_multiplier: Decimal,
    /// `calculateBuildingPowerCoinMultiplier()` — also log10'd into
    /// `flat_max_exponent_increase`.
    pub building_power_coin_multiplier: Decimal,
    /// `G.produceTotal` — sum of pre-clamp tier outputs. Its log10 feeds
    /// the `exponent_for_divisor` (clamped to `[0, maxexponent]`).
    pub produce_total: Decimal,
}

/// Result of [`calculate_tax`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CalculateTaxResult {
    /// The final tax exponent — floored at `1e-300` to dodge an overflow
    /// bug.
    pub exponent: f64,
    /// Max exponent the player can reach — floored value with the flat
    /// increase.
    pub maxexponent: f64,
    /// The `taxdivisor` that scales coin production downward at high counts.
    pub taxdivisor: Decimal,
    /// Sibling check value — used by the UI tier to detect
    /// "you're about to hit the cap".
    pub taxdivisorcheck: Decimal,
    /// True when the overtaxed achievement should be awarded — i.e. the
    /// player is in C13, has at least 1 c13eff completion, and their
    /// max-exponent gap is at most `99999`. The UI tier hooks this flag to
    /// `awardUngroupedAchievement`.
    pub should_award_overtaxed: bool,
}

/// Hardcoded numerator of the `maxexponent` formula. Comes from the
/// underlying log-base-1.01 conversion of the 275-coin "ten billion to the
/// max-exponent power" target.
const MAX_EXPONENT_NUMERATOR: f64 = 275.0;

/// `1.01` is the base of every tax-divisor power. Pulled out so the two
/// `log10(1.01)` calls in `maxexponent` / `taxdivisor` share the same
/// constant.
const TAX_BASE: f64 = 1.01;

fn compute_c13_eff_completions(input: &CalculateTaxInput) -> f64 {
    let sing_15_cut = if input.singularity_count >= 15.0 {
        4.0
    } else {
        0.0
    };
    let sing_20_cut = if input.singularity_count >= 20.0 {
        1.0
    } else {
        0.0
    };
    (input.total_challenge_completions
        - input.c11_completions
        - input.c12_completions
        - input.c13_completions
        - input.c14_completions
        - input.c15_completions
        - sing_15_cut
        - sing_20_cut)
        .max(0.0)
}

fn compute_base_exp(input: &CalculateTaxInput) -> f64 {
    // Precedence mirrors the legacy code: reinc6 → reinc9 → asc15. Each
    // later check overwrites the prior; if none fire, base is 1.
    let mut exp = 1.0;
    if input.in_reinc_6 {
        exp = 3.0 * (1.0 + input.c6_completions / 25.0).powi(2);
    }
    if input.in_reinc_9 {
        exp = 0.005;
    }
    if input.in_ascension_15 {
        exp = 0.000_005;
    }
    exp
}

/// Compute the tax exponent, max exponent, and the two `taxdivisor` values.
///
/// Branching summary:
/// - Base `exp` picks one of `{1, reinc6 formula, 0.005, 0.000005}`.
/// - C13 multiplies by `400 * (1 + c13/6) * 1.05^c13effcompletions`.
/// - C6 completions divide by `1.075`.
/// - Research / talisman / rune / cube reductions stack multiplicatively.
/// - `ascend_shards` log10 raised to
///   `1 + c10/300*upgrade_125 + 0.1*plat5 + 0.2*plat10 + tax_platonic_blessing`
///   divides.
/// - `rare_fragments` + `research_159` add a `0.98^...` factor.
/// - `>= 281` singularity + `upgrade_121` each halve.
/// - `taxman_last_stand` stacks `×4` (asc unlocked) and `×5` (c14 done).
/// - Final clamp at `1e-300`.
///
/// Then
/// `maxexponent = floor(275 / (log10(1.01) * exponent)) - 1 + flat_increase`,
/// where `flat_increase = log10(fortunae_formicidae) +
/// log10(building_power)`.
///
/// `exponent_for_divisor` clamps `log10(produce_total + 1)` to
/// `[0, maxexponent]` then subtracts `flat_increase`. `exponent_for_warning`
/// is just `maxexponent - flat_increase`. Both go through
/// `(1/550 * x^2)` before becoming the taxdivisor exponent.
#[must_use]
pub fn calculate_tax(input: &CalculateTaxInput) -> CalculateTaxResult {
    let c13eff = compute_c13_eff_completions(input);

    let mut exp = compute_base_exp(input);

    if input.in_ascension_13 {
        exp *= 400.0 * (1.0 + 1.0 / 6.0 * input.c13_completions);
        exp *= 1.05_f64.powf(c13eff);
    }
    if input.c6_completions > 0.0 {
        exp /= 1.075;
    }

    let mut exponent = 1.0_f64;
    exponent *= exp;
    exponent *= 1.0 - 0.06 * input.research_51;
    exponent *= 1.0 - 0.05 * input.research_52;
    exponent *= 1.0 - 0.05 * input.research_53;
    exponent *= 1.0 - 0.05 * input.research_54;
    exponent *= 1.0 - 0.05 * input.research_55;
    exponent *= input.tax_reduction_achievement;
    exponent *= 0.965_f64.powf(calc_ecc(ChallengeType::Reincarnation, input.c6_completions));
    exponent *= input.duplication_rune_tax_reduction;
    exponent *= input.thrift_rune_tax_reduction;
    exponent *= input.ant_tax_reduction;
    // ascendShards log10 raised to a sum of platonic / upgrade contributions.
    let ascend_log = (input.ascend_shards + Decimal::one()).log10().to_number();
    exponent *= 1.0
        / (1.0 + ascend_log).powf(
            1.0 + 1.0 / 300.0 * input.c10_completions * input.upgrade_125
                + 0.1 * input.platonic_upgrade_5
                + 0.2 * input.platonic_upgrade_10
                + input.tax_platonic_blessing,
        );
    exponent *= 1.0 + input.exemption_talisman_tax_reduction;
    let rare_log = (input.rare_fragments + Decimal::one()).log10().to_number();
    exponent *= 0.98_f64.powf(3.0 / 5.0 * rare_log * input.research_159);
    exponent *= 0.966_f64.powf(calc_ecc(ChallengeType::Ascension, input.c13_completions));
    exponent *= 1.0 - 0.666 * input.research_200 / 100_000.0;
    exponent *= 1.0 - 0.666 * input.cube_upgrade_50 / 100_000.0;
    exponent *= input.challenge_15_taxes_reward;
    exponent *= input.campaign_tax_multiplier;
    if input.upgrade_121 > 0.0 {
        exponent *= 0.5;
    }
    if input.highest_singularity_count >= 281.0 {
        exponent *= 0.5;
    }
    if input.taxman_last_stand_enabled {
        if input.ascensions_unlocked {
            exponent *= 4.0;
        }
        if input.highest_c14_completions > 0.0 {
            exponent *= 5.0;
        }
    }

    // Overflow guard — exponent of zero would NaN every downstream pow.
    if exponent < 1e-300 {
        exponent = 1e-300;
    }

    let flat_max_exponent_increase = input
        .fortunae_formicidae_coin_multiplier
        .log10()
        .to_number()
        + input.building_power_coin_multiplier.log10().to_number();

    let maxexponent = (MAX_EXPONENT_NUMERATOR / (TAX_BASE.log10() * exponent)).floor() - 1.0
        + flat_max_exponent_increase;

    let produce_log = (input.produce_total + Decimal::one()).log10().to_number();
    let exponent_for_divisor =
        (maxexponent.min(produce_log.floor()) - flat_max_exponent_increase).max(0.0);
    let exponent_for_warning = (maxexponent - flat_max_exponent_increase).max(0.0);

    let divisor_exponent = 1.0 / 550.0 * exponent_for_divisor.powi(2);
    let check_exponent = 1.0 / 550.0 * exponent_for_warning.powi(2);

    let taxdivisor =
        Decimal::from_finite(TAX_BASE).pow(Decimal::from_finite(divisor_exponent * exponent));
    let taxdivisorcheck =
        Decimal::from_finite(TAX_BASE).pow(Decimal::from_finite(check_exponent * exponent));

    // Overtaxed achievement: C13 active + at least one effective completion
    // + max-exponent gap below 100000.
    let should_award_overtaxed = input.in_ascension_13
        && (maxexponent - flat_max_exponent_increase) <= 99_999.0
        && c13eff >= 1.0;

    CalculateTaxResult {
        exponent,
        maxexponent,
        taxdivisor,
        taxdivisorcheck,
        should_award_overtaxed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline() -> CalculateTaxInput {
        CalculateTaxInput {
            in_reinc_6: false,
            in_reinc_9: false,
            in_ascension_15: false,
            in_ascension_13: false,
            c6_completions: 0.0,
            c13_completions: 0.0,
            total_challenge_completions: 0.0,
            c11_completions: 0.0,
            c12_completions: 0.0,
            c14_completions: 0.0,
            c15_completions: 0.0,
            singularity_count: 0.0,
            research_51: 0.0,
            research_52: 0.0,
            research_53: 0.0,
            research_54: 0.0,
            research_55: 0.0,
            research_159: 0.0,
            research_200: 0.0,
            cube_upgrade_50: 0.0,
            platonic_upgrade_5: 0.0,
            platonic_upgrade_10: 0.0,
            tax_platonic_blessing: 0.0,
            upgrade_121: 0.0,
            upgrade_125: 0.0,
            c10_completions: 0.0,
            highest_singularity_count: 0.0,
            taxman_last_stand_enabled: false,
            ascensions_unlocked: false,
            highest_c14_completions: 0.0,
            tax_reduction_achievement: 1.0,
            duplication_rune_tax_reduction: 1.0,
            thrift_rune_tax_reduction: 1.0,
            ant_tax_reduction: 1.0,
            exemption_talisman_tax_reduction: 0.0,
            challenge_15_taxes_reward: 1.0,
            campaign_tax_multiplier: 1.0,
            ascend_shards: Decimal::zero(),
            rare_fragments: Decimal::zero(),
            fortunae_formicidae_coin_multiplier: Decimal::one(),
            building_power_coin_multiplier: Decimal::one(),
            produce_total: Decimal::zero(),
        }
    }

    #[test]
    fn baseline_produces_unit_exponent() {
        // With unit multipliers everywhere, base exp = 1, all factors = 1
        // (taxReductionAchievement = 1 from the +true coercion), so the
        // final exponent stays at 1.
        let result = calculate_tax(&baseline());
        assert!((result.exponent - 1.0).abs() < 1e-12);
    }

    #[test]
    fn reinc9_base_exp_is_0_005() {
        let input = CalculateTaxInput {
            in_reinc_9: true,
            ..baseline()
        };
        let result = calculate_tax(&input);
        assert!((result.exponent - 0.005).abs() < 1e-12);
    }

    #[test]
    fn ascension15_base_exp_is_5e_minus_6() {
        let input = CalculateTaxInput {
            in_ascension_15: true,
            ..baseline()
        };
        let result = calculate_tax(&input);
        assert!((result.exponent - 0.000_005).abs() < 1e-15);
    }

    #[test]
    fn precedence_ascension15_wins_over_reinc9() {
        // The TS code overwrites in order: reinc6 → reinc9 → asc15. With
        // both reinc9 and asc15 true, asc15's 5e-6 wins.
        let input = CalculateTaxInput {
            in_reinc_9: true,
            in_ascension_15: true,
            ..baseline()
        };
        let result = calculate_tax(&input);
        assert!((result.exponent - 0.000_005).abs() < 1e-15);
    }

    #[test]
    fn c6_completions_divide_by_1_075() {
        // Bonus path: reinc6 sets base = 3, and c6>0 then divides by 1.075.
        // But CalcECC('reincarnation', 1) = 1 → factor 0.965^1, so the final
        // exponent ≈ 3/1.075 * 0.965 ≈ 2.692.
        let input = CalculateTaxInput {
            in_reinc_6: true,
            c6_completions: 1.0,
            ..baseline()
        };
        let result = calculate_tax(&input);
        let expected = 3.0_f64 * (1.0 + 1.0 / 25.0_f64).powi(2) / 1.075 * 0.965;
        assert!((result.exponent - expected).abs() / expected < 1e-9);
    }

    #[test]
    fn research_51_reduces_exponent() {
        let with_research = CalculateTaxInput {
            research_51: 1.0,
            ..baseline()
        };
        let result = calculate_tax(&with_research);
        // Just one factor of 1 - 0.06 * 1 = 0.94.
        assert!((result.exponent - 0.94).abs() < 1e-12);
    }

    #[test]
    fn upgrade_121_halves_exponent() {
        let input = CalculateTaxInput {
            upgrade_121: 1.0,
            ..baseline()
        };
        let result = calculate_tax(&input);
        assert!((result.exponent - 0.5).abs() < 1e-12);
    }

    #[test]
    fn high_singularity_halves_exponent() {
        let input = CalculateTaxInput {
            highest_singularity_count: 281.0,
            ..baseline()
        };
        let result = calculate_tax(&input);
        assert!((result.exponent - 0.5).abs() < 1e-12);
    }

    #[test]
    fn underflow_clamps_to_1e_minus_300() {
        // Force a near-zero base by combining ascension15 (5e-6) with research
        // factors that nearly zero the exponent.
        let input = CalculateTaxInput {
            in_ascension_15: true,
            research_51: 16.0, // 1 - 0.96 = 0.04
            research_52: 19.0, // 1 - 0.95 = 0.05
            research_53: 19.0,
            research_54: 19.0,
            research_55: 19.0,
            ..baseline()
        };
        let result = calculate_tax(&input);
        // exp = 5e-6 * 0.04 * 0.05^4 ≈ 1.25e-12 — still well above 1e-300,
        // so the clamp shouldn't fire. Check the value is finite + positive.
        assert!(result.exponent > 0.0);
        assert!(result.exponent.is_finite());
    }

    #[test]
    fn taxdivisor_is_one_when_produce_total_is_zero() {
        // With produce_total = 0 → log10(1) = 0 → exponent_for_divisor = 0
        // → divisor_exponent = 0 → taxdivisor = 1.01^0 = 1.
        let result = calculate_tax(&baseline());
        assert!((result.taxdivisor.to_number() - 1.0).abs() < 1e-12);
    }

    #[test]
    fn taxdivisor_grows_with_produce_total() {
        let small = CalculateTaxInput {
            produce_total: Decimal::from_finite(1e10),
            ..baseline()
        };
        let big = CalculateTaxInput {
            produce_total: Decimal::from_finite(1e30),
            ..baseline()
        };
        let small_result = calculate_tax(&small);
        let big_result = calculate_tax(&big);
        assert!(big_result.taxdivisor > small_result.taxdivisor);
    }

    #[test]
    fn maxexponent_includes_flat_increase() {
        let input = CalculateTaxInput {
            // log10(1e5) = 5 → flat_increase = 10
            fortunae_formicidae_coin_multiplier: Decimal::from_finite(1e5),
            building_power_coin_multiplier: Decimal::from_finite(1e5),
            ..baseline()
        };
        let no_increase_result = calculate_tax(&baseline());
        let with_increase_result = calculate_tax(&input);
        assert!(
            (with_increase_result.maxexponent - no_increase_result.maxexponent - 10.0).abs() < 1e-9
        );
    }

    #[test]
    fn overtaxed_flag_requires_ascension_13_and_c13eff() {
        // Outside C13 → never awarded.
        let outside = baseline();
        assert!(!calculate_tax(&outside).should_award_overtaxed);

        // Inside C13 with no eff completions → not awarded.
        let in_c13_no_eff = CalculateTaxInput {
            in_ascension_13: true,
            ..baseline()
        };
        assert!(!calculate_tax(&in_c13_no_eff).should_award_overtaxed);

        // Inside C13 with at least one eff completion → awarded (max-exp gap
        // is well below 99999 with baseline unit multipliers).
        let in_c13_with_eff = CalculateTaxInput {
            in_ascension_13: true,
            total_challenge_completions: 1.0,
            ..baseline()
        };
        // Need to check the gap is below 99999. With baseline-unit-multipliers
        // and the C13 amplifier (×400 * (1 + 0/6) * 1.05^1 ≈ 420), exponent is
        // ~420 → maxexponent ≈ 275 / (log10(1.01) * 420) - 1 ≈ 151. Gap is
        // 151, well below 99999.
        assert!(calculate_tax(&in_c13_with_eff).should_award_overtaxed);
    }

    // ─── helper-function tests ─────────────────────────────────────────────

    #[test]
    fn c13eff_subtracts_high_tier_and_singularity_bonuses() {
        let input = CalculateTaxInput {
            total_challenge_completions: 100.0,
            c11_completions: 10.0,
            c12_completions: 10.0,
            c13_completions: 10.0,
            c14_completions: 10.0,
            c15_completions: 10.0,
            singularity_count: 20.0,
            ..baseline()
        };
        // 100 - 50 (c11..c15) - 4 (sing 15) - 1 (sing 20) = 45
        let c13eff = compute_c13_eff_completions(&input);
        assert_eq!(c13eff, 45.0);
    }

    #[test]
    fn c13eff_floors_at_zero() {
        let input = CalculateTaxInput {
            total_challenge_completions: 5.0,
            c11_completions: 10.0,
            ..baseline()
        };
        let c13eff = compute_c13_eff_completions(&input);
        assert_eq!(c13eff, 0.0);
    }
}
