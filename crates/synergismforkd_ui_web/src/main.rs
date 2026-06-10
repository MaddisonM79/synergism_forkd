//! `dx serve` / `dx bundle` entry point for the browser build.
//!
//! All real work lives in the library crate; this binary only exists so the
//! Dioxus CLI has a `main` to build. On native targets (workspace test /
//! clippy builds) it compiles to a stub.

// The binary inherits every package dependency; it consumes them all through
// the library crate, so silence `unused_crate_dependencies` here.
use dioxus as _;
use synergismforkd_logic as _;
use synergismforkd_save as _;
use synergismforkd_ui as _;
#[cfg(not(target_arch = "wasm32"))]
use synergismforkd_ui_web as _;

fn main() {
    #[cfg(target_arch = "wasm32")]
    synergismforkd_ui_web::run();
}
