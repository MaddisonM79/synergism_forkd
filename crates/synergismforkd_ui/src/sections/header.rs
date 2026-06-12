//! Reset-gain indicators (Prestige / Transcend / Reincarnate) as compact
//! hover-icons, shown on the right of the section-tab row. Each icon appears
//! once its tier is reachable and, on hover/focus, writes its tier to the
//! bottom detail box (the shared "blurb" surface — what the reset does, the
//! gain, the requirement). Display-only — the actual reset buttons live in the
//! Buildings reset strip.

use dioxus::prelude::*;

use crate::bridge::use_slice;
use crate::components::{Resource, ResourceIcon};
use crate::detail::{use_detail, DetailTarget, ResetKind};

/// The reset-gain icon cluster, rendered right-aligned in the sub-nav row.
#[component]
pub fn ResetGains() -> Element {
    // Gates mirror the reset progression (a gain shows once that reset is
    // reachable): prestige at coin-four, transcend after a prestige,
    // reincarnate after a transcension.
    let show_prestige = use_slice(|s| s.reset_counters.coin_four_unlocked);
    let show_transcend = use_slice(|s| s.reset_counters.prestige_unlocked);
    let show_reincarnate = use_slice(|s| s.reset_counters.transcend_unlocked);

    rsx! {
        div { class: "sf-reset-gains",
            if show_prestige() {
                ResetGainIcon { resource: Resource::Diamonds, kind: ResetKind::Prestige }
            }
            if show_transcend() {
                ResetGainIcon { resource: Resource::Mythos, kind: ResetKind::Transcension }
            }
            if show_reincarnate() {
                ResetGainIcon { resource: Resource::Particles, kind: ResetKind::Reincarnation }
            }
        }
    }
}

/// One reset-gain icon: hovering/focusing it fills the bottom detail box with
/// that reset's blurb (via [`DetailTarget::Reset`]).
#[component]
fn ResetGainIcon(resource: Resource, kind: ResetKind) -> Element {
    let detail = use_detail();
    rsx! {
        span {
            class: "sf-header-icon",
            tabindex: "0",
            onmouseenter: move |_| detail.set(DetailTarget::Reset(kind)),
            onfocus: move |_| detail.set(DetailTarget::Reset(kind)),
            ResourceIcon { resource }
        }
    }
}
