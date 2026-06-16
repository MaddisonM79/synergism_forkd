//! Upgrades: the six shop sections (Coin / Diamond / Mythos / Particle /
//! Automation / Generator), each a [`Collapsible`] holding a dense grid of
//! numbered squares (mirrors the Achievements screen). A shared detail card at
//! the top describes the hovered/focused upgrade; clicking a square buys it.
//! Shop/reveal metadata lives in [`super::upgrade_data`]; live effect values in
//! [`super::upgrade_effects`].

use dioxus::prelude::*;
use synergismforkd_logic::{AutoToggle, PlayerAction};

use crate::bridge::{use_bridge, use_slice, use_slow_slice};
use crate::components::{Collapsible, Num, ResourceIcon};
use crate::derive;
use crate::detail::{use_detail, DetailBody, DetailTarget};
use crate::i18n::{t, t_args};

use super::upgrade_data::{meta, shop_upgrades, Shop};

#[component]
pub fn Upgrades() -> Element {
    // The shops with at least one revealed upgrade (reactive to unlocks). Coin
    // upgrades 1–5 are always revealed, so the Coin shop shows from the start —
    // the empty state is a defensive fallback only.
    let visible_shops = use_slice(|s| {
        Shop::ALL
            .into_iter()
            .filter(|&shop| shop_upgrades(shop).any(|m| m.revealed(s)))
            .collect::<Vec<_>>()
    });

    rsx! {
        div { class: "sf-section-head",
            h1 { {t("nav.section.upgrades")} }
        }
        if visible_shops().is_empty() {
            div { class: "sf-empty-state",
                div { class: "sf-empty-title", {t("upgrades.empty_title")} }
                div { class: "sf-empty-msg", {t("upgrades.empty_msg")} }
            }
        }
        for shop in visible_shops() {
            Collapsible {
                key: "{shop:?}",
                title: t(shop.title_key()).to_string(),
                action: rsx! { ShopAutoToggle { shop } },
                ShopGrid { shop }
            }
        }
    }
}

/// Per-shop autobuy toggle, shown on the section header once that family's
/// upgrade autobuyer is unlocked (its unlock upgrade is owned). Flips the
/// matching `shop_toggles` field that `auto_buy::auto_upgrades` reads. Renders
/// nothing for shops with no autobuyer (Automation) or while still locked.
#[component]
fn ShopAutoToggle(shop: Shop) -> Element {
    // All hooks run unconditionally (rules of hooks); the early returns below
    // are placed after every hook. `kind` is fixed per instance (the
    // Collapsible is keyed by shop); `unlocked` / `on` may change at runtime.
    let bridge = use_bridge();
    let kind = shop.autobuy_kind();
    let unlocked = use_slice(move |s| shop.autobuy_unlocked(s));
    let on = use_slice(move |s| kind.is_some_and(|k| s.automation.shop_toggles.get(k)));

    let Some(kind) = kind else {
        return rsx! {};
    };
    if !unlocked() {
        return rsx! {};
    }
    rsx! {
        button {
            class: if on() { "sf-auto-toggle on" } else { "sf-auto-toggle" },
            "aria-pressed": "{on()}",
            onclick: move |_| {
                bridge.dispatch(PlayerAction::ToggleAuto {
                    target: AutoToggle::ShopAutobuy(kind),
                    enabled: !on(),
                });
            },
            {t(if on() { "buildings.auto_on" } else { "buildings.auto_off" })}
        }
    }
}

/// The revealed upgrade squares for one shop, in ascending index (= legacy row)
/// order.
#[component]
fn ShopGrid(shop: Shop) -> Element {
    let indices = use_slice(move |s| {
        shop_upgrades(shop)
            .filter(|m| m.revealed(s))
            .map(|m| m.idx)
            .collect::<Vec<_>>()
    });
    rsx! {
        div { class: "sf-upg-grid",
            for idx in indices() {
                UpgradeCell { key: "{idx}", idx }
            }
        }
    }
}

/// Painted icon for an upgrade square, where art exists (see `tools/icongen/`).
/// Covers the Coin (1–20, 121–125), Diamond (21–40), Automation (81–100), and
/// Generator (101–120) shops. Mythos/Particle and any gap fall back to the
/// index number. Files are `upgrade<idx>.png`, named for the bitmap index.
fn upgrade_icon(idx: usize) -> Option<Asset> {
    Some(match idx {
        1 => asset!("/assets/pictures/upgrade/upgrade1.png"),
        2 => asset!("/assets/pictures/upgrade/upgrade2.png"),
        3 => asset!("/assets/pictures/upgrade/upgrade3.png"),
        4 => asset!("/assets/pictures/upgrade/upgrade4.png"),
        5 => asset!("/assets/pictures/upgrade/upgrade5.png"),
        6 => asset!("/assets/pictures/upgrade/upgrade6.png"),
        7 => asset!("/assets/pictures/upgrade/upgrade7.png"),
        8 => asset!("/assets/pictures/upgrade/upgrade8.png"),
        9 => asset!("/assets/pictures/upgrade/upgrade9.png"),
        10 => asset!("/assets/pictures/upgrade/upgrade10.png"),
        11 => asset!("/assets/pictures/upgrade/upgrade11.png"),
        12 => asset!("/assets/pictures/upgrade/upgrade12.png"),
        13 => asset!("/assets/pictures/upgrade/upgrade13.png"),
        14 => asset!("/assets/pictures/upgrade/upgrade14.png"),
        15 => asset!("/assets/pictures/upgrade/upgrade15.png"),
        16 => asset!("/assets/pictures/upgrade/upgrade16.png"),
        17 => asset!("/assets/pictures/upgrade/upgrade17.png"),
        18 => asset!("/assets/pictures/upgrade/upgrade18.png"),
        19 => asset!("/assets/pictures/upgrade/upgrade19.png"),
        20 => asset!("/assets/pictures/upgrade/upgrade20.png"),
        21 => asset!("/assets/pictures/upgrade/upgrade21.png"),
        22 => asset!("/assets/pictures/upgrade/upgrade22.png"),
        23 => asset!("/assets/pictures/upgrade/upgrade23.png"),
        24 => asset!("/assets/pictures/upgrade/upgrade24.png"),
        25 => asset!("/assets/pictures/upgrade/upgrade25.png"),
        26 => asset!("/assets/pictures/upgrade/upgrade26.png"),
        27 => asset!("/assets/pictures/upgrade/upgrade27.png"),
        28 => asset!("/assets/pictures/upgrade/upgrade28.png"),
        29 => asset!("/assets/pictures/upgrade/upgrade29.png"),
        30 => asset!("/assets/pictures/upgrade/upgrade30.png"),
        31 => asset!("/assets/pictures/upgrade/upgrade31.png"),
        32 => asset!("/assets/pictures/upgrade/upgrade32.png"),
        33 => asset!("/assets/pictures/upgrade/upgrade33.png"),
        34 => asset!("/assets/pictures/upgrade/upgrade34.png"),
        35 => asset!("/assets/pictures/upgrade/upgrade35.png"),
        36 => asset!("/assets/pictures/upgrade/upgrade36.png"),
        37 => asset!("/assets/pictures/upgrade/upgrade37.png"),
        38 => asset!("/assets/pictures/upgrade/upgrade38.png"),
        39 => asset!("/assets/pictures/upgrade/upgrade39.png"),
        40 => asset!("/assets/pictures/upgrade/upgrade40.png"),
        41 => asset!("/assets/pictures/upgrade/upgrade41.png"),
        42 => asset!("/assets/pictures/upgrade/upgrade42.png"),
        43 => asset!("/assets/pictures/upgrade/upgrade43.png"),
        44 => asset!("/assets/pictures/upgrade/upgrade44.png"),
        45 => asset!("/assets/pictures/upgrade/upgrade45.png"),
        46 => asset!("/assets/pictures/upgrade/upgrade46.png"),
        47 => asset!("/assets/pictures/upgrade/upgrade47.png"),
        48 => asset!("/assets/pictures/upgrade/upgrade48.png"),
        49 => asset!("/assets/pictures/upgrade/upgrade49.png"),
        50 => asset!("/assets/pictures/upgrade/upgrade50.png"),
        51 => asset!("/assets/pictures/upgrade/upgrade51.png"),
        52 => asset!("/assets/pictures/upgrade/upgrade52.png"),
        53 => asset!("/assets/pictures/upgrade/upgrade53.png"),
        54 => asset!("/assets/pictures/upgrade/upgrade54.png"),
        55 => asset!("/assets/pictures/upgrade/upgrade55.png"),
        56 => asset!("/assets/pictures/upgrade/upgrade56.png"),
        57 => asset!("/assets/pictures/upgrade/upgrade57.png"),
        58 => asset!("/assets/pictures/upgrade/upgrade58.png"),
        59 => asset!("/assets/pictures/upgrade/upgrade59.png"),
        60 => asset!("/assets/pictures/upgrade/upgrade60.png"),
        61 => asset!("/assets/pictures/upgrade/upgrade61.png"),
        62 => asset!("/assets/pictures/upgrade/upgrade62.png"),
        63 => asset!("/assets/pictures/upgrade/upgrade63.png"),
        64 => asset!("/assets/pictures/upgrade/upgrade64.png"),
        65 => asset!("/assets/pictures/upgrade/upgrade65.png"),
        66 => asset!("/assets/pictures/upgrade/upgrade66.png"),
        67 => asset!("/assets/pictures/upgrade/upgrade67.png"),
        68 => asset!("/assets/pictures/upgrade/upgrade68.png"),
        69 => asset!("/assets/pictures/upgrade/upgrade69.png"),
        70 => asset!("/assets/pictures/upgrade/upgrade70.png"),
        71 => asset!("/assets/pictures/upgrade/upgrade71.png"),
        72 => asset!("/assets/pictures/upgrade/upgrade72.png"),
        73 => asset!("/assets/pictures/upgrade/upgrade73.png"),
        74 => asset!("/assets/pictures/upgrade/upgrade74.png"),
        75 => asset!("/assets/pictures/upgrade/upgrade75.png"),
        76 => asset!("/assets/pictures/upgrade/upgrade76.png"),
        77 => asset!("/assets/pictures/upgrade/upgrade77.png"),
        78 => asset!("/assets/pictures/upgrade/upgrade78.png"),
        79 => asset!("/assets/pictures/upgrade/upgrade79.png"),
        80 => asset!("/assets/pictures/upgrade/upgrade80.png"),
        81 => asset!("/assets/pictures/upgrade/upgrade81.png"),
        82 => asset!("/assets/pictures/upgrade/upgrade82.png"),
        83 => asset!("/assets/pictures/upgrade/upgrade83.png"),
        84 => asset!("/assets/pictures/upgrade/upgrade84.png"),
        85 => asset!("/assets/pictures/upgrade/upgrade85.png"),
        86 => asset!("/assets/pictures/upgrade/upgrade86.png"),
        87 => asset!("/assets/pictures/upgrade/upgrade87.png"),
        88 => asset!("/assets/pictures/upgrade/upgrade88.png"),
        89 => asset!("/assets/pictures/upgrade/upgrade89.png"),
        90 => asset!("/assets/pictures/upgrade/upgrade90.png"),
        91 => asset!("/assets/pictures/upgrade/upgrade91.png"),
        92 => asset!("/assets/pictures/upgrade/upgrade92.png"),
        93 => asset!("/assets/pictures/upgrade/upgrade93.png"),
        94 => asset!("/assets/pictures/upgrade/upgrade94.png"),
        95 => asset!("/assets/pictures/upgrade/upgrade95.png"),
        96 => asset!("/assets/pictures/upgrade/upgrade96.png"),
        97 => asset!("/assets/pictures/upgrade/upgrade97.png"),
        98 => asset!("/assets/pictures/upgrade/upgrade98.png"),
        99 => asset!("/assets/pictures/upgrade/upgrade99.png"),
        100 => asset!("/assets/pictures/upgrade/upgrade100.png"),
        101 => asset!("/assets/pictures/upgrade/upgrade101.png"),
        102 => asset!("/assets/pictures/upgrade/upgrade102.png"),
        103 => asset!("/assets/pictures/upgrade/upgrade103.png"),
        104 => asset!("/assets/pictures/upgrade/upgrade104.png"),
        105 => asset!("/assets/pictures/upgrade/upgrade105.png"),
        106 => asset!("/assets/pictures/upgrade/upgrade106.png"),
        107 => asset!("/assets/pictures/upgrade/upgrade107.png"),
        108 => asset!("/assets/pictures/upgrade/upgrade108.png"),
        109 => asset!("/assets/pictures/upgrade/upgrade109.png"),
        110 => asset!("/assets/pictures/upgrade/upgrade110.png"),
        111 => asset!("/assets/pictures/upgrade/upgrade111.png"),
        112 => asset!("/assets/pictures/upgrade/upgrade112.png"),
        113 => asset!("/assets/pictures/upgrade/upgrade113.png"),
        114 => asset!("/assets/pictures/upgrade/upgrade114.png"),
        115 => asset!("/assets/pictures/upgrade/upgrade115.png"),
        116 => asset!("/assets/pictures/upgrade/upgrade116.png"),
        117 => asset!("/assets/pictures/upgrade/upgrade117.png"),
        118 => asset!("/assets/pictures/upgrade/upgrade118.png"),
        119 => asset!("/assets/pictures/upgrade/upgrade119.png"),
        120 => asset!("/assets/pictures/upgrade/upgrade120.png"),
        121 => asset!("/assets/pictures/upgrade/upgrade121.png"),
        122 => asset!("/assets/pictures/upgrade/upgrade122.png"),
        123 => asset!("/assets/pictures/upgrade/upgrade123.png"),
        124 => asset!("/assets/pictures/upgrade/upgrade124.png"),
        125 => asset!("/assets/pictures/upgrade/upgrade125.png"),
        _ => return None,
    })
}

/// One upgrade square: shows a painted icon (or the index number where no art
/// exists yet), lights by state (owned / affordable / locked), updates the
/// detail card on hover/focus, and buys on click.
#[component]
fn UpgradeCell(idx: usize) -> Element {
    let bridge = use_bridge();
    let detail = use_detail();
    let owned = use_slice(move |s| meta(idx).owned(s));
    // 5 Hz (legacy buttoncolorchange): the affordable→`.can` accent would
    // otherwise strobe at 20 Hz when the upgrade autobuyer is draining currency.
    let affordable = use_slow_slice(move |s| meta(idx).affordable(s));

    // Three visual states: owned (filled), affordable (accent — buy now), and
    // unaffordable (`cant` — dimmed/disabled so the buyable ones stand out).
    let cls = if owned() {
        "sf-upg-cell owned"
    } else if affordable() {
        "sf-upg-cell can"
    } else {
        "sf-upg-cell cant"
    };

    rsx! {
        div {
            class: cls,
            tabindex: "0",
            onmouseenter: move |_| detail.set(DetailTarget::Upgrade(idx)),
            onfocus: move |_| detail.set(DetailTarget::Upgrade(idx)),
            onclick: move |_| bridge.dispatch(derive::upgrade_buy(&bridge.state.peek(), idx)),
            if let Some(src) = upgrade_icon(idx) {
                img { class: "sf-upg-icon", src, alt: "", draggable: "false" }
            } else {
                "{idx}"
            }
        }
    }
}

/// The upgrade body for the shared bottom detail panel: name, status, cost,
/// and live effect for the focused upgrade. Reads state/derived inline.
#[component]
pub fn UpgradeDetailBody(idx: usize) -> Element {
    let bridge = use_bridge();
    let m = meta(idx);
    let state = bridge.state.read();
    let derived = bridge.derived.read();
    let notation = bridge.prefs.read().notation;

    let owned = m.owned(&state);
    let affordable = m.affordable(&state);
    let cost = m.cost();
    let name = t(&format!("upgrades.descriptions.{idx}")).to_string();
    let effect = super::upgrade_effects::effect_text(idx, &state, &derived, notation);

    let (status_key, status_cls) = if owned {
        ("upgrades.bought", "sf-detail-badge owned")
    } else if affordable {
        ("upgrades.status_affordable", "sf-detail-badge ok")
    } else {
        ("upgrades.status_locked", "sf-detail-badge")
    };

    rsx! {
        DetailBody {
            title: t_args("upgrades.short", &[("n", &idx.to_string())]),
            // Painted upgrade art where it exists; otherwise the cost-currency
            // icon (e.g. the not-yet-drawn Particle shop) so the card is never
            // icon-less.
            marker: Some(match upgrade_icon(idx) {
                Some(src) => rsx! {
                    span { class: "sf-icon sf-icon-img",
                        img { src, alt: "", draggable: "false" }
                    }
                },
                None => rsx! { ResourceIcon { resource: m.resource } },
            }),
            badge: Some(rsx! { span { class: status_cls, {t(status_key)} } }),
            description: Some(name),
            accent: Some(m.resource.css_color()),
            div { class: "sf-card-row",
                span { class: "label", {t("buildings.cost")} }
                span {
                    Num { value: cost }
                    " "
                    ResourceIcon { resource: m.resource }
                }
            }
            if let Some(line) = effect {
                div { class: "sf-upgrade-effect", "{line}" }
            }
        }
    }
}
