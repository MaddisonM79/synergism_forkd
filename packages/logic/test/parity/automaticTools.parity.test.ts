// Parity tests for the migrated pure cases of automaticTools.
// Old bodies transcribed verbatim from packages/web_ui/src/Helper.ts
// (automaticTools, addObtainium + addOfferings + antSacrifice timer
// portion + canAutoSacrifice pre-migration).

import { describe, expect, it } from 'vitest'
import type { CoreEvent } from '../../src/events/types'
import { Decimal } from '../../src/math/bignum'
import {
  type AddObtainiumInput,
  type AddObtainiumResult,
  addObtainium as newAddObtainium,
  type AddOfferingsInput,
  type AddOfferingsResult,
  addOfferings as newAddOfferings,
  type AdvanceAntSacrificeTimersInput,
  type AdvanceAntSacrificeTimersResult,
  advanceAntSacrificeTimers as newAdvanceAntSacrificeTimers,
  type CheckAntSacrificeReadyInput,
  type CheckAntSacrificeReadyResult,
  checkAntSacrificeReady as newCheckAntSacrificeReady
} from '../../src/tick/automaticTools'

// ─── addObtainium ───────────────────────────────────────────────────────

const oldAddObtainium = (input: AddObtainiumInput): AddObtainiumResult => {
  // Legacy body: aborts in c14, else conditionally clamps gain, then adds.
  // UI side effect (visualUpdateResearch) is gated by Tabs.Research in the
  // caller; the logic-tier translation surfaces it as an auto-tool-fired
  // event whenever the branch wasn't aborted.
  if (input.ascensionChallenge === 14) {
    return { obtainium: input.obtainium, events: [] }
  }
  let gain = input.obtainiumGain
  if (input.taxmanLastStandEnabled && input.taxmanLastStandCompletions >= 2) {
    gain = Decimal.min(gain, input.obtainium.times(100).plus(1))
  }
  return {
    obtainium: input.obtainium.add(gain),
    events: [{ kind: 'auto-tool-fired', tool: 'addObtainium' }]
  }
}

describe('parity: addObtainium', () => {
  const cases: Array<{ name: string, input: AddObtainiumInput }> = [
    {
      name: 'baseline — no challenge, no taxman, normal gain',
      input: {
        obtainium: new Decimal(1e6),
        obtainiumGain: new Decimal(100),
        ascensionChallenge: 0,
        taxmanLastStandEnabled: false,
        taxmanLastStandCompletions: 0
      }
    },
    {
      name: 'c14 abort — returns unchanged, no event',
      input: {
        obtainium: new Decimal(1e6),
        obtainiumGain: new Decimal(500),
        ascensionChallenge: 14,
        taxmanLastStandEnabled: false,
        taxmanLastStandCompletions: 0
      }
    },
    {
      name: 'taxman enabled, completions < 2 (no clamp)',
      input: {
        obtainium: new Decimal(1),
        obtainiumGain: new Decimal(1e30),
        ascensionChallenge: 0,
        taxmanLastStandEnabled: true,
        taxmanLastStandCompletions: 1
      }
    },
    {
      name: 'taxman enabled, completions = 2 (clamp engaged, gain capped)',
      input: {
        obtainium: new Decimal(10),
        // Without clamp, gain is huge; with clamp, ceiling is 10*100+1 = 1001
        obtainiumGain: new Decimal(1e30),
        ascensionChallenge: 0,
        taxmanLastStandEnabled: true,
        taxmanLastStandCompletions: 2
      }
    },
    {
      name: 'taxman enabled, completions = 3 (clamp still engaged)',
      input: {
        obtainium: new Decimal(0),
        // obtainium=0 makes ceiling = 0*100 + 1 = 1
        obtainiumGain: new Decimal(1e6),
        ascensionChallenge: 0,
        taxmanLastStandEnabled: true,
        taxmanLastStandCompletions: 3
      }
    },
    {
      name: 'taxman clamp inactive when gain already < ceiling',
      input: {
        obtainium: new Decimal(1e6),
        // Ceiling = 1e8 + 1, gain = 500, so clamp is a no-op
        obtainiumGain: new Decimal(500),
        ascensionChallenge: 0,
        taxmanLastStandEnabled: true,
        taxmanLastStandCompletions: 5
      }
    },
    {
      name: 'taxman disabled, completions = 5 (no clamp)',
      input: {
        obtainium: new Decimal(10),
        obtainiumGain: new Decimal(1e30),
        ascensionChallenge: 0,
        taxmanLastStandEnabled: false,
        taxmanLastStandCompletions: 5
      }
    },
    {
      name: 'fractional obtainium balance, normal gain',
      input: {
        obtainium: new Decimal('123.456e9'),
        obtainiumGain: new Decimal('7.89e5'),
        ascensionChallenge: 0,
        taxmanLastStandEnabled: false,
        taxmanLastStandCompletions: 0
      }
    },
    {
      name: 'zero gain (no-op add, event still fires)',
      input: {
        obtainium: new Decimal(1e9),
        obtainiumGain: new Decimal(0),
        ascensionChallenge: 0,
        taxmanLastStandEnabled: false,
        taxmanLastStandCompletions: 0
      }
    },
    {
      name: 'c14 abort overrides taxman state',
      input: {
        obtainium: new Decimal(1e6),
        obtainiumGain: new Decimal(1e30),
        ascensionChallenge: 14,
        taxmanLastStandEnabled: true,
        taxmanLastStandCompletions: 5
      }
    },
    {
      name: 'tiny obtainium balance, taxman engaged (ceiling near 1)',
      input: {
        obtainium: new Decimal('1e-6'),
        obtainiumGain: new Decimal(1e10),
        ascensionChallenge: 0,
        taxmanLastStandEnabled: true,
        taxmanLastStandCompletions: 2
      }
    }
  ]

  for (const c of cases) {
    it(c.name, () => {
      const newR = newAddObtainium(c.input)
      const oldR = oldAddObtainium(c.input)
      expect(newR.obtainium.toString()).toBe(oldR.obtainium.toString())
      expect(newR.events).toEqual(oldR.events)
    })
  }
})

// ─── addOfferings ───────────────────────────────────────────────────────

const oldAddOfferings = (input: AddOfferingsInput): AddOfferingsResult => {
  // Legacy body in Helper.ts: G.autoOfferingCounter += time; offerings +=
  // floor(counter); counter %= 1.
  let counter = input.autoOfferingCounter + input.time
  const offerings = input.offerings.add(Math.floor(counter))
  counter = counter % 1
  return { autoOfferingCounter: counter, offerings }
}

describe('parity: addOfferings', () => {
  const cases: Array<{ name: string, input: AddOfferingsInput }> = [
    {
      name: 'tiny tick, no overflow',
      input: { time: 0.025, autoOfferingCounter: 0, offerings: new Decimal(0) }
    },
    {
      name: 'tick fills the bucket exactly',
      input: { time: 0.5, autoOfferingCounter: 0.5, offerings: new Decimal(1e6) }
    },
    {
      name: 'tick overflows by integer + remainder',
      input: { time: 3.75, autoOfferingCounter: 0.5, offerings: new Decimal(1e6) }
    },
    {
      name: 'time=0 (counter stays put, no offerings gained)',
      input: { time: 0, autoOfferingCounter: 0.5, offerings: new Decimal(100) }
    },
    {
      name: 'large multi-second tick (challenge 3 + cube 1x2 boost)',
      input: { time: 25.4, autoOfferingCounter: 0.9, offerings: new Decimal(1e9) }
    },
    {
      name: 'counter already > 1 (legacy never reduces incoming counter pre-add)',
      input: { time: 0.025, autoOfferingCounter: 1.4, offerings: new Decimal(50) }
    },
    {
      name: 'fractional carry — counter stays as fractional remainder',
      input: { time: 0.125, autoOfferingCounter: 0.875, offerings: new Decimal(0) }
    },
    {
      name: 'huge offerings balance, single offering gained',
      input: { time: 1, autoOfferingCounter: 0.5, offerings: new Decimal('1.5e30') }
    }
  ]

  for (const c of cases) {
    it(c.name, () => {
      const newR = newAddOfferings(c.input)
      const oldR = oldAddOfferings(c.input)
      expect(newR.autoOfferingCounter).toBe(oldR.autoOfferingCounter)
      expect(newR.offerings.toString()).toBe(oldR.offerings.toString())
    })
  }
})

// ─── advanceAntSacrificeTimers ──────────────────────────────────────────

const oldAdvanceAntSacrificeTimers = (
  input: AdvanceAntSacrificeTimersInput
): AdvanceAntSacrificeTimersResult => {
  // Legacy body in Helper.ts: scaled timer advances by time*globalDelta,
  // wall-clock timer advances by raw time. The globalDelta is whatever the
  // caller pre-evaluated (halfMind ? 10 : calculateGlobalSpeedMult()).
  return {
    antSacrificeTimer: input.antSacrificeTimer + input.time * input.globalDelta,
    antSacrificeTimerReal: input.antSacrificeTimerReal + input.time
  }
}

describe('parity: advanceAntSacrificeTimers', () => {
  const cases: Array<{ name: string, input: AdvanceAntSacrificeTimersInput }> = [
    {
      name: 'baseline tick — globalDelta = 1, fresh timers',
      input: {
        time: 0.025,
        globalDelta: 1,
        antSacrificeTimer: 0,
        antSacrificeTimerReal: 0
      }
    },
    {
      name: 'halfMind unlocked — globalDelta = 10',
      input: {
        time: 0.025,
        globalDelta: 10,
        antSacrificeTimer: 5,
        antSacrificeTimerReal: 5
      }
    },
    {
      name: 'fractional globalDelta (slow time)',
      input: {
        time: 0.05,
        globalDelta: 0.25,
        antSacrificeTimer: 100,
        antSacrificeTimerReal: 100
      }
    },
    {
      name: 'large globalDelta (fast time)',
      input: {
        time: 0.025,
        globalDelta: 3600,
        antSacrificeTimer: 0,
        antSacrificeTimerReal: 0
      }
    },
    {
      name: 'zero tick (no-op)',
      input: {
        time: 0,
        globalDelta: 10,
        antSacrificeTimer: 42.5,
        antSacrificeTimerReal: 42.5
      }
    },
    {
      name: 'big offline catch-up tick',
      input: {
        time: 3600,
        globalDelta: 5,
        antSacrificeTimer: 0,
        antSacrificeTimerReal: 0
      }
    },
    {
      name: 'timers diverge (scaled vs. raw) — globalDelta != 1',
      input: {
        time: 1,
        globalDelta: 7.5,
        antSacrificeTimer: 50,
        antSacrificeTimerReal: 10
      }
    },
    {
      name: 'globalDelta = 0 (frozen time) — scaled timer stays put',
      input: {
        time: 1,
        globalDelta: 0,
        antSacrificeTimer: 30,
        antSacrificeTimerReal: 30
      }
    },
    {
      name: 'subsecond tick with fractional globalDelta',
      input: {
        time: 0.0125,
        globalDelta: 1.5,
        antSacrificeTimer: 0.5,
        antSacrificeTimerReal: 0.5
      }
    }
  ]

  for (const c of cases) {
    it(c.name, () => {
      const newR = newAdvanceAntSacrificeTimers(c.input)
      const oldR = oldAdvanceAntSacrificeTimers(c.input)
      expect(newR.antSacrificeTimer).toBe(oldR.antSacrificeTimer)
      expect(newR.antSacrificeTimerReal).toBe(oldR.antSacrificeTimerReal)
    })
  }
})

// ─── checkAntSacrificeReady ─────────────────────────────────────────────

// Verbatim legacy `canAutoSacrifice` body, returning the same `events` shape
// the new logic function returns so the parity test can compare 1:1.
const oldCheckAntSacrificeReady = (
  input: CheckAntSacrificeReadyInput
): CheckAntSacrificeReadyResult => {
  // Mirrors web_ui constants. Kept inline so the parity reference is
  // self-contained — the production constants live in
  // packages/web_ui/src/Features/Ants/AntSacrifice/constants.ts and
  // packages/web_ui/src/Features/Ants/Automation/sacrifice.ts (the 0.001
  // tolerance used by the maxRebornELO derivation + MaxRebornELO mode).
  const MIN_CRUMBS = 1e40
  const MIN_DELAY = 0.05
  const TOLERANCE = 0.001

  const availableRebornELO = input.availableRebornELO
  const maxRebornELO = availableRebornELO < TOLERANCE

  const onlySacrificeMaxReborn = input.onlySacrificeMaxRebornELO
  if (onlySacrificeMaxReborn && !maxRebornELO) {
    return { events: [] }
  }

  const hasEnoughCrumbs = input.crumbsThisSacrifice.gte(MIN_CRUMBS)
  const offCooldown = input.antSacrificeTimerReal >= MIN_DELAY
  const universalChecks = hasEnoughCrumbs && offCooldown && input.autoSacrificeEnabled

  let specificCheck = false
  switch (input.mode) {
    case 'InGameTime':
      specificCheck = input.antSacrificeTimer >= input.autoSacrificeThreshold
      break
    case 'RealTime':
      specificCheck = input.antSacrificeTimerReal >= input.autoSacrificeThreshold
      break
    case 'ImmortalELOGain':
      specificCheck = input.immortalELOGain >= input.autoSacrificeThreshold
      break
    case 'MaxRebornELO':
      specificCheck = (input.immortalELO - input.rebornELO) <= TOLERANCE
      break
  }

  const alwaysSacrificeMaxReborn = input.alwaysSacrificeMaxRebornELO
  const ready = alwaysSacrificeMaxReborn
    ? universalChecks && (maxRebornELO || specificCheck)
    : universalChecks && specificCheck

  const events: CoreEvent[] = ready ? [{ kind: 'ant-sacrifice-triggered' }] : []
  return { events }
}

const defaultCheckInput = (
  overrides: Partial<CheckAntSacrificeReadyInput> = {}
): CheckAntSacrificeReadyInput => ({
  // Default: late-game-feasible setup, InGameTime mode, all gates clear,
  // mode-specific check passes.
  mode: 'InGameTime',
  crumbsThisSacrifice: new Decimal(1e50),
  antSacrificeTimerReal: 60,
  autoSacrificeEnabled: true,
  availableRebornELO: 5,
  onlySacrificeMaxRebornELO: false,
  alwaysSacrificeMaxRebornELO: false,
  antSacrificeTimer: 120,
  autoSacrificeThreshold: 60,
  immortalELOGain: 0,
  immortalELO: 100,
  rebornELO: 50,
  ...overrides
})

describe('parity: checkAntSacrificeReady', () => {
  const cases: Array<{ name: string, input: CheckAntSacrificeReadyInput }> = [
    {
      name: 'InGameTime — all gates clear, specific check passes',
      input: defaultCheckInput()
    },
    {
      name: 'InGameTime — timer below threshold (no fire)',
      input: defaultCheckInput({ antSacrificeTimer: 30, autoSacrificeThreshold: 60 })
    },
    {
      name: 'RealTime mode — wall-clock timer crosses threshold',
      input: defaultCheckInput({
        mode: 'RealTime',
        antSacrificeTimer: 0,
        antSacrificeTimerReal: 75,
        autoSacrificeThreshold: 60
      })
    },
    {
      name: 'RealTime mode — wall-clock timer below threshold',
      input: defaultCheckInput({
        mode: 'RealTime',
        antSacrificeTimerReal: 30,
        autoSacrificeThreshold: 60
      })
    },
    {
      name: 'ImmortalELOGain mode — gain meets threshold',
      input: defaultCheckInput({
        mode: 'ImmortalELOGain',
        immortalELOGain: 1500,
        autoSacrificeThreshold: 1000
      })
    },
    {
      name: 'ImmortalELOGain mode — gain below threshold',
      input: defaultCheckInput({
        mode: 'ImmortalELOGain',
        immortalELOGain: 500,
        autoSacrificeThreshold: 1000
      })
    },
    {
      name: 'MaxRebornELO mode — delta within tolerance (fire)',
      input: defaultCheckInput({
        mode: 'MaxRebornELO',
        immortalELO: 100.0005,
        rebornELO: 100
      })
    },
    {
      name: 'MaxRebornELO mode — delta exceeds tolerance (no fire)',
      input: defaultCheckInput({
        mode: 'MaxRebornELO',
        immortalELO: 100.5,
        rebornELO: 100
      })
    },
    {
      name: 'master gate off — never fires',
      input: defaultCheckInput({ autoSacrificeEnabled: false })
    },
    {
      name: 'crumbs below 1e40 — never fires',
      input: defaultCheckInput({ crumbsThisSacrifice: new Decimal(1e39) })
    },
    {
      name: 'cooldown not elapsed (timerReal < 0.05) — never fires',
      input: defaultCheckInput({ antSacrificeTimerReal: 0.025 })
    },
    {
      name: 'onlySacrificeMaxRebornELO on, but reborn not maxed — never fires',
      input: defaultCheckInput({
        onlySacrificeMaxRebornELO: true,
        availableRebornELO: 5
      })
    },
    {
      name: 'onlySacrificeMaxRebornELO on, reborn is maxed (proceed with mode check)',
      input: defaultCheckInput({
        onlySacrificeMaxRebornELO: true,
        availableRebornELO: 0
      })
    },
    {
      name: 'alwaysSacrificeMaxRebornELO + reborn maxed (fires even when mode check fails)',
      input: defaultCheckInput({
        alwaysSacrificeMaxRebornELO: true,
        availableRebornELO: 0,
        antSacrificeTimer: 0,
        autoSacrificeThreshold: 60
      })
    },
    {
      name: 'alwaysSacrificeMaxRebornELO + reborn NOT maxed, mode check passes (fires)',
      input: defaultCheckInput({
        alwaysSacrificeMaxRebornELO: true,
        availableRebornELO: 10,
        antSacrificeTimer: 120,
        autoSacrificeThreshold: 60
      })
    },
    {
      name: 'alwaysSacrificeMaxRebornELO + reborn NOT maxed + mode fails (no fire)',
      input: defaultCheckInput({
        alwaysSacrificeMaxRebornELO: true,
        availableRebornELO: 10,
        antSacrificeTimer: 30,
        autoSacrificeThreshold: 60
      })
    },
    {
      name: 'cooldown exactly at 0.05 boundary (passes)',
      input: defaultCheckInput({ antSacrificeTimerReal: 0.05 })
    },
    {
      name: 'crumbs exactly at 1e40 boundary (passes)',
      input: defaultCheckInput({ crumbsThisSacrifice: new Decimal(1e40) })
    },
    {
      name: 'MaxRebornELO mode — delta exactly at 0.001 tolerance (passes)',
      input: defaultCheckInput({
        mode: 'MaxRebornELO',
        immortalELO: 100.001,
        rebornELO: 100
      })
    },
    {
      name: 'availableRebornELO exactly at 0.001 tolerance — not "maxed"',
      input: defaultCheckInput({
        mode: 'InGameTime',
        availableRebornELO: 0.001,
        onlySacrificeMaxRebornELO: true
      })
    },
    {
      name: 'all bells and whistles — autoSacrificeEnabled off blocks everything',
      input: defaultCheckInput({
        autoSacrificeEnabled: false,
        alwaysSacrificeMaxRebornELO: true,
        availableRebornELO: 0,
        antSacrificeTimer: 99999
      })
    }
  ]

  for (const c of cases) {
    it(c.name, () => {
      const newR = newCheckAntSacrificeReady(c.input)
      const oldR = oldCheckAntSacrificeReady(c.input)
      expect(newR.events).toEqual(oldR.events)
    })
  }
})
