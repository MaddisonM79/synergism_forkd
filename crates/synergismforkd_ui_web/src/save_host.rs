//! Host-tier save persistence + the autosave cadence (the `Synergism.ts`
//! save glue: `setNamedInterval('save', saveSynergy, 5000)` + load-on-boot +
//! export-on-demand).
//!
//! The host is **persistence-only**: the live [`GameState`] belongs to the
//! UI layer (a Dioxus signal); every method borrows it. The orchestration is
//! platform-agnostic and unit-tested on native; the only wasm-gated pieces
//! are the [`SaveStorage`] backend ([`web::LocalStorageBackend`]) and the
//! wall-clock ([`web::now_ms`]).

use synergismforkd_logic::{
    claim_export_rewards, tack, ExportRewardClaim, GameState, TackInput, TickOutput,
};
use synergismforkd_save::{export_to_string_at, import_from_string_with_meta, reset_save};

/// `localStorage` key holding the base64 save blob. The legacy game used
/// `Synergysave2`; the fork's save format is incompatible, so it gets its own
/// key to avoid colliding with a legacy save on the same origin.
pub const SAVE_KEY: &str = "SynergismForkdSave";

/// Autosave cadence in seconds — the host saves at most this often, matching
/// the legacy `setNamedInterval('save', saveSynergy, 5000)` (5 s).
pub const AUTOSAVE_INTERVAL_S: f64 = 5.0;

/// Where a [`SaveHost::boot`] state came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootOutcome {
    /// No save present — started from a fresh [`reset_save`].
    Fresh,
    /// A stored save was loaded. `saved_at_ms` is the host wall-clock it was
    /// written at (the offline-progress anchor), if it carried one.
    Loaded {
        /// Save-time wall-clock (Unix epoch ms), or `None` if unstamped.
        saved_at_ms: Option<u64>,
    },
    /// A save was present but could not be decoded; started fresh instead.
    /// The host should surface this rather than silently overwrite.
    Corrupt,
}

/// Persistent-storage backend. Abstracted so the autosave orchestration is
/// testable on native (with an in-memory fake) while the browser build wires
/// `localStorage`.
pub trait SaveStorage {
    /// Read the stored blob, or `None` if absent / unreadable.
    fn read(&self) -> Option<String>;
    /// Write (overwrite) the stored blob. Failures are swallowed — a dropped
    /// autosave is non-fatal and the next tick retries.
    fn write(&self, blob: &str);
    /// Remove the stored blob (the reset path).
    fn clear(&self);
}

/// Owns the storage backend and the autosave accumulator — NOT the game
/// state. The shell constructs one with [`SaveHost::boot`] (which hands the
/// loaded state back for the UI to own), calls [`SaveHost::tick`] each
/// frame, [`SaveHost::export`] on a manual export, and [`SaveHost::reset`]
/// on a hard reset.
pub struct SaveHost<S: SaveStorage> {
    storage: S,
    since_last_save_s: f64,
}

impl<S: SaveStorage> SaveHost<S> {
    /// Load the persisted save (recomputing achievement points via
    /// [`import_from_string_with_meta`]) or start fresh. Returns the loaded
    /// state (for the UI to own), a [`BootOutcome`], and the host.
    pub fn boot(storage: S) -> (GameState, BootOutcome, Self) {
        let (state, outcome) = match storage.read() {
            None => (reset_save(), BootOutcome::Fresh),
            Some(blob) => match import_from_string_with_meta(&blob) {
                Ok((state, meta)) => (
                    state,
                    BootOutcome::Loaded {
                        saved_at_ms: meta.saved_at_ms,
                    },
                ),
                // A corrupt/foreign blob must not be clobbered blindly — start
                // fresh in memory but leave the bytes for the host to inspect.
                Err(_) => (reset_save(), BootOutcome::Corrupt),
            },
        };
        (
            state,
            outcome,
            Self {
                storage,
                since_last_save_s: 0.0,
            },
        )
    }

    /// Advance the game by one tick, then autosave if at least
    /// [`AUTOSAVE_INTERVAL_S`] of game time has elapsed since the last save.
    /// `now_ms` is the host wall-clock used to stamp any save this tick.
    /// Returns the tick's [`TickOutput`] event stream for the UI.
    pub fn tick(&mut self, state: &mut GameState, input: &TackInput, now_ms: u64) -> TickOutput {
        let output = tack(state, input);
        self.since_last_save_s += input.dt;
        if self.since_last_save_s >= AUTOSAVE_INTERVAL_S {
            self.persist(state, now_ms);
        }
        output
    }

    /// Force a save now (e.g. on `visibilitychange` / `beforeunload`), stamping
    /// the host clock and resetting the autosave accumulator.
    pub fn persist(&mut self, state: &GameState, now_ms: u64) {
        self.since_last_save_s = 0.0;
        if let Ok(blob) = export_to_string_at(state, Some(now_ms)) {
            self.storage.write(&blob);
        }
    }

    /// A *real* export (the legacy `exportSynergism` happy path): claim the
    /// accrued export rewards into the live state, persist, and hand back the
    /// claim plus the export blob (for clipboard / download). `None` blob means
    /// encoding failed.
    pub fn export(
        &mut self,
        state: &mut GameState,
        now_ms: u64,
    ) -> (ExportRewardClaim, Option<String>) {
        let claim = claim_export_rewards(state);
        self.since_last_save_s = 0.0;
        let blob = export_to_string_at(state, Some(now_ms)).ok();
        if let Some(blob) = &blob {
            self.storage.write(blob);
        }
        (claim, blob)
    }

    /// Decode a foreign/clipboard save blob (recomputing on load) and persist
    /// it. Returns the imported state for the UI to install, or `None` for a
    /// corrupt paste (current state untouched, nothing persisted).
    pub fn import(&mut self, blob: &str, now_ms: u64) -> Option<GameState> {
        match import_from_string_with_meta(blob) {
            Ok((state, _meta)) => {
                self.persist(&state, now_ms);
                Some(state)
            }
            Err(_) => None,
        }
    }

    /// Hard reset: cleared storage + a fresh state for the UI to install
    /// (the legacy `resetGame`).
    pub fn reset(&mut self) -> GameState {
        self.since_last_save_s = 0.0;
        self.storage.clear();
        reset_save()
    }
}

/// Browser backend: `localStorage` + the `Date.now()` wall-clock. Wasm-only —
/// `web-sys` / `js-sys` are not in the native dependency graph.
#[cfg(target_arch = "wasm32")]
pub mod web {
    use super::SaveStorage;

    /// `localStorage`-backed [`SaveStorage`]. Reads `window.localStorage`
    /// lazily on each call so a host with storage disabled (private mode) just
    /// degrades to no-persistence instead of panicking at construction.
    pub struct LocalStorageBackend {
        key: String,
    }

    impl LocalStorageBackend {
        /// Back the given key (use [`super::SAVE_KEY`] for the main save).
        #[must_use]
        pub fn new(key: impl Into<String>) -> Self {
            Self { key: key.into() }
        }

        fn storage() -> Option<web_sys::Storage> {
            web_sys::window()?.local_storage().ok().flatten()
        }
    }

    impl SaveStorage for LocalStorageBackend {
        fn read(&self) -> Option<String> {
            Self::storage()?.get_item(&self.key).ok().flatten()
        }

        fn write(&self, blob: &str) {
            if let Some(storage) = Self::storage() {
                // A quota / security failure is non-fatal — the next autosave
                // retries.
                let _ = storage.set_item(&self.key, blob);
            }
        }

        fn clear(&self) {
            if let Some(storage) = Self::storage() {
                let _ = storage.remove_item(&self.key);
            }
        }
    }

    /// The host wall-clock (Unix epoch milliseconds) for stamping saves.
    #[must_use]
    pub fn now_ms() -> u64 {
        js_sys::Date::now() as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    /// In-memory [`SaveStorage`] standing in for `localStorage` on native. The
    /// slot is shared via `Rc` so a clone can be handed to the [`SaveHost`]
    /// (which owns its storage) while the test keeps a handle to assert on.
    #[derive(Default, Clone)]
    struct MemStorage {
        slot: Rc<RefCell<Option<String>>>,
    }

    impl MemStorage {
        fn with(blob: &str) -> Self {
            Self {
                slot: Rc::new(RefCell::new(Some(blob.to_string()))),
            }
        }
    }

    impl SaveStorage for MemStorage {
        fn read(&self) -> Option<String> {
            self.slot.borrow().clone()
        }
        fn write(&self, blob: &str) {
            *self.slot.borrow_mut() = Some(blob.to_string());
        }
        fn clear(&self) {
            *self.slot.borrow_mut() = None;
        }
    }

    fn tick_input(dt: f64) -> TackInput {
        TackInput {
            dt,
            ..TackInput::default()
        }
    }

    #[test]
    fn boot_fresh_when_storage_empty() {
        let store = MemStorage::default();
        let (_state, outcome, _host) = SaveHost::boot(store.clone());
        assert_eq!(outcome, BootOutcome::Fresh);
    }

    #[test]
    fn boot_loads_a_stamped_save_and_recovers_its_timestamp() {
        // Write a stamped save through the host, then boot a second host over
        // the same backing store.
        let store = MemStorage::default();
        {
            let (mut state, _, mut host) = SaveHost::boot(store.clone());
            state.challenges.challenge_completions[5] = 4.0;
            host.persist(&state, 1_700_000_000_000);
        }
        let (state, outcome, _host) = SaveHost::boot(store.clone());
        assert_eq!(
            outcome,
            BootOutcome::Loaded {
                saved_at_ms: Some(1_700_000_000_000)
            }
        );
        assert_eq!(state.challenges.challenge_completions[5], 4.0);
    }

    #[test]
    fn corrupt_blob_boots_fresh_without_clobbering() {
        let store = MemStorage::with("this is not a valid base64 postcard save !!!");
        let (state, outcome, _host) = SaveHost::boot(store.clone());
        assert_eq!(outcome, BootOutcome::Corrupt);
        // Fresh in memory…
        assert_eq!(state.challenges.challenge_completions[5], 0.0);
        // …and the original bytes are left untouched for the host to inspect.
        assert_eq!(
            store.slot.borrow().as_deref(),
            Some("this is not a valid base64 postcard save !!!")
        );
    }

    #[test]
    fn autosave_only_fires_after_the_interval() {
        let store = MemStorage::default();
        let (mut state, _, mut host) = SaveHost::boot(store.clone());
        assert!(store.read().is_none());

        // Four 1s ticks: under the 5s threshold, nothing persisted yet.
        for _ in 0..4 {
            host.tick(&mut state, &tick_input(1.0), 10);
        }
        assert!(store.read().is_none(), "no save before the 5s interval");

        // The fifth crosses 5s → one autosave.
        host.tick(&mut state, &tick_input(1.0), 12345);
        assert!(store.read().is_some(), "autosave at the interval");
    }

    #[test]
    fn autosave_accumulator_resets_between_saves() {
        let store = MemStorage::default();
        let (mut state, _, mut host) = SaveHost::boot(store.clone());
        // One big tick trips the save; clear the slot to detect the next one.
        host.tick(&mut state, &tick_input(6.0), 1);
        assert!(store.read().is_some());
        store.clear();
        // A small follow-up tick must NOT immediately re-save (accumulator was
        // reset, not left above threshold).
        host.tick(&mut state, &tick_input(1.0), 2);
        assert!(store.read().is_none(), "accumulator reset after a save");
    }

    #[test]
    fn export_claims_rewards_persists_and_returns_blob() {
        use synergismforkd_logic::state::golden_quarks::GQ_GOLDEN_QUARKS_3;
        use synergismforkd_logic::state::GoldenQuarkUpgrade;

        let store = MemStorage::default();
        let (mut state, _, mut host) = SaveHost::boot(store.clone());
        // Arm an export reward: goldenQuarks3 lvl 1 + a full timer window.
        state.golden_quarks.upgrades[GQ_GOLDEN_QUARKS_3] = GoldenQuarkUpgrade {
            level: 1.0,
            ..GoldenQuarkUpgrade::default()
        };
        state.golden_quarks.golden_quarks_timer = 3600.0;

        let (claim, blob) = host.export(&mut state, 9_999);
        assert_eq!(claim.golden_quarks, 1.0);
        assert!(blob.is_some());
        // The claim mutated the live state and the persisted blob reflects it.
        assert_eq!(state.golden_quarks.golden_quarks.to_number(), 1.0);
        assert!(store.read().is_some());
    }

    #[test]
    fn import_returns_state_and_persists() {
        // Build a donor blob from one host…
        let donor = MemStorage::default();
        let donor_blob = {
            let (mut state, _, mut host) = SaveHost::boot(donor.clone());
            state.challenges.challenge_completions[9] = 6.0;
            host.export(&mut state, 1).1.expect("export blob")
        };

        // …and import it into a fresh host over a clean store.
        let store = MemStorage::default();
        let (_state, _, mut host) = SaveHost::boot(store.clone());
        let imported = host.import(&donor_blob, 77).expect("valid import");
        assert_eq!(imported.challenges.challenge_completions[9], 6.0);
        assert!(store.read().is_some());

        // A garbage paste is rejected and persists nothing new.
        store.clear();
        assert!(host.import("garbage!!!", 78).is_none());
        assert!(store.read().is_none());
    }

    #[test]
    fn reset_clears_storage_and_hands_back_fresh_state() {
        let store = MemStorage::default();
        let (mut state, _, mut host) = SaveHost::boot(store.clone());
        state.challenges.challenge_completions[5] = 9.0;
        host.persist(&state, 1);
        assert!(store.read().is_some());

        let fresh = host.reset();
        assert_eq!(fresh.challenges.challenge_completions[5], 0.0);
        assert!(store.read().is_none(), "storage cleared on reset");
    }
}
