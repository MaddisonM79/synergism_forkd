//! Offline catch-up: chunked `time_warp` ticks on boot (and after long tab
//! throttles), with a live progress dialog.
//!
//! This is the host loop the logic tier was built to expect: the save
//! envelope stamps `saved_at_ms`, `TackInput::time_warp` skips the
//! automation head+middle while generation still runs, and the host chunks
//! elapsed wall time into fixed 1 s warp ticks so huge gaps stay numerically
//! identical to having ticked through them.

use dioxus::prelude::*;
use synergismforkd_logic::{tack, GameState, TackInput};
use synergismforkd_ui::bridge::{DialogKind, DialogProgress, DialogRequest, GameBridge};
use synergismforkd_ui::i18n::t;

/// Offline simulation cap (24 h). Tune once idle balance data exists.
pub const MAX_OFFLINE_S: f64 = 86_400.0;

/// Each warp tick simulates this much game time.
const WARP_TICK_S: f64 = 1.0;

/// Warp ticks per UI yield — keeps the progress dialog live without
/// thrashing the renderer (one `state.write()` per batch).
const TICKS_PER_BATCH: usize = 200;

/// Gaps shorter than this run silently (no dialog) — a tab throttle of a
/// few seconds doesn't deserve a modal.
const DIALOG_THRESHOLD_S: f64 = 30.0;

/// Signals the catch-up driver needs from the component scope (signals and
/// callbacks must be created there, not inside the loop task).
#[derive(Clone, Copy)]
pub struct CatchUpHandles {
    pub progress: Signal<DialogProgress>,
    pub skip: Signal<bool>,
    pub on_skip: Callback<()>,
}

/// Simulate `elapsed_s` of offline time. One `state.write()` per batch;
/// yields to the renderer between batches unless skipping.
pub async fn run(bridge: GameBridge, handles: CatchUpHandles, elapsed_s: f64) {
    let total = elapsed_s.min(MAX_OFFLINE_S);
    if total < WARP_TICK_S {
        return;
    }

    let mut skip = handles.skip;
    skip.set(false);
    let mut progress = handles.progress;
    progress.set(DialogProgress {
        done_s: 0.0,
        total_s: total,
    });

    let show_dialog = total >= DIALOG_THRESHOLD_S;
    if show_dialog {
        bridge.open_dialog(DialogRequest {
            title: t("dialogs.offline.title").to_string(),
            body: t("dialogs.offline.body").to_string(),
            kind: DialogKind::Progress {
                progress: handles.progress,
                on_skip: Some(handles.on_skip),
            },
        });
    }

    let mut done = 0.0;
    while done < total {
        let skipping = *skip.peek();
        let remaining_ticks = ((total - done) / WARP_TICK_S).ceil() as usize;
        let batch = if skipping {
            remaining_ticks
        } else {
            remaining_ticks.min(TICKS_PER_BATCH)
        };

        {
            let mut state = bridge.state;
            let mut state = state.write();
            done += run_warp_batch(&mut state, batch, total - done);
        }
        progress.set(DialogProgress {
            done_s: done,
            total_s: total,
        });

        if !skipping && done < total {
            // Yield so the progress bar paints.
            gloo_timers::future::sleep(std::time::Duration::from_millis(0)).await;
        }
    }

    if show_dialog {
        bridge.close_dialog();
    }
}

/// Run up to `batch` warp ticks (bounded by `remaining_s`); returns the
/// simulated seconds.
fn run_warp_batch(state: &mut GameState, batch: usize, remaining_s: f64) -> f64 {
    let mut simulated = 0.0;
    for _ in 0..batch {
        if simulated >= remaining_s {
            break;
        }
        let dt = WARP_TICK_S.min(remaining_s - simulated);
        let input = TackInput {
            dt,
            time_warp: true,
            ..TackInput::default()
        };
        // Events from warp ticks are deliberately dropped: hours of offline
        // generation would flood the toast layer, and the end state is
        // what the player sees.
        let _ = tack(state, &input);
        simulated += dt;
    }
    simulated
}
