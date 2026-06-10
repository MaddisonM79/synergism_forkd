//! `ResourceIcon` — the one consistent resource symbol system.
//!
//! Every place a resource amount appears renders `Num + ResourceIcon`, so
//! the pairing is uniform game-wide (the legacy UI mixed emoji, images, and
//! colored text glyphs). Glyphs are simple inline SVG shapes colored via
//! the `--res-*` theme tokens; real art can replace the paths later without
//! touching call sites.

use dioxus::prelude::*;

/// Every displayable resource. Grows as sections land.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Resource {
    Coins,
    Diamonds,
    Mythos,
    Particles,
    Offerings,
    Obtainium,
    Quarks,
    GoldenQuarks,
    Ambrosia,
}

impl Resource {
    /// The `--res-*` color token.
    #[must_use]
    pub fn css_color(self) -> &'static str {
        match self {
            Resource::Coins => "var(--res-coin)",
            Resource::Diamonds => "var(--res-diamond)",
            Resource::Mythos => "var(--res-mythos)",
            Resource::Particles => "var(--res-particle)",
            Resource::Offerings => "var(--res-offering)",
            Resource::Obtainium => "var(--res-obtainium)",
            Resource::Quarks => "var(--res-quark)",
            Resource::GoldenQuarks => "var(--res-golden-quark)",
            Resource::Ambrosia => "var(--res-ambrosia)",
        }
    }

    /// Translation key for tooltips/labels.
    #[must_use]
    pub fn label_key(self) -> &'static str {
        match self {
            Resource::Coins => "hud.coins",
            Resource::Diamonds => "hud.diamonds",
            Resource::Mythos => "hud.mythos",
            Resource::Particles => "hud.particles",
            Resource::Offerings => "hud.offerings",
            Resource::Obtainium => "hud.obtainium",
            Resource::Quarks => "hud.quarks",
            Resource::GoldenQuarks => "hud.golden_quarks",
            Resource::Ambrosia => "hud.ambrosia",
        }
    }

    /// SVG path data (24×24 viewBox, filled with `currentColor`).
    fn path(self) -> &'static str {
        match self {
            // Filled circle (coin).
            Resource::Coins => "M12 3a9 9 0 1 0 0 18 9 9 0 0 0 0-18z",
            // Rhombus (diamond).
            Resource::Diamonds => "M12 2 21 12 12 22 3 12z",
            // Four-point star (mythos).
            Resource::Mythos => "M12 2l2.5 7.5L22 12l-7.5 2.5L12 22l-2.5-7.5L2 12l7.5-2.5z",
            // Dot + orbit hint (particle).
            Resource::Particles => {
                "M12 8a4 4 0 1 0 0 8 4 4 0 0 0 0-8zM12 2a10 10 0 0 1 9.5 6.9l-1.9.6A8 8 0 0 0 12 4z"
            }
            // Droplet (offering).
            Resource::Offerings => "M12 2C12 2 5 10 5 15a7 7 0 0 0 14 0c0-5-7-13-7-13z",
            // Hexagon (obtainium).
            Resource::Obtainium => "M12 2 20.7 7v10L12 22 3.3 17V7z",
            // Triangle (quark).
            Resource::Quarks => "M12 3 22 20H2z",
            // Five-point star (golden quark).
            Resource::GoldenQuarks => {
                "M12 2l3.1 6.3 6.9 1-5 4.9 1.2 6.8L12 17.8 5.8 21l1.2-6.8-5-4.9 6.9-1z"
            }
            // Berry pair (ambrosia).
            Resource::Ambrosia => {
                "M8.5 9a5 5 0 1 0 0 10 5 5 0 0 0 0-10zM16 5a4 4 0 1 0 0 8 4 4 0 0 0 0-8z"
            }
        }
    }
}

/// Inline resource glyph, sized by font (1em) and colored by theme token.
#[component]
pub fn ResourceIcon(resource: Resource) -> Element {
    rsx! {
        span { class: "sf-icon", style: "color: {resource.css_color()}",
            svg {
                view_box: "0 0 24 24",
                xmlns: "http://www.w3.org/2000/svg",
                path { d: resource.path(), fill: "currentColor" }
            }
        }
    }
}
