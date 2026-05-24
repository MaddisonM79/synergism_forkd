// Parity tests for the auto-reset state machine.
// Old body transcribed verbatim from packages/web_ui/src/Synergism.ts
// (tack, the three auto-reset blocks at ~lines 4135-4226 pre-migration),
// minus the side-effects (resetAchievementCheck / reset) which stay in
// web_ui — those are translated from the event list by the UI dispatcher.

import { describe, expect, it } from 'vitest'
import { Decimal } from '../../src/math/bignum'
import {
  applyAutoResets as newApplyAutoResets,
  type ApplyAutoResetsInput,
  type ApplyAutoResetsResult
} from '../../src/tick/autoReset'

// Verbatim transcription of legacy logic without the side-effect calls.
// `events` mirrors the order of the legacy if-block sequence:
//   prestige amount → prestige time → transcend amount → transcend time
//   → (reincarnation timer accumulation) → reincarnation time → reincarnation amount
const oldApplyAutoResets = (input: ApplyAutoResetsInput): ApplyAutoResetsResult => {
  const events: ApplyAutoResetsResult['events'] = []
  let autoResetTimerPrestige = input.autoResetTimerPrestige
  let autoResetTimerTranscension = input.autoResetTimerTranscension
  let autoResetTimerReincarnation = input.autoResetTimerReincarnation

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

  if (input.ascensionChallenge !== 12) {
    autoResetTimerReincarnation += input.dt
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

// Default input — every gate blocks. Override in each case to test one
// condition at a time. dt=0.025 mirrors the typical 25ms tick.
const defaultInput = (): ApplyAutoResetsInput => ({
  dt: 0.025,
  prestigeMode: 'amount',
  toggle15: false,
  autoPrestigeMilestone: 0,
  prestigePoints: new Decimal(0),
  prestigePointGain: new Decimal(0),
  prestigeamount: 1,
  coinsThisPrestige: new Decimal(0),
  autoResetTimerPrestige: 0,
  transcendMode: 'amount',
  toggle21: false,
  upgrade89: 0,
  transcendPoints: new Decimal(0),
  transcendPointGain: new Decimal(0),
  transcendamount: 1,
  coinsThisTranscension: new Decimal(0),
  autoResetTimerTranscension: 0,
  reincarnationMode: 'amount',
  toggle27: false,
  research46: 0,
  reincarnationPoints: new Decimal(0),
  reincarnationPointGain: new Decimal(0),
  reincarnationamount: 1,
  transcendShards: new Decimal(0),
  autoResetTimerReincarnation: 0,
  ascensionChallenge: 0,
  transcensionChallenge: 0,
  reincarnationChallenge: 0
})

describe('parity: applyAutoResets', () => {
  const cases: Array<{ name: string, input: ApplyAutoResetsInput }> = [
    // ─── Baseline: nothing fires ──────────────────────────────────────
    { name: 'baseline — all gates blocking', input: defaultInput() },

    // ─── Prestige amount mode ─────────────────────────────────────────
    {
      name: 'prestige amount — all conditions met (fires)',
      input: {
        ...defaultInput(),
        prestigeMode: 'amount',
        toggle15: true,
        autoPrestigeMilestone: 1,
        prestigePoints: new Decimal(100),
        prestigePointGain: new Decimal(1e6),
        prestigeamount: 2, // 100 * 10^2 = 10000, gain 1e6 > 10000
        coinsThisPrestige: new Decimal(1e17)
      }
    },
    {
      name: 'prestige amount — toggle15 off (blocked)',
      input: {
        ...defaultInput(),
        prestigeMode: 'amount',
        toggle15: false,
        autoPrestigeMilestone: 1,
        prestigePoints: new Decimal(100),
        prestigePointGain: new Decimal(1e6),
        prestigeamount: 2,
        coinsThisPrestige: new Decimal(1e17)
      }
    },
    {
      name: 'prestige amount — autoPrestigeMilestone is 2 (blocked: strict === 1)',
      input: {
        ...defaultInput(),
        prestigeMode: 'amount',
        toggle15: true,
        autoPrestigeMilestone: 2, // not strictly === 1
        prestigePoints: new Decimal(100),
        prestigePointGain: new Decimal(1e6),
        prestigeamount: 2,
        coinsThisPrestige: new Decimal(1e17)
      }
    },
    {
      name: 'prestige amount — gain below threshold (blocked)',
      input: {
        ...defaultInput(),
        prestigeMode: 'amount',
        toggle15: true,
        autoPrestigeMilestone: 1,
        prestigePoints: new Decimal(100),
        prestigePointGain: new Decimal(9999), // < 100 * 10^2 = 10000
        prestigeamount: 2,
        coinsThisPrestige: new Decimal(1e17)
      }
    },
    {
      name: 'prestige amount — coinsThisPrestige just under 1e16 (blocked)',
      input: {
        ...defaultInput(),
        prestigeMode: 'amount',
        toggle15: true,
        autoPrestigeMilestone: 1,
        prestigePoints: new Decimal(100),
        prestigePointGain: new Decimal(1e6),
        prestigeamount: 2,
        coinsThisPrestige: new Decimal('9.99e15')
      }
    },
    {
      name: 'prestige amount — gain exactly at threshold (fires)',
      input: {
        ...defaultInput(),
        prestigeMode: 'amount',
        toggle15: true,
        autoPrestigeMilestone: 1,
        prestigePoints: new Decimal(1),
        prestigePointGain: new Decimal(10), // 1 * 10^1 = 10
        prestigeamount: 1,
        coinsThisPrestige: new Decimal(1e16)
      }
    },

    // ─── Prestige time mode ───────────────────────────────────────────
    {
      name: 'prestige time — fires once timer reaches threshold',
      input: {
        ...defaultInput(),
        prestigeMode: 'time',
        toggle15: true,
        autoPrestigeMilestone: 1,
        prestigeamount: 5,
        autoResetTimerPrestige: 4.99, // + dt=0.025 → 5.015 ≥ 5
        coinsThisPrestige: new Decimal(1e17)
      }
    },
    {
      name: 'prestige time — timer not yet at threshold (timer accumulates, no event)',
      input: {
        ...defaultInput(),
        prestigeMode: 'time',
        toggle15: true,
        autoPrestigeMilestone: 1,
        prestigeamount: 60,
        autoResetTimerPrestige: 1.5,
        coinsThisPrestige: new Decimal(1e17)
      }
    },
    {
      name: 'prestige time — prestigeamount = 0 (floor at 0.01)',
      input: {
        ...defaultInput(),
        prestigeMode: 'time',
        toggle15: true,
        autoPrestigeMilestone: 1,
        prestigeamount: 0, // max(0.01, 0) = 0.01
        autoResetTimerPrestige: 0,
        dt: 0.025, // → 0.025 ≥ 0.01
        coinsThisPrestige: new Decimal(1e17)
      }
    },
    {
      name: 'prestige time — prestigeamount = -5 (floor at 0.01)',
      input: {
        ...defaultInput(),
        prestigeMode: 'time',
        toggle15: true,
        autoPrestigeMilestone: 1,
        prestigeamount: -5,
        autoResetTimerPrestige: 0,
        dt: 0.025,
        coinsThisPrestige: new Decimal(1e17)
      }
    },
    {
      name: 'prestige time — mode is amount, prestige timer NOT incremented',
      input: {
        ...defaultInput(),
        prestigeMode: 'amount',
        autoResetTimerPrestige: 10
      }
    },

    // ─── Transcend amount mode ────────────────────────────────────────
    {
      name: 'transcend amount — all conditions met (fires)',
      input: {
        ...defaultInput(),
        transcendMode: 'amount',
        toggle21: true,
        upgrade89: 1,
        transcendPoints: new Decimal(100),
        transcendPointGain: new Decimal(1e6),
        transcendamount: 2,
        coinsThisTranscension: new Decimal(1e101),
        transcensionChallenge: 0
      }
    },
    {
      name: 'transcend amount — upgrade89 = 2 (blocked: strict === 1)',
      input: {
        ...defaultInput(),
        transcendMode: 'amount',
        toggle21: true,
        upgrade89: 2,
        transcendPoints: new Decimal(100),
        transcendPointGain: new Decimal(1e6),
        transcendamount: 2,
        coinsThisTranscension: new Decimal(1e101),
        transcensionChallenge: 0
      }
    },
    {
      name: 'transcend amount — transcensionChallenge !== 0 (blocked)',
      input: {
        ...defaultInput(),
        transcendMode: 'amount',
        toggle21: true,
        upgrade89: 1,
        transcendPoints: new Decimal(100),
        transcendPointGain: new Decimal(1e6),
        transcendamount: 2,
        coinsThisTranscension: new Decimal(1e101),
        transcensionChallenge: 3
      }
    },
    {
      name: 'transcend amount — coinsThisTranscension just under 1e100 (blocked)',
      input: {
        ...defaultInput(),
        transcendMode: 'amount',
        toggle21: true,
        upgrade89: 1,
        transcendPoints: new Decimal(100),
        transcendPointGain: new Decimal(1e6),
        transcendamount: 2,
        coinsThisTranscension: new Decimal('9.99e99')
      }
    },

    // ─── Transcend time mode ──────────────────────────────────────────
    {
      name: 'transcend time — fires once timer reaches threshold',
      input: {
        ...defaultInput(),
        transcendMode: 'time',
        toggle21: true,
        upgrade89: 1,
        transcendamount: 60,
        autoResetTimerTranscension: 59.99,
        dt: 0.025,
        coinsThisTranscension: new Decimal(1e101)
      }
    },
    {
      name: 'transcend time — transcensionChallenge gate blocks',
      input: {
        ...defaultInput(),
        transcendMode: 'time',
        toggle21: true,
        upgrade89: 1,
        transcendamount: 1,
        autoResetTimerTranscension: 100,
        coinsThisTranscension: new Decimal(1e101),
        transcensionChallenge: 1
      }
    },

    // ─── Reincarnation block (ascensionChallenge === 12 gate) ─────────
    {
      name: 'reincarnation block — ascensionChallenge=12 (timer not accumulated, no fires)',
      input: {
        ...defaultInput(),
        ascensionChallenge: 12,
        reincarnationMode: 'time',
        toggle27: true,
        research46: 1,
        reincarnationamount: 0,
        transcendShards: new Decimal('1e301'),
        autoResetTimerReincarnation: 100
      }
    },
    {
      name: 'reincarnation block — ascensionChallenge=0, amount mode unsatisfied (timer still accumulates)',
      input: {
        ...defaultInput(),
        ascensionChallenge: 0,
        reincarnationMode: 'amount',
        toggle27: false, // blocked
        autoResetTimerReincarnation: 5
      }
    },

    // ─── Reincarnation time mode ──────────────────────────────────────
    {
      name: 'reincarnation time — all conditions met (fires)',
      input: {
        ...defaultInput(),
        reincarnationMode: 'time',
        toggle27: true,
        research46: 1,
        reincarnationamount: 10,
        autoResetTimerReincarnation: 9.99,
        dt: 0.025,
        transcendShards: new Decimal('1e301')
      }
    },
    {
      name: 'reincarnation time — research46 = 0.5 exactly (blocked: > 0.5)',
      input: {
        ...defaultInput(),
        reincarnationMode: 'time',
        toggle27: true,
        research46: 0.5, // not > 0.5
        reincarnationamount: 0,
        autoResetTimerReincarnation: 100,
        transcendShards: new Decimal('1e301')
      }
    },
    {
      name: 'reincarnation time — transcendShards just under 1e300 (blocked)',
      input: {
        ...defaultInput(),
        reincarnationMode: 'time',
        toggle27: true,
        research46: 1,
        reincarnationamount: 0,
        autoResetTimerReincarnation: 100,
        transcendShards: new Decimal('9.99e299')
      }
    },
    {
      name: 'reincarnation time — transcensionChallenge !== 0 (blocked)',
      input: {
        ...defaultInput(),
        reincarnationMode: 'time',
        toggle27: true,
        research46: 1,
        reincarnationamount: 0,
        autoResetTimerReincarnation: 100,
        transcendShards: new Decimal('1e301'),
        transcensionChallenge: 1
      }
    },
    {
      name: 'reincarnation time — reincarnationChallenge !== 0 (blocked)',
      input: {
        ...defaultInput(),
        reincarnationMode: 'time',
        toggle27: true,
        research46: 1,
        reincarnationamount: 0,
        autoResetTimerReincarnation: 100,
        transcendShards: new Decimal('1e301'),
        reincarnationChallenge: 5
      }
    },

    // ─── Reincarnation amount mode ────────────────────────────────────
    {
      name: 'reincarnation amount — (rPoints+1) * 10^amount threshold (fires)',
      input: {
        ...defaultInput(),
        reincarnationMode: 'amount',
        toggle27: true,
        research46: 1,
        reincarnationPoints: new Decimal(9), // (9+1)*10^2 = 1000
        reincarnationPointGain: new Decimal(2000),
        reincarnationamount: 2,
        transcendShards: new Decimal('1e301')
      }
    },
    {
      name: 'reincarnation amount — rPoints=0, threshold = 1 * 10^amount (the +1 matters at 0)',
      input: {
        ...defaultInput(),
        reincarnationMode: 'amount',
        toggle27: true,
        research46: 1,
        reincarnationPoints: new Decimal(0), // (0+1)*10^1 = 10
        reincarnationPointGain: new Decimal(10), // exactly at threshold
        reincarnationamount: 1,
        transcendShards: new Decimal('1e301')
      }
    },
    {
      name: 'reincarnation amount — gain just below (+1 boundary blocks)',
      input: {
        ...defaultInput(),
        reincarnationMode: 'amount',
        toggle27: true,
        research46: 1,
        reincarnationPoints: new Decimal(0),
        reincarnationPointGain: new Decimal(9.99), // < 10
        reincarnationamount: 1,
        transcendShards: new Decimal('1e301')
      }
    },

    // ─── Multi-tier simultaneous trigger ──────────────────────────────
    {
      name: 'all three tiers fire in the same tick',
      input: {
        dt: 0.025,
        prestigeMode: 'amount',
        toggle15: true,
        autoPrestigeMilestone: 1,
        prestigePoints: new Decimal(100),
        prestigePointGain: new Decimal(1e6),
        prestigeamount: 2,
        coinsThisPrestige: new Decimal(1e17),
        autoResetTimerPrestige: 0,
        transcendMode: 'amount',
        toggle21: true,
        upgrade89: 1,
        transcendPoints: new Decimal(100),
        transcendPointGain: new Decimal(1e6),
        transcendamount: 2,
        coinsThisTranscension: new Decimal(1e101),
        autoResetTimerTranscension: 0,
        reincarnationMode: 'amount',
        toggle27: true,
        research46: 1,
        reincarnationPoints: new Decimal(9),
        reincarnationPointGain: new Decimal(2000),
        reincarnationamount: 2,
        transcendShards: new Decimal('1e301'),
        autoResetTimerReincarnation: 0,
        ascensionChallenge: 0,
        transcensionChallenge: 0,
        reincarnationChallenge: 0
      }
    },

    // ─── dt edge cases ────────────────────────────────────────────────
    {
      name: 'dt = 0 — timers untouched, no fires',
      input: {
        ...defaultInput(),
        dt: 0,
        prestigeMode: 'time',
        autoResetTimerPrestige: 5
      }
    },
    {
      name: 'large dt overshoots threshold in one tick',
      input: {
        ...defaultInput(),
        dt: 100,
        prestigeMode: 'time',
        toggle15: true,
        autoPrestigeMilestone: 1,
        prestigeamount: 5,
        autoResetTimerPrestige: 0,
        coinsThisPrestige: new Decimal(1e17)
      }
    }
  ]

  for (const c of cases) {
    it(c.name, () => {
      const newR = newApplyAutoResets(c.input)
      const oldR = oldApplyAutoResets(c.input)
      expect(newR.autoResetTimerPrestige).toBe(oldR.autoResetTimerPrestige)
      expect(newR.autoResetTimerTranscension).toBe(oldR.autoResetTimerTranscension)
      expect(newR.autoResetTimerReincarnation).toBe(oldR.autoResetTimerReincarnation)
      expect(newR.events).toEqual(oldR.events)
    })
  }
})
