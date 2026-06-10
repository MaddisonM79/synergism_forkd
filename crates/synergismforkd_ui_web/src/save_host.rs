//! Host-tier save persistence + the autosave loop (the `Synergism.ts` save
//! glue: `setNamedInterval('save', saveSynergy, 5000)` + load-on-boot +
//! export-on-demand).
//!
//! The game logic and the save format are headless and time-free; this is the
//! host seam that drives them on the browser's clock and parks the bytes in
//! `localStorage`. The orchestration ([`SaveHost`]) is platform-agnostic and
//! unit-tested on native; the only wasm-gated pieces are the [`SaveStorage`]
//! backend ([`web::LocalStorageBackend`]) and the wall-clock ([`web::now_ms`]).

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

/// Owns the live [`GameState`], the autosave accumulator, and the storage
/// backend. The browser app constructs one with [`SaveHost::boot`], calls
/// [`SaveHost::tick`] each frame, [`SaveHost::export`] on a manual export, and
/// [`SaveHost::reset`] on a hard reset.
pub struct SaveHost<S: SaveStorage> {
    state: GameState,
    storage: S,
    since_last_save_s: f64,
}

impl<S: SaveStorage> SaveHost<S> {
    /// Load the persisted save (recomputing achievement points via
    /// [`import_from_string_with_meta`]) or start fresh. Returns the host plus
    /// a [`BootOutcome`] describing what happened.
    pub fn boot(storage: S) -> (Self, BootOutcome) {
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
            Self {
                state,
                storage,
                since_last_save_s: 0.0,
            },
            outcome,
        )
    }

    /// Borrow the live game state (for the UI render).
    pub fn state(&self) -> &GameState {
        &self.state
    }

    /// Mutably borrow the live game state (rarely needed; prefer [`Self::tick`]).
    pub fn state_mut(&mut self) -> &mut GameState {
        &mut self.state
    }

    /// Advance the game by one tick, then autosave if at least
    /// [`AUTOSAVE_INTERVAL_S`] of game time has elapsed since the last save.
    /// `now_ms` is the host wall-clock used to stamp any save this tick.
    /// Returns the tick's [`TickOutput`] event stream for the UI.
    pub fn tick(&mut self, input: &TackInput, now_ms: u64) -> TickOutput {
        let output = tack(&mut self.state, input);
        self.since_last_save_s += input.dt;
        if self.since_last_save_s >= AUTOSAVE_INTERVAL_S {
            self.persist(now_ms);
        }
        output
    }

    /// Force a save now (e.g. on `visibilitychange` / `beforeunload`), stamping
    /// the host clock and resetting the autosave accumulator.
    pub fn persist(&mut self, now_ms: u64) {
        self.since_last_save_s = 0.0;
        if let Ok(blob) = export_to_string_at(&self.state, Some(now_ms)) {
            self.storage.write(&blob);
        }
    }

    /// A *real* export (the legacy `exportSynergism` happy path): claim the
    /// accrued export rewards into the live state, persist, and hand back the
    /// claim plus the export blob (for clipboard / download). `None` blob means
    /// encoding failed.
    pub fn export(&mut self, now_ms: u64) -> (ExportRewardClaim, Option<String>) {
        let claim = claim_export_rewards(&mut self.state);
        self.since_last_save_s = 0.0;
        let blob = export_to_string_at(&self.state, Some(now_ms)).ok();
        if let Some(blob) = &blob {
            self.storage.write(blob);
        }
        (claim, blob)
    }

    /// Replace a foreign/clipboard save blob, recomputing on load. Returns the
    /// [`BootOutcome`] (so the caller can reject a corrupt paste) and, on
    /// success, persists it under [`SAVE_KEY`].
    pub fn import(&mut self, blob: &str, now_ms: u64) -> BootOutcome {
        match import_from_string_with_meta(blob) {
            Ok((state, meta)) => {
                self.state = state;
                self.since_last_save_s = 0.0;
                self.persist(now_ms);
                BootOutcome::Loaded {
                    saved_at_ms: meta.saved_at_ms,
                }
            }
            Err(_) => BootOutcome::Corrupt,
        }
    }

    /// Hard reset: fresh state + cleared storage (the legacy `resetGame`).
    pub fn reset(&mut self) {
        self.state = reset_save();
        self.since_last_save_s = 0.0;
        self.storage.clear();
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

    /// The host wall-clock (Unix epoch milliseconds) for stamping saves. The
    /// Dioxus root passes this into [`super::SaveHost::tick`] /
    /// [`super::SaveHost::export`].
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
        let (_host, outcome) = SaveHost::boot(store.clone());
        assert_eq!(outcome, BootOutcome::Fresh);
    }

    #[test]
    fn boot_loads_a_stamped_save_and_recovers_its_timestamp() {
        // Write a stamped save through the host, then boot a second host over
        // the same backing store.
        let store = MemStorage::default();
        {
            let (mut host, _) = SaveHost::boot(store.clone());
            host.state_mut().challenges.challenge_completions[5] = 4.0;
            host.persist(1_700_000_000_000);
        }
        let (host, outcome) = SaveHost::boot(store.clone());
        assert_eq!(
            outcome,
            BootOutcome::Loaded {
                saved_at_ms: Some(1_700_000_000_000)
            }
        );
        assert_eq!(host.state().challenges.challenge_completions[5], 4.0);
    }

    #[test]
    fn corrupt_blob_boots_fresh_without_clobbering() {
        let store = MemStorage::with("this is not a valid base64 postcard save !!!");
        let (host, outcome) = SaveHost::boot(store.clone());
        assert_eq!(outcome, BootOutcome::Corrupt);
        // Fresh in memory…
        assert_eq!(host.state().challenges.challenge_completions[5], 0.0);
        // …and the original bytes are left untouched for the host to inspect.
        assert_eq!(
            store.slot.borrow().as_deref(),
            Some("this is not a valid base64 postcard save !!!")
        );
    }

    #[test]
    fn autosave_only_fires_after_the_interval() {
        let store = MemStorage::default();
        let (mut host, _) = SaveHost::boot(store.clone());
        assert!(store.read().is_none());

        // Four 1s ticks: under the 5s threshold, nothing persisted yet.
        for _ in 0..4 {
            host.tick(&tick_input(1.0), 10);
        }
        assert!(store.read().is_none(), "no save before the 5s interval");

        // The fifth crosses 5s → one autosave.
        host.tick(&tick_input(1.0), 12345);
        assert!(store.read().is_some(), "autosave at the interval");
    }

    #[test]
    fn autosave_accumulator_resets_between_saves() {
        let store = MemStorage::default();
        let (mut host, _) = SaveHost::boot(store.clone());
        // One big tick trips the save; clear the slot to detect the next one.
        host.tick(&tick_input(6.0), 1);
        assert!(store.read().is_some());
        store.clear();
        // A small follow-up tick must NOT immediately re-save (accumulator was
        // reset, not left above threshold).
        host.tick(&tick_input(1.0), 2);
        assert!(store.read().is_none(), "accumulator reset after a save");
    }

    #[test]
    fn export_claims_rewards_persists_and_returns_blob() {
        use synergismforkd_logic::state::golden_quarks::GQ_GOLDEN_QUARKS_3;
        use synergismforkd_logic::state::GoldenQuarkUpgrade;

        let store = MemStorage::default();
        let (mut host, _) = SaveHost::boot(store.clone());
        // Arm an export reward: goldenQuarks3 lvl 1 + a full timer window.
        host.state_mut().golden_quarks.upgrades[GQ_GOLDEN_QUARKS_3] = GoldenQuarkUpgrade {
            level: 1.0,
            ..GoldenQuarkUpgrade::default()
        };
        host.state_mut().golden_quarks.golden_quarks_timer = 3600.0;

        let (claim, blob) = host.export(9_999);
        assert_eq!(claim.golden_quarks, 1.0);
        assert!(blob.is_some());
        // The claim mutated the live state and the persisted blob reflects it.
        assert_eq!(host.state().golden_quarks.golden_quarks.to_number(), 1.0);
        assert!(store.read().is_some());
    }

    #[test]
    fn import_replaces_state_and_persists() {
        // Build a donor blob from one host…
        let donor = MemStorage::default();
        let donor_blob = {
            let (mut host, _) = SaveHost::boot(donor.clone());
            host.state_mut().challenges.challenge_completions[9] = 6.0;
            host.export(1).1.expect("export blob")
        };

        // …and import it into a fresh host over a clean store.
        let store = MemStorage::default();
        let (mut host, _) = SaveHost::boot(store.clone());
        let outcome = host.import(&donor_blob, 77);
        assert!(matches!(outcome, BootOutcome::Loaded { .. }));
        assert_eq!(host.state().challenges.challenge_completions[9], 6.0);
        assert!(store.read().is_some());

        // A garbage paste is rejected and leaves the state untouched.
        assert_eq!(host.import("garbage!!!", 78), BootOutcome::Corrupt);
        assert_eq!(host.state().challenges.challenge_completions[9], 6.0);
    }

    #[test]
    fn reset_clears_state_and_storage() {
        let store = MemStorage::default();
        let (mut host, _) = SaveHost::boot(store.clone());
        host.state_mut().challenges.challenge_completions[5] = 9.0;
        host.persist(1);
        assert!(store.read().is_some());

        host.reset();
        assert_eq!(host.state().challenges.challenge_completions[5], 0.0);
        assert!(store.read().is_none(), "storage cleared on reset");
    }
}
