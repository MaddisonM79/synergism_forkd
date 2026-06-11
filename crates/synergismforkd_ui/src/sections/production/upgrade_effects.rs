//! Live effect values for the Upgrades tab — a direct port of the legacy
//! `upgradetexts` array (`Upgrades.ts:61`). Each upgrade's effect string
//! (`upgrades.effects.{idx}`) may carry `{{x}}` / `{{y}}` placeholders filled
//! by these values; entries that returned `null` in the legacy (and the
//! `81..=119` automation/generator block, which always shows `effects.81`)
//! render the static string.
//!
//! Inputs come from `&GameState` plus the tick's derived surface
//! ([`DerivedTickStats`] / [`BuildingsDerived`]) — the same `G.*` aggregates
//! the legacy read. A few formulas reference state the port doesn't track yet
//! (lifetime max obtainium/offerings, corruption deflation); those return
//! `None` and their (templated) effect line is hidden until the input lands.

use synergismforkd_bignum::Decimal;
use synergismforkd_logic::{DerivedTickStats, GameState};

use crate::format::{format_count, format_value, Notation};
use crate::i18n::{t, t_args};

/// Interpolation args for an effect string. `X` fills `{{x}}`; `XY` fills both.
pub enum EffectArgs {
    X(String),
    XY(String, String),
}

#[inline]
fn fd(x: f64) -> Decimal {
    Decimal::from_finite(x)
}

/// `10 ^ exp`, for the large `Decimal.min` caps the legacy writes as string
/// literals (`'1e5000'` etc.) that overflow `f64`.
#[inline]
fn pow10(exp: f64) -> Decimal {
    fd(10.0).pow(fd(exp))
}

/// Sum of the five owned coin producers (`calculateTotalCoinOwned`).
fn total_coin_owned(s: &GameState) -> f64 {
    (1..=5u8).map(|i| s.coin_producers.owned(i)).sum()
}

/// The legacy `(totalCoinOwned+1) * min(1e30, 1.008^totalCoinOwned)` base
/// shared by coin upgrades 1–6 and (raised to the 10th) upgrade 20.
fn coin_upgrade_base(s: &GameState) -> Decimal {
    let tco = total_coin_owned(s);
    fd(tco + 1.0) * fd(1.008).pow(fd(tco)).min(fd(1e30))
}

/// The live value(s) for upgrade `idx`'s effect string, or `None` to render
/// the static string. Mirrors `upgradetexts[idx - 1]()`.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn upgrade_effect(
    idx: usize,
    s: &GameState,
    d: &DerivedTickStats,
    notation: Notation,
) -> Option<EffectArgs> {
    let b = &d.buildings;
    let one = Decimal::one();
    // Formatting shims mirroring the legacy `format(x, digits)`.
    let x2 = |v: Decimal| EffectArgs::X(format_value(v, notation));
    let x0 = |v: Decimal| EffectArgs::X(format_count(v, notation));

    let coins = s.upgrades.coins;
    let prestige_points = s.upgrades.prestige_points;
    let transcend_points = s.upgrades.transcend_points;
    let reincarnation_points = s.upgrades.reincarnation_points;
    let transcend_shards = s.reset_counters.transcend_shards;

    match idx {
        // Coin upgrades 1–6: producer-output multiplier.
        1..=6 => Some(x2(coin_upgrade_base(s))),
        // 7: free Multipliers from Alchemies — min(4, 1+floor(log10(fifth+1))).
        7 => {
            let v = (1.0 + (s.coin_producers.owned(5) + 1.0).log10().floor()).min(4.0);
            Some(x0(fd(v)))
        }
        // 8: free Multipliers — floor(multiplierBought / 7).
        8 => Some(x0(fd((s.multiplier.multiplier_bought / 7.0).floor()))),
        // 9: free Accelerators — floor(acceleratorBought / 10).
        9 => Some(x0(fd((s.accelerator.accelerator_bought / 10.0).floor()))),
        // 10: 2 ^ min(50, secondOwnedCoin / 15).
        10 => Some(x2(
            fd(2.0).pow(fd((s.coin_producers.owned(2) / 15.0).min(50.0)))
        )),
        // 11: 1.02 ^ freeAccelerator.
        11 => Some(x2(fd(1.02).pow(fd(b.free_accelerator)))),
        // 12: min(1e4, 1.01 ^ prestigeCount).
        12 => Some(x2(fd(1.01)
            .pow(fd(s.reset_counters.prestige_count))
            .min(fd(1e4)))),
        // 13: min(1e50, (firstGenMythos + firstOwnedMythos + 1)^(4/3) * 1e22).
        13 => {
            let base =
                s.mythos_producers.tiers[0].generated + fd(s.mythos_producers.owned(1)) + one;
            Some(x2((base.pow(fd(4.0 / 3.0)) * fd(1e22)).min(fd(1e50))))
        }
        // 14, 15: 1.15 ^ freeAccelerator * 1e5.
        14 | 15 => Some(x2(fd(1.15).pow(fd(b.free_accelerator)) * fd(1e5))),
        // 16: corruption deflation — unported, render static.
        16 => None,
        // 17: min(1e125, transcendShards + 1), integer.
        17 => Some(x0((transcend_shards + one).min(fd(1e125)))),
        // 18: min(1e125, transcendShards + 1).
        18 => Some(x0((transcend_shards + one).min(fd(1e125)))),
        // 19: min(1e200, transcendPoints * 1e30 + 1).
        19 => Some(x0((transcend_points * fd(1e30) + one).min(fd(1e200)))),
        // 20: coin_upgrade_base ^ 10.
        20 => Some(x2(coin_upgrade_base(s).pow(fd(10.0)))),
        // 21–25: {x: 1 + freeMultiplier/101, y: N + freeAccelerator/101}.
        21..=25 => {
            let y_base = f64::from(26 - idx as u32); // u21→5 … u25→1
            let x = (1.0 + b.free_multiplier / 101.0).floor();
            let y = (y_base + b.free_accelerator / 101.0).floor();
            Some(EffectArgs::XY(
                format_count(fd(x), notation),
                format_count(fd(y), notation),
            ))
        }
        // 27: log_1e10(coins+1) [≤50] + max(0, log_1e50(coins+1) - 10 [≤50]).
        27 => {
            let a = (coins + one).log(fd(1e10)).to_number().floor().min(50.0);
            let bx = ((coins + one).log(fd(1e50)).to_number().floor() - 10.0).clamp(0.0, 50.0);
            Some(x0(fd(a + bx)))
        }
        // 28: min(100, floor(sumOwnedCoins / 400)).
        28 => {
            let sum: f64 = (1..=5u8).map(|i| s.coin_producers.owned(i)).sum();
            Some(x0(fd((sum / 400.0).floor().min(100.0))))
        }
        // 29: floor(min(100, sumOwnedCoins / 400)).
        29 => {
            let sum: f64 = (1..=5u8).map(|i| s.coin_producers.owned(i)).sum();
            Some(x0(fd((sum / 400.0).min(100.0).floor())))
        }
        // 30: log_1e30(coins+1) [≤50] + log_1e300(coins+1) [≤50].
        30 => {
            let a = (coins + one).log(fd(1e30)).to_number().floor().min(50.0);
            let bx = (coins + one).log(fd(1e300)).to_number().floor().min(50.0);
            Some(x0(fd(a + bx)))
        }
        // 31: floor(totalCoinOwned / 2000).
        31 => Some(x0(fd((total_coin_owned(s) / 2000.0).floor()))),
        // 32: min(500, floor(log_1e25(prestigePoints + 1))).
        32 => Some(x0(fd((prestige_points + one)
            .log(fd(1e25))
            .to_number()
            .floor()
            .min(500.0)))),
        // 33: G.totalAcceleratorBoost.
        33 => Some(x0(fd(b.total_accelerator_boost))),
        // 34: floor(3/103 * freeMultiplier).
        34 => Some(x0(fd((3.0 / 103.0 * b.free_multiplier).floor()))),
        // 35: floor(2/102 * freeMultiplier).
        35 => Some(x0(fd((2.0 / 102.0 * b.free_multiplier).floor()))),
        // 36: min(1e5000, prestigePoints ^ (1/500)).
        36 => Some(x2(prestige_points.pow(fd(1.0 / 500.0)).min(pow10(5000.0)))),
        // 37: log10(prestigePoints + 10) ^ 2.
        37 => Some(x2((prestige_points + fd(10.0)).log10().pow(fd(2.0)))),
        // 41: min(1e30, (transcendPoints + 4) ^ 0.5).
        41 => Some(x0((transcend_points + fd(4.0)).pow(fd(0.5)).min(fd(1e30)))),
        // 42: min(1e50, (prestigePoints + 1)^(1/50) / 2.5 + 1).
        42 => Some(x2(((prestige_points + one).pow(fd(1.0 / 50.0)) / fd(2.5)
            + one)
            .min(fd(1e50)))),
        // 43: min(1e30, 1.01 ^ transcendCount).
        43 => Some(x2(fd(1.01)
            .pow(fd(s.reset_counters.transcend_count))
            .min(fd(1e30)))),
        // 44: min(1e6, 1.01 ^ transcendCount).
        44 => Some(x2(fd(1.01)
            .pow(fd(s.reset_counters.transcend_count))
            .min(fd(1e6)))),
        // 45: min(2500, floor(log10(transcendShards + 1))).
        45 => Some(x0(fd((transcend_shards + one)
            .log10()
            .to_number()
            .floor()
            .min(2500.0)))),
        // 47: 1.01^AP * (AP/5 + 1).
        47 => {
            let ap = s.achievements.achievement_points;
            Some(x2(fd(1.01_f64.powf(ap) * (ap / 5.0 + 1.0))))
        }
        // 48: (min(1e25, totalMultiplier * totalAccelerator) / 1000 + 1) ^ 8.
        48 => {
            let prod = (b.total_multiplier * b.total_accelerator).min(1e25);
            Some(x0(fd(prod / 1000.0 + 1.0).pow(fd(8.0))))
        }
        // 49: min(50, floor(log_1e10(transcendPoints + 1))).
        49 => Some(x0(fd((transcend_points + one)
            .log(fd(1e10))
            .to_number()
            .floor()
            .min(50.0)))),
        // 51: G.totalAcceleratorBoost ^ 2.
        51 => Some(x2(fd(b.total_accelerator_boost).pow(fd(2.0)))),
        // 52: globalMythosMultiplier ^ 0.025.
        52 => Some(x2(b.global_mythos_multiplier.pow(fd(0.025)))),
        // 53: min(1e1250, acceleratorEffect ^ (1/125)).
        53 => Some(x2(b
            .accelerator_effect
            .pow(fd(1.0 / 125.0))
            .min(pow10(1250.0)))),
        // 54: min(1e2000, multiplierEffect ^ (1/180)).
        54 => Some(x2(b
            .multiplier_effect
            .pow(fd(1.0 / 180.0))
            .min(pow10(2000.0)))),
        // 55: 1e1000 ^ min(1000, buildingPower - 1) = 10 ^ (1000 * exp).
        55 => {
            let exp = (b.building_power - 1.0).min(1000.0);
            Some(x2(pow10(1000.0 * exp)))
        }
        // 62: min(12, floor(sumContents(challengecompletions) / 50)).
        62 => {
            let sum: f64 = s.challenges.challenge_completions.iter().sum();
            Some(x0(fd((sum / 50.0).floor().min(12.0))))
        }
        // 63: min(1e6000, (reincarnationPoints + 1) ^ 6).
        63 => Some(x0((reincarnation_points + one)
            .pow(fd(6.0))
            .min(pow10(6000.0)))),
        // 64: (reincarnationPoints + 1) ^ 2.
        64 => Some(x0((reincarnation_points + one).pow(fd(2.0)))),
        // 67: 1.03 ^ sumOwnedParticles.
        67 => {
            let sum: f64 = (1..=5u8).map(|i| s.particle_producers.owned(i)).sum();
            Some(x2(fd(1.03).pow(fd(sum))))
        }
        // 68: min(2500, floor(log10(taxdivisor) / 1000)).
        68 => Some(x0(fd((b.tax_divisor.log10().to_number() / 1000.0)
            .floor()
            .min(2500.0)))),
        // 69: a = sqrt(log10(reincarnationPointGain + 10)); {x: min(10,a), y: min(3,a)}.
        69 => {
            let a = (d.reincarnation_point_gain + fd(10.0))
                .log10()
                .pow(fd(0.5))
                .to_number();
            Some(EffectArgs::XY(
                format_value(fd(a.min(10.0)), notation),
                format_value(fd(a.min(3.0)), notation),
            ))
        }
        // 72: min(50, 1 + 2*(cc6+cc7+cc8+cc9+cc10)).
        72 => {
            let cc = &s.challenges.challenge_completions;
            let v = (1.0 + 2.0 * (cc[6] + cc[7] + cc[8] + cc[9] + cc[10])).min(50.0);
            Some(x0(fd(v)))
        }
        // 77: 1.004 ^ ants.producers[Workers].purchased.
        77 => Some(x2(fd(1.004).pow(fd(s.ants.producers[0].purchased)))),
        // 79: max(1, globalSpeedMult ^ 3).
        79 => Some(x2(fd(b.global_speed_mult).pow(fd(3.0)).max(one))),
        // 80: ant-sacrifice base-offering bonus (tiered).
        80 => {
            let asc = s.ants.ant_sacrifice_count;
            let v = 10.0 * asc.min(50.0)
                + 5.0 * (asc - 50.0).clamp(0.0, 50.0)
                + (asc - 100.0).clamp(0.0, 250.0);
            Some(x0(fd(v)))
        }
        // 124, 125: 0.333 * challengecompletions[10].
        124 | 125 => Some(x0(fd(0.333 * s.challenges.challenge_completions[10]))),
        // Everything else (legacy `null`) renders the static effect string.
        // 70/74/75/78 reference lifetime max obtainium/offerings (unported).
        _ => None,
    }
}

/// The effect line to render for upgrade `idx`, or `None` for no line.
/// `81..=119` always show the static `effects.81` (legacy special-case); other
/// indices fill the templated string, or show the static string when the
/// formula is `None` — but a *templated* string we can't fill is hidden rather
/// than shown with raw `{{…}}`.
#[must_use]
pub fn effect_text(
    idx: usize,
    s: &GameState,
    d: &DerivedTickStats,
    notation: Notation,
) -> Option<String> {
    if (81..=119).contains(&idx) {
        return Some(t("upgrades.effects.81").to_string());
    }
    let key = format!("upgrades.effects.{idx}");
    match upgrade_effect(idx, s, d, notation) {
        Some(EffectArgs::X(x)) => Some(t_args(&key, &[("x", &x)])),
        Some(EffectArgs::XY(x, y)) => Some(t_args(&key, &[("x", &x), ("y", &y)])),
        None => {
            let txt = t(&key);
            // Missing key (echoed) or an unfillable templated string → no line.
            if txt == key || txt.contains("{{") {
                None
            } else {
                Some(txt.to_string())
            }
        }
    }
}

/// Crystal-upgrade log10 base costs / per-level increments
/// (`G.crystalUpgradesCost` / `G.crystalUpgradeCostIncrement`).
const CRYSTAL_BASE_COST: [f64; 8] = [6.0, 15.0, 20.0, 40.0, 100.0, 200.0, 500.0, 1000.0];
const CRYSTAL_COST_INCREMENT: [f64; 8] = [8.0, 15.0, 20.0, 40.0, 100.0, 200.0, 500.0, 1000.0];

/// Display cost of the next crystal-upgrade purchase (`i` in `1..=8`) —
/// `10 ^ (base − prism + increment · floor((level + 0.5 − c)² / 2))`, with the
/// Prism divisor at `0` until runes are surfaced. Mirrors the legacy
/// `crystalupgradedescriptions` cost line.
#[must_use]
pub fn crystal_cost(i: u8, s: &GameState) -> Decimal {
    let u = usize::from(i - 1);
    let level = s.crystal_upgrades.crystal_upgrades[u];
    let c = if s.upgrades.upgrades[73] > 0 && s.challenges.current_reincarnation_challenge != 0 {
        10.0
    } else {
        0.0
    };
    let exp = CRYSTAL_BASE_COST[u]
        + CRYSTAL_COST_INCREMENT[u] * ((level + 0.5 - c).powi(2) / 2.0).floor();
    pow10(exp)
}

/// The live effect line for crystal upgrade `i` (`1..=8`), or `None` to hide it.
/// Effects 1/2/5 are computable from state; 3/4 need crystal-power helpers and
/// are hidden for now; 6–8 are unimplemented ("Coming SOON").
#[must_use]
pub fn crystal_effect_text(i: u8, s: &GameState, notation: Notation) -> Option<String> {
    let cu = &s.crystal_upgrades.crystal_upgrades;
    let one = Decimal::one();
    let value = match i {
        // (1 + 0.01·level) ^ achievementPoints.
        1 => {
            let ap = s.achievements.achievement_points;
            Some(fd(1.0 + 0.01 * cu[0]).pow(fd(ap)))
        }
        // (1 + level·log10(coins+1)/100) ^ (2 + log2(level+1)).
        2 => {
            let coins_log = (s.upgrades.coins + one).log10().to_number();
            let base = 1.0 + cu[1] * coins_log / 100.0;
            let exp = 2.0 + (cu[1] + 1.0).log2();
            Some(fd(base).pow(fd(exp)))
        }
        // (1 + level/20) ^ (cc1+cc2+cc3+cc4+cc5).
        5 => {
            let cc = &s.challenges.challenge_completions;
            let sum = cc[1] + cc[2] + cc[3] + cc[4] + cc[5];
            Some(fd(1.0 + cu[4] / 20.0).pow(fd(sum)))
        }
        _ => None,
    }?;
    Some(t_args(
        &format!("upgrades.crystalEffects.{i}"),
        &[("x", &format_value(value, notation))],
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coin_upgrade_one_effect_is_templated() {
        let s = GameState::default();
        let d = DerivedTickStats::default();
        let txt =
            effect_text(1, &s, &d, Notation::default()).expect("coin upgrade 1 has an effect");
        assert!(txt.starts_with("Effect: Worker Production x"));
        assert!(!txt.contains("{{"), "placeholder must be filled");
    }

    #[test]
    fn automation_block_uses_effect_81() {
        let s = GameState::default();
        let d = DerivedTickStats::default();
        for idx in [81usize, 100, 119] {
            let txt = effect_text(idx, &s, &d, Notation::default()).unwrap();
            assert_eq!(txt, t("upgrades.effects.81"));
        }
    }

    #[test]
    fn unported_input_effects_are_hidden_not_raw() {
        let s = GameState::default();
        let d = DerivedTickStats::default();
        // 16 (corruption) and 74 (max offerings) have templated strings we
        // can't fill yet — they must hide, never show raw {{x}}.
        for idx in [16usize, 70, 74, 75, 78] {
            assert!(effect_text(idx, &s, &d, Notation::default()).is_none());
        }
    }

    #[test]
    fn xy_effect_fills_both_placeholders() {
        let s = GameState::default();
        let d = DerivedTickStats::default();
        let txt = effect_text(21, &s, &d, Notation::default()).unwrap();
        assert!(!txt.contains("{{x}}") && !txt.contains("{{y}}"));
    }

    #[test]
    fn crystal_cost_at_level_zero_is_base() {
        let s = GameState::default();
        // base[0] = 6, level 0, c = 0 → floor(0.5²/2) = 0 → 10^6.
        assert_eq!(crystal_cost(1, &s), Decimal::from_finite(1e6));
    }

    #[test]
    fn crystal_effect_fills_or_hides() {
        let s = GameState::default();
        // 1/2/5 are computable → templated string filled.
        for i in [1u8, 2, 5] {
            let txt = crystal_effect_text(i, &s, Notation::default())
                .unwrap_or_else(|| panic!("crystal {i} should have an effect"));
            assert!(
                !txt.contains("{{"),
                "crystal {i} placeholder must be filled"
            );
        }
        // 3/4 need unported helpers → hidden, never raw.
        for i in [3u8, 4] {
            assert!(crystal_effect_text(i, &s, Notation::default()).is_none());
        }
    }
}
