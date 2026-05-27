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

use postcard::{from_bytes, to_allocvec};
use serde::{Deserialize, Serialize};
use synergismforkd_common as _;
use synergismforkd_logic::GameState;

/// Current save schema version. Bump when a breaking schema change ships
/// and a new `SaveV<N>` migration arm is added.
pub const CURRENT_VERSION: u16 = 1;

/// Wire-format envelope. The first thing serialized in every save; the
/// `version` field is how a reader knows which `SaveV<N>` shape follows.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SaveEnvelope {
    version: u16,
    payload: SaveV1,
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
}

impl core::fmt::Display for SaveError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Postcard(e) => write!(f, "postcard: {e}"),
            Self::UnknownVersion(v) => write!(
                f,
                "save version {v} is newer than this build's CURRENT_VERSION ({CURRENT_VERSION})"
            ),
        }
    }
}

impl std::error::Error for SaveError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Postcard(e) => Some(e),
            Self::UnknownVersion(_) => None,
        }
    }
}

impl From<postcard::Error> for SaveError {
    fn from(e: postcard::Error) -> Self {
        Self::Postcard(e)
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
    let envelope = SaveEnvelope {
        version: CURRENT_VERSION,
        payload: SaveV1 {
            state: state.clone(),
        },
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
    let envelope: SaveEnvelope = from_bytes(bytes)?;
    if envelope.version > CURRENT_VERSION {
        return Err(SaveError::UnknownVersion(envelope.version));
    }
    // Single-arm dispatch today; the migration chain ladder lives here
    // when v2+ lands.
    Ok(envelope.payload.state)
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
}
