//! Synergism Forkd — headless game logic.
//!
//! Subdirs mirror the legacy `packages/logic/src/` tree. Boundary rules
//! carry over: no DOM, no UI imports, no i18n, no modal helpers. Public
//! functions follow the `(state, input) -> (state, events)` shape so
//! side effects are routed through the UI tier.

pub mod events;
pub mod math;
pub mod mechanics;
pub mod state;
pub mod tick;

pub fn placeholder() {}
