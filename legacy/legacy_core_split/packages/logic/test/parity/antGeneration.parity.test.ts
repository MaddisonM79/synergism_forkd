// Parity tests for generateAntsAndCrumbs.
// Old body transcribed verbatim from
// packages/web_ui/src/Features/Ants/AntProducers/lib/generate-ant-producers.ts
// (pre-migration), minus the `activateELO(dt)` tail call which stays
// in web_ui (depends on Date.now / DOM / quark-credit side effects).

import { describe, expect, it } from 'vitest'
import { Decimal } from '../../src/math/bignum'
import { antMasteryData, calculateSelfSpeedFromMastery } from '../../src/mechanics/antMasteries'
import { antProducerData, calculateBaseAntsToBeGenerated } from '../../src/mechanics/antProducers'
import {
  generateAntsAndCrumbs as newGenerate,
  type GenerateAntsAndCrumbsInput,
  type GenerateAntsAndCrumbsResult
} from '../../src/tick/antGeneration'

const LAST_ANT_PRODUCER = 8

// Verbatim transcription mirroring the legacy producer loop + crumb math.
const oldGenerate = (input: GenerateAntsAndCrumbsInput): GenerateAntsAndCrumbsResult => {
  const updatedGenerated: Decimal[] = input.producers.map((p) => p.generated)

  for (let antType = LAST_ANT_PRODUCER; antType > 0; antType--) {
    const selfSpeedMult = calculateSelfSpeedFromMastery({
      antData: antMasteryData[antType],
      masteryLevel: input.producers[antType].masteryLevel,
      purchased: input.producers[antType].purchased
    })
    const baseGeneration = calculateBaseAntsToBeGenerated({
      generated: updatedGenerated[antType],
      purchased: input.producers[antType].purchased,
      baseProduction: antProducerData[antType].baseProduction,
      selfSpeedMult,
      antSpeedMult: input.antSpeedMult
    })
    const producedAnt = antProducerData[antType].produces!
    updatedGenerated[producedAnt] = updatedGenerated[producedAnt].add(baseGeneration.times(input.dt))
  }

  const workersSelfSpeed = calculateSelfSpeedFromMastery({
    antData: antMasteryData[0],
    masteryLevel: input.producers[0].masteryLevel,
    purchased: input.producers[0].purchased
  })
  const crumbsToGenerate = calculateBaseAntsToBeGenerated({
    generated: updatedGenerated[0],
    purchased: input.producers[0].purchased,
    baseProduction: antProducerData[0].baseProduction,
    selfSpeedMult: workersSelfSpeed,
    antSpeedMult: input.antSpeedMult
  }).times(input.dt)

  return {
    producersGenerated: updatedGenerated,
    crumbs: Decimal.add(input.crumbs, crumbsToGenerate),
    crumbsThisSacrifice: Decimal.add(input.crumbsThisSacrifice, crumbsToGenerate),
    crumbsEverMade: Decimal.add(input.crumbsEverMade, crumbsToGenerate)
  }
}

const blankProducers = () => {
  const arr = []
  for (let i = 0; i <= LAST_ANT_PRODUCER; i++) {
    arr.push({
      generated: new Decimal(0),
      purchased: 0,
      masteryLevel: 0
    })
  }
  return arr
}

const defaultInput = (overrides: Partial<GenerateAntsAndCrumbsInput> = {}): GenerateAntsAndCrumbsInput => ({
  dt: 0.025,
  antSpeedMult: new Decimal(1),
  producers: blankProducers(),
  crumbs: new Decimal(0),
  crumbsThisSacrifice: new Decimal(0),
  crumbsEverMade: new Decimal(0),
  ...overrides
})

describe('parity: generateAntsAndCrumbs', () => {
  const cases: Array<{ name: string, input: GenerateAntsAndCrumbsInput }> = [
    {
      name: 'cold start — no producers, no mastery (zero work)',
      input: defaultInput()
    },
    {
      name: 'Workers only (purchased 1, others empty) — produces only crumbs',
      input: (() => {
        const producers = blankProducers()
        producers[0] = { generated: new Decimal(0), purchased: 1, masteryLevel: 0 }
        return defaultInput({ producers })
      })()
    },
    {
      name: 'Breeders 1 + Workers 1 — Breeders produce Workers, Workers produce crumbs',
      input: (() => {
        const producers = blankProducers()
        producers[0] = { generated: new Decimal(0), purchased: 1, masteryLevel: 0 }
        producers[1] = { generated: new Decimal(0), purchased: 1, masteryLevel: 0 }
        return defaultInput({ producers })
      })()
    },
    {
      name: 'mid-game — first 5 tiers populated',
      input: (() => {
        const producers = blankProducers()
        producers[0] = { generated: new Decimal('1e6'), purchased: 100, masteryLevel: 5 }
        producers[1] = { generated: new Decimal('1e4'), purchased: 50, masteryLevel: 4 }
        producers[2] = { generated: new Decimal('1e2'), purchased: 25, masteryLevel: 3 }
        producers[3] = { generated: new Decimal(10), purchased: 10, masteryLevel: 2 }
        producers[4] = { generated: new Decimal(5), purchased: 5, masteryLevel: 1 }
        return defaultInput({
          producers,
          dt: 0.1,
          antSpeedMult: new Decimal(10),
          crumbs: new Decimal('1e10'),
          crumbsThisSacrifice: new Decimal('1e8'),
          crumbsEverMade: new Decimal('1e12')
        })
      })()
    },
    {
      name: 'late-game — all 9 tiers populated, max mastery',
      input: (() => {
        const producers = blankProducers()
        for (let i = 0; i <= LAST_ANT_PRODUCER; i++) {
          producers[i] = {
            generated: new Decimal(Math.pow(10, 20 - i * 2)),
            purchased: 200 - i * 10,
            masteryLevel: 12
          }
        }
        return defaultInput({
          producers,
          dt: 0.025,
          antSpeedMult: new Decimal(1000),
          crumbs: new Decimal('1e50'),
          crumbsThisSacrifice: new Decimal('1e40'),
          crumbsEverMade: new Decimal('1e100')
        })
      })()
    },
    {
      name: 'producer-loop cascade — HolySpirit only, watch propagation',
      input: (() => {
        const producers = blankProducers()
        producers[8] = { generated: new Decimal(1), purchased: 1, masteryLevel: 5 }
        return defaultInput({ producers, dt: 1 })
      })()
    },
    {
      name: 'large dt (offline catch-up)',
      input: (() => {
        const producers = blankProducers()
        producers[0] = { generated: new Decimal('1e10'), purchased: 1000, masteryLevel: 8 }
        producers[1] = { generated: new Decimal('1e8'), purchased: 500, masteryLevel: 7 }
        return defaultInput({ producers, dt: 3600 })
      })()
    },
    {
      name: 'high antSpeedMult (galaxy speed)',
      input: (() => {
        const producers = blankProducers()
        producers[0] = { generated: new Decimal('1e8'), purchased: 100, masteryLevel: 6 }
        producers[1] = { generated: new Decimal('1e6'), purchased: 80, masteryLevel: 6 }
        producers[2] = { generated: new Decimal('1e4'), purchased: 60, masteryLevel: 6 }
        return defaultInput({
          producers,
          antSpeedMult: new Decimal('1e30'),
          dt: 0.025
        })
      })()
    },
    {
      name: 'high mastery only on lower tiers (mid-progression)',
      input: (() => {
        const producers = blankProducers()
        producers[0] = { generated: new Decimal('1e15'), purchased: 100, masteryLevel: 12 }
        producers[1] = { generated: new Decimal('1e10'), purchased: 80, masteryLevel: 12 }
        producers[2] = { generated: new Decimal('1e5'), purchased: 60, masteryLevel: 10 }
        producers[3] = { generated: new Decimal(100), purchased: 40, masteryLevel: 0 }
        return defaultInput({
          producers,
          antSpeedMult: new Decimal(50),
          dt: 0.5
        })
      })()
    }
  ]

  for (const c of cases) {
    it(c.name, () => {
      const newR = newGenerate(c.input)
      const oldR = oldGenerate(c.input)
      // Per-tier generated values match
      for (let i = 0; i <= LAST_ANT_PRODUCER; i++) {
        expect(newR.producersGenerated[i].toString())
          .toBe(oldR.producersGenerated[i].toString())
      }
      expect(newR.crumbs.toString()).toBe(oldR.crumbs.toString())
      expect(newR.crumbsThisSacrifice.toString()).toBe(oldR.crumbsThisSacrifice.toString())
      expect(newR.crumbsEverMade.toString()).toBe(oldR.crumbsEverMade.toString())
    })
  }
})
