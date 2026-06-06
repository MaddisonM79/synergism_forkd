//! Core event enum — one variant per tick outcome or purchase confirmation.
//! The UI tier consumes the event stream and orchestrates side effects.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use synergismforkd_bignum::Decimal;

/// Which producer family a [`CoreEvent::ProducersPurchased`] event refers to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProducerType {
    /// Coin tier (base game).
    Coin,
    /// Diamonds tier (prestige).
    Diamonds,
    /// Mythos tier (transcension).
    Mythos,
    /// Particles tier (reincarnation).
    Particles,
}

/// Which resource tier a [`CoreEvent::UpgradePurchased`] event refers to.
/// Mirrors the legacy `UpgradeTier` string union — coin / prestige
/// (Diamonds) / transcend (Mythos) / reincarnation (Particles).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpgradeTier {
    /// Bought with coins.
    Coin,
    /// Bought with prestige points (Diamonds layer).
    Prestige,
    /// Bought with transcend points (Mythos layer).
    Transcend,
    /// Bought with reincarnation points (Particles layer).
    Reincarnation,
}

/// Achievement-group identifier — passed to `awardAchievementGroup()` in the
/// legacy UI tier. Closed enum because every emitter names the group at
/// compile time. Extend with new variants as more groups are wired up.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum AchievementGroup {
    /// `'constant'` — awarded by [`resourceGain`](crate::mechanics::resource_gain)
    /// when `ascensionCount > 0`.
    Constant,
}

/// Which reset tier auto-fired this tick. Payload of
/// [`CoreEvent::AutoResetTriggered`]. Mirrors the legacy
/// `'prestige' | 'transcension' | 'reincarnation' | 'ascension'` union.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutoResetTier {
    /// Prestige auto-reset.
    Prestige,
    /// Transcension auto-reset.
    Transcension,
    /// Reincarnation auto-reset.
    Reincarnation,
    /// Ascension auto-reset.
    Ascension,
}

/// Whether the auto-reset gate that fired was point-amount based or
/// wall-clock based. Payload of [`CoreEvent::AutoResetTriggered`].
///
/// Derives serde so it can double as the persisted reset-mode setting in
/// [`crate::state::AutomationState`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoResetMode {
    /// Resource-amount threshold ("autoPrestigeAmount" etc.).
    Amount,
    /// Wall-clock threshold ("autoPrestigeTime" etc.).
    Time,
}

/// Which `automaticTools()` branch fired. Payload of
/// [`CoreEvent::AutoToolFired`]. Mirrors the legacy
/// `'runeSacrifice' | 'antSacrifice' | 'addObtainium' | 'addOfferings'`
/// union.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutoTool {
    /// Auto-rune-sacrifice fired this tick.
    RuneSacrifice,
    /// Auto-ant-sacrifice fired this tick.
    AntSacrifice,
    /// `addObtainium` branch fired this tick.
    AddObtainium,
    /// `addOfferings` branch fired this tick.
    AddOfferings,
}

/// Which legacy `revealStuff()` trigger fired. Payload of
/// [`CoreEvent::RevealNeeded`]. The TS names are the four coin-tier
/// reveal checks in `resourceGain`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RevealTrigger {
    /// `'coinone'` — first coin-tier reveal check.
    CoinOne,
    /// `'cointwo'` — second coin-tier reveal check.
    CoinTwo,
    /// `'cointhree'` — third coin-tier reveal check.
    CoinThree,
    /// `'coinfour'` — fourth coin-tier reveal check.
    CoinFour,
}

/// Which side of the auto-potion dispenser fired. Payload of
/// [`CoreEvent::AutoPotionFired`]. The UI dispatcher maps to
/// `useConsumable('offeringPotion' | 'obtainiumPotion', …)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutoPotionType {
    /// Offering potion.
    Offering,
    /// Obtainium potion.
    Obtainium,
}

/// State of the auto-challenge sweep machine. Mirrors the legacy
/// `SweepStates` discriminated union in
/// `legacy/core_split/packages/logic/src/tick/challengeSweep.ts`.
///
/// The `Active` and `EnterWait` variants carry an `explored` set so a
/// single sweep cycle doesn't repeat challenges. `BTreeSet<u8>` matches
/// the TS `Set<number>` with the small fixed range of challenge indices
/// (1..=10).
///
/// Derives serde so it can double as the persisted sweep-machine state
/// in [`crate::state::AutomationState`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SweepState {
    /// Sweep is off — autoChallenge toggle is disabled.
    Idle,
    /// Initial 5-second pause before the first sweep starts.
    InitialWait,
    /// About to enter `to_index`. `explored` tracks which challenges
    /// have already been visited this cycle.
    EnterWait {
        /// 1-based challenge index the sweep is about to enter.
        to_index: u8,
        /// Set of challenge indices already visited this cycle.
        explored: BTreeSet<u8>,
    },
    /// Currently running challenge `index`. `explored` tracks which
    /// challenges have already been visited this cycle.
    Active {
        /// 1-based challenge index currently active.
        index: u8,
        /// Set of challenge indices already visited this cycle.
        explored: BTreeSet<u8>,
    },
    /// 5-second pause when the player can auto-gain challenge-15
    /// exponent (autoAscend + cubeUpgrade 10 + realAscensionTime mode).
    C15Wait,
    /// Every regular challenge (1-10) is maxed; sweep parks until a
    /// max-completions cap changes.
    Finished,
}

/// Events emitted by mechanic functions. The closed set lets the UI dispatch
/// on the variant without a string-typed kind field, and `#[non_exhaustive]`
/// means new variants can land without breaking downstream `match` arms.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum CoreEvent {
    /// Accelerators were purchased — `after - before` units in total at
    /// a coin cost of `spent`.
    AcceleratorsPurchased {
        /// Owned count before the purchase loop ran.
        before: f64,
        /// Owned count after the purchase loop ran.
        after: f64,
        /// Coins removed from the player's balance.
        spent: Decimal,
    },
    /// Multipliers were purchased — same shape as
    /// [`CoreEvent::AcceleratorsPurchased`].
    MultipliersPurchased {
        /// Owned count before the purchase loop ran.
        before: f64,
        /// Owned count after the purchase loop ran.
        after: f64,
        /// Coins removed from the player's balance.
        spent: Decimal,
    },
    /// One position of a producer family was purchased.
    ProducersPurchased {
        /// Which family (Coin / Diamonds / Mythos / Particles).
        producer_type: ProducerType,
        /// Tier index, 1..=5 (1-based to match the legacy `buyMax(index)`
        /// parameter).
        index: u8,
        /// Owned count before the purchase loop ran.
        before: f64,
        /// Owned count after the purchase loop ran.
        after: f64,
        /// Family resource removed from the player's balance.
        spent: Decimal,
    },
    /// One of the five particle buildings was purchased.
    ParticleBuildingsPurchased {
        /// Tier index, 1..=5.
        index: u8,
        /// Owned count before the purchase loop ran.
        before: f64,
        /// Owned count after the purchase loop ran.
        after: f64,
        /// Reincarnation points removed from the player's balance.
        spent: Decimal,
    },
    /// One crystal upgrade leveled up (zero-or-more levels at once via
    /// the closed-form max-affordable solve).
    CrystalUpgradePurchased {
        /// 1-based crystal-upgrade index.
        i: u8,
        /// Level before the purchase.
        before: f64,
        /// Level after the purchase (includes any +10 bonus from owning
        /// upgrade-73 while in a reincarnation challenge).
        after: f64,
        /// Prestige shards removed from the player's balance.
        spent: Decimal,
    },
    /// One cube upgrade was leveled up (zero-or-more levels at once via the
    /// summation cost solver). `spent` is in wow cubes — an `f64` mirroring
    /// the legacy `Number(player.wowCubes)` cost comparison.
    CubeUpgradePurchased {
        /// 1-based cube-upgrade index (1..=80).
        index: u8,
        /// Level before the purchase.
        before: f64,
        /// Level after the purchase.
        after: f64,
        /// Wow cubes removed from the player's balance.
        spent: f64,
    },
    /// One platonic upgrade gained a level. The spend spans seven resources
    /// (obtainium / offerings / cubes / tesseracts / hypercubes / platonics /
    /// abyssals), so no single `spent` value is carried.
    PlatonicUpgradePurchased {
        /// 1-based platonic-upgrade index (1..=20).
        index: u8,
        /// Level before the purchase.
        before: f64,
        /// Level after the purchase.
        after: f64,
    },
    /// One golden-quark (singularity) upgrade gained a level. `spent` is in
    /// golden quarks — an `f64` mirroring the legacy `Number(goldenQuarks)`
    /// cost comparison.
    GoldenQuarkUpgradePurchased {
        /// GQ-upgrade index (0..80, via the `GQ_*` constants).
        index: u32,
        /// Level before the purchase.
        before: f64,
        /// Level after the purchase.
        after: f64,
        /// Golden quarks removed from the player's balance.
        spent: f64,
    },
    /// One octeract upgrade gained a level — the single-level step of the
    /// legacy `buyOcteractUpgradeLevel` loop. `spent` is in `wow_octeracts`,
    /// an `f64` mirroring the legacy `player.wowOcteracts < cost` number
    /// comparison.
    OcteractUpgradePurchased {
        /// Octeract-upgrade index (0..47, via the `OCTERACT_*` constants).
        index: u32,
        /// Level before the purchase.
        before: f64,
        /// Level after the purchase.
        after: f64,
        /// Octeracts removed from the player's balance.
        spent: f64,
    },
    /// One ambrosia (blueberry) upgrade gained a level — the single-level
    /// step of the legacy `buyAmbrosiaUpgradeLevel` loop. `spent` is in
    /// ambrosia (`f64`); the first level out of level 0 also debits the
    /// upgrade's blueberry-slot cost to `spent_blueberries` (reflected in
    /// state, not on this event).
    AmbrosiaUpgradePurchased {
        /// Ambrosia-upgrade index (0..36, via the `AMBROSIA_*` constants).
        index: u32,
        /// Level before the purchase.
        before: f64,
        /// Level after the purchase.
        after: f64,
        /// Ambrosia removed from the player's balance.
        spent: f64,
    },
    /// One shop upgrade gained a level (or a consumable a unit of stock —
    /// the buy is uniform). `spent` is in quarks — an `f64` mirroring the
    /// legacy `Number(player.worlds)` cost comparison.
    ShopUpgradePurchased {
        /// Shop-upgrade index (0..83, via the `SHOP_*` constants).
        index: u32,
        /// Level / stock before the purchase.
        before: f64,
        /// Level / stock after the purchase.
        after: f64,
        /// Quarks removed from the player's balance.
        spent: f64,
    },
    /// One research slot was leveled up (zero-or-more levels at once via
    /// the closed-form max-affordable solve). `spent` is in obtainium.
    ResearchPurchased {
        /// 1-based research index.
        index: u32,
        /// Level before the purchase.
        before: f64,
        /// Level after the purchase.
        after: f64,
        /// Obtainium removed from the player's balance.
        spent: Decimal,
    },
    /// Rune `index` gained levels by spending offerings — the legacy
    /// `sacrificeOfferings` flow. `before`/`after` are purchased levels (which
    /// may be equal if the budget only banked partial EXP); `spent` is the
    /// offerings consumed.
    RuneLevelsPurchased {
        /// Rune index (0..7, via the `RUNE_*` constants).
        index: u32,
        /// Purchased level before the spend.
        before: f64,
        /// Purchased level after the spend (re-derived from EXP).
        after: f64,
        /// Offerings removed from the player's balance.
        spent: Decimal,
    },
    /// One ant-producer tier was bought (single click or buy-max) — the
    /// legacy `buyAntProducers`. `before`/`after` are the `purchased` count;
    /// `spent` is in galactic crumbs.
    AntProducersPurchased {
        /// Ant-producer index (0..9, Workers..HolySpirit).
        index: u32,
        /// `purchased` count before the buy.
        before: f64,
        /// `purchased` count after the buy.
        after: f64,
        /// Crumbs removed from the player's balance.
        spent: Decimal,
    },
    /// One ant upgrade gained level(s) (single click or buy-max) — the legacy
    /// `buyAntUpgrade`. `spent` is in galactic crumbs.
    AntUpgradePurchased {
        /// Ant-upgrade index (0..16, AntSpeed..Mortuus2).
        index: u32,
        /// Level before the buy.
        before: f64,
        /// Level after the buy.
        after: f64,
        /// Crumbs removed from the player's balance.
        spent: Decimal,
    },
    /// Hepteract `index` was crafted toward its cap — the legacy
    /// `craftHepteracts`. `before`/`after` are the craft's balance; `amount`
    /// is the units crafted (the multi-resource spend lands on state).
    HepteractCrafted {
        /// Hepteract index (0..8, chronos..multiplier).
        index: u32,
        /// Balance before the craft.
        before: f64,
        /// Balance after the craft.
        after: f64,
        /// Units crafted this action.
        amount: f64,
    },
    /// Hepteract `index` had its cap doubled, a full bar spent — the legacy
    /// `expandHepteracts`.
    HepteractCapExpanded {
        /// Hepteract index (0..8, chronos..multiplier).
        index: u32,
        /// Balance left after spending one cap's worth.
        bal_after: f64,
        /// The new (doubled) cap.
        cap_after: f64,
    },
    /// Talisman `index` gained a level — the legacy `buyTalismanLevel`. The
    /// spend lands on the seven talisman resources (shards + six fragment
    /// tiers) in state.
    TalismanLevelPurchased {
        /// Talisman index (0..7, via the `TALISMAN_*` constants).
        index: u32,
        /// Level before the buy.
        before: f64,
        /// Level after the buy.
        after: f64,
    },
    /// A single-bit upgrade was purchased. The `spent` value is the cost
    /// in the tier's currency.
    UpgradePurchased {
        /// Which resource tier paid for the upgrade.
        tier: UpgradeTier,
        /// Upgrade position in the bitmap.
        pos: u32,
        /// Currency removed from the player's balance.
        spent: Decimal,
    },
    /// One tier of the tesseract (ascension) building family was
    /// purchased. `spent` is in `wow_tesseracts` (an `f64` because the
    /// resource caps out well below `1e308`).
    TesseractBuildingsPurchased {
        /// Tier index, 1..=5.
        index: u8,
        /// Owned count before the purchase loop ran.
        before: f64,
        /// Owned count after the purchase loop ran.
        after: f64,
        /// `wow_tesseracts` removed from the player's balance.
        spent: f64,
    },
    /// An achievement group should be checked/awarded. The UI tier maps
    /// the group identifier to its `awardAchievementGroup()` call.
    AchievementGroupAwarded {
        /// Which group to evaluate.
        group: AchievementGroup,
    },
    /// One of challenges 1..=5 was auto-completed this tick. Fires when
    /// the research-71..75 gates are met and the coin threshold is crossed.
    ChallengeAutoCompleted {
        /// 1-based challenge index, `1..=5`.
        challenge_index: u8,
        /// New completion count after the increment.
        new_completions: f64,
    },
    /// Per-tick resource gain delta. Emitted by the tick orchestrator once
    /// the resource-cascade pass completes. All fields are *deltas* applied
    /// this tick (zero when the gate didn't fire).
    ///
    /// The tick orchestrator emitter isn't ported yet — variant declared
    /// for the closed-set match contract.
    ResourcesGained {
        /// Per-tick coin gain (after `taxdivisor` + `maxexponent` clamp).
        /// Zero if `produceTotal < 0.001`.
        coins: Decimal,
        /// Per-tick prestige-point gain from upgrade-93 (zero otherwise).
        prestige_points: Decimal,
        /// Per-tick transcend-point gain from upgrade-100 (zero otherwise).
        transcend_points: Decimal,
        /// Per-tick reincarnation-point gain from `cubeUpgrade-28` (zero otherwise).
        reincarnation_points: Decimal,
        /// Per-tick `prestigeShards` gain (zero in t-chal 3 / r-chal 10).
        prestige_shards: Decimal,
        /// Per-tick `transcendShards` gain (zero in t-chal 3 / r-chal 10).
        transcend_shards: Decimal,
        /// Per-tick `reincarnationShards` gain.
        reincarnation_shards: Decimal,
        /// Per-tick `ascendShards` gain from the first ascension building.
        ascend_shards: Decimal,
    },
    /// One of the four reset tiers auto-fired this tick.
    ///
    /// Emitted by the auto-reset state machine (tick-side, not yet
    /// ported).
    AutoResetTriggered {
        /// Which reset tier auto-fired this tick.
        tier: AutoResetTier,
        /// Whether the threshold check was point-amount based or wall-clock based.
        mode: AutoResetMode,
    },
    /// A reset tier was *executed* this tick — the player's currencies,
    /// producers, and counters have already been mutated and the
    /// prestige-family currency credited. Distinct from
    /// [`Self::AutoResetTriggered`], which is only the auto-reset *intent*
    /// (nothing applies it yet). Emitted by the manual-reset dispatch.
    ResetPerformed {
        /// Which reset tier executed.
        tier: AutoResetTier,
        /// Prestige-family points credited by the reset (`0` for an empty
        /// reset — the execution is ungated).
        points_gained: Decimal,
    },
    /// One of the four `automaticTools()` branches fired this tick.
    ///
    /// Emitted by the auto-tool state machine (tick-side, not yet
    /// ported).
    AutoToolFired {
        /// Which auto-tool branch fired this tick.
        tool: AutoTool,
    },
    /// The auto-challenge sweep machine transitioned from one state to
    /// another. The UI dispatcher routes resetCheck by `from.index` when
    /// `from` is `Active`, and `toggleChallenges(to.index, true)` when
    /// `to` is `Active`.
    ///
    /// Emitted by the auto-challenge sweep machine (tick-side, not yet
    /// ported).
    ChallengeSweepTransitioned {
        /// Full sweep state transitioned out of.
        from: SweepState,
        /// Full sweep state transitioned into.
        to: SweepState,
    },
    /// A legacy `revealStuff()` trigger fired — coin-tier visibility
    /// gate the UI should re-evaluate.
    ///
    /// Emitted by the resource-gain branch when a tier-visibility
    /// threshold is crossed (tick-side, not yet ported).
    RevealNeeded {
        /// Which trigger fired.
        trigger: RevealTrigger,
    },
    /// Total ambrosia gained this tick (sum across all loop iterations).
    ///
    /// Emitted by the ambrosia tick branch (not yet ported).
    AmbrosiaGained {
        /// Amount of ambrosia gained this tick.
        amount: f64,
    },
    /// Total red ambrosia gained this tick (sum across all loop
    /// iterations).
    ///
    /// Emitted by the red-ambrosia tick branch (not yet ported).
    RedAmbrosiaGained {
        /// Amount of red ambrosia gained this tick.
        amount: f64,
    },
    /// One or more integer 1-second giveaway buckets crossed this tick
    /// for the octeract subsystem. Always `≥ 1` when emitted.
    ///
    /// Emitted by the octeract tick branch (not yet ported).
    OcteractTickFired {
        /// Number of 1-second giveaway buckets that crossed this tick.
        amount_of_giveaways: u32,
    },
    /// The auto-potion dispenser fired. The UI dispatcher maps to
    /// `useConsumable('offeringPotion' | 'obtainiumPotion', …)`.
    ///
    /// Emitted by the auto-potion tick branch (not yet ported).
    AutoPotionFired {
        /// Which side of the dispenser fired this tick.
        potion_type: AutoPotionType,
        /// Number of potions to dispense this tick. Always `≥ 1`.
        amount: u32,
        /// Whether fast mode was active for this dispense (skips
        /// `shopUpgrades` count decrement when `false`).
        fast_mode: bool,
    },
    /// The auto-ant-sacrifice gate's conditions were met this tick.
    /// Pure intent signal — the UI dispatcher invokes `sacrificeAnts()`
    /// which re-reads the latest player state itself.
    ///
    /// Emitted by `checkAntSacrificeReady` (tick-side, not yet ported).
    AntSacrificeTriggered,
    /// The auto-rune-sacrifice timer crossed `autoSacrificeInterval`
    /// and offerings > 0. Pure intent signal — the UI dispatcher runs
    /// the blessing/spirit/talisman/per-rune-or-all purchase fan-out.
    ///
    /// Emitted by `advanceRuneSacrifice` (tick-side, not yet ported).
    RuneSacrificeTriggered,
    /// The autoResearch toggle is on and the mode is `manual`. The UI
    /// dispatcher calls `buyResearch(autoResearch, true, false)` +
    /// `updateResearchAuto`.
    ///
    /// Emitted by `processAutoResearchTick` (tick-side, not yet
    /// ported).
    AutoResearchManualRequested,
    /// The autoResearch toggle is on, the Roomba gates pass, and the
    /// mode is `cheapest`. The UI dispatcher runs the bounded
    /// while-loop in `runRoombaResearchSweep(max_count)`.
    ///
    /// Emitted by `processAutoResearchTick` (tick-side, not yet
    /// ported).
    AutoResearchRoombaRequested {
        /// Max iterations for the Roomba sweep this tick — `1 +
        /// floor(CalcECC('ascension', challengecompletions[14]))`.
        max_count: u32,
    },
    /// `tackMiddle`'s obtainium branch fires this when `research61 !=
    /// 1`, mirroring the vestigial `else { calculateObtainium() }` arm
    /// of the legacy code. The UI dispatcher invokes
    /// `calculateObtainium()` to preserve existing behavior; the return
    /// value is discarded.
    ///
    /// Emitted by `tackMiddle` (tick-side, not yet ported).
    ObtainiumMultiplierRecomputeRequested,
    /// A corruption's *next-ascension* loadout level was set (legacy
    /// `CorruptionLoadout.setLevel`). `index` is the corruption slot
    /// (viscosity = 0), `level` the clamped new level.
    CorruptionLevelSet {
        /// Corruption slot index (`0..8`; viscosity = 0).
        index: usize,
        /// The clamped new level written to `corruptions.next`.
        level: u32,
    },
    /// A challenge was entered (legacy `toggleChallenges`): the
    /// `current_*_challenge` slot was set and the tier reset ran. `challenge`
    /// is `0..=15` (`0` exits the transcension slot). The accompanying
    /// `ResetPerformed` carries the tier-reset detail.
    ChallengeEntered {
        /// Challenge index (`1..=5` transcension, `6..=10` reincarnation;
        /// `0` exits the transcension slot).
        challenge: u32,
    },
    /// A challenge auto-completed in the tick (legacy `resetCheck` completion):
    /// the goal was met, completions were awarded, and the challenge exited.
    /// An accompanying `ResetPerformed` carries the reset-out (unless
    /// `instantChallenge` is unlocked).
    ChallengeCompleted {
        /// Challenge index that completed (`1..=5` transcension).
        challenge: u32,
        /// New `challenge_completions[challenge]` after the award.
        completions: f64,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_event_variants_construct() {
        // Compile-time assertion that the no-payload variants are usable
        // without struct-init syntax.
        let _ = CoreEvent::AntSacrificeTriggered;
        let _ = CoreEvent::RuneSacrificeTriggered;
        let _ = CoreEvent::AutoResearchManualRequested;
        let _ = CoreEvent::ObtainiumMultiplierRecomputeRequested;
    }

    #[test]
    fn payload_event_variants_construct_and_equate() {
        let a = CoreEvent::AmbrosiaGained { amount: 1.5 };
        let b = CoreEvent::AmbrosiaGained { amount: 1.5 };
        assert_eq!(a, b);
        assert_ne!(
            CoreEvent::AmbrosiaGained { amount: 1.5 },
            CoreEvent::RedAmbrosiaGained { amount: 1.5 }
        );
    }

    #[test]
    fn sweep_state_variants_construct() {
        let _ = SweepState::Idle;
        let _ = SweepState::InitialWait;
        let _ = SweepState::Active {
            index: 3,
            explored: BTreeSet::from([1u8, 2]),
        };
        let _ = SweepState::EnterWait {
            to_index: 5,
            explored: BTreeSet::new(),
        };
        let _ = SweepState::C15Wait;
        let _ = SweepState::Finished;
    }

    #[test]
    fn sweep_state_equality_compares_explored_set() {
        let a = SweepState::Active {
            index: 1,
            explored: BTreeSet::from([1u8, 2]),
        };
        let b = SweepState::Active {
            index: 1,
            explored: BTreeSet::from([2u8, 1]),
        };
        assert_eq!(a, b); // BTreeSet is order-independent
        let c = SweepState::Active {
            index: 1,
            explored: BTreeSet::from([1u8]),
        };
        assert_ne!(a, c);
    }

    #[test]
    fn challenge_sweep_transitioned_carries_both_states() {
        let event = CoreEvent::ChallengeSweepTransitioned {
            from: SweepState::Idle,
            to: SweepState::InitialWait,
        };
        match event {
            CoreEvent::ChallengeSweepTransitioned { from, to } => {
                assert_eq!(from, SweepState::Idle);
                assert_eq!(to, SweepState::InitialWait);
            }
            _ => panic!("variant mismatch"),
        }
    }

    #[test]
    fn auto_reset_triggered_combines_tier_and_mode() {
        let event = CoreEvent::AutoResetTriggered {
            tier: AutoResetTier::Reincarnation,
            mode: AutoResetMode::Time,
        };
        match event {
            CoreEvent::AutoResetTriggered { tier, mode } => {
                assert_eq!(tier, AutoResetTier::Reincarnation);
                assert_eq!(mode, AutoResetMode::Time);
            }
            _ => panic!("variant mismatch"),
        }
    }
}
