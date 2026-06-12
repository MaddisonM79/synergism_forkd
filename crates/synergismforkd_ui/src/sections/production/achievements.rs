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
use crate::components::{Collapsible, Num, Progress};
use crate::detail::{use_detail, DetailBody, DetailTarget};
use crate::format::format_value;
use crate::i18n::t;

use super::achievements_text;

/// Discrete Synergism-Level milestone unlocks: `(level, name key)`, in
/// ascending order. These mirror the logic tier's `LevelMilestoneKey`
/// level gates (`level_milestones.rs`) — the level at which each feature
/// unlocks. Continuous per-level rewards (salvage/quark/offering scaling)
/// aren't listed; they have no discrete unlock event.
const LEVEL_MILESTONES: &[(u32, &str)] = &[
    (5, "achievements.milestone.offering_timer"),
    (6, "achievements.milestone.crystal_autobuy_1"),
    (7, "achievements.milestone.auto_prestige"),
    (9, "achievements.milestone.crystal_autobuy_2"),
    (12, "achievements.milestone.crystal_autobuy_3"),
    (15, "achievements.milestone.crystal_autobuy_4"),
    (20, "achievements.milestone.crystal_autobuy_5"),
    (20, "achievements.milestone.speed_rune"),
    (40, "achievements.milestone.duplication_rune"),
    (60, "achievements.milestone.prism_rune"),
    (65, "achievements.milestone.ant_speed_2"),
    (80, "achievements.milestone.thrift_rune"),
    (80, "achievements.milestone.wow_cubes_auto"),
    (80, "achievements.milestone.ascension_score_auto"),
    (100, "achievements.milestone.si_rune"),
    (100, "achievements.milestone.achievement_talisman"),
    (130, "achievements.milestone.rune_autobuy_dx"),
    (160, "achievements.milestone.talisman_enhancement"),
    (225, "achievements.milestone.mortuus_2"),
];

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
        LevelRewards { points: points() }
        div { class: "sf-ach-grid",
            for i in 0..ACHIEVEMENTS_LEN {
                AchievementCell {
                    key: "{i}",
                    index: i,
                    earned: earned()[i] != 0,
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
            NextReward { level }
        }
    }
}

/// The next milestone unlock above the current level (if any).
#[component]
fn NextReward(level: f64) -> Element {
    let next = LEVEL_MILESTONES
        .iter()
        .find(|&&(lv, _)| f64::from(lv) > level);
    rsx! {
        if let Some(&(lv, key)) = next {
            div { class: "sf-ach-next-reward",
                span { class: "sf-ach-req-label", {t("achievements.next_reward")} ": " }
                span { class: "sf-ach-next-lv", {t("achievements.level")} " {lv}" }
                " — "
                {t(key)}
            }
        }
    }
}

/// Collapsible roadmap of all level-milestone unlocks — earned ones lit,
/// upcoming ones muted. Doubles as "what previous levels unlocked".
#[component]
fn LevelRewards(points: f64) -> Element {
    let level = achievement_level_from_points(points);
    rsx! {
        Collapsible { title: t("achievements.level_rewards").to_string(), open: false,
            div { class: "sf-ach-rewards-list",
                for (i, &(lv, key)) in LEVEL_MILESTONES.iter().enumerate() {
                    {
                        let unlocked = level >= f64::from(lv);
                        let cls = if unlocked { "sf-ach-reward-row unlocked" } else { "sf-ach-reward-row" };
                        rsx! {
                            div { key: "{i}", class: cls,
                                span { class: "sf-ach-reward-lv", {t("achievements.level")} " {lv}" }
                                span { class: "sf-ach-reward-name", {t(key)} }
                                span { class: "sf-ach-reward-state",
                                    if unlocked { "✓" } else { "🔒" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// The achievement body for the shared bottom detail panel: `#N - Name
/// [status]`, then the requirement + live progress. Reads earned/progress
/// from state inline (the panel re-renders at tick rate).
#[component]
pub fn AchievementDetailBody(index: usize) -> Element {
    let bridge = use_bridge();
    let points = ACHIEVEMENT_POINT_VALUES[index];
    let is_earned = bridge.state.read().achievements.achievements[index] != 0;
    let status_cls = if is_earned {
        "sf-detail-badge ok"
    } else {
        "sf-detail-badge"
    };
    let status_key = if is_earned {
        "achievements.completed"
    } else {
        "achievements.locked"
    };
    // Live numeric progress (e.g. "34 / 100") for the threshold-backed
    // achievements.
    let notation = bridge.prefs.read().notation;
    let progress = achievement_progress(&bridge.state.read(), index);

    rsx! {
        DetailBody {
            title: achievements_text::name(index).to_string(),
            marker: Some(rsx! { span { class: "sf-detail-marker", "#{index + 1}" } }),
            badge: Some(rsx! { span { class: status_cls, {t(status_key)} } }),
            description: Some(achievements_text::requirement(index).to_string()),
            div { class: "sf-card-row",
                span { class: "label", {t("detail.reward")} }
                span { "{points} {t(\"achievements.ap\")}" }
            }
            div { class: "sf-card-row",
                span { class: "label", {t("achievements.progress")} }
                span {
                    if is_earned {
                        {t("achievements.progress_done")}
                    } else if let Some((current, target)) = progress {
                        {format_value(Decimal::from_finite(current.min(target)), notation)}
                        " / "
                        {format_value(Decimal::from_finite(target), notation)}
                    } else {
                        {t("achievements.progress_pending")}
                    }
                }
            }
        }
    }
}

#[component]
fn AchievementCell(index: usize, earned: bool) -> Element {
    let detail = use_detail();
    let cls = if earned {
        "sf-ach-cell earned"
    } else {
        "sf-ach-cell"
    };
    rsx! {
        div {
            class: cls,
            tabindex: "0",
            onmouseenter: move |_| detail.set(DetailTarget::Achievement(index)),
            onfocus: move |_| detail.set(DetailTarget::Achievement(index)),
            // 1-based label, matching the legacy numbering.
            "{index + 1}"
        }
    }
}
