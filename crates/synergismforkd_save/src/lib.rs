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
use postcard::{from_bytes, take_from_bytes, to_allocvec};
use serde::{Deserialize, Serialize};
use synergismforkd_bignum::Decimal;
use synergismforkd_common as _;
use synergismforkd_logic::mechanics::rune_data::{RuneUpgradeKind, CORE_RUNE_COUNT};
use synergismforkd_logic::mechanics::rune_upgrade_progression::rune_upgrade_exp_to_level;
use synergismforkd_logic::state::*;
use synergismforkd_logic::{recompute_achievement_points, seed_blank_save, GameState};

/// Current save schema version. Bump when a breaking schema change ships
/// and a new `SaveV<N>` migration arm is added.
///
/// - **v1**: pre-rune-EXP `GameState` (no blessing/spirit EXP).
/// - **v2**: adds `RunesState::rune_blessing_exp` / `rune_spirit_exp`.
pub const CURRENT_VERSION: u16 = 2;

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
struct SaveEnvelope<P> {
    version: u16,
    payload: P,
    saved_at_ms: Option<u64>,
}

/// Decodes just the leading `version` field. Postcard is positional, so the
/// `version` (the envelope's first field) reads first; this lets [`load`]
/// route to the right `SaveV<N>` payload shape before decoding the rest.
#[derive(Deserialize)]
struct VersionPeek {
    version: u16,
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

/// Save schema version 2 — the live [`GameState`] shape (the current writer
/// always emits this).
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SaveV2 {
    state: GameState,
}

/// Save schema version 1 — **frozen** at the pre-rune-EXP shape. Loaded only
/// when migrating an old save; [`GameStateV1`] mirrors what `GameState` looked
/// like before the blessing/spirit EXP fields were added. Migrated to the live
/// state via `From<GameStateV1> for GameState`.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SaveV1 {
    state: GameStateV1,
}

/// Frozen v1 [`GameState`] mirror. Field order **must** match the v1 wire
/// layout (postcard is positional): identical to the live `GameState` except
/// `runes` is the pre-EXP [`RunesStateV1`]. Every other slice type is unchanged
/// since v1, so they reuse the live types.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct GameStateV1 {
    accelerator: AcceleratorState,
    achievements: AchievementsState,
    ambrosia: AmbrosiaState,
    ants: AntsState,
    automation: AutomationState,
    cube_blessings: BlessingValues,
    tesseract_blessings: BlessingValues,
    hypercube_blessings: BlessingValues,
    platonic_blessings: PlatonicBlessings,
    campaigns: CampaignsState,
    challenges: ChallengesState,
    coin_counters: CoinCountersState,
    corruptions: CorruptionsState,
    crystal_upgrades: CrystalUpgradesState,
    cube_balances: CubeBalancesState,
    cube_upgrade_levels: CubeUpgradeLevelsState,
    event_buffs: EventBuffsState,
    g_cache: GCacheState,
    golden_quarks: GoldenQuarksState,
    hepteracts: HepteractsState,
    level: LevelState,
    multiplier: MultiplierState,
    octeract_upgrades: OcteractUpgradesState,
    particle_buildings: ParticleBuildingsState,
    coin_producers: ProducerFamilyState,
    diamond_producers: ProducerFamilyState,
    mythos_producers: ProducerFamilyState,
    particle_producers: ProducerFamilyState,
    quarks: QuarksState,
    red_ambrosia: RedAmbrosiaState,
    researches: ResearchesState,
    reset_counters: ResetCountersState,
    rng: RngState,
    runes: RunesStateV1,
    shop: ShopState,
    singularity: SingularityState,
    talismans: TalismansState,
    tesseract_buildings: TesseractBuildingsState,
    upgrades: UpgradesState,
}

/// Frozen v1 [`RunesState`] mirror — the shape before `rune_blessing_exp` /
/// `rune_spirit_exp` were added. Field order matches the v1 wire layout.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RunesStateV1 {
    rune_levels: [f64; RUNE_COUNT],
    rune_exp: [f64; RUNE_COUNT],
    rune_shards: Decimal,
    rune_blessing_levels: [f64; RUNE_COUNT],
    rune_spirit_levels: [f64; RUNE_COUNT],
    rune_free_levels: [f64; RUNE_COUNT],
}

impl From<GameStateV1> for GameState {
    fn from(v: GameStateV1) -> Self {
        GameState {
            runes: migrate_runes_v1(v.runes),
            accelerator: v.accelerator,
            achievements: v.achievements,
            ambrosia: v.ambrosia,
            ants: v.ants,
            automation: v.automation,
            cube_blessings: v.cube_blessings,
            tesseract_blessings: v.tesseract_blessings,
            hypercube_blessings: v.hypercube_blessings,
            platonic_blessings: v.platonic_blessings,
            campaigns: v.campaigns,
            challenges: v.challenges,
            coin_counters: v.coin_counters,
            corruptions: v.corruptions,
            crystal_upgrades: v.crystal_upgrades,
            cube_balances: v.cube_balances,
            cube_upgrade_levels: v.cube_upgrade_levels,
            event_buffs: v.event_buffs,
            g_cache: v.g_cache,
            golden_quarks: v.golden_quarks,
            hepteracts: v.hepteracts,
            level: v.level,
            multiplier: v.multiplier,
            octeract_upgrades: v.octeract_upgrades,
            particle_buildings: v.particle_buildings,
            coin_producers: v.coin_producers,
            diamond_producers: v.diamond_producers,
            mythos_producers: v.mythos_producers,
            particle_producers: v.particle_producers,
            quarks: v.quarks,
            red_ambrosia: v.red_ambrosia,
            researches: v.researches,
            reset_counters: v.reset_counters,
            rng: v.rng,
            shop: v.shop,
            singularity: v.singularity,
            talismans: v.talismans,
            tesseract_buildings: v.tesseract_buildings,
            upgrades: v.upgrades,
        }
    }
}

/// v1 → v2 runes migration: copy the existing fields and seed the new
/// blessing/spirit EXP arrays from the stored *level* — `expToLevel(level)` —
/// so the level survives the round-trip through `levelFromEXP` (seeding `0`
/// would reset a non-zero level to 0). Only the five core runes have
/// blessings/spirits; the rest stay `0`.
fn migrate_runes_v1(v: RunesStateV1) -> RunesState {
    let mut rune_blessing_exp = [0.0_f64; RUNE_COUNT];
    let mut rune_spirit_exp = [0.0_f64; RUNE_COUNT];
    for i in 0..CORE_RUNE_COUNT {
        rune_blessing_exp[i] = rune_upgrade_exp_to_level(
            RuneUpgradeKind::Blessing.cost_coefficient(i),
            v.rune_blessing_levels[i],
            RuneUpgradeKind::Blessing.levels_per_oom(i),
        )
        .to_number();
        rune_spirit_exp[i] = rune_upgrade_exp_to_level(
            RuneUpgradeKind::Spirit.cost_coefficient(i),
            v.rune_spirit_levels[i],
            RuneUpgradeKind::Spirit.levels_per_oom(i),
        )
        .to_number();
    }
    RunesState {
        rune_levels: v.rune_levels,
        rune_exp: v.rune_exp,
        rune_shards: v.rune_shards,
        rune_blessing_levels: v.rune_blessing_levels,
        rune_blessing_exp,
        rune_spirit_levels: v.rune_spirit_levels,
        rune_spirit_exp,
        rune_free_levels: v.rune_free_levels,
    }
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
        payload: SaveV2 {
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
    // Peek the leading version field, then decode the matching payload shape
    // and run the migration chain up to the current version.
    let (peek, _) = take_from_bytes::<VersionPeek>(bytes)?;
    if peek.version > CURRENT_VERSION {
        return Err(SaveError::UnknownVersion(peek.version));
    }
    match peek.version {
        1 => {
            let envelope: SaveEnvelope<SaveV1> = from_bytes(bytes)?;
            let meta = SaveMeta {
                saved_at_ms: envelope.saved_at_ms,
            };
            Ok((GameState::from(envelope.payload.state), meta))
        }
        // 2 (CURRENT) — and any unreachable lower version decodes as latest.
        _ => {
            let envelope: SaveEnvelope<SaveV2> = from_bytes(bytes)?;
            let meta = SaveMeta {
                saved_at_ms: envelope.saved_at_ms,
            };
            Ok((envelope.payload.state, meta))
        }
    }
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
/// state portion). [`GameState::default`] plus the `blankSave` starting
/// economy ([`seed_blank_save`]: 100 coins, producer/accelerator/multiplier
/// base costs — without it a fresh game is soft-locked). Clearing persistent
/// storage is the host's responsibility (filesystem / browser storage are
/// UI-tier).
#[must_use]
pub fn reset_save() -> GameState {
    let mut state = GameState::default();
    seed_blank_save(&mut state);
    state
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
    fn migrate_runes_v1_seeds_exp_from_level() {
        use synergismforkd_logic::mechanics::rune_upgrade_progression::rune_upgrade_level_from_exp;

        let mut rune_blessing_levels = [0.0_f64; RUNE_COUNT];
        let mut rune_spirit_levels = [0.0_f64; RUNE_COUNT];
        rune_blessing_levels[0] = 8.0; // speed blessing
        rune_spirit_levels[0] = 4.0; // speed spirit
        let migrated = migrate_runes_v1(RunesStateV1 {
            rune_levels: [0.0; RUNE_COUNT],
            rune_exp: [0.0; RUNE_COUNT],
            rune_shards: Decimal::zero(),
            rune_blessing_levels,
            rune_spirit_levels,
            rune_free_levels: [0.0; RUNE_COUNT],
        });

        // Levels survive the migration…
        assert_eq!(migrated.rune_blessing_levels[0], 8.0);
        assert_eq!(migrated.rune_spirit_levels[0], 4.0);
        // …and the seeded EXP re-derives the same level (seeding 0 would reset it).
        let d = rune_upgrade_level_from_exp(
            Decimal::from_finite(migrated.rune_blessing_exp[0]),
            RuneUpgradeKind::Blessing.cost_coefficient(0),
            RuneUpgradeKind::Blessing.levels_per_oom(0),
        );
        let level = if d.needs_float_bump {
            d.levels + 1.0
        } else {
            d.levels
        };
        assert_eq!(level, 8.0);
    }

    #[test]
    fn v1_save_blob_loads_and_migrates_to_current() {
        // Build a frozen-v1 envelope with a non-zero speed-blessing level, encode
        // it, and load through the version dispatch + migration.
        let g = GameState::default();
        let mut rune_blessing_levels = [0.0_f64; RUNE_COUNT];
        rune_blessing_levels[0] = 6.0;
        let v1_state = GameStateV1 {
            accelerator: g.accelerator,
            achievements: g.achievements,
            ambrosia: g.ambrosia,
            ants: g.ants,
            automation: g.automation,
            cube_blessings: g.cube_blessings,
            tesseract_blessings: g.tesseract_blessings,
            hypercube_blessings: g.hypercube_blessings,
            platonic_blessings: g.platonic_blessings,
            campaigns: g.campaigns,
            challenges: g.challenges,
            coin_counters: g.coin_counters,
            corruptions: g.corruptions,
            crystal_upgrades: g.crystal_upgrades,
            cube_balances: g.cube_balances,
            cube_upgrade_levels: g.cube_upgrade_levels,
            event_buffs: g.event_buffs,
            g_cache: g.g_cache,
            golden_quarks: g.golden_quarks,
            hepteracts: g.hepteracts,
            level: g.level,
            multiplier: g.multiplier,
            octeract_upgrades: g.octeract_upgrades,
            particle_buildings: g.particle_buildings,
            coin_producers: g.coin_producers,
            diamond_producers: g.diamond_producers,
            mythos_producers: g.mythos_producers,
            particle_producers: g.particle_producers,
            quarks: g.quarks,
            red_ambrosia: g.red_ambrosia,
            researches: g.researches,
            reset_counters: g.reset_counters,
            rng: g.rng,
            runes: RunesStateV1 {
                rune_levels: [0.0; RUNE_COUNT],
                rune_exp: [0.0; RUNE_COUNT],
                rune_shards: Decimal::zero(),
                rune_blessing_levels,
                rune_spirit_levels: [0.0; RUNE_COUNT],
                rune_free_levels: [0.0; RUNE_COUNT],
            },
            shop: g.shop,
            singularity: g.singularity,
            talismans: g.talismans,
            tesseract_buildings: g.tesseract_buildings,
            upgrades: g.upgrades,
        };
        let envelope = SaveEnvelope {
            version: 1,
            payload: SaveV1 { state: v1_state },
            saved_at_ms: Some(123),
        };
        let bytes = to_allocvec(&envelope).expect("encode v1 envelope");

        let (restored, meta) = load_with_meta(&bytes).expect("load v1 blob");
        assert_eq!(meta.saved_at_ms, Some(123));
        // The blessing level survived migration, and its EXP was seeded.
        assert_eq!(restored.runes.rune_blessing_levels[0], 6.0);
        assert!(restored.runes.rune_blessing_exp[0] > 0.0);
    }

    #[test]
    fn current_v2_save_round_trips_blessing_spirit_exp() {
        let mut original = GameState::default();
        original.runes.rune_blessing_exp[0] = 1234.5;
        original.runes.rune_spirit_exp[1] = 6789.0;
        let bytes = save(&original).expect("save v2");
        let restored = load(&bytes).expect("load v2");
        assert_eq!(restored.runes.rune_blessing_exp[0], 1234.5);
        assert_eq!(restored.runes.rune_spirit_exp[1], 6789.0);
    }

    #[test]
    fn reset_save_returns_seeded_blank_save() {
        let fresh = reset_save();
        // blankSave starting economy (Synergism.ts:307-345): 100 coins and
        // playable base costs — NOT the all-zero GameState::default().
        assert_eq!(fresh.upgrades.coins.to_number(), 100.0);
        assert_eq!(fresh.coin_producers.cost(1).to_number(), 100.0);
        assert_eq!(fresh.coin_producers.cost(5).to_number(), 8e6);
        assert_eq!(fresh.diamond_producers.cost(1).to_number(), 100.0);
        assert_eq!(fresh.accelerator.accelerator_cost.to_number(), 500.0);
        assert_eq!(fresh.multiplier.multiplier_cost.to_number(), 10_000.0);
        assert_eq!(fresh.achievements.achievement_points, 0.0);
    }
}
