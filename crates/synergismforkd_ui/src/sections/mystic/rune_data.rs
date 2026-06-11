//! UI metadata for the five core runes (display names, effect blurbs, and
//! reveal gates). Numeric cost data lives in the logic tier
//! (`synergismforkd_logic::mechanics::rune_data`); this is the presentation
//! layer only.

use synergismforkd_bignum::Decimal;
use synergismforkd_logic::mechanics::rune_blessing_effects::{
    duplication_rune_blessing_effects, prism_rune_blessing_effects, speed_rune_blessing_effects,
    superior_intellect_rune_blessing_effects, thrift_rune_blessing_effects,
};
use synergismforkd_logic::mechanics::rune_spirit_effects::{
    duplication_rune_spirit_effects, prism_rune_spirit_effects, speed_rune_spirit_effects,
    superior_intellect_rune_spirit_effects, thrift_rune_spirit_effects,
};
use synergismforkd_logic::tick::{rune_blessing_power, rune_spirit_power};
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

/// Live effect line for a rune blessing (`0..5`) — the blessing's single
/// effect at its current power, mirroring the legacy `effectsDescription`.
#[must_use]
pub fn blessing_effect_line(state: &GameState, index: usize, notation: Notation) -> String {
    let p = rune_blessing_power(state, index);
    match index {
        0 => t_args(
            "runes.blessing.speed",
            &[(
                "val",
                &pct(speed_rune_blessing_effects(p).global_speed, notation),
            )],
        ),
        1 => t_args(
            "runes.blessing.duplication",
            &[(
                "val",
                &pct(
                    duplication_rune_blessing_effects(p).multiplier_boosts,
                    notation,
                ),
            )],
        ),
        2 => t_args(
            "runes.blessing.prism",
            &[(
                "val",
                &pct(prism_rune_blessing_effects(p).ant_sacrifice_mult, notation),
            )],
        ),
        3 => t_args(
            "runes.blessing.thrift",
            &[(
                "val",
                &pct(
                    thrift_rune_blessing_effects(p).accel_boost_cost_delay,
                    notation,
                ),
            )],
        ),
        _ => t_args(
            "runes.blessing.si",
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
            "runes.spirit.speed",
            &[(
                "val",
                &pct(speed_rune_spirit_effects(p).global_speed, notation),
            )],
        ),
        1 => t_args(
            "runes.spirit.duplication",
            &[(
                "val",
                &pct(duplication_rune_spirit_effects(p).wow_cubes, notation),
            )],
        ),
        2 => t_args(
            "runes.spirit.prism",
            &[(
                "val",
                &format_value(
                    Decimal::from_finite(prism_rune_spirit_effects(p).crystal_caps),
                    notation,
                ),
            )],
        ),
        3 => t_args(
            "runes.spirit.thrift",
            &[(
                "val",
                &pct(thrift_rune_spirit_effects(p).offerings, notation),
            )],
        ),
        _ => t_args(
            "runes.spirit.si",
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
        0 => "runes.name.speed",
        1 => "runes.name.duplication",
        2 => "runes.name.prism",
        3 => "runes.name.thrift",
        _ => "runes.name.si",
    }
}

/// i18n key for a core rune's short effect blurb (`0..5`).
#[must_use]
pub fn rune_effect_key(index: usize) -> &'static str {
    match index {
        0 => "runes.effect.speed",
        1 => "runes.effect.duplication",
        2 => "runes.effect.prism",
        3 => "runes.effect.thrift",
        _ => "runes.effect.si",
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
