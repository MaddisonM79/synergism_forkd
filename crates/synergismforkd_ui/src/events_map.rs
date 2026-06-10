//! CoreEvent → UI side-effect mapping.
//!
//! Most of the 103 `CoreEvent` variants need NO UI reaction — state changes
//! already reach the screen through the memo selectors. Only events whose
//! feedback isn't visible state (or deserves celebration) surface here.
//! Grows with each milestone (achievement toasts land with the
//! Achievements section).

use synergismforkd_logic::CoreEvent;

use crate::bridge::{GameBridge, ToastKind};
use crate::i18n::t;

/// Dispatch one tick's event stream.
pub fn apply(bridge: &GameBridge, events: &[CoreEvent]) {
    for event in events {
        #[allow(clippy::single_match)]
        match event {
            // Singularity is the one reset rare enough to always celebrate.
            CoreEvent::SingularityPerformed { .. } => {
                bridge.toast(ToastKind::Achievement, t("toasts.singularity_done"));
            }
            // Everything else: state-visible already, or a later milestone.
            _ => {}
        }
    }
}
