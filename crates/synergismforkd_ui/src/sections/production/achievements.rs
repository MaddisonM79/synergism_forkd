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
            // Painted medallion where it exists; otherwise the legacy `#N` tag.
            marker: Some(match achievement_icon(index) {
                Some(src) => rsx! {
                    span { class: "sf-icon sf-icon-img",
                        img { src, alt: "", draggable: "false" }
                    }
                },
                None => rsx! { span { class: "sf-detail-marker", "#{index + 1}" } },
            }),
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

/// Painted medallion for an achievement, where art exists (see `tools/icongen/`).
/// `index` is 0-based; the file is `ach<index+1>.png`. Cells without art fall
/// back to the index label. Currently achievements 1–50.
fn achievement_icon(index: usize) -> Option<Asset> {
    Some(match index {
        0 => asset!("/assets/pictures/achievement/ach1.png"),
        1 => asset!("/assets/pictures/achievement/ach2.png"),
        2 => asset!("/assets/pictures/achievement/ach3.png"),
        3 => asset!("/assets/pictures/achievement/ach4.png"),
        4 => asset!("/assets/pictures/achievement/ach5.png"),
        5 => asset!("/assets/pictures/achievement/ach6.png"),
        6 => asset!("/assets/pictures/achievement/ach7.png"),
        7 => asset!("/assets/pictures/achievement/ach8.png"),
        8 => asset!("/assets/pictures/achievement/ach9.png"),
        9 => asset!("/assets/pictures/achievement/ach10.png"),
        10 => asset!("/assets/pictures/achievement/ach11.png"),
        11 => asset!("/assets/pictures/achievement/ach12.png"),
        12 => asset!("/assets/pictures/achievement/ach13.png"),
        13 => asset!("/assets/pictures/achievement/ach14.png"),
        14 => asset!("/assets/pictures/achievement/ach15.png"),
        15 => asset!("/assets/pictures/achievement/ach16.png"),
        16 => asset!("/assets/pictures/achievement/ach17.png"),
        17 => asset!("/assets/pictures/achievement/ach18.png"),
        18 => asset!("/assets/pictures/achievement/ach19.png"),
        19 => asset!("/assets/pictures/achievement/ach20.png"),
        20 => asset!("/assets/pictures/achievement/ach21.png"),
        21 => asset!("/assets/pictures/achievement/ach22.png"),
        22 => asset!("/assets/pictures/achievement/ach23.png"),
        23 => asset!("/assets/pictures/achievement/ach24.png"),
        24 => asset!("/assets/pictures/achievement/ach25.png"),
        25 => asset!("/assets/pictures/achievement/ach26.png"),
        26 => asset!("/assets/pictures/achievement/ach27.png"),
        27 => asset!("/assets/pictures/achievement/ach28.png"),
        28 => asset!("/assets/pictures/achievement/ach29.png"),
        29 => asset!("/assets/pictures/achievement/ach30.png"),
        30 => asset!("/assets/pictures/achievement/ach31.png"),
        31 => asset!("/assets/pictures/achievement/ach32.png"),
        32 => asset!("/assets/pictures/achievement/ach33.png"),
        33 => asset!("/assets/pictures/achievement/ach34.png"),
        34 => asset!("/assets/pictures/achievement/ach35.png"),
        35 => asset!("/assets/pictures/achievement/ach36.png"),
        36 => asset!("/assets/pictures/achievement/ach37.png"),
        37 => asset!("/assets/pictures/achievement/ach38.png"),
        38 => asset!("/assets/pictures/achievement/ach39.png"),
        39 => asset!("/assets/pictures/achievement/ach40.png"),
        40 => asset!("/assets/pictures/achievement/ach41.png"),
        41 => asset!("/assets/pictures/achievement/ach42.png"),
        42 => asset!("/assets/pictures/achievement/ach43.png"),
        43 => asset!("/assets/pictures/achievement/ach44.png"),
        44 => asset!("/assets/pictures/achievement/ach45.png"),
        45 => asset!("/assets/pictures/achievement/ach46.png"),
        46 => asset!("/assets/pictures/achievement/ach47.png"),
        47 => asset!("/assets/pictures/achievement/ach48.png"),
        48 => asset!("/assets/pictures/achievement/ach49.png"),
        49 => asset!("/assets/pictures/achievement/ach50.png"),
        50 => asset!("/assets/pictures/achievement/ach51.png"),
        51 => asset!("/assets/pictures/achievement/ach52.png"),
        52 => asset!("/assets/pictures/achievement/ach53.png"),
        53 => asset!("/assets/pictures/achievement/ach54.png"),
        54 => asset!("/assets/pictures/achievement/ach55.png"),
        55 => asset!("/assets/pictures/achievement/ach56.png"),
        56 => asset!("/assets/pictures/achievement/ach57.png"),
        57 => asset!("/assets/pictures/achievement/ach58.png"),
        58 => asset!("/assets/pictures/achievement/ach59.png"),
        59 => asset!("/assets/pictures/achievement/ach60.png"),
        60 => asset!("/assets/pictures/achievement/ach61.png"),
        61 => asset!("/assets/pictures/achievement/ach62.png"),
        62 => asset!("/assets/pictures/achievement/ach63.png"),
        63 => asset!("/assets/pictures/achievement/ach64.png"),
        64 => asset!("/assets/pictures/achievement/ach65.png"),
        65 => asset!("/assets/pictures/achievement/ach66.png"),
        66 => asset!("/assets/pictures/achievement/ach67.png"),
        67 => asset!("/assets/pictures/achievement/ach68.png"),
        68 => asset!("/assets/pictures/achievement/ach69.png"),
        69 => asset!("/assets/pictures/achievement/ach70.png"),
        70 => asset!("/assets/pictures/achievement/ach71.png"),
        71 => asset!("/assets/pictures/achievement/ach72.png"),
        72 => asset!("/assets/pictures/achievement/ach73.png"),
        73 => asset!("/assets/pictures/achievement/ach74.png"),
        74 => asset!("/assets/pictures/achievement/ach75.png"),
        75 => asset!("/assets/pictures/achievement/ach76.png"),
        76 => asset!("/assets/pictures/achievement/ach77.png"),
        77 => asset!("/assets/pictures/achievement/ach78.png"),
        78 => asset!("/assets/pictures/achievement/ach79.png"),
        79 => asset!("/assets/pictures/achievement/ach80.png"),
        80 => asset!("/assets/pictures/achievement/ach81.png"),
        81 => asset!("/assets/pictures/achievement/ach82.png"),
        82 => asset!("/assets/pictures/achievement/ach83.png"),
        83 => asset!("/assets/pictures/achievement/ach84.png"),
        84 => asset!("/assets/pictures/achievement/ach85.png"),
        85 => asset!("/assets/pictures/achievement/ach86.png"),
        86 => asset!("/assets/pictures/achievement/ach87.png"),
        87 => asset!("/assets/pictures/achievement/ach88.png"),
        88 => asset!("/assets/pictures/achievement/ach89.png"),
        89 => asset!("/assets/pictures/achievement/ach90.png"),
        90 => asset!("/assets/pictures/achievement/ach91.png"),
        91 => asset!("/assets/pictures/achievement/ach92.png"),
        92 => asset!("/assets/pictures/achievement/ach93.png"),
        93 => asset!("/assets/pictures/achievement/ach94.png"),
        94 => asset!("/assets/pictures/achievement/ach95.png"),
        95 => asset!("/assets/pictures/achievement/ach96.png"),
        96 => asset!("/assets/pictures/achievement/ach97.png"),
        97 => asset!("/assets/pictures/achievement/ach98.png"),
        98 => asset!("/assets/pictures/achievement/ach99.png"),
        99 => asset!("/assets/pictures/achievement/ach100.png"),
        100 => asset!("/assets/pictures/achievement/ach101.png"),
        101 => asset!("/assets/pictures/achievement/ach102.png"),
        102 => asset!("/assets/pictures/achievement/ach103.png"),
        103 => asset!("/assets/pictures/achievement/ach104.png"),
        104 => asset!("/assets/pictures/achievement/ach105.png"),
        105 => asset!("/assets/pictures/achievement/ach106.png"),
        106 => asset!("/assets/pictures/achievement/ach107.png"),
        107 => asset!("/assets/pictures/achievement/ach108.png"),
        108 => asset!("/assets/pictures/achievement/ach109.png"),
        109 => asset!("/assets/pictures/achievement/ach110.png"),
        110 => asset!("/assets/pictures/achievement/ach111.png"),
        111 => asset!("/assets/pictures/achievement/ach112.png"),
        112 => asset!("/assets/pictures/achievement/ach113.png"),
        113 => asset!("/assets/pictures/achievement/ach114.png"),
        114 => asset!("/assets/pictures/achievement/ach115.png"),
        115 => asset!("/assets/pictures/achievement/ach116.png"),
        116 => asset!("/assets/pictures/achievement/ach117.png"),
        117 => asset!("/assets/pictures/achievement/ach118.png"),
        118 => asset!("/assets/pictures/achievement/ach119.png"),
        119 => asset!("/assets/pictures/achievement/ach120.png"),
        120 => asset!("/assets/pictures/achievement/ach121.png"),
        121 => asset!("/assets/pictures/achievement/ach122.png"),
        122 => asset!("/assets/pictures/achievement/ach123.png"),
        123 => asset!("/assets/pictures/achievement/ach124.png"),
        124 => asset!("/assets/pictures/achievement/ach125.png"),
        125 => asset!("/assets/pictures/achievement/ach126.png"),
        126 => asset!("/assets/pictures/achievement/ach127.png"),
        127 => asset!("/assets/pictures/achievement/ach128.png"),
        128 => asset!("/assets/pictures/achievement/ach129.png"),
        129 => asset!("/assets/pictures/achievement/ach130.png"),
        130 => asset!("/assets/pictures/achievement/ach131.png"),
        131 => asset!("/assets/pictures/achievement/ach132.png"),
        132 => asset!("/assets/pictures/achievement/ach133.png"),
        133 => asset!("/assets/pictures/achievement/ach134.png"),
        134 => asset!("/assets/pictures/achievement/ach135.png"),
        135 => asset!("/assets/pictures/achievement/ach136.png"),
        136 => asset!("/assets/pictures/achievement/ach137.png"),
        137 => asset!("/assets/pictures/achievement/ach138.png"),
        138 => asset!("/assets/pictures/achievement/ach139.png"),
        139 => asset!("/assets/pictures/achievement/ach140.png"),
        140 => asset!("/assets/pictures/achievement/ach141.png"),
        141 => asset!("/assets/pictures/achievement/ach142.png"),
        142 => asset!("/assets/pictures/achievement/ach143.png"),
        143 => asset!("/assets/pictures/achievement/ach144.png"),
        144 => asset!("/assets/pictures/achievement/ach145.png"),
        145 => asset!("/assets/pictures/achievement/ach146.png"),
        146 => asset!("/assets/pictures/achievement/ach147.png"),
        147 => asset!("/assets/pictures/achievement/ach148.png"),
        148 => asset!("/assets/pictures/achievement/ach149.png"),
        149 => asset!("/assets/pictures/achievement/ach150.png"),
        150 => asset!("/assets/pictures/achievement/ach151.png"),
        151 => asset!("/assets/pictures/achievement/ach152.png"),
        152 => asset!("/assets/pictures/achievement/ach153.png"),
        153 => asset!("/assets/pictures/achievement/ach154.png"),
        154 => asset!("/assets/pictures/achievement/ach155.png"),
        155 => asset!("/assets/pictures/achievement/ach156.png"),
        156 => asset!("/assets/pictures/achievement/ach157.png"),
        157 => asset!("/assets/pictures/achievement/ach158.png"),
        158 => asset!("/assets/pictures/achievement/ach159.png"),
        159 => asset!("/assets/pictures/achievement/ach160.png"),
        160 => asset!("/assets/pictures/achievement/ach161.png"),
        161 => asset!("/assets/pictures/achievement/ach162.png"),
        162 => asset!("/assets/pictures/achievement/ach163.png"),
        163 => asset!("/assets/pictures/achievement/ach164.png"),
        164 => asset!("/assets/pictures/achievement/ach165.png"),
        165 => asset!("/assets/pictures/achievement/ach166.png"),
        166 => asset!("/assets/pictures/achievement/ach167.png"),
        167 => asset!("/assets/pictures/achievement/ach168.png"),
        168 => asset!("/assets/pictures/achievement/ach169.png"),
        169 => asset!("/assets/pictures/achievement/ach170.png"),
        170 => asset!("/assets/pictures/achievement/ach171.png"),
        171 => asset!("/assets/pictures/achievement/ach172.png"),
        172 => asset!("/assets/pictures/achievement/ach173.png"),
        173 => asset!("/assets/pictures/achievement/ach174.png"),
        174 => asset!("/assets/pictures/achievement/ach175.png"),
        175 => asset!("/assets/pictures/achievement/ach176.png"),
        176 => asset!("/assets/pictures/achievement/ach177.png"),
        177 => asset!("/assets/pictures/achievement/ach178.png"),
        178 => asset!("/assets/pictures/achievement/ach179.png"),
        179 => asset!("/assets/pictures/achievement/ach180.png"),
        180 => asset!("/assets/pictures/achievement/ach181.png"),
        181 => asset!("/assets/pictures/achievement/ach182.png"),
        182 => asset!("/assets/pictures/achievement/ach183.png"),
        183 => asset!("/assets/pictures/achievement/ach184.png"),
        184 => asset!("/assets/pictures/achievement/ach185.png"),
        185 => asset!("/assets/pictures/achievement/ach186.png"),
        186 => asset!("/assets/pictures/achievement/ach187.png"),
        187 => asset!("/assets/pictures/achievement/ach188.png"),
        188 => asset!("/assets/pictures/achievement/ach189.png"),
        189 => asset!("/assets/pictures/achievement/ach190.png"),
        190 => asset!("/assets/pictures/achievement/ach191.png"),
        191 => asset!("/assets/pictures/achievement/ach192.png"),
        192 => asset!("/assets/pictures/achievement/ach193.png"),
        193 => asset!("/assets/pictures/achievement/ach194.png"),
        194 => asset!("/assets/pictures/achievement/ach195.png"),
        195 => asset!("/assets/pictures/achievement/ach196.png"),
        196 => asset!("/assets/pictures/achievement/ach197.png"),
        197 => asset!("/assets/pictures/achievement/ach198.png"),
        198 => asset!("/assets/pictures/achievement/ach199.png"),
        199 => asset!("/assets/pictures/achievement/ach200.png"),
        200 => asset!("/assets/pictures/achievement/ach201.png"),
        201 => asset!("/assets/pictures/achievement/ach202.png"),
        202 => asset!("/assets/pictures/achievement/ach203.png"),
        203 => asset!("/assets/pictures/achievement/ach204.png"),
        204 => asset!("/assets/pictures/achievement/ach205.png"),
        205 => asset!("/assets/pictures/achievement/ach206.png"),
        206 => asset!("/assets/pictures/achievement/ach207.png"),
        207 => asset!("/assets/pictures/achievement/ach208.png"),
        208 => asset!("/assets/pictures/achievement/ach209.png"),
        209 => asset!("/assets/pictures/achievement/ach210.png"),
        210 => asset!("/assets/pictures/achievement/ach211.png"),
        211 => asset!("/assets/pictures/achievement/ach212.png"),
        212 => asset!("/assets/pictures/achievement/ach213.png"),
        213 => asset!("/assets/pictures/achievement/ach214.png"),
        214 => asset!("/assets/pictures/achievement/ach215.png"),
        215 => asset!("/assets/pictures/achievement/ach216.png"),
        216 => asset!("/assets/pictures/achievement/ach217.png"),
        217 => asset!("/assets/pictures/achievement/ach218.png"),
        218 => asset!("/assets/pictures/achievement/ach219.png"),
        219 => asset!("/assets/pictures/achievement/ach220.png"),
        220 => asset!("/assets/pictures/achievement/ach221.png"),
        221 => asset!("/assets/pictures/achievement/ach222.png"),
        222 => asset!("/assets/pictures/achievement/ach223.png"),
        223 => asset!("/assets/pictures/achievement/ach224.png"),
        224 => asset!("/assets/pictures/achievement/ach225.png"),
        225 => asset!("/assets/pictures/achievement/ach226.png"),
        226 => asset!("/assets/pictures/achievement/ach227.png"),
        227 => asset!("/assets/pictures/achievement/ach228.png"),
        228 => asset!("/assets/pictures/achievement/ach229.png"),
        229 => asset!("/assets/pictures/achievement/ach230.png"),
        230 => asset!("/assets/pictures/achievement/ach231.png"),
        231 => asset!("/assets/pictures/achievement/ach232.png"),
        232 => asset!("/assets/pictures/achievement/ach233.png"),
        233 => asset!("/assets/pictures/achievement/ach234.png"),
        234 => asset!("/assets/pictures/achievement/ach235.png"),
        235 => asset!("/assets/pictures/achievement/ach236.png"),
        236 => asset!("/assets/pictures/achievement/ach237.png"),
        237 => asset!("/assets/pictures/achievement/ach238.png"),
        238 => asset!("/assets/pictures/achievement/ach239.png"),
        239 => asset!("/assets/pictures/achievement/ach240.png"),
        240 => asset!("/assets/pictures/achievement/ach241.png"),
        241 => asset!("/assets/pictures/achievement/ach242.png"),
        242 => asset!("/assets/pictures/achievement/ach243.png"),
        243 => asset!("/assets/pictures/achievement/ach244.png"),
        244 => asset!("/assets/pictures/achievement/ach245.png"),
        245 => asset!("/assets/pictures/achievement/ach246.png"),
        246 => asset!("/assets/pictures/achievement/ach247.png"),
        247 => asset!("/assets/pictures/achievement/ach248.png"),
        248 => asset!("/assets/pictures/achievement/ach249.png"),
        249 => asset!("/assets/pictures/achievement/ach250.png"),
        250 => asset!("/assets/pictures/achievement/ach251.png"),
        251 => asset!("/assets/pictures/achievement/ach252.png"),
        252 => asset!("/assets/pictures/achievement/ach253.png"),
        253 => asset!("/assets/pictures/achievement/ach254.png"),
        254 => asset!("/assets/pictures/achievement/ach255.png"),
        255 => asset!("/assets/pictures/achievement/ach256.png"),
        256 => asset!("/assets/pictures/achievement/ach257.png"),
        257 => asset!("/assets/pictures/achievement/ach258.png"),
        258 => asset!("/assets/pictures/achievement/ach259.png"),
        259 => asset!("/assets/pictures/achievement/ach260.png"),
        260 => asset!("/assets/pictures/achievement/ach261.png"),
        261 => asset!("/assets/pictures/achievement/ach262.png"),
        262 => asset!("/assets/pictures/achievement/ach263.png"),
        263 => asset!("/assets/pictures/achievement/ach264.png"),
        264 => asset!("/assets/pictures/achievement/ach265.png"),
        265 => asset!("/assets/pictures/achievement/ach266.png"),
        266 => asset!("/assets/pictures/achievement/ach267.png"),
        267 => asset!("/assets/pictures/achievement/ach268.png"),
        268 => asset!("/assets/pictures/achievement/ach269.png"),
        269 => asset!("/assets/pictures/achievement/ach270.png"),
        270 => asset!("/assets/pictures/achievement/ach271.png"),
        271 => asset!("/assets/pictures/achievement/ach272.png"),
        272 => asset!("/assets/pictures/achievement/ach273.png"),
        273 => asset!("/assets/pictures/achievement/ach274.png"),
        274 => asset!("/assets/pictures/achievement/ach275.png"),
        275 => asset!("/assets/pictures/achievement/ach276.png"),
        276 => asset!("/assets/pictures/achievement/ach277.png"),
        277 => asset!("/assets/pictures/achievement/ach278.png"),
        278 => asset!("/assets/pictures/achievement/ach279.png"),
        279 => asset!("/assets/pictures/achievement/ach280.png"),
        280 => asset!("/assets/pictures/achievement/ach281.png"),
        281 => asset!("/assets/pictures/achievement/ach282.png"),
        282 => asset!("/assets/pictures/achievement/ach283.png"),
        283 => asset!("/assets/pictures/achievement/ach284.png"),
        284 => asset!("/assets/pictures/achievement/ach285.png"),
        285 => asset!("/assets/pictures/achievement/ach286.png"),
        286 => asset!("/assets/pictures/achievement/ach287.png"),
        287 => asset!("/assets/pictures/achievement/ach288.png"),
        288 => asset!("/assets/pictures/achievement/ach289.png"),
        289 => asset!("/assets/pictures/achievement/ach290.png"),
        290 => asset!("/assets/pictures/achievement/ach291.png"),
        291 => asset!("/assets/pictures/achievement/ach292.png"),
        292 => asset!("/assets/pictures/achievement/ach293.png"),
        293 => asset!("/assets/pictures/achievement/ach294.png"),
        294 => asset!("/assets/pictures/achievement/ach295.png"),
        295 => asset!("/assets/pictures/achievement/ach296.png"),
        296 => asset!("/assets/pictures/achievement/ach297.png"),
        297 => asset!("/assets/pictures/achievement/ach298.png"),
        298 => asset!("/assets/pictures/achievement/ach299.png"),
        299 => asset!("/assets/pictures/achievement/ach300.png"),
        300 => asset!("/assets/pictures/achievement/ach301.png"),
        301 => asset!("/assets/pictures/achievement/ach302.png"),
        302 => asset!("/assets/pictures/achievement/ach303.png"),
        303 => asset!("/assets/pictures/achievement/ach304.png"),
        304 => asset!("/assets/pictures/achievement/ach305.png"),
        305 => asset!("/assets/pictures/achievement/ach306.png"),
        306 => asset!("/assets/pictures/achievement/ach307.png"),
        307 => asset!("/assets/pictures/achievement/ach308.png"),
        308 => asset!("/assets/pictures/achievement/ach309.png"),
        309 => asset!("/assets/pictures/achievement/ach310.png"),
        310 => asset!("/assets/pictures/achievement/ach311.png"),
        311 => asset!("/assets/pictures/achievement/ach312.png"),
        312 => asset!("/assets/pictures/achievement/ach313.png"),
        313 => asset!("/assets/pictures/achievement/ach314.png"),
        314 => asset!("/assets/pictures/achievement/ach315.png"),
        315 => asset!("/assets/pictures/achievement/ach316.png"),
        316 => asset!("/assets/pictures/achievement/ach317.png"),
        317 => asset!("/assets/pictures/achievement/ach318.png"),
        318 => asset!("/assets/pictures/achievement/ach319.png"),
        319 => asset!("/assets/pictures/achievement/ach320.png"),
        320 => asset!("/assets/pictures/achievement/ach321.png"),
        321 => asset!("/assets/pictures/achievement/ach322.png"),
        322 => asset!("/assets/pictures/achievement/ach323.png"),
        323 => asset!("/assets/pictures/achievement/ach324.png"),
        324 => asset!("/assets/pictures/achievement/ach325.png"),
        325 => asset!("/assets/pictures/achievement/ach326.png"),
        326 => asset!("/assets/pictures/achievement/ach327.png"),
        327 => asset!("/assets/pictures/achievement/ach328.png"),
        328 => asset!("/assets/pictures/achievement/ach329.png"),
        329 => asset!("/assets/pictures/achievement/ach330.png"),
        330 => asset!("/assets/pictures/achievement/ach331.png"),
        331 => asset!("/assets/pictures/achievement/ach332.png"),
        332 => asset!("/assets/pictures/achievement/ach333.png"),
        333 => asset!("/assets/pictures/achievement/ach334.png"),
        334 => asset!("/assets/pictures/achievement/ach335.png"),
        335 => asset!("/assets/pictures/achievement/ach336.png"),
        336 => asset!("/assets/pictures/achievement/ach337.png"),
        337 => asset!("/assets/pictures/achievement/ach338.png"),
        338 => asset!("/assets/pictures/achievement/ach339.png"),
        339 => asset!("/assets/pictures/achievement/ach340.png"),
        340 => asset!("/assets/pictures/achievement/ach341.png"),
        341 => asset!("/assets/pictures/achievement/ach342.png"),
        342 => asset!("/assets/pictures/achievement/ach343.png"),
        343 => asset!("/assets/pictures/achievement/ach344.png"),
        344 => asset!("/assets/pictures/achievement/ach345.png"),
        345 => asset!("/assets/pictures/achievement/ach346.png"),
        346 => asset!("/assets/pictures/achievement/ach347.png"),
        347 => asset!("/assets/pictures/achievement/ach348.png"),
        348 => asset!("/assets/pictures/achievement/ach349.png"),
        349 => asset!("/assets/pictures/achievement/ach350.png"),
        350 => asset!("/assets/pictures/achievement/ach351.png"),
        351 => asset!("/assets/pictures/achievement/ach352.png"),
        352 => asset!("/assets/pictures/achievement/ach353.png"),
        353 => asset!("/assets/pictures/achievement/ach354.png"),
        354 => asset!("/assets/pictures/achievement/ach355.png"),
        355 => asset!("/assets/pictures/achievement/ach356.png"),
        356 => asset!("/assets/pictures/achievement/ach357.png"),
        357 => asset!("/assets/pictures/achievement/ach358.png"),
        358 => asset!("/assets/pictures/achievement/ach359.png"),
        359 => asset!("/assets/pictures/achievement/ach360.png"),
        360 => asset!("/assets/pictures/achievement/ach361.png"),
        361 => asset!("/assets/pictures/achievement/ach362.png"),
        362 => asset!("/assets/pictures/achievement/ach363.png"),
        363 => asset!("/assets/pictures/achievement/ach364.png"),
        364 => asset!("/assets/pictures/achievement/ach365.png"),
        365 => asset!("/assets/pictures/achievement/ach366.png"),
        366 => asset!("/assets/pictures/achievement/ach367.png"),
        367 => asset!("/assets/pictures/achievement/ach368.png"),
        368 => asset!("/assets/pictures/achievement/ach369.png"),
        369 => asset!("/assets/pictures/achievement/ach370.png"),
        370 => asset!("/assets/pictures/achievement/ach371.png"),
        371 => asset!("/assets/pictures/achievement/ach372.png"),
        372 => asset!("/assets/pictures/achievement/ach373.png"),
        373 => asset!("/assets/pictures/achievement/ach374.png"),
        374 => asset!("/assets/pictures/achievement/ach375.png"),
        375 => asset!("/assets/pictures/achievement/ach376.png"),
        376 => asset!("/assets/pictures/achievement/ach377.png"),
        377 => asset!("/assets/pictures/achievement/ach378.png"),
        378 => asset!("/assets/pictures/achievement/ach379.png"),
        379 => asset!("/assets/pictures/achievement/ach380.png"),
        380 => asset!("/assets/pictures/achievement/ach381.png"),
        381 => asset!("/assets/pictures/achievement/ach382.png"),
        382 => asset!("/assets/pictures/achievement/ach383.png"),
        383 => asset!("/assets/pictures/achievement/ach384.png"),
        384 => asset!("/assets/pictures/achievement/ach385.png"),
        385 => asset!("/assets/pictures/achievement/ach386.png"),
        386 => asset!("/assets/pictures/achievement/ach387.png"),
        387 => asset!("/assets/pictures/achievement/ach388.png"),
        388 => asset!("/assets/pictures/achievement/ach389.png"),
        389 => asset!("/assets/pictures/achievement/ach390.png"),
        390 => asset!("/assets/pictures/achievement/ach391.png"),
        391 => asset!("/assets/pictures/achievement/ach392.png"),
        392 => asset!("/assets/pictures/achievement/ach393.png"),
        393 => asset!("/assets/pictures/achievement/ach394.png"),
        394 => asset!("/assets/pictures/achievement/ach395.png"),
        395 => asset!("/assets/pictures/achievement/ach396.png"),
        396 => asset!("/assets/pictures/achievement/ach397.png"),
        397 => asset!("/assets/pictures/achievement/ach398.png"),
        398 => asset!("/assets/pictures/achievement/ach399.png"),
        399 => asset!("/assets/pictures/achievement/ach400.png"),
        400 => asset!("/assets/pictures/achievement/ach401.png"),
        401 => asset!("/assets/pictures/achievement/ach402.png"),
        402 => asset!("/assets/pictures/achievement/ach403.png"),
        403 => asset!("/assets/pictures/achievement/ach404.png"),
        404 => asset!("/assets/pictures/achievement/ach405.png"),
        405 => asset!("/assets/pictures/achievement/ach406.png"),
        406 => asset!("/assets/pictures/achievement/ach407.png"),
        407 => asset!("/assets/pictures/achievement/ach408.png"),
        408 => asset!("/assets/pictures/achievement/ach409.png"),
        409 => asset!("/assets/pictures/achievement/ach410.png"),
        410 => asset!("/assets/pictures/achievement/ach411.png"),
        411 => asset!("/assets/pictures/achievement/ach412.png"),
        412 => asset!("/assets/pictures/achievement/ach413.png"),
        413 => asset!("/assets/pictures/achievement/ach414.png"),
        414 => asset!("/assets/pictures/achievement/ach415.png"),
        415 => asset!("/assets/pictures/achievement/ach416.png"),
        416 => asset!("/assets/pictures/achievement/ach417.png"),
        417 => asset!("/assets/pictures/achievement/ach418.png"),
        418 => asset!("/assets/pictures/achievement/ach419.png"),
        419 => asset!("/assets/pictures/achievement/ach420.png"),
        420 => asset!("/assets/pictures/achievement/ach421.png"),
        421 => asset!("/assets/pictures/achievement/ach422.png"),
        422 => asset!("/assets/pictures/achievement/ach423.png"),
        423 => asset!("/assets/pictures/achievement/ach424.png"),
        424 => asset!("/assets/pictures/achievement/ach425.png"),
        425 => asset!("/assets/pictures/achievement/ach426.png"),
        426 => asset!("/assets/pictures/achievement/ach427.png"),
        427 => asset!("/assets/pictures/achievement/ach428.png"),
        428 => asset!("/assets/pictures/achievement/ach429.png"),
        429 => asset!("/assets/pictures/achievement/ach430.png"),
        430 => asset!("/assets/pictures/achievement/ach431.png"),
        431 => asset!("/assets/pictures/achievement/ach432.png"),
        432 => asset!("/assets/pictures/achievement/ach433.png"),
        433 => asset!("/assets/pictures/achievement/ach434.png"),
        434 => asset!("/assets/pictures/achievement/ach435.png"),
        435 => asset!("/assets/pictures/achievement/ach436.png"),
        436 => asset!("/assets/pictures/achievement/ach437.png"),
        437 => asset!("/assets/pictures/achievement/ach438.png"),
        438 => asset!("/assets/pictures/achievement/ach439.png"),
        439 => asset!("/assets/pictures/achievement/ach440.png"),
        440 => asset!("/assets/pictures/achievement/ach441.png"),
        441 => asset!("/assets/pictures/achievement/ach442.png"),
        442 => asset!("/assets/pictures/achievement/ach443.png"),
        443 => asset!("/assets/pictures/achievement/ach444.png"),
        444 => asset!("/assets/pictures/achievement/ach445.png"),
        445 => asset!("/assets/pictures/achievement/ach446.png"),
        446 => asset!("/assets/pictures/achievement/ach447.png"),
        447 => asset!("/assets/pictures/achievement/ach448.png"),
        448 => asset!("/assets/pictures/achievement/ach449.png"),
        449 => asset!("/assets/pictures/achievement/ach450.png"),
        _ => return None,
    })
}

#[component]
fn AchievementCell(index: usize, earned: bool) -> Element {
    let detail = use_detail();
    let icon = achievement_icon(index);
    // `has-icon` lets the stylesheet swap the gold "earned" fill for a thin
    // gold border (so the painted medallion isn't washed out) and dim the
    // locked ones.
    let cls = match (icon.is_some(), earned) {
        (true, true) => "sf-ach-cell has-icon earned",
        (true, false) => "sf-ach-cell has-icon",
        (false, true) => "sf-ach-cell earned",
        (false, false) => "sf-ach-cell",
    };
    rsx! {
        div {
            class: cls,
            tabindex: "0",
            onmouseenter: move |_| detail.set(DetailTarget::Achievement(index)),
            onfocus: move |_| detail.set(DetailTarget::Achievement(index)),
            if let Some(src) = icon {
                img { class: "sf-ach-icon", src, alt: "", draggable: "false" }
            } else {
                // 1-based label, matching the legacy numbering.
                "{index + 1}"
            }
        }
    }
}
