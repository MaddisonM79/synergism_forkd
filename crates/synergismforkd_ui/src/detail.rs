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
use crate::i18n::t;

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

    /// The current target (`None` until the first hover).
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

/// The persistent bottom panel: renders the current target's full detail by
/// dispatching to each section's body component, or a hint before anything is
/// hovered. Section body components read live state/derived inline.
#[component]
pub fn DetailPanel() -> Element {
    let detail = use_detail();
    rsx! {
        aside { class: "sf-detail",
            match detail.get() {
                None => rsx! {
                    div { class: "sf-detail-card muted", {t("detail.hint")} }
                },
                Some(DetailTarget::Resource(resource)) => rsx! {
                    crate::stats::ResourceDetailBody { resource }
                },
                Some(DetailTarget::Achievement(index)) => rsx! {
                    crate::sections::production::achievements::AchievementDetailBody { index }
                },
                Some(DetailTarget::Upgrade(idx)) => rsx! {
                    crate::sections::production::upgrades::UpgradeDetailBody { idx }
                },
                Some(DetailTarget::CrystalUpgrade(i)) => rsx! {
                    crate::sections::production::buildings::CrystalDetailBody { i }
                },
                Some(DetailTarget::Rune { family, index }) => rsx! {
                    crate::sections::mystic::runes::RuneDetailBody { family, index }
                },
                Some(DetailTarget::Building(which)) => rsx! {
                    crate::sections::production::buildings::BuildingDetailBody { which }
                },
            }
        }
    }
}
