//! Automation state slice — toggles, modes, thresholds, and per-tick
//! accumulators for the Phase 5 tick-automation layer.
//!
//! Mirrors the scattered `player.toggles[N]`, `player.autoX`,
//! `player.resetToggleModes`, the `G.*` auto-reset / auto-offering
//! timers, and the challenge-sweep machine state from the legacy
//! `tick/` package. These have no single mechanic-family home, so they
//! live together here. Backs the orchestration in [`crate::tick`].
//!
//! Fields that clearly belong to a mechanic family live in that
//! family's slice instead: the prestige / transcend / reincarnation
//! reset counters in [`super::ResetCountersState`]; the quark, golden-
//! quark, ambrosia, red-ambrosia, octeract, and ant-sacrifice timers in
//! their own slices; the auto-research toggle in
//! [`super::ResearchesState`].

use serde::{Deserialize, Serialize};

use synergismforkd_bignum::Decimal;

use crate::events::{AutoResetMode, SweepState};

/// `player.autoChallengeTimer` — the three challenge-sweep phase durations
/// (seconds). The sweep state machine advances when the time-since-last-change
/// crosses the matching threshold.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct AutoChallengeTimer {
    /// `initial_wait → active` threshold.
    pub start: f64,
    /// `active → next-stage` threshold.
    pub exit: f64,
    /// `enter_wait → active` threshold.
    pub enter: f64,
}

impl Default for AutoChallengeTimer {
    /// Legacy blank-save values `{ start: 10, exit: 2, enter: 2 }`.
    fn default() -> Self {
        Self {
            start: 10.0,
            exit: 2.0,
            enter: 2.0,
        }
    }
}

/// `player.autoAscendMode` — when the auto-ascend reset fires (legacy
/// `AutoAscensionResetModes`).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AutoAscendMode {
    /// `c10Completions = 0` — reset on challenge-10 completions (legacy
    /// blank-save default).
    #[default]
    C10Completions,
    /// `realAscensionTime = 1` — reset on real ascension time.
    RealAscensionTime,
}

/// `player.resetToggleModes.ascension` (legacy `AutoAscensionModes`) — the
/// tesseract-autobuyer mode. `Amount` (the `updateAll` Family-12 path) spends
/// down to a flat `tesseract_auto_buyer_amount` reserve; `Percentage` is the
/// on-ascension `autoBuyTesseracts` path (`Reset.ts:825`, not driven here).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AutoAscensionMode {
    /// `amount = 0` — keep a flat reserve.
    #[default]
    Amount,
    /// `percentage = 1` — keep a percentage reserve.
    Percentage,
}

/// `player.shoptoggles` — the five category gates read by the upgrade-tab
/// autobuyer (`autoUpgrades`). Distinct from the indexed `toggles` array:
/// these are the shop's "auto-buy this upgrade family" switches.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShopToggles {
    /// `player.shoptoggles.coin` — auto-buy coin-tab upgrades.
    pub coin: bool,
    /// `player.shoptoggles.prestige` — auto-buy prestige-tab upgrades.
    pub prestige: bool,
    /// `player.shoptoggles.transcend` — auto-buy transcension-tab upgrades.
    pub transcend: bool,
    /// `player.shoptoggles.generators` — auto-buy generator upgrades.
    pub generators: bool,
    /// `player.shoptoggles.reincarnate` — auto-buy reincarnation-tab upgrades.
    pub reincarnate: bool,
}

impl Default for ShopToggles {
    /// Legacy blank-save values — every category auto-buy on.
    fn default() -> Self {
        Self {
            coin: true,
            prestige: true,
            transcend: true,
            generators: true,
            reincarnate: true,
        }
    }
}

/// Selects one of the five [`ShopToggles`] fields — the upgrade-family
/// autobuy switches surfaced to the UI. Lets the toggle action and the
/// field-read share one mapping ([`ShopToggles::get`] / [`ShopToggles::set`])
/// rather than duplicating the field match per call site.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShopAutobuyKind {
    /// `shop_toggles.coin` — coin-tab upgrades.
    Coin,
    /// `shop_toggles.prestige` — diamond/prestige-tab upgrades.
    Diamond,
    /// `shop_toggles.transcend` — mythos/transcension-tab upgrades.
    Mythos,
    /// `shop_toggles.generators` — generator upgrades.
    Generators,
    /// `shop_toggles.reincarnate` — reincarnation-tab upgrades.
    Reincarnation,
}

impl ShopToggles {
    /// Current on/off state of the toggle `kind` selects.
    #[must_use]
    pub fn get(&self, kind: ShopAutobuyKind) -> bool {
        match kind {
            ShopAutobuyKind::Coin => self.coin,
            ShopAutobuyKind::Diamond => self.prestige,
            ShopAutobuyKind::Mythos => self.transcend,
            ShopAutobuyKind::Generators => self.generators,
            ShopAutobuyKind::Reincarnation => self.reincarnate,
        }
    }

    /// Set the toggle `kind` selects to `on`.
    pub fn set(&mut self, kind: ShopAutobuyKind, on: bool) {
        match kind {
            ShopAutobuyKind::Coin => self.coin = on,
            ShopAutobuyKind::Diamond => self.prestige = on,
            ShopAutobuyKind::Mythos => self.transcend = on,
            ShopAutobuyKind::Generators => self.generators = on,
            ShopAutobuyKind::Reincarnation => self.reincarnate = on,
        }
    }
}

/// Slice of `GameState` for cross-cutting automation toggles + timers.
///
/// Not `Copy` (holds a [`SweepState`], which carries a `BTreeSet`) and
/// not `#[derive(Default)]` (holds a [`Decimal`], which has no
/// `Default`); the manual [`Default`] impl below sets inert values.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct AutomationState {
    // ── Auto-reset toggles / modes / thresholds ──────────────────────
    /// `player.toggles[15]` — auto-prestige enabled.
    pub auto_prestige_enabled: bool,
    /// `player.toggles[21]` — auto-transcend enabled.
    pub auto_transcend_enabled: bool,
    /// `player.toggles[27]` — auto-reincarnate enabled.
    pub auto_reincarnate_enabled: bool,
    /// `player.resetToggleModes.prestige` — amount- vs time-gated.
    pub prestige_reset_mode: AutoResetMode,
    /// `player.resetToggleModes.transcend`.
    pub transcend_reset_mode: AutoResetMode,
    /// `player.resetToggleModes.reincarnation`.
    pub reincarnation_reset_mode: AutoResetMode,
    /// `player.prestigeamount` — exponent (amount mode) or seconds
    /// (time mode) threshold.
    pub prestige_amount: f64,
    /// `player.transcendamount`.
    pub transcend_amount: f64,
    /// `player.reincarnationamount`.
    pub reincarnation_amount: f64,
    /// `G.autoResetTimers.prestige` — time-mode accumulator (seconds).
    pub auto_reset_timer_prestige: f64,
    /// `G.autoResetTimers.transcension`.
    pub auto_reset_timer_transcension: f64,
    /// `G.autoResetTimers.reincarnation`.
    pub auto_reset_timer_reincarnation: f64,

    // ── Auto-potion dispenser ────────────────────────────────────────
    /// `player.toggles[42]` — auto-offering-potion fast mode.
    pub auto_potion_toggle_offering: bool,
    /// `player.toggles[43]` — auto-obtainium-potion fast mode.
    pub auto_potion_toggle_obtainium: bool,
    /// `player.autoPotionTimer` — offering-potion accumulator (seconds).
    pub auto_potion_timer: f64,
    /// `player.autoPotionTimerObtainium` — obtainium-potion accumulator.
    pub auto_potion_timer_obtainium: f64,

    // ── Rune sacrifice ───────────────────────────────────────────────
    /// `player.autoSacrifice` — auto-rune-sacrifice enabled.
    pub rune_sacrifice_auto_enabled: bool,
    /// `player.autoSacrificeInterval` — seconds between auto-sacrifices.
    pub auto_sacrifice_interval: f64,
    /// `G.sacrificeTimer` — auto-rune-sacrifice accumulator (seconds).
    pub sacrifice_timer: f64,

    // ── Auto offerings ───────────────────────────────────────────────
    /// `player.offerings` — offering currency (rune-sacrifice fuel).
    pub offerings: Decimal,
    /// `G.autoOfferingCounter` — fractional auto-offering accumulator.
    pub auto_offering_counter: f64,

    // ── Challenge sweep machine ──────────────────────────────────────
    /// `player.autoChallengeRunning` — sweep armed.
    pub auto_challenge_running: bool,
    /// `player.retrychallenges` — when true, completed challenges do not
    /// exit their slot; the game re-runs the same challenge immediately on
    /// the next tick (the standard grind path for farming completions).
    /// The structural reset still fires each completion cycle.
    pub retry_challenges: bool,
    /// Challenge-sweep machine state (mirrors the legacy `SweepStates`).
    pub sweep_state: SweepState,
    /// Seconds elapsed since the last sweep-state transition.
    pub sweep_time_since_last_change: f64,
    /// `player.autoChallengeTimer` — the sweep phase-duration thresholds.
    pub auto_challenge_timer: AutoChallengeTimer,
    /// `player.autoChallengeToggles` — per-challenge sweep enables, indexed by
    /// challenge number (slot 0 unused; `1..=10` are the regular challenges the
    /// sweep cycles, `11..=15` the ascension challenges). Drives
    /// `getNextRegularChallenge`.
    pub auto_challenge_toggles: [bool; 16],

    // ── Auto ascension (challenge-15 sweep guard) ────────────────────
    /// `player.autoAscend` — auto-ascend reset enabled.
    pub auto_ascend: bool,
    /// `player.autoAscendMode`.
    pub auto_ascend_mode: AutoAscendMode,
    /// `player.autoAscendThreshold` — real-ascension-time mode threshold.
    pub auto_ascend_threshold: f64,

    // ── Building / upgrade autobuyers ────────────────────────────────
    /// `player.toggles[1..=26]` — the per-building / accelerator / multiplier
    /// / boost autobuy enables read by `updateAll`. Indexed by the legacy
    /// toggle number (slot 0 unused). Slots 15 and 21 exist for index parity
    /// but are NOT read here: `player.toggles[15]` / `[21]` (auto-prestige /
    /// auto-transcend) are owned by the named `auto_prestige_enabled` /
    /// `auto_transcend_enabled` fields above, and `toggles[27]`
    /// (auto-reincarnate) is out of range and likewise named.
    pub toggles: [bool; 27],
    /// `player.tesseractAutoBuyerToggle` — tesseract-building autobuyer armed.
    pub tesseract_auto_buyer_toggle: bool,
    /// `player.tesseractAutoBuyerAmount` — wow-tesseract floor the autobuyer
    /// keeps in reserve (spend budget = balance − this).
    pub tesseract_auto_buyer_amount: f64,
    /// `player.tesseractbuyamount` — tesseract buy quantity (also the per-tier
    /// step the autobuyer rounds to).
    pub tesseract_buy_amount: f64,
    /// `player.autoTesseracts` — per-tier tesseract autobuy enables (slot 0
    /// unused; `1..=5` are the five buildings).
    pub auto_tesseracts: [bool; 6],
    /// `player.resetToggleModes.ascension` — the tesseract-autobuyer mode. Only
    /// `Amount` is driven by the `updateAll` autobuyer.
    pub ascension_reset_mode: AutoAscensionMode,
    /// `player.autoFortifyToggle` — talisman level→rarity autobuyer armed.
    pub auto_fortify_toggle: bool,
    /// `player.shoptoggles` — the five upgrade-tab auto-buy category gates.
    pub shop_toggles: ShopToggles,
}

impl Default for AutomationState {
    /// Inert defaults — every auto-toggle off, every reset mode
    /// `Amount`, every timer `0`, the sweep machine `Idle`, and zero
    /// offerings.
    fn default() -> Self {
        Self {
            auto_prestige_enabled: false,
            auto_transcend_enabled: false,
            auto_reincarnate_enabled: false,
            prestige_reset_mode: AutoResetMode::Amount,
            transcend_reset_mode: AutoResetMode::Amount,
            reincarnation_reset_mode: AutoResetMode::Amount,
            prestige_amount: 0.0,
            transcend_amount: 0.0,
            reincarnation_amount: 0.0,
            auto_reset_timer_prestige: 0.0,
            auto_reset_timer_transcension: 0.0,
            auto_reset_timer_reincarnation: 0.0,
            auto_potion_toggle_offering: false,
            auto_potion_toggle_obtainium: false,
            auto_potion_timer: 0.0,
            auto_potion_timer_obtainium: 0.0,
            rune_sacrifice_auto_enabled: false,
            auto_sacrifice_interval: 0.0,
            sacrifice_timer: 0.0,
            offerings: Decimal::zero(),
            auto_offering_counter: 0.0,
            auto_challenge_running: false,
            retry_challenges: false,
            sweep_state: SweepState::Idle,
            sweep_time_since_last_change: 0.0,
            auto_challenge_timer: AutoChallengeTimer::default(),
            // Legacy blank save: slot 0 + the ascension slots off, the ten
            // regular challenges on.
            auto_challenge_toggles: [
                false, true, true, true, true, true, true, true, true, true, true, false, false,
                false, false, false,
            ],
            auto_ascend: false,
            auto_ascend_mode: AutoAscendMode::C10Completions,
            auto_ascend_threshold: 1.0,
            toggles: [false; 27],
            tesseract_auto_buyer_toggle: false,
            tesseract_auto_buyer_amount: 0.0,
            tesseract_buy_amount: 1.0,
            auto_tesseracts: [false; 6],
            ascension_reset_mode: AutoAscensionMode::Amount,
            auto_fortify_toggle: false,
            shop_toggles: ShopToggles::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_inert() {
        let s = AutomationState::default();
        assert!(!s.auto_prestige_enabled);
        assert!(!s.auto_challenge_running);
        assert_eq!(s.prestige_reset_mode, AutoResetMode::Amount);
        assert_eq!(s.offerings.to_number(), 0.0);
        assert_eq!(s.sweep_state, SweepState::Idle);
        assert_eq!(s.sacrifice_timer, 0.0);
        assert_eq!(s.auto_offering_counter, 0.0);
    }

    #[test]
    fn challenge_sweep_state_defaults_match_legacy_blank_save() {
        let s = AutomationState::default();
        // autoChallengeTimer = { start: 10, exit: 2, enter: 2 }.
        assert_eq!(
            s.auto_challenge_timer,
            AutoChallengeTimer {
                start: 10.0,
                exit: 2.0,
                enter: 2.0
            }
        );
        // autoChallengeToggles: slot 0 + ascension slots off, regular 1..=10 on.
        assert!(!s.auto_challenge_toggles[0]);
        assert!(s.auto_challenge_toggles[1..=10].iter().all(|&t| t));
        assert!(s.auto_challenge_toggles[11..].iter().all(|&t| !t));
        // autoAscend / mode / threshold.
        assert!(!s.auto_ascend);
        assert_eq!(s.auto_ascend_mode, AutoAscendMode::C10Completions);
        assert_eq!(s.auto_ascend_threshold, 1.0);
    }

    #[test]
    fn autobuy_toggles_default_off_shop_toggles_on() {
        let s = AutomationState::default();
        // Legacy player.toggles[1..=26] all default false → autobuyers dormant.
        assert!(s.toggles.iter().all(|&t| !t));
        assert!(!s.tesseract_auto_buyer_toggle);
        assert_eq!(s.tesseract_auto_buyer_amount, 0.0);
        assert_eq!(s.tesseract_buy_amount, 1.0);
        assert!(s.auto_tesseracts.iter().all(|&t| !t));
        assert!(!s.auto_fortify_toggle);
        // player.shoptoggles default on (legacy blank save).
        assert_eq!(s.shop_toggles, ShopToggles::default());
        assert!(s.shop_toggles.coin);
        assert!(s.shop_toggles.prestige);
        assert!(s.shop_toggles.transcend);
        assert!(s.shop_toggles.generators);
        assert!(s.shop_toggles.reincarnate);
    }

    #[test]
    fn shop_toggle_get_set_round_trips_every_kind() {
        let kinds = [
            (
                ShopAutobuyKind::Coin,
                ShopToggles {
                    coin: false,
                    ..ShopToggles::default()
                },
            ),
            (
                ShopAutobuyKind::Diamond,
                ShopToggles {
                    prestige: false,
                    ..ShopToggles::default()
                },
            ),
            (
                ShopAutobuyKind::Mythos,
                ShopToggles {
                    transcend: false,
                    ..ShopToggles::default()
                },
            ),
            (
                ShopAutobuyKind::Generators,
                ShopToggles {
                    generators: false,
                    ..ShopToggles::default()
                },
            ),
            (
                ShopAutobuyKind::Reincarnation,
                ShopToggles {
                    reincarnate: false,
                    ..ShopToggles::default()
                },
            ),
        ];
        for (kind, expected) in kinds {
            let mut t = ShopToggles::default();
            assert!(t.get(kind), "defaults on");
            t.set(kind, false);
            assert!(!t.get(kind));
            assert_eq!(t, expected, "only the selected field flips");
            t.set(kind, true);
            assert_eq!(t, ShopToggles::default());
        }
    }
}
