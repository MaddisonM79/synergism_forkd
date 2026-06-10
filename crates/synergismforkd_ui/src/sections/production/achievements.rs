//! Achievements: a dense grid of all 509 achievements, earned ones lit.
//! Hovering (or focusing) a cell fills the detail box below the grid with
//! its name/description, point value, and earned state — no popup. Header
//! summarizes earned count, points, and completion percent.

use dioxus::prelude::*;
use synergismforkd_bignum::Decimal;
use synergismforkd_logic::mechanics::achievement_awards::achievement_progress;
use synergismforkd_logic::mechanics::achievement_levels::{
    achievement_level_from_points, to_next_achievement_level_exp,
};
use synergismforkd_logic::mechanics::achievement_point_values::ACHIEVEMENT_POINT_VALUES;
use synergismforkd_logic::state::achievements::ACHIEVEMENTS_LEN;
use synergismforkd_logic::PlayerAction;

use crate::bridge::{use_bridge, use_slice};
use crate::components::{Num, Progress};
use crate::format::format_value;
use crate::i18n::t;

use super::achievements_text;

/// Total achievement points available (sum of the per-achievement table).
fn max_points() -> f64 {
    ACHIEVEMENT_POINT_VALUES.iter().sum()
}

#[component]
pub fn Achievements() -> Element {
    let bridge = use_bridge();
    // Opening the screen awards "Achievement Hunter" (the legacy
    // participationTrophy, fired on showing the tab). Runs once per mount.
    use_hook(|| bridge.dispatch(PlayerAction::OpenedAchievements));
    // The earned bitmap: re-renders the grid only when an achievement flips.
    let earned = use_slice(|s| s.achievements.achievements.to_vec());
    let points = use_slice(|s| s.achievements.achievement_points);
    let earned_count = use_memo(move || earned().iter().filter(|&&e| e != 0).count());

    // Which achievement the detail box describes (hover/focus driven).
    let mut focused = use_signal(|| None::<usize>);

    let total = max_points();
    let percent = if total > 0.0 {
        points() / total * 100.0
    } else {
        0.0
    };
    let notation = bridge.prefs.read().notation;

    rsx! {
        div { class: "sf-section-head",
            h1 { {t("nav.section.achievements")} }
        }
        div { class: "sf-ach-summary",
            span {
                "{earned_count()} / {ACHIEVEMENTS_LEN} "
                {t("achievements.earned")}
            }
            span { class: "sf-ach-points",
                Num { value: Decimal::from_finite(points()) }
                " / "
                Num { value: Decimal::from_finite(total) }
                " "
                {t("achievements.points")}
                " ({format_value(Decimal::from_finite(percent), notation)}%)"
            }
        }
        LevelBar { points: points() }
        AchievementDetail { focused: focused(), earned: focused().map(|i| earned()[i] != 0) }
        div { class: "sf-ach-grid",
            for i in 0..ACHIEVEMENTS_LEN {
                AchievementCell {
                    key: "{i}",
                    index: i,
                    earned: earned()[i] != 0,
                    on_focus: move |idx| focused.set(Some(idx)),
                }
            }
        }
    }
}

/// Synergism Level + a bar of progress toward the next level. Levels are
/// 50 AP apart below 2500 points, 100 apart above (the legacy two-regime
/// curve); the bar fills with this level's AP gained out of the AP the next
/// level costs.
#[component]
fn LevelBar(points: f64) -> Element {
    let bridge = use_bridge();
    let notation = bridge.prefs.read().notation;
    let level = achievement_level_from_points(points);
    let next_level = level + 1.0;
    let to_next = to_next_achievement_level_exp(points);
    let per_level = if points < 2_500.0 { 50.0 } else { 100.0 };
    let gained = per_level - to_next;
    let fraction = (gained / per_level).clamp(0.0, 1.0);

    rsx! {
        div { class: "sf-ach-level",
            div { class: "sf-ach-level-row",
                span { class: "sf-ach-level-name",
                    {t("achievements.level")} " "
                    span { class: "sf-ach-level-num", "{level}" }
                }
                span { class: "sf-ach-level-to-next",
                    {format_value(Decimal::from_finite(gained), notation)}
                    " / "
                    {format_value(Decimal::from_finite(per_level), notation)}
                    " "
                    {t("achievements.ap")}
                    " → "
                    {t("achievements.level")} " {next_level}"
                }
            }
            Progress { fraction }
        }
    }
}

/// The persistent info box above the grid: `#N - Name [status]`, the name's
/// flavor below, then the requirement + progress. Shows a prompt when
/// nothing is hovered.
#[component]
fn AchievementDetail(focused: Option<usize>, earned: Option<bool>) -> Element {
    let Some(index) = focused else {
        return rsx! {
            div { class: "sf-ach-detail muted", {t("achievements.hover_hint")} }
        };
    };
    let points = ACHIEVEMENT_POINT_VALUES[index];
    let is_earned = earned == Some(true);
    let status_cls = if is_earned {
        "sf-ach-status earned"
    } else {
        "sf-ach-status"
    };
    let status_key = if is_earned {
        "achievements.completed"
    } else {
        "achievements.locked"
    };
    // Live numeric progress (e.g. "34 / 100") for the threshold-backed
    // achievements. Reading state here subscribes only this small detail box
    // to the tick — the 509-cell grid stays on its own earned-bitmap memo.
    let bridge = use_bridge();
    let notation = bridge.prefs.read().notation;
    let progress = achievement_progress(&bridge.state.read(), index);

    rsx! {
        div { class: "sf-ach-detail",
            div { class: "sf-ach-detail-head",
                span { class: "sf-ach-detail-num", "#{index + 1}" }
                span { class: "sf-ach-detail-name", " - " {achievements_text::name(index)} }
                span { class: status_cls, {t(status_key)} }
                span { class: "sf-ach-detail-pts", "{points} {t(\"achievements.ap\")}" }
            }
            div { class: "sf-ach-detail-req",
                span { class: "sf-ach-req-label", {t("achievements.requirement")} ": " }
                span { {achievements_text::requirement(index)} }
            }
            div { class: "sf-ach-detail-progress",
                if is_earned {
                    span { class: "sf-ach-progress-done", {t("achievements.progress_done")} }
                } else if let Some((current, target)) = progress {
                    span { class: "sf-ach-progress-count",
                        {format_value(Decimal::from_finite(current.min(target)), notation)}
                        " / "
                        {format_value(Decimal::from_finite(target), notation)}
                    }
                } else {
                    span { class: "sf-ach-progress-pending", {t("achievements.progress_pending")} }
                }
            }
        }
    }
}

#[component]
fn AchievementCell(index: usize, earned: bool, on_focus: EventHandler<usize>) -> Element {
    let cls = if earned {
        "sf-ach-cell earned"
    } else {
        "sf-ach-cell"
    };
    rsx! {
        div {
            class: cls,
            tabindex: "0",
            onmouseenter: move |_| on_focus.call(index),
            onfocus: move |_| on_focus.call(index),
            // 1-based label, matching the legacy numbering.
            "{index + 1}"
        }
    }
}
