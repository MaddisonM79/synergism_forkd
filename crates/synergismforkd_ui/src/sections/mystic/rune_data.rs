//! UI metadata for the five core runes (display names, effect blurbs, and
//! reveal gates). Numeric cost data lives in the logic tier
//! (`synergismforkd_logic::mechanics::rune_data`); this is the presentation
//! layer only.

use synergismforkd_bignum::Decimal;
use synergismforkd_logic::mechanics::rune_blessing_effects::{
    duplication_rune_blessing_effects, prism_rune_blessing_effects, speed_rune_blessing_effects,
    superior_intellect_rune_blessing_effects, thrift_rune_blessing_effects,
};
use synergismforkd_logic::mechanics::rune_effects::{
    duplication_rune_effects, prism_rune_effects, speed_rune_effects,
    superior_intellect_rune_effects, thrift_rune_effects, DuplicationRuneKey, PrismRuneKey,
    SpeedRuneKey, SuperiorIntellectRuneKey, ThriftRuneKey,
};
use synergismforkd_logic::mechanics::rune_spirit_effects::{
    duplication_rune_spirit_effects, prism_rune_spirit_effects, speed_rune_spirit_effects,
    superior_intellect_rune_spirit_effects, thrift_rune_spirit_effects,
};
use synergismforkd_logic::tick::{
    first_five_effective_rune_level, rune_blessing_power, rune_spirit_power,
};
use synergismforkd_logic::GameState;

use crate::format::{format_value, Notation};
use crate::i18n::t_args;

/// Core runes surfaced in the UI: Speed, Duplication, Prism, Thrift, SI.
pub const CORE_RUNES: usize = 5;

/// Format a multiplier as a percent-increase string (`formatAsPercentIncrease`):
/// `(mult - 1) × 100` with a trailing `%`.
fn pct(mult: f64, notation: Notation) -> String {
    format!(
        "+{}%",
        format_value(Decimal::from_finite((mult - 1.0) * 100.0), notation)
    )
}

/// Format a bare numeric value (no `+`/`%` decoration) for the multi-value
/// effect lines — the decoration lives in the i18n template.
fn num(value: f64, notation: Notation) -> String {
    format_value(Decimal::from_finite(value), notation)
}

/// Live multi-effect summary for a core rune (`0..5`), computed at the rune's
/// effective level — the equivalent of the legacy `runes.<rune>.effect`
/// aggregate line, replacing the former static blurb.
#[must_use]
pub fn rune_effect_line(state: &GameState, index: usize, notation: Notation) -> String {
    let n = first_five_effective_rune_level(state, index);
    match index {
        0 => {
            let accel = 100.0 * speed_rune_effects(n, SpeedRuneKey::AcceleratorPower);
            let free =
                (speed_rune_effects(n, SpeedRuneKey::MultiplicativeAccelerators) - 1.0) * 100.0;
            let speed = (speed_rune_effects(n, SpeedRuneKey::GlobalSpeed) - 1.0) * 100.0;
            t_args(
                "runes.speed.effect",
                &[
                    ("val", &num(accel, notation)),
                    ("val2", &num(free, notation)),
                    ("val3", &num(speed, notation)),
                ],
            )
        }
        1 => {
            let boosts = duplication_rune_effects(n, DuplicationRuneKey::MultiplierBoosts);
            let free = (duplication_rune_effects(n, DuplicationRuneKey::MultiplicativeMultipliers)
                - 1.0)
                * 100.0;
            let tax = 100.0 * (1.0 - duplication_rune_effects(n, DuplicationRuneKey::TaxReduction));
            t_args(
                "runes.duplication.effect",
                &[
                    ("val", &num(boosts, notation)),
                    ("val2", &num(free, notation)),
                    ("val3", &num(tax, notation)),
                ],
            )
        }
        2 => {
            let prod = 10f64.powf(prism_rune_effects(n, PrismRuneKey::ProductionLog10));
            let cost = 10f64.powf(prism_rune_effects(n, PrismRuneKey::CostDivisorLog10));
            t_args(
                "runes.prism.effect",
                &[
                    ("val", &num(prod, notation)),
                    ("val2", &num(cost, notation)),
                ],
            )
        }
        3 => {
            let delay = thrift_rune_effects(n, ThriftRuneKey::CostDelay);
            let salvage = thrift_rune_effects(n, ThriftRuneKey::Salvage);
            let tax = 100.0 * (1.0 - thrift_rune_effects(n, ThriftRuneKey::TaxReduction));
            t_args(
                "runes.thrift.effect",
                &[
                    ("val", &num(delay, notation)),
                    ("val2", &num(salvage, notation)),
                    ("val3", &num(tax, notation)),
                ],
            )
        }
        _ => {
            let off = superior_intellect_rune_effects(n, SuperiorIntellectRuneKey::OfferingMult);
            let obt = superior_intellect_rune_effects(n, SuperiorIntellectRuneKey::ObtainiumMult);
            let ant = superior_intellect_rune_effects(n, SuperiorIntellectRuneKey::AntSpeed);
            t_args(
                "runes.si.effect",
                &[
                    ("val", &num(off, notation)),
                    ("val2", &num(obt, notation)),
                    ("val3", &num(ant, notation)),
                ],
            )
        }
    }
}

/// Live effect line for a rune blessing (`0..5`) — the blessing's single
/// effect at its current power, mirroring the legacy `effectsDescription`.
#[must_use]
pub fn blessing_effect_line(state: &GameState, index: usize, notation: Notation) -> String {
    let p = rune_blessing_power(state, index);
    match index {
        0 => t_args(
            "runes.speed.blessing",
            &[(
                "val",
                &pct(speed_rune_blessing_effects(p).global_speed, notation),
            )],
        ),
        1 => t_args(
            "runes.duplication.blessing",
            &[(
                "val",
                &pct(
                    duplication_rune_blessing_effects(p).multiplier_boosts,
                    notation,
                ),
            )],
        ),
        2 => t_args(
            "runes.prism.blessing",
            &[(
                "val",
                &pct(prism_rune_blessing_effects(p).ant_sacrifice_mult, notation),
            )],
        ),
        3 => t_args(
            "runes.thrift.blessing",
            &[(
                "val",
                &pct(
                    thrift_rune_blessing_effects(p).accel_boost_cost_delay,
                    notation,
                ),
            )],
        ),
        _ => t_args(
            "runes.si.blessing",
            &[(
                "val",
                &format_value(
                    Decimal::from_finite(
                        superior_intellect_rune_blessing_effects(p).obt_to_ant_exponent,
                    ),
                    notation,
                ),
            )],
        ),
    }
}

/// Live effect line for a rune spirit (`0..5`).
#[must_use]
pub fn spirit_effect_line(state: &GameState, index: usize, notation: Notation) -> String {
    let p = rune_spirit_power(state, index);
    match index {
        0 => t_args(
            "runes.speed.spirit",
            &[(
                "val",
                &pct(speed_rune_spirit_effects(p).global_speed, notation),
            )],
        ),
        1 => t_args(
            "runes.duplication.spirit",
            &[(
                "val",
                &pct(duplication_rune_spirit_effects(p).wow_cubes, notation),
            )],
        ),
        2 => t_args(
            "runes.prism.spirit",
            &[(
                "val",
                &format_value(
                    Decimal::from_finite(prism_rune_spirit_effects(p).crystal_caps),
                    notation,
                ),
            )],
        ),
        3 => t_args(
            "runes.thrift.spirit",
            &[(
                "val",
                &pct(thrift_rune_spirit_effects(p).offerings, notation),
            )],
        ),
        _ => t_args(
            "runes.si.spirit",
            &[(
                "val",
                &pct(
                    superior_intellect_rune_spirit_effects(p).obtainium,
                    notation,
                ),
            )],
        ),
    }
}

/// i18n key for a core rune's display name (`0..5`).
#[must_use]
pub fn rune_name_key(index: usize) -> &'static str {
    match index {
        0 => "runes.speed.name",
        1 => "runes.duplication.name",
        2 => "runes.prism.name",
        3 => "runes.thrift.name",
        _ => "runes.si.name",
    }
}

/// Whether a core rune is revealed. Speed/Duplication/Prism/Thrift show with
/// the Runes section (the per-rune achievement-reward gates are unported —
/// they unlock early, before this section opens); Superior Intellect gates on
/// research 82 (`player.researches[82] > 0`), as in legacy.
#[must_use]
pub fn rune_unlocked(state: &GameState, index: usize) -> bool {
    match index {
        4 => state.researches.researches[82] > 0.0,
        _ => true,
    }
}

/// Whether the Rune Blessings panel is revealed. The precise legacy gate
/// (challenge-9 completion) is unported; for now blessings reveal with the
/// section.
#[must_use]
pub fn blessings_unlocked(_state: &GameState) -> bool {
    true
}

/// Whether the Rune Spirits panel is revealed (legacy gate unported; reveals
/// with the section for now).
#[must_use]
pub fn spirits_unlocked(_state: &GameState) -> bool {
    true
}
