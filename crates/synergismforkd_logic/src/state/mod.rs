//! Sliced game-state types and the composed [`GameState`] that owns one
//! instance of each.
//!
//! Each mechanic family has its own slice; `GameState` is the union the
//! tick orchestrator threads through every per-tick call. Mechanic
//! functions may take either the full `&GameState` (aggregators) or a
//! narrower `&XState` slice (single-family operations).

use serde::{Deserialize, Serialize};

pub mod accelerator;
pub mod achievements;
pub mod ambrosia;
pub mod ants;
pub mod blessings;
pub mod campaigns;
pub mod challenges;
pub mod coin_counters;
pub mod corruptions;
pub mod crystal_upgrades;
pub mod cube_balances;
pub mod cube_upgrade_levels;
pub mod event_buffs;
pub mod golden_quarks;
pub mod hepteracts;
pub mod level;
pub mod multiplier;
pub mod octeract_upgrades;
pub mod particle_buildings;
pub mod producer;
pub mod quarks;
pub mod red_ambrosia;
pub mod researches;
pub mod reset_counters;
pub mod rng;
pub mod runes;
pub mod shop;
pub mod singularity;
pub mod talismans;
pub mod tesseract_buildings;
pub mod upgrades;

pub use accelerator::AcceleratorState;
pub use achievements::{AchievementsState, ProgressiveAchievementCache};
pub use ambrosia::{AmbrosiaState, AmbrosiaUpgrade};
pub use ants::{
    AntsState, AntsToggles, AutoSacrificeMode, PlayerAntMastery, PlayerAntProducer, RebornELOEntry,
};
pub use blessings::{BlessingValues, PlatonicBlessings};
pub use campaigns::CampaignsState;
pub use challenges::ChallengesState;
pub use coin_counters::CoinCountersState;
pub use corruptions::{
    CorruptionLoadout, CorruptionsState, DEFLATION_INDEX, DILATION_INDEX, DROUGHT_INDEX,
    EXTINCTION_INDEX, HYPERCHALLENGE_INDEX, ILLITERACY_INDEX, RECESSION_INDEX, VISCOSITY_INDEX,
};
pub use crystal_upgrades::{CrystalUpgradesState, CRYSTAL_UPGRADES_DEFAULT_LEN};
pub use cube_balances::CubeBalancesState;
pub use cube_upgrade_levels::CubeUpgradeLevelsState;
pub use event_buffs::EventBuffsState;
pub use golden_quarks::{GoldenQuarkUpgrade, GoldenQuarksState, StoredSpecialCostForm};
pub use hepteracts::{HepteractCraft, HepteractsState};
pub use level::LevelState;
pub use multiplier::MultiplierState;
pub use octeract_upgrades::{OcteractUpgrade, OcteractUpgradesState};
pub use particle_buildings::ParticleBuildingsState;
pub use producer::{BuyAmount, ProducerFamilyState};
pub use quarks::QuarksState;
pub use red_ambrosia::{RedAmbrosiaState, RedAmbrosiaUpgrade};
pub use researches::ResearchesState;
pub use reset_counters::ResetCountersState;
pub use rng::{RngPurpose, RngState};
pub use runes::{RunesState, RUNE_COUNT};
pub use shop::{ShopBuyMaxMode, ShopState};
pub use singularity::{SingularityChallengeState, SingularityState};
pub use talismans::{TalismanRuneAssignment, TalismansState, TALISMAN_COUNT};
pub use tesseract_buildings::{AscendBuildingState, TesseractBuildingsState};
pub use upgrades::{UpgradesState, UPGRADES_DEFAULT_LEN};

/// Composed game state — one instance of every slice the tick orchestrator
/// threads through per-tick calls.
///
/// The 3 cube-layer blessings (`cube_blessings`, `tesseract_blessings`,
/// `hypercube_blessings`) share the [`BlessingValues`] shape but are
/// independent instances. The 4 producer families
/// (`coin_producers`, `diamond_producers`, `mythos_producers`,
/// `particle_producers`) similarly share [`ProducerFamilyState`].
///
/// `Default` constructs a fresh-start game state with zeroed counters,
/// achievement flags set `true` where appropriate, and the legacy default
/// vector lengths (see [`UPGRADES_DEFAULT_LEN`],
/// [`CRYSTAL_UPGRADES_DEFAULT_LEN`]).
///
/// `PartialEq` is intentionally not derived because [`RngState`] holds
/// `Xoshiro256PlusPlus` internals that don't usefully compare. Tests that
/// need to compare states should assert on individual slices.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GameState {
    /// Accelerator purchases — `player.acceleratorBought` and friends.
    pub accelerator: AcceleratorState,
    /// Achievements (point totals + per-id bitmap + progressive cache).
    pub achievements: AchievementsState,
    /// Ambrosia balance / blueberry timer / per-upgrade levels.
    pub ambrosia: AmbrosiaState,
    /// Ants — producers, masteries, ELO, upgrades, sacrifice state.
    pub ants: AntsState,
    /// Cube blessing values (`player.cubeBlessings`).
    pub cube_blessings: BlessingValues,
    /// Tesseract blessing values (`player.tesseractBlessings`).
    pub tesseract_blessings: BlessingValues,
    /// Hypercube blessing values (`player.hypercubeBlessings`).
    pub hypercube_blessings: BlessingValues,
    /// Platonic blessing values (distinct shape from the other three).
    pub platonic_blessings: PlatonicBlessings,
    /// Campaigns + constant upgrades.
    pub campaigns: CampaignsState,
    /// Challenge completions (1..=15) + current-challenge gates.
    pub challenges: ChallengesState,
    /// Coin counters across the 4 reset lineages + total.
    pub coin_counters: CoinCountersState,
    /// Per-name corruption levels (used + next preview).
    pub corruptions: CorruptionsState,
    /// Crystal-upgrade levels + prestige-shard spend resource.
    pub crystal_upgrades: CrystalUpgradesState,
    /// Per-layer cube balances (`wowCubes` / `wowTesseracts` / …).
    pub cube_balances: CubeBalancesState,
    /// `player.cubeUpgrades[]` + `player.platonicUpgrades[]` level arrays.
    pub cube_upgrade_levels: CubeUpgradeLevelsState,
    /// Event-buff state (used coupons, day-check timer).
    pub event_buffs: EventBuffsState,
    /// Golden quarks balance + 80-entry singularity-upgrade map.
    pub golden_quarks: GoldenQuarksState,
    /// Hepteract craft state across all 8 craft types + overflux balances.
    pub hepteracts: HepteractsState,
    /// Player level / tier / XP.
    pub level: LevelState,
    /// Multiplier purchases (`player.multiplierBought` and friends).
    pub multiplier: MultiplierState,
    /// 42-entry octeract-upgrade map + octeract timer.
    pub octeract_upgrades: OcteractUpgradesState,
    /// Particle building family (5 tiers + reincarnation-points spend).
    pub particle_buildings: ParticleBuildingsState,
    /// Coin tier producer family (Coin-currency-paid 5-tier producers).
    pub coin_producers: ProducerFamilyState,
    /// Diamonds tier producer family (prestige-paid).
    pub diamond_producers: ProducerFamilyState,
    /// Mythos tier producer family (transcend-paid).
    pub mythos_producers: ProducerFamilyState,
    /// Particles tier producer family (reincarnation-paid).
    pub particle_producers: ProducerFamilyState,
    /// Quark balances (`worlds`, lifetime totals).
    pub quarks: QuarksState,
    /// Red-ambrosia balance + 27-entry upgrade map.
    pub red_ambrosia: RedAmbrosiaState,
    /// 200-slot research level vec + obtainium/research-points balances.
    pub researches: ResearchesState,
    /// Reset counters (prestige / transcend / reincarnation / ascension).
    pub reset_counters: ResetCountersState,
    /// Per-purpose PRNG state.
    pub rng: RngState,
    /// Rune levels + EXP + blessings + spirits + shards spend resource.
    pub runes: RunesState,
    /// Shop upgrades (83-entry level map + potions-consumed counter).
    pub shop: ShopState,
    /// Singularity counters + per-challenge completion state.
    pub singularity: SingularityState,
    /// Talisman levels + rarities + fragment balances.
    pub talismans: TalismansState,
    /// Tesseract (ascension-tier) building family + `wowTesseracts` spend.
    pub tesseract_buildings: TesseractBuildingsState,
    /// Single-bit upgrade bitmap across all 4 resource tiers + reset flags.
    pub upgrades: UpgradesState,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_game_state_has_legacy_vec_lengths() {
        let state = GameState::default();
        assert_eq!(state.upgrades.upgrades.len(), UPGRADES_DEFAULT_LEN);
        assert_eq!(
            state.crystal_upgrades.crystal_upgrades.len(),
            CRYSTAL_UPGRADES_DEFAULT_LEN
        );
    }

    #[test]
    fn default_game_state_starts_with_no_purchases_made() {
        let state = GameState::default();
        assert_eq!(state.accelerator.accelerator_bought, 0.0);
        assert_eq!(state.multiplier.multiplier_bought, 0.0);
        // Achievement flags start true — nothing has been bought yet.
        assert!(state.accelerator.prestige_no_accelerator);
        assert!(state.multiplier.prestige_no_multiplier);
        assert!(state.upgrades.prestige_no_coin_upgrades);
    }

    #[test]
    fn default_game_state_starts_with_no_runes_or_talismans() {
        let state = GameState::default();
        assert_eq!(state.runes.rune_levels, [0.0; RUNE_COUNT]);
        assert_eq!(state.talismans.talisman_levels, [0.0; TALISMAN_COUNT]);
    }

    #[test]
    fn each_blessing_layer_is_independent() {
        let mut state = GameState::default();
        state.cube_blessings.accelerator = 5.0;
        state.tesseract_blessings.accelerator = 7.0;
        state.hypercube_blessings.accelerator = 11.0;
        assert_eq!(state.cube_blessings.accelerator, 5.0);
        assert_eq!(state.tesseract_blessings.accelerator, 7.0);
        assert_eq!(state.hypercube_blessings.accelerator, 11.0);
    }

    #[test]
    fn each_producer_family_is_independent() {
        let mut state = GameState::default();
        state.coin_producers.first_owned = 1.0;
        state.diamond_producers.first_owned = 2.0;
        state.mythos_producers.first_owned = 3.0;
        state.particle_producers.first_owned = 4.0;
        assert_eq!(state.coin_producers.first_owned, 1.0);
        assert_eq!(state.diamond_producers.first_owned, 2.0);
        assert_eq!(state.mythos_producers.first_owned, 3.0);
        assert_eq!(state.particle_producers.first_owned, 4.0);
    }
}
