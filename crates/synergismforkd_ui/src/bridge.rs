//! The UI ↔ host ↔ logic seam.
//!
//! One [`GameBridge`] lives in Dioxus context. The host (web/desktop shell)
//! creates it, drives the tick loop against `state`, drains `actions` into
//! each [`TackInput`](synergismforkd_logic::TackInput), and executes
//! [`HostCommand`]s. Components read state ONLY through [`use_slice`]-style
//! memo selectors — never `state.read()` in a render body, or the component
//! subscribes to every tick. The loop performs exactly one `state.write()`
//! per tick, so memo selectors are the entire re-render firewall.

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use synergismforkd_bignum::Decimal;
use synergismforkd_logic::{GameState, PlayerAction};

use crate::format::Notation;
use crate::gating::Route;
use crate::theme::Theme;

/// Side effects only the host can perform (storage, clipboard, wall-clock).
/// Queued by components, drained by the host loop each tick.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostCommand {
    /// Claim export rewards, persist, and copy the blob to the clipboard.
    ExportSave,
    /// Replace the current save with a pasted export blob.
    ImportSave(String),
    /// Persist immediately (settings actions that shouldn't wait 5 s).
    ForceSave,
    /// Wipe the save and start over.
    HardReset,
}

/// Toast severity → styling + auto-dismiss behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastKind {
    Info,
    Success,
    Warn,
    Achievement,
}

/// One toast in the bottom-right stack.
#[derive(Debug, Clone, PartialEq)]
pub struct Toast {
    /// Monotonic id — list key + dismissal handle.
    pub id: u64,
    pub kind: ToastKind,
    pub text: String,
}

/// Modal dialog request. One layer, one dialog at a time (nested modals are
/// a legacy non-feature we keep not supporting).
#[derive(Clone, PartialEq)]
pub struct DialogRequest {
    pub title: String,
    pub body: String,
    pub kind: DialogKind,
}

#[derive(Clone, PartialEq)]
pub enum DialogKind {
    /// OK only.
    Alert,
    /// OK/Cancel; resolves with the choice.
    Confirm { on_result: Callback<bool> },
    /// Text input; resolves with `Some(input)` on OK, `None` on cancel.
    Prompt { on_submit: Callback<Option<String>> },
    /// Live progress (offline catch-up). The driver updates `progress`;
    /// `on_skip` runs the remainder without yielding.
    Progress {
        progress: Signal<DialogProgress>,
        on_skip: Option<Callback<()>>,
    },
}

/// Progress dialog state, in simulated seconds.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct DialogProgress {
    pub done_s: f64,
    pub total_s: f64,
}

/// UI-only preferences. Persisted by the host (localStorage / config file),
/// NEVER part of `GameState` or the save schema.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct UiPrefs {
    pub notation: Notation,
    pub buy_amount: BuyAmount,
    pub theme: Theme,
    /// Ask before prestiging ("don't ask again" unchecks this).
    pub confirm_resets: bool,
}

impl Default for UiPrefs {
    fn default() -> Self {
        Self {
            notation: Notation::default(),
            buy_amount: BuyAmount::default(),
            theme: Theme::default(),
            confirm_resets: true,
        }
    }
}

/// Per-click purchase cap selector (the legacy 1x/10x/100x/1k toggle, plus
/// Max which routes to the buy-max request instead).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum BuyAmount {
    One,
    Ten,
    Hundred,
    #[default]
    Thousand,
    Max,
}

impl BuyAmount {
    pub const ALL: [BuyAmount; 5] = [
        BuyAmount::One,
        BuyAmount::Ten,
        BuyAmount::Hundred,
        BuyAmount::Thousand,
        BuyAmount::Max,
    ];

    /// The `buyamount` cap the buy functions expect. `None` = use buy-max.
    #[must_use]
    pub fn cap(self) -> Option<f64> {
        match self {
            BuyAmount::One => Some(1.0),
            BuyAmount::Ten => Some(10.0),
            BuyAmount::Hundred => Some(100.0),
            BuyAmount::Thousand => Some(1000.0),
            BuyAmount::Max => None,
        }
    }

    /// Toggle-button label (numbers, not prose — no i18n key needed).
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            BuyAmount::One => "1",
            BuyAmount::Ten => "10",
            BuyAmount::Hundred => "100",
            BuyAmount::Thousand => "1k",
            BuyAmount::Max => "MAX",
        }
    }
}

/// Per-tick derived numbers the HUD shows (filled by the host loop from
/// `TickOutput`; not stored in `GameState`).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct DerivedStats {
    /// Coins gained per real second (smoothed by the loop).
    pub coins_per_sec: Decimal,
    /// Prestige points the next prestige would award.
    pub prestige_gain: Decimal,
}

/// Everything the component tree needs, `Copy` (all fields are signals).
#[derive(Clone, Copy)]
pub struct GameBridge {
    /// THE game state. Components: use selectors, never `.read()` in render.
    pub state: Signal<GameState>,
    /// Player actions queued for the next tick (FIFO).
    pub actions: Signal<Vec<PlayerAction>>,
    /// Host side effects queued for the next loop iteration.
    pub host: Signal<Vec<HostCommand>>,
    /// Active toasts, newest last.
    pub toasts: Signal<Vec<Toast>>,
    /// Active modal, if any.
    pub dialog: Signal<Option<DialogRequest>>,
    /// UI preferences (host-persisted).
    pub prefs: Signal<UiPrefs>,
    /// Current nav position.
    pub route: Signal<Route>,
    /// HUD numbers derived per tick.
    pub derived: Signal<DerivedStats>,
    /// Toast id counter.
    toast_seq: Signal<u64>,
}

impl GameBridge {
    /// Create the bridge and install it in context. Host-root only.
    pub fn provide(initial_state: GameState, prefs: UiPrefs) -> Self {
        use_context_provider(|| GameBridge {
            state: Signal::new(initial_state),
            actions: Signal::new(Vec::new()),
            host: Signal::new(Vec::new()),
            toasts: Signal::new(Vec::new()),
            dialog: Signal::new(None),
            prefs: Signal::new(prefs),
            route: Signal::new(Route::default()),
            derived: Signal::new(DerivedStats::default()),
            toast_seq: Signal::new(0),
        })
    }

    /// Queue a player action for the next tick.
    pub fn dispatch(&self, action: PlayerAction) {
        let mut actions = self.actions;
        actions.write().push(action);
    }

    /// Queue a host command for the next loop iteration.
    pub fn dispatch_host(&self, command: HostCommand) {
        let mut host = self.host;
        host.write().push(command);
    }

    /// Show a toast.
    pub fn toast(&self, kind: ToastKind, text: impl Into<String>) {
        let mut seq = self.toast_seq;
        let id = {
            let mut seq = seq.write();
            *seq += 1;
            *seq
        };
        let mut toasts = self.toasts;
        toasts.write().push(Toast {
            id,
            kind,
            text: text.into(),
        });
    }

    /// Remove a toast (dismiss button / auto-expiry).
    pub fn dismiss_toast(&self, id: u64) {
        let mut toasts = self.toasts;
        toasts.write().retain(|t| t.id != id);
    }

    /// Open a modal (replaces any current one).
    pub fn open_dialog(&self, request: DialogRequest) {
        let mut dialog = self.dialog;
        dialog.set(Some(request));
    }

    /// Close the modal layer.
    pub fn close_dialog(&self) {
        let mut dialog = self.dialog;
        dialog.set(None);
    }

    /// Navigate, clamped to currently visible sections.
    pub fn navigate(&self, route: Route) {
        let clamped = route.clamped(&self.state.peek());
        let mut current = self.route;
        current.set(clamped);
    }
}

/// Grab the bridge from context.
#[must_use]
pub fn use_bridge() -> GameBridge {
    use_context()
}

/// THE state-read pattern: a memo selector. Recomputes every tick (cheap
/// field reads) but only re-renders subscribers when the selected value
/// actually changed. `Decimal` is `PartialEq`, so resource selectors are
/// fine.
#[must_use]
pub fn use_slice<T, F>(select: F) -> Memo<T>
where
    T: PartialEq + 'static,
    F: Fn(&GameState) -> T + 'static,
{
    let bridge = use_bridge();
    use_memo(move || select(&bridge.state.read()))
}
