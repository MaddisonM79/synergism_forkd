// Parity tests for the event-buff selectors. Old bodies transcribed verbatim
// from packages/web_ui/src/Event.ts (BuffType enum, getEventBuff,
// consumableEventBuff, calculateEventSourceBuff).

import { describe, expect, it } from 'vitest'
import {
  BuffType,
  calculateEventSourceBuff as newCalcEventSource,
  consumableEventBuff as newConsumable,
  type GameEventBuffs,
  getEventBuff as newGetEventBuff
} from '../../src/mechanics/eventBuffs'

// ─── Old implementations ──────────────────────────────────────────────────

const oldGetEventBuff = (
  buff: BuffType,
  event: GameEventBuffs | null,
  oneMindUnlocked: boolean
): number => {
  if (event === null) return 0
  switch (buff) {
    case BuffType.Quark: return event.quark
    case BuffType.GoldenQuark: return event.goldenQuark
    case BuffType.Cubes: return event.cubes
    case BuffType.PowderConversion: return event.powderConversion
    case BuffType.AscensionSpeed: return event.ascensionSpeed
    case BuffType.GlobalSpeed: return event.globalSpeed
    case BuffType.AscensionScore: return event.ascensionScore
    case BuffType.AntSacrifice: return event.antSacrifice
    case BuffType.Offering: return event.offering
    case BuffType.Obtainium: return event.obtainium
    case BuffType.Octeract: return event.octeract
    case BuffType.OneMind: return oneMindUnlocked ? event.oneMind : 0
    case BuffType.BlueberryTime: return event.blueberryTime
    case BuffType.AmbrosiaLuck: return event.ambrosiaLuck
    default: throw new Error(`unhandled: ${buff as number}`)
  }
}

const oldConsumableEventBuff = (buff: BuffType, amount: number): number => {
  if (amount === 0) return 0
  const interval = amount - 1
  switch (buff) {
    case BuffType.Quark: return 0.25 + 0.025 * interval
    case BuffType.GoldenQuark: return 0
    case BuffType.Cubes: return 0.5 + 0.05 * interval
    case BuffType.PowderConversion: return 0
    case BuffType.AscensionSpeed: return 0
    case BuffType.GlobalSpeed: return 0
    case BuffType.AscensionScore: return 0
    case BuffType.AntSacrifice: return 0
    case BuffType.Offering: return 0.5 + 0.05 * interval
    case BuffType.Obtainium: return 0.5 + 0.05 * interval
    case BuffType.Octeract: return 0
    case BuffType.OneMind: return 0
    case BuffType.BlueberryTime: return 0.1 + 0.01 * interval
    case BuffType.AmbrosiaLuck: return 0.1 + 0.01 * interval
    default: throw new Error(`unhandled: ${buff as number}`)
  }
}

const oldCalcEventSource = (
  buff: BuffType,
  event: GameEventBuffs | null,
  oneMindUnlocked: boolean,
  amount: number
): number => oldGetEventBuff(buff, event, oneMindUnlocked) + oldConsumableEventBuff(buff, amount)

// ─── Test fixtures ────────────────────────────────────────────────────────

const sampleEvent: GameEventBuffs = {
  quark: 0.1,
  goldenQuark: 0.05,
  cubes: 0.2,
  powderConversion: 0.15,
  ascensionSpeed: 0.25,
  globalSpeed: 0.3,
  ascensionScore: 0.35,
  antSacrifice: 0.4,
  offering: 0.45,
  obtainium: 0.5,
  octeract: 0.55,
  blueberryTime: 0.6,
  ambrosiaLuck: 0.65,
  oneMind: 0.7
}

const allBuffs: BuffType[] = [
  BuffType.Quark,
  BuffType.GoldenQuark,
  BuffType.Cubes,
  BuffType.PowderConversion,
  BuffType.AscensionSpeed,
  BuffType.GlobalSpeed,
  BuffType.AscensionScore,
  BuffType.AntSacrifice,
  BuffType.Offering,
  BuffType.Obtainium,
  BuffType.Octeract,
  BuffType.BlueberryTime,
  BuffType.AmbrosiaLuck,
  BuffType.OneMind
]

// ─── getEventBuff parity ──────────────────────────────────────────────────

describe('parity: getEventBuff', () => {
  describe('null event returns 0 for every buff', () => {
    for (const buff of allBuffs) {
      it(`buff=${BuffType[buff]}`, () => {
        expect(newGetEventBuff(buff, null, true)).toBe(oldGetEventBuff(buff, null, true))
      })
    }
  })

  describe('active event with oneMindUnlocked=false', () => {
    for (const buff of allBuffs) {
      it(`buff=${BuffType[buff]}`, () => {
        expect(newGetEventBuff(buff, sampleEvent, false))
          .toBe(oldGetEventBuff(buff, sampleEvent, false))
      })
    }
  })

  describe('active event with oneMindUnlocked=true', () => {
    for (const buff of allBuffs) {
      it(`buff=${BuffType[buff]}`, () => {
        expect(newGetEventBuff(buff, sampleEvent, true))
          .toBe(oldGetEventBuff(buff, sampleEvent, true))
      })
    }
  })
})

// ─── consumableEventBuff parity ──────────────────────────────────────────

describe('parity: consumableEventBuff', () => {
  const amounts = [0, 1, 5, 10, 100]
  for (const amount of amounts) {
    describe(`happyHourBellAmount=${amount}`, () => {
      for (const buff of allBuffs) {
        it(`buff=${BuffType[buff]}`, () => {
          expect(newConsumable(buff, amount)).toBe(oldConsumableEventBuff(buff, amount))
        })
      }
    })
  }
})

// ─── calculateEventSourceBuff parity ─────────────────────────────────────

describe('parity: calculateEventSourceBuff', () => {
  const cases = [
    { event: null, oneMindUnlocked: false, amount: 0 },
    { event: null, oneMindUnlocked: true, amount: 5 },
    { event: sampleEvent, oneMindUnlocked: false, amount: 0 },
    { event: sampleEvent, oneMindUnlocked: true, amount: 3 },
    { event: sampleEvent, oneMindUnlocked: true, amount: 10 }
  ] as const
  for (const c of cases) {
    describe(JSON.stringify({ event: c.event ? '<event>' : null, oneMindUnlocked: c.oneMindUnlocked, amount: c.amount }), () => {
      for (const buff of allBuffs) {
        it(`buff=${BuffType[buff]}`, () => {
          expect(newCalcEventSource(buff, c.event, c.oneMindUnlocked, c.amount))
            .toBe(oldCalcEventSource(buff, c.event, c.oneMindUnlocked, c.amount))
        })
      }
    })
  }
})
