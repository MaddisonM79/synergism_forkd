// Parity tests for the 2 simple cases of automaticTools.
// Old bodies transcribed verbatim from packages/web_ui/src/Helper.ts
// (automaticTools, addObtainium + addOfferings cases pre-migration).

import { describe, expect, it } from 'vitest'
import { Decimal } from '../../src/math/bignum'
import {
  type AddObtainiumInput,
  type AddObtainiumResult,
  addObtainium as newAddObtainium,
  type AddOfferingsInput,
  type AddOfferingsResult,
  addOfferings as newAddOfferings
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
