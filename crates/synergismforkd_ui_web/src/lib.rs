//! Synergism Forkd — browser shell.
//!
//! The `dx`-built bin target (`main.rs`) calls [`run`] on
//! `wasm32-unknown-unknown` to mount the `synergismforkd_ui` Dioxus tree.
//! The lib form exists for native unit tests (SaveHost) and any host that
//! wants the persistence seam.
//!
//! Hosts the [`save_host`] seam: `localStorage` persistence + the autosave
//! loop that drives the headless game logic and the fresh save format on the
//! browser clock.

use synergismforkd_ui as _;

// Silence `unused_crate_dependencies` on wasm32 — `getrandom` is listed
// purely to enable its `js` feature for the transitive
// `rand_chacha`/`rand_xoshiro` chain via Cargo feature unification, not
// consumed directly here. The dep is cfg-gated to wasm32 in Cargo.toml,
// so the silencer must match.
#[cfg(target_arch = "wasm32")]
use getrandom as _;

// On native, `dioxus` is present only so the renderer-free feature set
// unifies with `synergismforkd_ui`; the launch path below is wasm-only.
#[cfg(not(target_arch = "wasm32"))]
use dioxus as _;

pub mod save_host;

#[cfg(target_arch = "wasm32")]
pub mod catch_up;
#[cfg(target_arch = "wasm32")]
pub mod loop_driver;
#[cfg(target_arch = "wasm32")]
pub mod platform;
#[cfg(target_arch = "wasm32")]
pub mod root;

pub use save_host::{BootOutcome, SaveHost, SaveStorage, AUTOSAVE_INTERVAL_S, SAVE_KEY};

/// Mount the app in the browser. Called by the `dx` binary entry (`main.rs`).
#[cfg(target_arch = "wasm32")]
pub fn run() {
    console_error_panic_hook::set_once();
    dioxus::launch(root::WebRoot);
}
