//! Achievements: a dense grid of all 509 achievements, earned ones lit.
//! Hovering (or focusing) a cell fills the detail box below the grid with
//! its name/description, point value, and earned state — no popup. Header
//! summarizes earned count, points, and completion percent.

use dioxus::prelude::*;
use synergismforkd_bignum::Decimal;
use synergismforkd_logic::mechanics::achievement_point_values::ACHIEVEMENT_POINT_VALUES;
use synergismforkd_logic::state::achievements::ACHIEVEMENTS_LEN;

use crate::bridge::{use_bridge, use_slice};
use crate::components::Num;
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

/// The persistent info box above the grid. Shows the focused achievement's
/// full text + points, or a prompt when nothing is hovered.
#[component]
fn AchievementDetail(focused: Option<usize>, earned: Option<bool>) -> Element {
    let Some(index) = focused else {
        return rsx! {
            div { class: "sf-ach-detail muted", {t("achievements.hover_hint")} }
        };
    };
    let points = ACHIEVEMENT_POINT_VALUES[index];
    let status_cls = if earned == Some(true) {
        "sf-ach-status earned"
    } else {
        "sf-ach-status"
    };
    let status_key = if earned == Some(true) {
        "achievements.completed"
    } else {
        "achievements.locked"
    };
    rsx! {
        div { class: "sf-ach-detail",
            div { class: "sf-ach-detail-head",
                span { class: "sf-ach-detail-num", "#{index + 1}" }
                span { class: status_cls, {t(status_key)} }
                span { class: "sf-ach-detail-pts", "{points} {t(\"achievements.ap\")}" }
            }
            div { class: "sf-ach-detail-text", {achievements_text::full(index)} }
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
