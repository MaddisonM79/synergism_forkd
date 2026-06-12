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
    /// Crystals (`prestigeShards`) — the diamond-layer sub-currency that
    /// crystal upgrades cost.
    Crystals,
    Mythos,
    /// Mythos Shards (`transcendShards`) — the transcend-layer production
    /// resource, the analogue of Crystals one layer up.
    MythosShards,
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
            Resource::Crystals => "var(--res-crystal)",
            Resource::Mythos => "var(--res-mythos)",
            Resource::MythosShards => "var(--res-mythos)",
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
            Resource::Coins => "resources.coins.name",
            Resource::Diamonds => "resources.diamonds.name",
            Resource::Crystals => "resources.crystals.name",
            Resource::Mythos => "resources.mythos.name",
            Resource::MythosShards => "resources.mythos_shards.name",
            Resource::Particles => "resources.particles.name",
            Resource::Offerings => "resources.offerings.name",
            Resource::Obtainium => "resources.obtainium.name",
            Resource::Quarks => "resources.quarks.name",
            Resource::GoldenQuarks => "resources.golden_quarks.name",
            Resource::Ambrosia => "resources.ambrosia.name",
        }
    }

    /// Painted raster icon for this resource, if one has been produced (see
    /// `tools/icongen/`). Resources with real art render it as an `<img>`;
    /// the rest fall back to the inline SVG glyph below. Paths resolve
    /// relative to the UI crate root and are bundled by `asset!`.
    fn raster(self) -> Option<Asset> {
        Some(match self {
            Resource::Coins => asset!("/assets/pictures/currency/coins.png"),
            Resource::Diamonds => asset!("/assets/pictures/currency/diamonds.png"),
            Resource::Crystals => asset!("/assets/pictures/currency/crystals.png"),
            Resource::Mythos => asset!("/assets/pictures/currency/mythos.png"),
            Resource::MythosShards => asset!("/assets/pictures/currency/mythosshards.png"),
            Resource::Particles => asset!("/assets/pictures/currency/particles.png"),
            Resource::Offerings => asset!("/assets/pictures/currency/offerings.png"),
            Resource::Obtainium => asset!("/assets/pictures/currency/obtainium.png"),
            Resource::Quarks => asset!("/assets/pictures/currency/quarks.png"),
            Resource::GoldenQuarks => asset!("/assets/pictures/currency/goldenquarks.png"),
            Resource::Ambrosia => asset!("/assets/pictures/currency/ambrosia.png"),
        })
    }

    /// SVG path data (24×24 viewBox, filled with `currentColor`).
    fn path(self) -> &'static str {
        match self {
            // Filled circle (coin).
            Resource::Coins => "M12 3a9 9 0 1 0 0 18 9 9 0 0 0 0-18z",
            // Rhombus (diamond).
            Resource::Diamonds => "M12 2 21 12 12 22 3 12z",
            // Faceted gem (crystal): a smaller cut-stone silhouette.
            Resource::Crystals => "M7 3h10l3 5-8 13L4 8z",
            // Four-point star (mythos).
            Resource::Mythos => "M12 2l2.5 7.5L22 12l-7.5 2.5L12 22l-2.5-7.5L2 12l7.5-2.5z",
            // Twin shards (mythos shards): a shard pair, the transcend-layer
            // production resource.
            Resource::MythosShards => "M11 2 4 22l5-3 1-12zM13 2l1 12 5 3z",
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
/// Explicit `width`/`height` attributes keep the SVG at icon size even
/// before the stylesheets load — without them an unstyled `<svg>` defaults
/// to ~300×150px, flashing a huge glyph (e.g. the coin's filled circle) on
/// the first paint after a refresh.
#[component]
pub fn ResourceIcon(resource: Resource) -> Element {
    // Painted art where it exists; the colored SVG glyph everywhere else.
    if let Some(src) = resource.raster() {
        return rsx! {
            span { class: "sf-icon sf-icon-img",
                img { src, alt: "", draggable: "false" }
            }
        };
    }
    rsx! {
        span { class: "sf-icon", style: "color: {resource.css_color()}",
            svg {
                view_box: "0 0 24 24",
                width: "1em",
                height: "1em",
                xmlns: "http://www.w3.org/2000/svg",
                path { d: resource.path(), fill: "currentColor" }
            }
        }
    }
}
