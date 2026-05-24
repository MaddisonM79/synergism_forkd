// Per-tick auto-reset state machine. Lifted from packages/web_ui/src/Synergism.ts
// (`tack`, the three auto-reset blocks at lines ~4135-4226 pre-migration).
//
// Decides whether prestige / transcend / reincarnation auto-resets should
// fire this tick, accumulates time-mode counters, and emits
// `auto-reset-triggered` events for the UI tier to dispatch. The actual
// `reset(tier, true)` call and `resetAchievementCheck(...)` award stay
// in web_ui — `reset()` is its own migration with deep player mutation,
// achievement awards funnel through web_ui's `awardAchievementGroup`.
//
// Bug-for-bug parity note: the legacy prestige *time-mode* branch awards
// the **transcension** achievement check (line 4159 in the original) —
// almost certainly a copy-paste typo, but it ships in production so the
// migration preserves it. The logic-tier emits a clean
// `{ tier: 'prestige', mode: 'time' }` event; the UI dispatcher
// hard-codes the typo so the side-effect is identical.

import type { CoreEvent } from '../events/types'
import { Decimal } from '../math/bignum'

export interface ApplyAutoResetsInput {
  /** Tick delta in seconds (already scaled by globalSpeedMult by the caller — same `dt` fed to resourceGain etc.). */
  dt: number

  // ─── Prestige tier inputs ────────────────────────────────────────────

  /** player.resetToggleModes.prestige — `amount` checks point-gain
   * threshold, `time` accumulates `autoResetTimerPrestige` and fires on
   * threshold. */
  prestigeMode: 'amount' | 'time'
  /** player.toggles[15] — prestige autobuyer master switch. */
  toggle15: boolean
  /** getLevelMilestone('autoPrestige') — strictly === 1 to unlock the
   * prestige autobuyer; legacy uses === 1 not >= 1. */
  autoPrestigeMilestone: number
  /** player.prestigePoints — currently-held prestige points.
   * Amount-mode threshold is `prestigePoints * 10^prestigeamount`. */
  prestigePoints: Decimal
  /** G.prestigePointGain — per-tick prestige-point gain candidate
   * (from logic `resetCurrency`). */
  prestigePointGain: Decimal
  /** player.prestigeamount — exponent for amount-mode threshold;
   * also the time-mode threshold (in seconds, with `max(0.01, x)` floor). */
  prestigeamount: number
  /** player.coinsThisPrestige — must be ≥ 1e16 for either mode to fire. */
  coinsThisPrestige: Decimal
  /** G.autoResetTimers.prestige — wall-clock since last prestige (time mode). */
  autoResetTimerPrestige: number

  // ─── Transcend tier inputs ───────────────────────────────────────────

  /** player.resetToggleModes.transcend — same shape as prestige. */
  transcendMode: 'amount' | 'time'
  /** player.toggles[21] — transcend autobuyer master switch. */
  toggle21: boolean
  /** player.upgrades[89] — strictly === 1 to unlock the transcend autobuyer. */
  upgrade89: number
  /** player.transcendPoints — current transcend points balance. */
  transcendPoints: Decimal
  /** G.transcendPointGain — per-tick transcend-point gain candidate. */
  transcendPointGain: Decimal
  /** player.transcendamount — exponent + time threshold (same dual use as prestigeamount). */
  transcendamount: number
  /** player.coinsThisTranscension — must be ≥ 1e100 to fire. */
  coinsThisTranscension: Decimal
  /** G.autoResetTimers.transcension — time-mode counter. */
  autoResetTimerTranscension: number

  // ─── Reincarnation tier inputs ───────────────────────────────────────

  /** player.resetToggleModes.reincarnation — same shape. */
  reincarnationMode: 'amount' | 'time'
  /** player.toggles[27] — reincarnation autobuyer master switch. */
  toggle27: boolean
  /** player.researches[46] — must be > 0.5 (i.e. ≥ 1) to unlock. */
  research46: number
  /** player.reincarnationPoints — current reincarnation points; note the
   * amount-mode threshold adds 1 first (`(rPoints + 1) * 10^amount`),
   * unlike prestige/transcend which use the raw points. */
  reincarnationPoints: Decimal
  /** G.reincarnationPointGain — per-tick reincarnation-point gain candidate. */
  reincarnationPointGain: Decimal
  /** player.reincarnationamount — exponent + time threshold. */
  reincarnationamount: number
  /** player.transcendShards — must be ≥ 1e300 for either reincarnation mode. */
  transcendShards: Decimal
  /** G.autoResetTimers.reincarnation — wall-clock; legacy accumulates this
   * regardless of mode (any mode, every tick when ascensionChallenge !== 12). */
  autoResetTimerReincarnation: number

  // ─── Shared challenge gates ──────────────────────────────────────────

  /** player.currentChallenge.ascension — when === 12, the entire
   * reincarnation block (timer accumulation + both mode checks) is
   * short-circuited. Doesn't affect prestige/transcend. */
  ascensionChallenge: number
  /** player.currentChallenge.transcension — must === 0 for transcend or
   * reincarnation to fire (either mode). */
  transcensionChallenge: number
  /** player.currentChallenge.reincarnation — must === 0 for reincarnation
   * to fire. */
  reincarnationChallenge: number
}

export interface ApplyAutoResetsResult {
  /** Updated G.autoResetTimers.prestige (only time mode mutates it). */
  autoResetTimerPrestige: number
  /** Updated G.autoResetTimers.transcension (only time mode mutates it). */
  autoResetTimerTranscension: number
  /** Updated G.autoResetTimers.reincarnation (accumulates regardless of
   * mode when ascensionChallenge !== 12; untouched otherwise). */
  autoResetTimerReincarnation: number
  /** `auto-reset-triggered` events — zero or more per tick. The UI
   * dispatcher translates each into the corresponding
   * `resetAchievementCheck(...)` + `reset(tier, true)` pair. */
  events: CoreEvent[]
}

/**
 * Check all three reset tiers for auto-fire conditions. Pure — given the
 * full input bundle, returns the post-tick timer values and the event
 * list. Mirrors the legacy three `if`-block sequence exactly: prestige
 * amount/time → transcend amount/time → reincarnation (gated by
 * ascensionChallenge !== 12) time/amount.
 */
export function applyAutoResets (input: ApplyAutoResetsInput): ApplyAutoResetsResult {
  const events: CoreEvent[] = []
  let autoResetTimerPrestige = input.autoResetTimerPrestige
  let autoResetTimerTranscension = input.autoResetTimerTranscension
  let autoResetTimerReincarnation = input.autoResetTimerReincarnation

  // ─── Prestige amount mode ──────────────────────────────────────────
  if (input.prestigeMode === 'amount') {
    if (
      input.toggle15
      && input.autoPrestigeMilestone === 1
      && input.prestigePointGain.gte(
        input.prestigePoints.times(Decimal.pow(10, input.prestigeamount))
      )
      && input.coinsThisPrestige.gte(1e16)
    ) {
      events.push({ kind: 'auto-reset-triggered', tier: 'prestige', mode: 'amount' })
    }
  }

  // ─── Prestige time mode ────────────────────────────────────────────
  if (input.prestigeMode === 'time') {
    autoResetTimerPrestige += input.dt
    const time = Math.max(0.01, input.prestigeamount)
    if (
      input.toggle15
      && input.autoPrestigeMilestone === 1
      && autoResetTimerPrestige >= time
      && input.coinsThisPrestige.gte(1e16)
    ) {
      events.push({ kind: 'auto-reset-triggered', tier: 'prestige', mode: 'time' })
    }
  }

  // ─── Transcend amount mode ─────────────────────────────────────────
  if (input.transcendMode === 'amount') {
    if (
      input.toggle21
      && input.upgrade89 === 1
      && input.transcendPointGain.gte(
        input.transcendPoints.times(Decimal.pow(10, input.transcendamount))
      )
      && input.coinsThisTranscension.gte(1e100)
      && input.transcensionChallenge === 0
    ) {
      events.push({ kind: 'auto-reset-triggered', tier: 'transcension', mode: 'amount' })
    }
  }

  // ─── Transcend time mode ───────────────────────────────────────────
  if (input.transcendMode === 'time') {
    autoResetTimerTranscension += input.dt
    const time = Math.max(0.01, input.transcendamount)
    if (
      input.toggle21
      && input.upgrade89 === 1
      && autoResetTimerTranscension >= time
      && input.coinsThisTranscension.gte(1e100)
      && input.transcensionChallenge === 0
    ) {
      events.push({ kind: 'auto-reset-triggered', tier: 'transcension', mode: 'time' })
    }
  }

  // ─── Reincarnation block (gated by ascensionChallenge !== 12) ──────
  if (input.ascensionChallenge !== 12) {
    // Timer accumulates UNCONDITIONALLY here — both modes share the same
    // counter. (Legacy quirk: it ticks even in amount mode where it's
    // never read; preserved for bug-for-bug parity.)
    autoResetTimerReincarnation += input.dt

    // Reincarnation time mode
    if (input.reincarnationMode === 'time') {
      const time = Math.max(0.01, input.reincarnationamount)
      if (
        input.toggle27
        && input.research46 > 0.5
        && input.transcendShards.gte('1e300')
        && autoResetTimerReincarnation >= time
        && input.transcensionChallenge === 0
        && input.reincarnationChallenge === 0
      ) {
        events.push({ kind: 'auto-reset-triggered', tier: 'reincarnation', mode: 'time' })
      }
    }

    // Reincarnation amount mode — note the `+1` shift before exponentiation
    // (unique to this tier; prestige and transcend don't add 1 to their
    // current-point value).
    if (input.reincarnationMode === 'amount') {
      if (
        input.toggle27
        && input.research46 > 0.5
        && input.reincarnationPointGain.gte(
          input.reincarnationPoints
            .add(1)
            .times(Decimal.pow(10, input.reincarnationamount))
        )
        && input.transcendShards.gte(1e300)
        && input.transcensionChallenge === 0
        && input.reincarnationChallenge === 0
      ) {
        events.push({ kind: 'auto-reset-triggered', tier: 'reincarnation', mode: 'amount' })
      }
    }
  }

  return {
    autoResetTimerPrestige,
    autoResetTimerTranscension,
    autoResetTimerReincarnation,
    events
  }
}
