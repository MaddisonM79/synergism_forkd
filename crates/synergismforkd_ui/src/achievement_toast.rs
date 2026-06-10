//! Achievement-unlock toasts. The logic tier emits no per-achievement event
//! (awards are set across many `award_*` paths that return counts), so the
//! host loop diffs the earned bitmap each tick and calls [`toast_new_unlocks`]
//! with the previous snapshot. Driving it from the loop (rather than a Dioxus
//! effect) makes it deterministic — every tick, regardless of the open screen.

use dioxus::prelude::*;
use synergismforkd_logic::mechanics::achievement_awards::get_achievement_quarks;
use synergismforkd_logic::mechanics::achievement_point_values::ACHIEVEMENT_POINT_VALUES;
use synergismforkd_logic::GameState;

use crate::bridge::{GameBridge, ToastKind};
use crate::format::{format_value, Notation};
use crate::i18n::t;
use crate::sections::production::achievements_text;

/// Above this many simultaneous unlocks (e.g. a save import), collapse into
/// one summary toast instead of flooding the stack.
const SUMMARY_THRESHOLD: usize = 3;

/// Toast every achievement that flipped 0→1 between `prev` (the previous
/// tick's bitmap) and `state.achievements.achievements`. Shows the same data
/// the TS notification did — full description + AP + quark reward.
pub fn toast_new_unlocks(bridge: &GameBridge, prev: &[u8], state: &GameState) {
    let cur = &state.achievements.achievements;
    let newly: Vec<usize> = cur
        .iter()
        .enumerate()
        .filter(|&(i, &c)| c != 0 && prev.get(i).copied().unwrap_or(0) == 0)
        .map(|(i, _)| i)
        .collect();
    if newly.is_empty() {
        return;
    }

    // Per-award quark reward (legacy `getAchievementQuarks`) — the same for
    // every achievement this tick.
    let quarks_each = get_achievement_quarks(state.quarks.quark_bonus);
    let notation = bridge.prefs.peek().notation;

    if newly.len() > SUMMARY_THRESHOLD {
        let total_ap: f64 = newly.iter().map(|&i| ACHIEVEMENT_POINT_VALUES[i]).sum();
        let total_quarks = quarks_each * newly.len() as f64;
        bridge.toast_rich(
            ToastKind::Achievement,
            Some(format!(
                "{} {}",
                newly.len(),
                t("toasts.achievements_unlocked_n")
            )),
            reward_line(total_ap, total_quarks, notation),
        );
    } else {
        for i in newly {
            let body = format!(
                "{}\n{}",
                achievements_text::full(i),
                reward_line(ACHIEVEMENT_POINT_VALUES[i], quarks_each, notation),
            );
            bridge.toast_rich(
                ToastKind::Achievement,
                Some(t("toasts.achievement_unlocked").to_string()),
                body,
            );
        }
    }
}

/// "Reward: +5 AP, +5 Quarks".
fn reward_line(ap: f64, quarks: f64, notation: Notation) -> String {
    use synergismforkd_bignum::Decimal;
    format!(
        "{} +{} {}, +{} {}",
        t("toasts.reward"),
        format_value(Decimal::from_finite(ap), notation),
        t("achievements.ap"),
        format_value(Decimal::from_finite(quarks), notation),
        t("toasts.quarks"),
    )
}
