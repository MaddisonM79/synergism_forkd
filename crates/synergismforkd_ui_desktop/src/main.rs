//! Synergism Forkd — desktop shell placeholder.
//!
//! The follow-up Tauri-integration PR replaces this `main` with a
//! `tauri::Builder` that loads the `synergismforkd_ui_web` WASM bundle
//! and exposes native commands (file pickers, Steam SDK, Discord RPC).

use synergismforkd_logic as _;
use synergismforkd_save as _;
use synergismforkd_ui as _;

fn main() {
    println!("synergismforkd-desktop: placeholder (Tauri wiring lands in a follow-up)");
}
