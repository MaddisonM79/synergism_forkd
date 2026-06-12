//! The shared "what is the player inspecting" target + the persistent bottom
//! detail panel.
//!
//! Any hoverable surface (achievement cell, upgrade square, building/rune
//! card) calls `use_detail().set(target)` on hover/focus; the [`DetailPanel`]
//! at the bottom of the shell renders that target's full description. This
//! replaces the per-section info boxes so cards can stay minimal.
//!
//! **Sticky by construction**: [`Detail`] only ever sets `Some(..)` — there is
//! no "clear", and no `onmouseleave` is wired anywhere, so the last-hovered
//! item stays shown while the player moves to click Buy.
//!
//! The target is ephemeral/UI-only and deliberately NOT part of
//! [`GameBridge`](crate::bridge::GameBridge) (the host seam) — it lives in its
//! own context provided at the App root.

use dioxus::prelude::*;

use crate::components::Resource;

/// Which thing the bottom detail panel is describing. Carries only the
/// identifier; the panel reads live numbers from state/derived itself.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DetailTarget {
    /// A currency in the top resource bar.
    Resource(Resource),
    /// Achievement by 0-based index.
    Achievement(usize),
    /// Shop upgrade by legacy index.
    Upgrade(usize),
    /// Crystal upgrade `1..=5`.
    CrystalUpgrade(u8),
    /// A rune / blessing / spirit.
    Rune { family: RuneKind, index: usize },
    /// A building-family card.
    Building(BuildingDetail),
    /// A reset card (prestige / transcension / reincarnation).
    Reset(ResetKind),
}

/// Reset tier for [`DetailTarget::Reset`] — a pub mirror of the buildings
/// section's private `ResetTier`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResetKind {
    Prestige,
    Transcension,
    Reincarnation,
}

/// Rune family for [`DetailTarget::Rune`] — a pub mirror of the rune section's
/// private `RuneFamily`, so this leaf module needn't import rune internals.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuneKind {
    Rune,
    Blessing,
    Spirit,
}

/// Which building card a [`DetailTarget::Building`] points at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildingDetail {
    /// Coin producer tier `1..=5`.
    CoinProducer(u8),
    /// Diamond producer tier `1..=5`.
    Diamond(u8),
    /// Mythos producer tier `1..=5`.
    Mythos(u8),
    Accelerator,
    Multiplier,
    AcceleratorBoost,
}

/// App-root context holding the current detail target.
#[derive(Clone, Copy)]
pub struct Detail(Signal<Option<DetailTarget>>);

impl Detail {
    /// Point the panel at `target`. The only mutator — the absence of a
    /// "clear" is what makes the panel sticky.
    pub fn set(self, target: DetailTarget) {
        let mut sig = self.0;
        sig.set(Some(target));
    }

    /// Dismiss the panel (the close button). It stays gone until the next
    /// hover/focus sets a target again — so the panel is sticky *until* the
    /// player explicitly closes it.
    pub fn clear(self) {
        let mut sig = self.0;
        sig.set(None);
    }

    /// The current target (`None` until the first hover, or after a close).
    #[must_use]
    pub fn get(self) -> Option<DetailTarget> {
        *self.0.read()
    }
}

/// Install the detail context. Call once at the App root, before the tree
/// that reads it renders.
pub fn provide_detail() {
    use_context_provider(|| Detail(Signal::new(None)));
}

/// Grab the detail context.
#[must_use]
pub fn use_detail() -> Detail {
    use_context()
}

/// The standardized detail-body shell every hover routes through, so the box
/// looks identical regardless of source:
///
/// ```text
/// [marker]  Title  ............  [badge]
/// Description (optional)
/// Formula (optional)
/// <rows: cost / level / effect / …>   (children)
/// ```
///
/// `marker` is a leading `#N` tag or resource icon; `badge` is a right-aligned
/// status; `accent` tints the card's left edge to the relevant resource. Slots
/// left unset are simply omitted — fill what's relevant per thing.
#[component]
pub fn DetailBody(
    title: String,
    #[props(default)] marker: Option<Element>,
    #[props(default)] badge: Option<Element>,
    #[props(default)] description: Option<String>,
    #[props(default)] formula: Option<String>,
    #[props(default)] accent: Option<&'static str>,
    children: Element,
) -> Element {
    let style = accent
        .map(|a| format!("--card-accent: {a}"))
        .unwrap_or_default();
    rsx! {
        div { class: "sf-detail-card", style: "{style}",
            div { class: "sf-detail-head",
                if let Some(marker) = marker {
                    {marker}
                }
                span { class: "sf-detail-title", "{title}" }
                if let Some(badge) = badge {
                    {badge}
                }
            }
            if let Some(description) = description {
                div { class: "sf-detail-desc", "{description}" }
            }
            if let Some(formula) = formula {
                div { class: "sf-upgrade-formula", "{formula}" }
            }
            {children}
        }
    }
}

/// The detail panel: a floating, toast-style card pinned to the bottom-right
/// that shows the current target's full detail. It does NOT auto-dismiss —
/// hovering anything updates it in place; the ✕ closes it until the next
/// hover. Renders nothing while there's no target (no reserved space).
/// Section body components read live state/derived inline.
#[component]
pub fn DetailPanel() -> Element {
    let detail = use_detail();
    // Nothing floats until something is hovered (or after a close).
    let Some(target) = detail.get() else {
        return rsx! {};
    };
    rsx! {
        aside { class: "sf-detail",
            button {
                class: "sf-detail-close",
                "aria-label": "Close",
                onclick: move |_| detail.clear(),
                "✕"
            }
            match target {
                DetailTarget::Resource(resource) => rsx! {
                    crate::stats::ResourceDetailBody { resource }
                },
                DetailTarget::Achievement(index) => rsx! {
                    crate::sections::production::achievements::AchievementDetailBody { index }
                },
                DetailTarget::Upgrade(idx) => rsx! {
                    crate::sections::production::upgrades::UpgradeDetailBody { idx }
                },
                DetailTarget::CrystalUpgrade(i) => rsx! {
                    crate::sections::production::buildings::CrystalDetailBody { i }
                },
                DetailTarget::Rune { family, index } => rsx! {
                    crate::sections::mystic::runes::RuneDetailBody { family, index }
                },
                DetailTarget::Building(which) => rsx! {
                    crate::sections::production::buildings::BuildingDetailBody { which }
                },
                DetailTarget::Reset(kind) => rsx! {
                    crate::sections::production::buildings::ResetDetailBody { kind }
                },
            }
        }
    }
}
