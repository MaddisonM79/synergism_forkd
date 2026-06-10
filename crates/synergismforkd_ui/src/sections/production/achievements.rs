//! Achievements: a dense grid of all 509 achievements, earned ones lit,
//! each with its name/description + point value on hover. Header summarizes
//! earned count, points, and completion percent.

use dioxus::prelude::*;
use synergismforkd_bignum::Decimal;
use synergismforkd_logic::mechanics::achievement_point_values::ACHIEVEMENT_POINT_VALUES;
use synergismforkd_logic::state::achievements::ACHIEVEMENTS_LEN;

use crate::bridge::{use_bridge, use_slice};
use crate::components::{Num, Tooltip};
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
        div { class: "sf-ach-grid",
            for i in 0..ACHIEVEMENTS_LEN {
                AchievementCell { key: "{i}", index: i, earned: earned()[i] != 0 }
            }
        }
    }
}

#[component]
fn AchievementCell(index: usize, earned: bool) -> Element {
    let points = ACHIEVEMENT_POINT_VALUES[index];
    let cls = if earned {
        "sf-ach-cell earned"
    } else {
        "sf-ach-cell"
    };
    rsx! {
        Tooltip {
            tip: rsx! {
                span { class: "sf-ach-tip-name", {achievements_text::full(index)} }
                span { class: "sf-ach-tip-pts", "{points} {t(\"achievements.ap\")}" }
            },
            div { class: cls,
                // 1-based label, matching the legacy numbering.
                "{index + 1}"
            }
        }
    }
}
