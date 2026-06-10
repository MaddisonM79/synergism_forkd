//! App root: stylesheets, theme attribute, shell grid (rail / HUD /
//! sub-nav / content), toast + dialog layers.
//!
//! The host shell (web/desktop) calls [`crate::bridge::GameBridge::provide`]
//! BEFORE rendering `App`, and drives the tick loop against the bridge's
//! signals. `App` itself is pure presentation — platform-agnostic.

use dioxus::prelude::*;

use crate::bridge::use_bridge;
use crate::components::{DialogLayer, ToastStack};
use crate::hud::ResourceHud;
use crate::nav::{GroupedNav, SubNav};
use crate::sections::SectionView;

#[component]
pub fn App() -> Element {
    let bridge = use_bridge();
    let theme = bridge.prefs.read().theme;

    rsx! {
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
        }
    }
}
