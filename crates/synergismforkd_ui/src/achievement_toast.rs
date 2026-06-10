//! Achievement-unlock toasts. The logic tier doesn't emit a per-achievement
//! event (awards are set in many `award_*` paths that return counts, not
//! events), so the UI detects unlocks by diffing the earned bitmap each tick.
//! Mounted once in the app root, it fires globally regardless of the current
//! screen.

use dioxus::prelude::*;

use crate::bridge::{use_bridge, use_slice, ToastKind};
use crate::i18n::t;
use crate::sections::production::achievements_text;

/// Above this many simultaneous unlocks (e.g. a save import), collapse into
/// one summary toast instead of flooding the stack.
const SUMMARY_THRESHOLD: usize = 3;

/// Renders nothing; watches the achievement bitmap and toasts new unlocks.
#[component]
pub fn AchievementToaster() -> Element {
    let bridge = use_bridge();
    let earned = use_slice(|s| s.achievements.achievements.to_vec());
    // Seed the baseline from the booted state so we don't toast everything
    // already earned on first load.
    let mut prev = use_signal(|| earned.peek().clone());

    use_effect(move || {
        let current = earned();
        let previous = prev.peek().clone();
        // The bitmap can grow (length only changes across schema versions);
        // compare index-by-index up to the shorter length.
        let newly: Vec<usize> = current
            .iter()
            .zip(previous.iter())
            .enumerate()
            .filter(|(_, (cur, old))| **cur != 0 && **old == 0)
            .map(|(i, _)| i)
            .collect();

        if !newly.is_empty() {
            if newly.len() > SUMMARY_THRESHOLD {
                bridge.toast(
                    ToastKind::Achievement,
                    format!("{} {}", newly.len(), t("toasts.achievements_unlocked_n")),
                );
            } else {
                for i in newly {
                    bridge.toast(
                        ToastKind::Achievement,
                        format!(
                            "{} {}",
                            t("toasts.achievement_unlocked"),
                            achievements_text::name(i)
                        ),
                    );
                }
            }
            prev.set(current);
        }
    });

    rsx! {}
}
