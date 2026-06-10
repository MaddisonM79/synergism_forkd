//! Synergism Forkd — WASM entry point.
//!
//! Built as `cdylib` for `wasm32-unknown-unknown`; mounts the
//! `synergismforkd_ui` Dioxus root component into the page. Also built
//! as `rlib` so the desktop crate can depend on it.
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

pub mod save_host;

pub use save_host::{BootOutcome, SaveHost, SaveStorage, AUTOSAVE_INTERVAL_S, SAVE_KEY};

pub fn placeholder() {}
