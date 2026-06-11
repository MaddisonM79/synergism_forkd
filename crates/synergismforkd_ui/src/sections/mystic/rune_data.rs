//! UI metadata for the five core runes (display names, effect blurbs, and
//! reveal gates). Numeric cost data lives in the logic tier
//! (`synergismforkd_logic::mechanics::rune_data`); this is the presentation
//! layer only.

use synergismforkd_logic::GameState;

/// Core runes surfaced in the UI: Speed, Duplication, Prism, Thrift, SI.
pub const CORE_RUNES: usize = 5;

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
