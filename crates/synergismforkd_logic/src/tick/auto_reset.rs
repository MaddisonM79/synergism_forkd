//! Per-tick auto-reset state machine. The prestige / transcend /
//! reincarnation blocks are a direct port of
//! `legacy/core_split/packages/logic/src/tick/autoReset.ts`; the
//! **auto-ascension** block ports the web_ui-tier decision at
//! `legacy/core_split/packages/web_ui/src/Synergism.ts:3867-3909` (the logic
//! `autoReset.ts` has no ascension block), lifted here so the tick can decide
//! it without the UI tier.
//!
//! Decides whether prestige / transcend / reincarnation / ascension
//! auto-resets fire this tick, accumulates the time-mode counters, and emits
//! `AutoResetTriggered` intents. Execution lives in the tick tier:
//! `phase_automation` performs the matching reset (`perform_*_reset`) for
//! every fired intent — all four tiers are wired today.

use smallvec::SmallVec;

use synergismforkd_bignum::Decimal;

use crate::events::{AutoResetMode, AutoResetTier, CoreEvent};
use crate::state::AutoAscendMode;

/// `coinsThisPrestige` floor for a prestige auto-reset.
const PRESTIGE_COINS_THRESHOLD: f64 = 1e16;
/// `coinsThisTranscension` floor for a transcend auto-reset.
const TRANSCEND_COINS_THRESHOLD: f64 = 1e100;
/// `transcendShards` floor for a reincarnation auto-reset.
const REINCARNATION_SHARDS_THRESHOLD: f64 = 1e300;
/// Lower bound on the time-mode threshold (`max(0.01, amount)`).
const RESET_TIME_FLOOR: f64 = 0.01;
/// Ascension challenge that fully suppresses the reincarnation block.
const ASCENSION_CHALLENGE_NO_REINCARNATION: u32 = 12;
/// Reincarnation challenge that suppresses the auto-ascension block
/// (`currentChallenge.reincarnation !== 10`, Synergism.ts:3871).
const REINCARNATION_CHALLENGE_NO_ASCENSION: u32 = 10;
/// Lower bound on the auto-ascension threshold in `c10Completions` mode
/// (`max(1, autoAscendThreshold)`, Synergism.ts:3876).
const AUTO_ASCEND_C10_FLOOR: f64 = 1.0;
/// Lower bound on the auto-ascension threshold in `realAscensionTime` mode
/// (`max(0.1, autoAscendThreshold)`, Synergism.ts:3883).
const AUTO_ASCEND_TIME_FLOOR: f64 = 0.1;

/// Inputs to [`apply_auto_resets`]. Mirrors the legacy
/// `ApplyAutoResetsInput` field-for-field.
pub(crate) struct ApplyAutoResetsInput {
    /// Tick delta (seconds).
    pub dt: f64,

    // ─── Prestige ────────────────────────────────────────────────────
    /// `resetToggleModes.prestige`.
    pub prestige_mode: AutoResetMode,
    /// `player.toggles[15]`.
    pub auto_prestige_enabled: bool,
    /// `getLevelMilestone('autoPrestige')` — strictly `== 1` to unlock.
    pub auto_prestige_milestone: f64,
    /// `player.prestigePoints`.
    pub prestige_points: Decimal,
    /// `G.prestigePointGain` (from `reset_currency`).
    pub prestige_point_gain: Decimal,
    /// `player.prestigeamount` — amount exponent + time threshold.
    pub prestige_amount: f64,
    /// `player.coinsThisPrestige`.
    pub coins_this_prestige: Decimal,
    /// `G.autoResetTimers.prestige`.
    pub auto_reset_timer_prestige: f64,

    // ─── Transcend ───────────────────────────────────────────────────
    /// `resetToggleModes.transcend`.
    pub transcend_mode: AutoResetMode,
    /// `player.toggles[21]`.
    pub auto_transcend_enabled: bool,
    /// `player.upgrades[89]` — strictly `== 1` to unlock.
    pub upgrade_89: u8,
    /// `player.transcendPoints`.
    pub transcend_points: Decimal,
    /// `G.transcendPointGain`.
    pub transcend_point_gain: Decimal,
    /// `player.transcendamount`.
    pub transcend_amount: f64,
    /// `player.coinsThisTranscension`.
    pub coins_this_transcension: Decimal,
    /// `G.autoResetTimers.transcension`.
    pub auto_reset_timer_transcension: f64,

    // ─── Reincarnation ───────────────────────────────────────────────
    /// `resetToggleModes.reincarnation`.
    pub reincarnation_mode: AutoResetMode,
    /// `player.toggles[27]`.
    pub auto_reincarnate_enabled: bool,
    /// `player.researches[46]` — must be `> 0.5`.
    pub research_46: f64,
    /// `player.reincarnationPoints` — note the amount-mode `+1` shift.
    pub reincarnation_points: Decimal,
    /// `G.reincarnationPointGain`.
    pub reincarnation_point_gain: Decimal,
    /// `player.reincarnationamount`.
    pub reincarnation_amount: f64,
    /// `player.transcendShards`.
    pub transcend_shards: Decimal,
    /// `G.autoResetTimers.reincarnation` — accumulates regardless of mode
    /// (legacy quirk) whenever `ascension_challenge != 12`.
    pub auto_reset_timer_reincarnation: f64,

    // ─── Shared challenge gates ──────────────────────────────────────
    /// `player.currentChallenge.ascension` — `12` suppresses reincarnation;
    /// the auto-ascension block also requires it to be `0` (plain ascension).
    pub ascension_challenge: u32,
    /// `player.currentChallenge.transcension` — must be `0` for transcend
    /// or reincarnation.
    pub transcension_challenge: u32,
    /// `player.currentChallenge.reincarnation` — must be `0` for reincarnation
    /// and `!= 10` for auto-ascension.
    pub reincarnation_challenge: u32,

    // ─── Auto-ascension (Synergism.ts:3867-3909) ─────────────────────
    /// `player.autoAscend`.
    pub auto_ascend: bool,
    /// `player.autoAscendMode` — completions vs real-ascension-time.
    pub auto_ascend_mode: AutoAscendMode,
    /// `player.autoAscendThreshold`.
    pub auto_ascend_threshold: f64,
    /// `player.challengecompletions[10]` — the ascension unlock gate (must
    /// be `> 0`) and the `c10Completions`-mode comparand.
    pub challenge_completions_10: f64,
    /// `player.challengecompletions[11]` — outer guard, must be `> 0`.
    pub challenge_completions_11: f64,
    /// `player.cubeUpgrades[10]` — outer guard, must be `> 0`.
    pub cube_upgrade_10: f64,
    /// `player.ascensionCounterRealReal` — the `realAscensionTime`-mode
    /// comparand (advanced by the timer phase; no accumulation here).
    pub ascension_counter_real_real: f64,
}

/// Result of [`apply_auto_resets`].
pub(crate) struct ApplyAutoResetsResult {
    /// Updated `autoResetTimers.prestige` (only time mode mutates it).
    pub auto_reset_timer_prestige: f64,
    /// Updated `autoResetTimers.transcension`.
    pub auto_reset_timer_transcension: f64,
    /// Updated `autoResetTimers.reincarnation`.
    pub auto_reset_timer_reincarnation: f64,
    /// Zero or more `AutoResetTriggered` events (one per fired tier).
    pub events: SmallVec<[CoreEvent; 3]>,
}

/// Check the reset tiers for auto-fire conditions, in legacy order:
/// prestige (amount/time) → transcend (amount/time) → reincarnation
/// (gated by `ascension_challenge != 12`, time then amount) → ascension
/// (Synergism.ts, gated by the challenge-11 / cube-upgrade-10 outer guard).
/// The reincarnation amount mode uses the unique `(points + 1)` shift.
pub(crate) fn apply_auto_resets(input: &ApplyAutoResetsInput) -> ApplyAutoResetsResult {
    let mut events = SmallVec::new();
    let mut auto_reset_timer_prestige = input.auto_reset_timer_prestige;
    let mut auto_reset_timer_transcension = input.auto_reset_timer_transcension;
    let mut auto_reset_timer_reincarnation = input.auto_reset_timer_reincarnation;

    let ten = Decimal::from_finite(10.0);

    // ─── Prestige amount mode ───────────────────────────────────────
    if input.prestige_mode == AutoResetMode::Amount
        && input.auto_prestige_enabled
        && input.auto_prestige_milestone == 1.0
        && input.prestige_point_gain
            >= input.prestige_points * ten.pow(Decimal::from_finite(input.prestige_amount))
        && input.coins_this_prestige >= Decimal::from_finite(PRESTIGE_COINS_THRESHOLD)
    {
        events.push(CoreEvent::AutoResetTriggered {
            tier: AutoResetTier::Prestige,
            mode: AutoResetMode::Amount,
        });
    }

    // ─── Prestige time mode ─────────────────────────────────────────
    if input.prestige_mode == AutoResetMode::Time {
        auto_reset_timer_prestige += input.dt;
        let time = RESET_TIME_FLOOR.max(input.prestige_amount);
        if input.auto_prestige_enabled
            && input.auto_prestige_milestone == 1.0
            && auto_reset_timer_prestige >= time
            && input.coins_this_prestige >= Decimal::from_finite(PRESTIGE_COINS_THRESHOLD)
        {
            events.push(CoreEvent::AutoResetTriggered {
                tier: AutoResetTier::Prestige,
                mode: AutoResetMode::Time,
            });
        }
    }

    // ─── Transcend amount mode ──────────────────────────────────────
    if input.transcend_mode == AutoResetMode::Amount
        && input.auto_transcend_enabled
        && input.upgrade_89 == 1
        && input.transcend_point_gain
            >= input.transcend_points * ten.pow(Decimal::from_finite(input.transcend_amount))
        && input.coins_this_transcension >= Decimal::from_finite(TRANSCEND_COINS_THRESHOLD)
        && input.transcension_challenge == 0
    {
        events.push(CoreEvent::AutoResetTriggered {
            tier: AutoResetTier::Transcension,
            mode: AutoResetMode::Amount,
        });
    }

    // ─── Transcend time mode ────────────────────────────────────────
    if input.transcend_mode == AutoResetMode::Time {
        auto_reset_timer_transcension += input.dt;
        let time = RESET_TIME_FLOOR.max(input.transcend_amount);
        if input.auto_transcend_enabled
            && input.upgrade_89 == 1
            && auto_reset_timer_transcension >= time
            && input.coins_this_transcension >= Decimal::from_finite(TRANSCEND_COINS_THRESHOLD)
            && input.transcension_challenge == 0
        {
            events.push(CoreEvent::AutoResetTriggered {
                tier: AutoResetTier::Transcension,
                mode: AutoResetMode::Time,
            });
        }
    }

    // ─── Reincarnation block (suppressed entirely in ascension c12) ──
    if input.ascension_challenge != ASCENSION_CHALLENGE_NO_REINCARNATION {
        // Timer accumulates unconditionally here (legacy quirk: it ticks
        // even in amount mode, where it is never read).
        auto_reset_timer_reincarnation += input.dt;

        if input.reincarnation_mode == AutoResetMode::Time {
            let time = RESET_TIME_FLOOR.max(input.reincarnation_amount);
            if input.auto_reincarnate_enabled
                && input.research_46 > 0.5
                && input.transcend_shards >= Decimal::from_finite(REINCARNATION_SHARDS_THRESHOLD)
                && auto_reset_timer_reincarnation >= time
                && input.transcension_challenge == 0
                && input.reincarnation_challenge == 0
            {
                events.push(CoreEvent::AutoResetTriggered {
                    tier: AutoResetTier::Reincarnation,
                    mode: AutoResetMode::Time,
                });
            }
        }

        // Amount mode — the `(points + 1)` shift is unique to this tier.
        if input.reincarnation_mode == AutoResetMode::Amount
            && input.auto_reincarnate_enabled
            && input.research_46 > 0.5
            && input.reincarnation_point_gain
                >= (input.reincarnation_points + Decimal::one())
                    * ten.pow(Decimal::from_finite(input.reincarnation_amount))
            && input.transcend_shards >= Decimal::from_finite(REINCARNATION_SHARDS_THRESHOLD)
            && input.transcension_challenge == 0
            && input.reincarnation_challenge == 0
        {
            events.push(CoreEvent::AutoResetTriggered {
                tier: AutoResetTier::Reincarnation,
                mode: AutoResetMode::Amount,
            });
        }
    }

    // ─── Auto-ascension (Synergism.ts:3867-3909) ────────────────────
    // Inert at default — the outer guard needs `challengecompletions[11] > 0`
    // and `cubeUpgrades[10] > 0`. Only the plain `reset('ascension')` path is
    // ported (`currentChallenge.ascension == 0`); with an ascension challenge
    // active the legacy rotates / `reset('ascensionChallenge')` through helpers
    // that aren't ported, so we emit nothing there (deferred). No timer
    // accumulates — `realAscensionTime` reads the timer-phase counter directly.
    if input.auto_ascend
        && input.challenge_completions_11 > 0.0
        && input.cube_upgrade_10 > 0.0
        && input.reincarnation_challenge != REINCARNATION_CHALLENGE_NO_ASCENSION
        && input.ascension_challenge == 0
    {
        let fires = match input.auto_ascend_mode {
            AutoAscendMode::C10Completions => {
                input.challenge_completions_10
                    >= AUTO_ASCEND_C10_FLOOR.max(input.auto_ascend_threshold)
            }
            AutoAscendMode::RealAscensionTime => {
                input.ascension_counter_real_real
                    >= AUTO_ASCEND_TIME_FLOOR.max(input.auto_ascend_threshold)
            }
        };
        // The `&& c10 > 0` gate mirrors the legacy `if (ascension && c10 > 0)`
        // — redundant in `C10Completions` mode (`c10 >= max(1, …) ⇒ c10 > 0`)
        // but load-bearing in `RealAscensionTime` mode.
        if fires && input.challenge_completions_10 > 0.0 {
            events.push(CoreEvent::AutoResetTriggered {
                tier: AutoResetTier::Ascension,
                // Ascension's native modes (`C10Completions` /
                // `RealAscensionTime`) don't map onto `AutoResetMode`; the
                // dispatch ignores `mode`, so we record the closest analogue
                // for the UI (`Amount` ≈ completions, `Time` ≈ real time).
                mode: match input.auto_ascend_mode {
                    AutoAscendMode::C10Completions => AutoResetMode::Amount,
                    AutoAscendMode::RealAscensionTime => AutoResetMode::Time,
                },
            });
        }
    }

    ApplyAutoResetsResult {
        auto_reset_timer_prestige,
        auto_reset_timer_transcension,
        auto_reset_timer_reincarnation,
        events,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Base input: prestige-amount conditions all pass; every other tier
    /// disabled. Override fields per test.
    fn base() -> ApplyAutoResetsInput {
        ApplyAutoResetsInput {
            dt: 1.0,
            prestige_mode: AutoResetMode::Amount,
            auto_prestige_enabled: true,
            auto_prestige_milestone: 1.0,
            prestige_points: Decimal::from_finite(1.0),
            prestige_point_gain: Decimal::from_finite(1e9),
            prestige_amount: 0.0,
            coins_this_prestige: Decimal::from_finite(1e16),
            auto_reset_timer_prestige: 0.0,
            transcend_mode: AutoResetMode::Amount,
            auto_transcend_enabled: false,
            upgrade_89: 0,
            transcend_points: Decimal::zero(),
            transcend_point_gain: Decimal::zero(),
            transcend_amount: 0.0,
            coins_this_transcension: Decimal::zero(),
            auto_reset_timer_transcension: 0.0,
            reincarnation_mode: AutoResetMode::Amount,
            auto_reincarnate_enabled: false,
            research_46: 0.0,
            reincarnation_points: Decimal::zero(),
            reincarnation_point_gain: Decimal::zero(),
            reincarnation_amount: 0.0,
            transcend_shards: Decimal::zero(),
            auto_reset_timer_reincarnation: 0.0,
            ascension_challenge: 0,
            transcension_challenge: 0,
            reincarnation_challenge: 0,
            auto_ascend: false,
            auto_ascend_mode: AutoAscendMode::C10Completions,
            auto_ascend_threshold: 1.0,
            challenge_completions_10: 0.0,
            challenge_completions_11: 0.0,
            cube_upgrade_10: 0.0,
            ascension_counter_real_real: 0.0,
        }
    }

    fn is_reset(e: &CoreEvent, tier: AutoResetTier, mode: AutoResetMode) -> bool {
        matches!(e, CoreEvent::AutoResetTriggered { tier: t, mode: m } if *t == tier && *m == mode)
    }

    #[test]
    fn prestige_amount_fires_when_all_conditions_met() {
        let r = apply_auto_resets(&base());
        assert_eq!(r.events.len(), 1);
        assert!(is_reset(
            &r.events[0],
            AutoResetTier::Prestige,
            AutoResetMode::Amount
        ));
    }

    #[test]
    fn prestige_blocked_when_toggle_off() {
        let r = apply_auto_resets(&ApplyAutoResetsInput {
            auto_prestige_enabled: false,
            ..base()
        });
        assert!(r.events.is_empty());
    }

    #[test]
    fn prestige_time_mode_accumulates_then_fires() {
        let r = apply_auto_resets(&ApplyAutoResetsInput {
            prestige_mode: AutoResetMode::Time,
            prestige_amount: 0.5, // threshold max(0.01, 0.5) = 0.5
            dt: 1.0,
            ..base()
        });
        assert_eq!(r.auto_reset_timer_prestige, 1.0); // accumulated
        assert!(r
            .events
            .iter()
            .any(|e| is_reset(e, AutoResetTier::Prestige, AutoResetMode::Time)));
    }

    #[test]
    fn reincarnation_amount_uses_plus_one_shift() {
        // Threshold = (0 + 1) × 10^0 = 1. gain 1.0 fires; gain 0.5 does not.
        let fires = apply_auto_resets(&ApplyAutoResetsInput {
            auto_prestige_enabled: false, // isolate reincarnation
            reincarnation_mode: AutoResetMode::Amount,
            auto_reincarnate_enabled: true,
            research_46: 1.0,
            reincarnation_points: Decimal::zero(),
            reincarnation_point_gain: Decimal::from_finite(1.0),
            transcend_shards: Decimal::from_finite(1e300),
            ..base()
        });
        assert!(fires.events.iter().any(|e| is_reset(
            e,
            AutoResetTier::Reincarnation,
            AutoResetMode::Amount
        )));

        let blocked = apply_auto_resets(&ApplyAutoResetsInput {
            auto_prestige_enabled: false,
            reincarnation_mode: AutoResetMode::Amount,
            auto_reincarnate_enabled: true,
            research_46: 1.0,
            reincarnation_points: Decimal::zero(),
            reincarnation_point_gain: Decimal::from_finite(0.5), // < 1 → no fire
            transcend_shards: Decimal::from_finite(1e300),
            ..base()
        });
        assert!(!blocked.events.iter().any(|e| is_reset(
            e,
            AutoResetTier::Reincarnation,
            AutoResetMode::Amount
        )));
    }

    #[test]
    fn reincarnation_suppressed_in_ascension_challenge_12() {
        let r = apply_auto_resets(&ApplyAutoResetsInput {
            auto_prestige_enabled: false,
            reincarnation_mode: AutoResetMode::Time,
            auto_reincarnate_enabled: true,
            research_46: 1.0,
            reincarnation_amount: 0.0,
            transcend_shards: Decimal::from_finite(1e300),
            ascension_challenge: 12, // suppresses the whole block
            ..base()
        });
        assert!(!r.events.iter().any(|e| is_reset(
            e,
            AutoResetTier::Reincarnation,
            AutoResetMode::Time
        )));
        // Timer is NOT accumulated in c12.
        assert_eq!(r.auto_reset_timer_reincarnation, 0.0);
    }

    // ─── Auto-ascension ──────────────────────────────────────────────

    /// Outer guard satisfied + a met mode threshold. `auto_prestige_enabled`
    /// is cleared so only the ascension intent can fire.
    fn ascend_base() -> ApplyAutoResetsInput {
        ApplyAutoResetsInput {
            auto_prestige_enabled: false,
            auto_ascend: true,
            challenge_completions_11: 1.0,
            cube_upgrade_10: 1.0,
            ..base()
        }
    }

    fn fired_ascension(r: &ApplyAutoResetsResult) -> bool {
        r.events.iter().any(|e| {
            matches!(
                e,
                CoreEvent::AutoResetTriggered {
                    tier: AutoResetTier::Ascension,
                    ..
                }
            )
        })
    }

    #[test]
    fn auto_ascend_c10_mode_fires_when_completions_meet_threshold() {
        let r = apply_auto_resets(&ApplyAutoResetsInput {
            auto_ascend_mode: AutoAscendMode::C10Completions,
            auto_ascend_threshold: 3.0,
            challenge_completions_10: 3.0, // >= max(1, 3) = 3
            ..ascend_base()
        });
        assert!(r.events.iter().any(|e| is_reset(
            e,
            AutoResetTier::Ascension,
            AutoResetMode::Amount
        )));
    }

    #[test]
    fn auto_ascend_realtime_mode_fires_when_time_meets_threshold() {
        let r = apply_auto_resets(&ApplyAutoResetsInput {
            auto_ascend_mode: AutoAscendMode::RealAscensionTime,
            auto_ascend_threshold: 5.0,
            ascension_counter_real_real: 5.0, // >= max(0.1, 5) = 5
            challenge_completions_10: 1.0,    // unlock gate
            ..ascend_base()
        });
        assert!(r.events.iter().any(|e| is_reset(
            e,
            AutoResetTier::Ascension,
            AutoResetMode::Time
        )));
    }

    #[test]
    fn auto_ascend_blocked_without_challenge_11_or_cube_10() {
        // Mode threshold met, but the outer guard fails.
        let r = apply_auto_resets(&ApplyAutoResetsInput {
            challenge_completions_10: 5.0,
            challenge_completions_11: 0.0, // outer guard fails
            ..ascend_base()
        });
        assert!(!fired_ascension(&r));
    }

    #[test]
    fn auto_ascend_deferred_inside_ascension_challenge() {
        // With an ascension challenge active, the legacy takes the
        // `reset('ascensionChallenge')` / sweep-rotation branch we don't port.
        let r = apply_auto_resets(&ApplyAutoResetsInput {
            challenge_completions_10: 5.0,
            ascension_challenge: 11,
            ..ascend_base()
        });
        assert!(!fired_ascension(&r));
    }

    #[test]
    fn auto_ascend_realtime_still_requires_c10_unlock() {
        // Real-time threshold met, but challenge 10 is not yet completed, so
        // ascension is not unlocked.
        let r = apply_auto_resets(&ApplyAutoResetsInput {
            auto_ascend_mode: AutoAscendMode::RealAscensionTime,
            auto_ascend_threshold: 5.0,
            ascension_counter_real_real: 100.0,
            challenge_completions_10: 0.0, // not unlocked
            ..ascend_base()
        });
        assert!(!fired_ascension(&r));
    }

    #[test]
    fn auto_ascend_suppressed_in_reincarnation_challenge_10() {
        let r = apply_auto_resets(&ApplyAutoResetsInput {
            auto_ascend_mode: AutoAscendMode::C10Completions,
            challenge_completions_10: 5.0,
            reincarnation_challenge: 10, // outer guard: reincarnation !== 10
            ..ascend_base()
        });
        assert!(!fired_ascension(&r));
    }
}
