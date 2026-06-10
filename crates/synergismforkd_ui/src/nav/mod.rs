//! Grouped navigation: left rail (groups) + horizontal sub-nav (sections).
//! Both render only what's unlocked; gating lives in [`crate::gating`].

use dioxus::prelude::*;

use crate::bridge::{use_bridge, use_slice};
use crate::gating::{Group, Route};
use crate::i18n::t;

/// Left rail of group buttons. Locked groups don't render at all (the
/// unlock-reveal pattern); the active group carries its accent color.
#[component]
pub fn GroupedNav() -> Element {
    let bridge = use_bridge();
    let visible_groups = use_slice(|state| {
        Group::ALL
            .into_iter()
            .filter(|g| g.visible(state))
            .collect::<Vec<_>>()
    });
    let active = bridge.route.read().group;

    rsx! {
        nav { class: "sf-rail",
            div { class: "sf-rail-title", "Synergism Forkd" }
            for group in visible_groups() {
                button {
                    key: "{group.css_value()}",
                    class: if group == active { "sf-rail-btn active" } else { "sf-rail-btn" },
                    style: "--accent: var(--accent-{group.css_value()})",
                    onclick: move |_| {
                        let section = group.sections()[0];
                        bridge.navigate(Route { group, section, subsection: 0 });
                    },
                    span { class: "sf-dot" }
                    {t(group.label_key())}
                }
            }
        }
    }
}

/// Horizontal section tabs for the active group.
#[component]
pub fn SubNav() -> Element {
    let bridge = use_bridge();
    let route = *bridge.route.read();
    let group = route.group;
    let visible_sections = use_slice(move |state| {
        group
            .sections()
            .iter()
            .copied()
            .filter(|s| s.visible(state))
            .collect::<Vec<_>>()
    });

    rsx! {
        div {
            class: "sf-subnav",
            style: "--accent: var(--accent-{group.css_value()})",
            for section in visible_sections() {
                button {
                    key: "{section.label_key()}",
                    class: if section == route.section { "sf-subnav-btn active" } else { "sf-subnav-btn" },
                    onclick: move |_| {
                        bridge.navigate(Route { group, section, subsection: 0 });
                    },
                    {t(section.label_key())}
                }
            }
        }
    }
}

// Re-export for the router in `sections`.
pub use crate::gating::Section as NavSection;
