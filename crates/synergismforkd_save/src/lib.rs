#![cfg_attr(not(test), deny(clippy::unwrap_used))]

//! Synergism Forkd — save format, (de)serialization, and migrations.
//!
//! Owns a fresh, version-tagged save schema (no compatibility with the
//! legacy TS savefile format). Postcard binary encoding per the locked
//! plan: ~3-5× smaller than JSON, ~30% smaller than bincode,
//! format-stable.
//!
//! ## Versioning contract
//!
//! Every save starts with a private `SaveEnvelope` that pins a `u16`
//! version number on the wire. The current writer always emits the
//! latest version; readers dispatch on the version field and run the
//! appropriate migration chain to reach the current shape.
//!
//! Today there is one version: `SaveV1` is the [`GameState`] composed
//! struct directly. At the first schema break, `SaveV2` becomes a
//! separately-defined struct whose fields mirror what GameState looked
//! like at that break, with a `SaveV1::migrate_to` method (added in
//! that future PR) handling the conversion. The envelope's `version`
//! field is what lets a fresh reader spot an old save and route
//! through the migration chain.
//!
//! ## SavedNumber design (intentionally omitted)
//!
//! The locked plan mentioned a `SavedNumber` enum for magnitude-based
//! compaction (small values stored as u64/f64, large as Decimal). Per
//! Ledger Finding 10, magnitude-based dynamic bucketing introduces
//! precision loss at the threshold and was rejected — each field's
//! static Rust type is now the source of truth for its on-disk shape.
//! `Decimal` fields always encode as Decimal; `u32` fields always encode
//! as u32; etc. There is no `SavedNumber`.

use base64::prelude::{Engine as _, BASE64_STANDARD};
use postcard::{from_bytes, to_allocvec};
use serde::{Deserialize, Serialize};
use synergismforkd_common as _;
use synergismforkd_logic::{recompute_achievement_points, GameState};

/// Current save schema version. Bump when a breaking schema change ships
/// and a new `SaveV<N>` migration arm is added.
pub const CURRENT_VERSION: u16 = 1;

/// Wire-format envelope. The first thing serialized in every save; the
/// `version` field is how a reader knows which `SaveV<N>` shape follows.
///
/// `saved_at_ms` is the host-stamped wall-clock (Unix epoch milliseconds) at
/// save time — the Rust analogue of the legacy `player.offlinetick` /
/// `lastExportedSave`. It lives on the envelope, not in [`GameState`], because
/// the logic crate has no time-of-day: the host (which owns the clock) passes
/// it in via [`save_at`] / [`export_to_string_at`] and reads it back via
/// [`load_with_meta`] to compute offline-elapsed on load. `None` when the save
/// was written without a timestamp (the plain [`save`] path).
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SaveEnvelope {
    version: u16,
    payload: SaveV1,
    saved_at_ms: Option<u64>,
}

/// Transport-level metadata recovered from a save, independent of the
/// versioned [`GameState`] payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SaveMeta {
    /// Host wall-clock at save time (Unix epoch milliseconds), or `None` if
    /// the save was written without a timestamp. The host diffs this against
    /// "now" to size offline progress.
    pub saved_at_ms: Option<u64>,
}

/// Save schema version 1 — today, equal to [`GameState`] verbatim.
///
/// At the first schema break, `SaveV1` stays frozen at its current shape
/// (so old saves can still be read) and a new `SaveV2` struct gets
/// introduced. A `From<SaveV1> for SaveV2` impl handles the migration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveV1 {
    /// Composed game state. Today this is the live [`GameState`]; once a
    /// schema break ships, this field becomes a stable mirror that
    /// records what `GameState` looked like at v1.
    pub state: GameState,
}

/// Errors from [`save`] / [`load`].
#[derive(Debug)]
#[non_exhaustive]
pub enum SaveError {
    /// Serialization or deserialization failed. Postcard's error is
    /// wrapped verbatim.
    Postcard(postcard::Error),
    /// The save header declared a version this build does not know how
    /// to read. The migration chain stops one version short of the
    /// current build.
    UnknownVersion(u16),
    /// The import string was not valid standard base64 (the export/import
    /// string path only; the raw [`load`] byte path never produces this).
    Base64(base64::DecodeError),
}

impl core::fmt::Display for SaveError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Postcard(e) => write!(f, "postcard: {e}"),
            Self::UnknownVersion(v) => write!(
                f,
                "save version {v} is newer than this build's CURRENT_VERSION ({CURRENT_VERSION})"
            ),
            Self::Base64(e) => write!(f, "base64: {e}"),
        }
    }
}

impl std::error::Error for SaveError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Postcard(e) => Some(e),
            Self::UnknownVersion(_) => None,
            Self::Base64(e) => Some(e),
        }
    }
}

impl From<postcard::Error> for SaveError {
    fn from(e: postcard::Error) -> Self {
        Self::Postcard(e)
    }
}

impl From<base64::DecodeError> for SaveError {
    fn from(e: base64::DecodeError) -> Self {
        Self::Base64(e)
    }
}

/// Encode a [`GameState`] to a postcard byte stream.
///
/// The encoded payload starts with a private envelope header carrying
/// [`CURRENT_VERSION`]; readers route on that field.
///
/// # Errors
///
/// Returns [`SaveError::Postcard`] if the underlying postcard encoder
/// fails. In practice this requires allocator exhaustion or a misbehaving
/// `Serialize` impl on a state field — the happy-path GameState shape
/// is always encodable.
pub fn save(state: &GameState) -> Result<Vec<u8>, SaveError> {
    save_at(state, None)
}

/// Encode a [`GameState`] with a host-stamped save timestamp (Unix epoch
/// milliseconds). The timestamp rides on the envelope, not the state, and is
/// recovered with [`load_with_meta`]. Use this on the host save path where the
/// wall-clock is available; [`save`] is the `None`-timestamp shorthand.
///
/// # Errors
///
/// Returns [`SaveError::Postcard`] if the underlying postcard encoder fails
/// (see [`save`]).
pub fn save_at(state: &GameState, saved_at_ms: Option<u64>) -> Result<Vec<u8>, SaveError> {
    let envelope = SaveEnvelope {
        version: CURRENT_VERSION,
        payload: SaveV1 {
            state: state.clone(),
        },
        saved_at_ms,
    };
    Ok(to_allocvec(&envelope)?)
}

/// Decode a postcard byte stream back into a [`GameState`], running the
/// version migration chain as needed.
///
/// Today there is only `SaveV1`; older versions are unreachable. When
/// `SaveV2` ships, this function gains a `match envelope.version` arm
/// that deserializes the old struct shape and runs the migration.
///
/// # Errors
///
/// - [`SaveError::Postcard`] if the bytes are not a well-formed postcard
///   stream or the encoded shape does not match the current envelope.
/// - [`SaveError::UnknownVersion`] if the header reports a version
///   greater than [`CURRENT_VERSION`] (a save written by a newer build).
pub fn load(bytes: &[u8]) -> Result<GameState, SaveError> {
    Ok(load_with_meta(bytes)?.0)
}

/// Decode a postcard byte stream into a [`GameState`] **and** its transport
/// [`SaveMeta`] (the host-stamped save timestamp). The host uses the meta to
/// compute offline-elapsed; [`load`] is the meta-dropping shorthand.
///
/// # Errors
///
/// - [`SaveError::Postcard`] if the bytes are not a well-formed postcard
///   stream or the encoded shape does not match the current envelope.
/// - [`SaveError::UnknownVersion`] if the header reports a version greater than
///   [`CURRENT_VERSION`] (a save written by a newer build).
pub fn load_with_meta(bytes: &[u8]) -> Result<(GameState, SaveMeta), SaveError> {
    let envelope: SaveEnvelope = from_bytes(bytes)?;
    if envelope.version > CURRENT_VERSION {
        return Err(SaveError::UnknownVersion(envelope.version));
    }
    // Single-arm dispatch today; the migration chain ladder lives here
    // when v2+ lands.
    let meta = SaveMeta {
        saved_at_ms: envelope.saved_at_ms,
    };
    Ok((envelope.payload.state, meta))
}

/// Encode a [`GameState`] to a portable base64 string — the user-facing export
/// blob (clipboard / file download), mirroring the TS `exportSynergism`
/// operation. Standard base64 over the postcard bytes from [`save`]; no
/// compression for v1 (postcard is already compact).
///
/// # Errors
///
/// [`SaveError::Postcard`] if encoding the state fails (see [`save`]).
pub fn export_to_string(state: &GameState) -> Result<String, SaveError> {
    Ok(BASE64_STANDARD.encode(save(state)?))
}

/// Like [`export_to_string`] but stamps the host wall-clock (Unix epoch
/// milliseconds) into the envelope, recoverable with
/// [`import_from_string_with_meta`].
///
/// # Errors
///
/// [`SaveError::Postcard`] if encoding the state fails (see [`save`]).
pub fn export_to_string_at(
    state: &GameState,
    saved_at_ms: Option<u64>,
) -> Result<String, SaveError> {
    Ok(BASE64_STANDARD.encode(save_at(state, saved_at_ms)?))
}

/// Decode a base64 export blob back into a [`GameState`], then rebuild the
/// achievement-points total from the loaded bitmap. A loaded save must not
/// trust a possibly-drifted running points field, so
/// [`recompute_achievement_points`] runs on the import path (audit H5).
///
/// # Errors
///
/// - [`SaveError::Base64`] if the string is not valid standard base64.
/// - [`SaveError::Postcard`] / [`SaveError::UnknownVersion`] from [`load`].
pub fn import_from_string(s: &str) -> Result<GameState, SaveError> {
    Ok(import_from_string_with_meta(s)?.0)
}

/// Like [`import_from_string`] but also returns the transport [`SaveMeta`] (the
/// host-stamped save timestamp), for offline-progress sizing on load. The
/// achievement-points recompute (audit H5) runs here too.
///
/// # Errors
///
/// - [`SaveError::Base64`] if the string is not valid standard base64.
/// - [`SaveError::Postcard`] / [`SaveError::UnknownVersion`] from
///   [`load_with_meta`].
pub fn import_from_string_with_meta(s: &str) -> Result<(GameState, SaveMeta), SaveError> {
    let bytes = BASE64_STANDARD.decode(s)?;
    let (mut state, meta) = load_with_meta(&bytes)?;
    recompute_achievement_points(&mut state);
    Ok((state, meta))
}

/// A fresh-start game state — the "reset save" operation (TS `resetGame`'s
/// state portion). Returns [`GameState::default`]; clearing persistent storage
/// is the host's responsibility (filesystem / browser storage are UI-tier).
#[must_use]
pub fn reset_save() -> GameState {
    GameState::default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_default_state() {
        let original = GameState::default();
        let bytes = save(&original).expect("save should succeed for default state");
        let restored = load(&bytes).expect("load should succeed for fresh-saved bytes");

        // Spot-check a handful of slices that should survive byte-for-byte
        // through the round trip. (Can't `assert_eq!(original, restored)`
        // because GameState doesn't derive PartialEq — see state/mod.rs.)
        assert_eq!(original.upgrades.coins, restored.upgrades.coins);
        assert_eq!(original.upgrades.upgrades, restored.upgrades.upgrades);
        assert_eq!(
            original.challenges.challenge_completions,
            restored.challenges.challenge_completions
        );
        assert_eq!(
            original.coin_counters.coins_total,
            restored.coin_counters.coins_total
        );
    }

    #[test]
    fn round_trip_preserves_modified_state() {
        use synergismforkd_logic::state::UpgradesState;

        let mut original = GameState::default();
        original.upgrades = UpgradesState {
            coins: synergismforkd_bignum::Decimal::from_finite(1.234e42),
            ..original.upgrades
        };
        original.challenges.challenge_completions[3] = 7.0;

        let bytes = save(&original).expect("save round-trip");
        let restored = load(&bytes).expect("load round-trip");

        assert_eq!(restored.upgrades.coins, original.upgrades.coins);
        assert_eq!(restored.challenges.challenge_completions[3], 7.0);
    }

    #[test]
    fn save_header_carries_current_version() {
        let bytes = save(&GameState::default()).expect("save default");
        // Postcard's varint encoding for a u16 value of `1` is a single
        // byte `0x01`. The envelope's first field is the version, so
        // byte 0 must be `0x01` today.
        assert_eq!(
            bytes.first(),
            Some(&(CURRENT_VERSION as u8)),
            "envelope must lead with version u16 (varint-encoded)"
        );
    }

    #[test]
    fn unknown_version_is_a_clean_error() {
        // Craft an envelope-shaped byte stream with an impossibly-high
        // version number; load must reject it without panicking.
        // u16::MAX = 65535, postcard varint = `[0xff, 0xff, 0x03]`. Then
        // the body has to be a valid SaveV1 — easier to just write a
        // valid envelope and bump its version field on the wire.
        let mut bytes = save(&GameState::default()).expect("save default");
        // Byte 0 is the version's first varint byte (currently 0x01).
        // Bump it to a value > CURRENT_VERSION.
        bytes[0] = 0x7f; // varint terminator bit clear → value = 127
        match load(&bytes) {
            Err(SaveError::UnknownVersion(v)) => {
                assert_eq!(v, 127, "decoded version should match what we wrote");
            }
            other => panic!("expected UnknownVersion(127), got {other:?}"),
        }
    }

    #[test]
    fn export_import_string_round_trip() {
        let mut original = GameState::default();
        original.upgrades.coins = synergismforkd_bignum::Decimal::from_finite(9.87e30);
        original.challenges.challenge_completions[5] = 3.0;

        let blob = export_to_string(&original).expect("export should succeed");
        let restored = import_from_string(&blob).expect("import should succeed");

        assert_eq!(restored.upgrades.coins, original.upgrades.coins);
        assert_eq!(restored.challenges.challenge_completions[5], 3.0);
    }

    #[test]
    fn import_rejects_non_base64() {
        // Spaces / `!` are not in the standard base64 alphabet.
        match import_from_string("not valid base64 !!!") {
            Err(SaveError::Base64(_)) => {}
            other => panic!("expected Base64 error, got {other:?}"),
        }
    }

    #[test]
    fn import_recomputes_achievement_points() {
        let mut state = GameState::default();
        // Unlock achievement 0 (5 pts) + 173 (25 pts); drift the running total.
        state.achievements.achievements[0] = 1;
        state.achievements.achievements[173] = 1;
        state.achievements.achievement_points = 999.0;

        let blob = export_to_string(&state).expect("export");
        let restored = import_from_string(&blob).expect("import");

        // The loaded total is rebuilt from the bitmap, discarding the drift.
        assert_eq!(restored.achievements.achievement_points, 30.0);
    }

    #[test]
    fn save_timestamp_round_trips_through_envelope() {
        let state = GameState::default();
        // Stamped save carries the timestamp back out via the meta path.
        let bytes = save_at(&state, Some(1_700_000_000_000)).expect("save_at");
        let (_restored, meta) = load_with_meta(&bytes).expect("load_with_meta");
        assert_eq!(meta.saved_at_ms, Some(1_700_000_000_000));

        // The plain save path leaves the timestamp absent.
        let plain = save(&state).expect("save");
        let (_s, meta) = load_with_meta(&plain).expect("load_with_meta plain");
        assert_eq!(meta.saved_at_ms, None);
    }

    #[test]
    fn export_string_carries_timestamp_and_recomputes_points() {
        let mut state = GameState::default();
        state.achievements.achievements[0] = 1; // 5 pts
        state.achievements.achievement_points = 999.0; // drift

        let blob = export_to_string_at(&state, Some(42)).expect("export_at");
        let (restored, meta) = import_from_string_with_meta(&blob).expect("import_with_meta");

        assert_eq!(meta.saved_at_ms, Some(42));
        // H5 recompute still runs on the meta import path.
        assert_eq!(restored.achievements.achievement_points, 5.0);
    }

    #[test]
    fn reset_save_returns_default() {
        let fresh = reset_save();
        let default = GameState::default();
        assert_eq!(fresh.upgrades.coins, default.upgrades.coins);
        assert_eq!(fresh.achievements.achievement_points, 0.0);
    }
}
