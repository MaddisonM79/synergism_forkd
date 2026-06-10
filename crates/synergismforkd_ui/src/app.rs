//! App root: stylesheets, theme attribute, shell grid (rail / HUD /
//! sub-nav / content), toast + dialog layers.
//!
//! The host shell (web/desktop) calls [`crate::bridge::GameBridge::provide`]
//! BEFORE rendering `App`, and drives the tick loop against the bridge's
//! signals. `App` itself is pure presentation — platform-agnostic.

use dioxus::prelude::*;

use crate::achievement_toast::AchievementToaster;
use crate::bridge::use_bridge;
use crate::components::{DialogLayer, ToastStack};
use crate::hud::ResourceHud;
use crate::nav::{GroupedNav, SubNav};
use crate::sections::SectionView;

/// Crate version, stamped at build time — shown in the corner badge.
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Inline critical CSS: rendered synchronously in `<head>` (unlike the
/// async `document::Stylesheet` links), so the page paints with the app
/// background immediately instead of flashing unstyled content on refresh.
const CRITICAL_CSS: &str = "html,body{margin:0;background:#1a1325;}";

#[component]
pub fn App() -> Element {
    let bridge = use_bridge();
    let theme = bridge.prefs.read().theme;

    rsx! {
        document::Style { {CRITICAL_CSS} }
        document::Stylesheet { href: asset!("/assets/styles/tokens.css") }
        document::Stylesheet { href: asset!("/assets/styles/themes.css") }
        document::Stylesheet { href: asset!("/assets/styles/base.css") }
        document::Stylesheet { href: asset!("/assets/styles/components.css") }
        document::Stylesheet { href: asset!("/assets/styles/sections.css") }

        div { class: "sf-app", "data-theme": theme.css_value(),
            GroupedNav {}
            ResourceHud {}
            SubNav {}
            SectionView {}
            ToastStack {}
            DialogLayer {}
            AchievementToaster {}
            div { class: "sf-version", "v{VERSION}" }
        }
    }
}
