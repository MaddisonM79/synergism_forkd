// Parity tests for resetCurrency. Old body transcribed verbatim from
// packages/web_ui/src/Synergism.ts (resetCurrency, ~line 3466 pre-migration).

import Decimal from 'break_infinity.js'
import { describe, expect, it } from 'vitest'
import {
  resetCurrency as newResetCurrency,
  type ResetCurrencyInput
} from '../../src/mechanics/resetCurrency'

const oldResetCurrency = (input: ResetCurrencyInput) => {
  let prestigePow = 0.5 + input.ecc5 / 100
  let transcendPow = 0.03

  if (input.transcensionChallenge === 5) {
    prestigePow = 0.01
  }
  if (input.reincarnationChallenge === 10) {
    prestigePow = 1e-4
    transcendPow = 0.001
  }
  prestigePow *= input.deflationMultiplier

  let prestigePointGain = Decimal.floor(
    Decimal.pow(input.coinsThisPrestige.dividedBy(1e12), prestigePow)
  )
  if (
    input.upgrade16 > 0.5
    && input.transcensionChallenge !== 5
    && input.reincarnationChallenge !== 10
  ) {
    prestigePointGain = prestigePointGain.times(
      Decimal.min(
        Decimal.pow(10, 1e33),
        Decimal.pow(
          input.acceleratorEffect,
          (1 / 3) * input.deflationMultiplier
        )
      )
    )
  }

  let transcendPointGain = Decimal.floor(
    Decimal.pow(input.coinsThisTranscension.dividedBy(1e100), transcendPow)
  )
  if (
    input.upgrade44 > 0.5
    && input.transcensionChallenge !== 5
    && input.reincarnationChallenge !== 10
  ) {
    transcendPointGain = transcendPointGain.times(
      Decimal.min(1e6, Decimal.pow(1.01, input.transcendCount))
    )
  }

  let reincarnationPointGain = Decimal.floor(
    Decimal.pow(input.transcendShards.dividedBy(1e300), 0.01)
  )
  if (input.reincarnationChallenge !== 0) {
    reincarnationPointGain = Decimal.pow(reincarnationPointGain, 0.01)
  }
  reincarnationPointGain = reincarnationPointGain.times(input.particleGainReward)
  if (input.upgrade65 > 0.5) {
    reincarnationPointGain = reincarnationPointGain.times(5)
  }
  if (input.ascensionChallenge === 12) {
    reincarnationPointGain = new Decimal('0')
  }

  return { prestigePointGain, transcendPointGain, reincarnationPointGain }
}

const decimalEq = (a: Decimal, b: Decimal): boolean => a.eq(b)

const baseInput: ResetCurrencyInput = {
  ecc5: 0,
  transcensionChallenge: 0,
  reincarnationChallenge: 0,
  ascensionChallenge: 0,
  deflationMultiplier: 1,
  coinsThisPrestige: new Decimal('1e20'),
  coinsThisTranscension: new Decimal('1e120'),
  transcendShards: new Decimal('1e320'),
  upgrade16: 0,
  upgrade44: 0,
  upgrade65: 0,
  transcendCount: 0,
  acceleratorEffect: new Decimal(1),
  particleGainReward: 1
}

const cases: Array<{ name: string, input: ResetCurrencyInput }> = [
  { name: 'baseline (no upgrades, no challenges)', input: baseInput },

  // ─── Challenge overrides ──────────────────────────────────────────────
  {
    name: 't-chal 5 forces prestigePow to 0.01 and disables upgrade-16 mult',
    input: { ...baseInput, transcensionChallenge: 5, upgrade16: 1, acceleratorEffect: new Decimal('1e10') }
  },
  {
    name: 'r-chal 10 forces prestigePow to 1e-4, transcendPow to 0.001, disables both upgrade mults',
    input: { ...baseInput, reincarnationChallenge: 10, upgrade16: 1, upgrade44: 1, transcendCount: 100 }
  },
  {
    name: 'r-chal non-zero (not 10) re-exponents reincarnationPointGain by 0.01',
    input: { ...baseInput, reincarnationChallenge: 4 }
  },
  {
    name: 'a-chal 12 zeroes reincarnationPointGain after all other math',
    input: { ...baseInput, ascensionChallenge: 12, upgrade65: 1, particleGainReward: 5 }
  },

  // ─── Upgrade gates ────────────────────────────────────────────────────
  {
    name: 'upgrade 16 active multiplies prestigePointGain by acceleratorEffect-derived factor',
    input: { ...baseInput, upgrade16: 1, acceleratorEffect: new Decimal('1e10') }
  },
  {
    name: 'upgrade 16 saturates against 10^1e33 cap when acceleratorEffect is huge',
    input: { ...baseInput, upgrade16: 1, acceleratorEffect: new Decimal('1e1000') }
  },
  {
    name: 'upgrade 44 active multiplies transcendPointGain by 1.01^transcendCount',
    input: { ...baseInput, upgrade44: 1, transcendCount: 200 }
  },
  {
    name: 'upgrade 44 saturates at 1e6 when transcendCount is huge',
    input: { ...baseInput, upgrade44: 1, transcendCount: 100000 }
  },
  {
    name: 'upgrade 65 multiplies reincarnationPointGain by 5',
    input: { ...baseInput, upgrade65: 1 }
  },

  // ─── Other inputs ─────────────────────────────────────────────────────
  {
    name: 'ecc5 raises prestigePow (0.5 + ecc5/100)',
    input: { ...baseInput, ecc5: 25 }
  },
  {
    name: 'deflationMultiplier 0.5 halves prestigePow',
    input: { ...baseInput, deflationMultiplier: 0.5 }
  },
  {
    name: 'deflationMultiplier 0.1 with upgrade 16 (both prestigePow and 16-mult exponent scaled)',
    input: { ...baseInput, deflationMultiplier: 0.1, upgrade16: 1, acceleratorEffect: new Decimal('1e10') }
  },
  {
    name: 'particleGainReward 5 multiplies reincarnationPointGain by 5',
    input: { ...baseInput, particleGainReward: 5 }
  },
  {
    name: 'particleGainReward 0 zeroes reincarnationPointGain',
    input: { ...baseInput, particleGainReward: 0 }
  },

  // ─── Resource extremes ────────────────────────────────────────────────
  {
    name: 'coinsThisPrestige < 1e12 → fractional base (< 1) → prestigePointGain floors to 0',
    input: { ...baseInput, coinsThisPrestige: new Decimal('1e10') }
  },
  {
    name: 'transcendShards < 1e300 → reincarnationPointGain floors to 0',
    input: { ...baseInput, transcendShards: new Decimal('1e200') }
  },
  {
    name: 'all-on combo (ecc5+upgrades+nonzero r-chal+deflation)',
    input: {
      ...baseInput,
      ecc5: 20,
      deflationMultiplier: 0.7,
      upgrade16: 1,
      upgrade44: 1,
      upgrade65: 1,
      transcendCount: 50,
      acceleratorEffect: new Decimal('1e8'),
      particleGainReward: 3,
      reincarnationChallenge: 5
    }
  }
]

describe('parity: resetCurrency', () => {
  for (const c of cases) {
    it(c.name, () => {
      const newRes = newResetCurrency(c.input)
      const oldRes = oldResetCurrency(c.input)
      expect(decimalEq(newRes.prestigePointGain, oldRes.prestigePointGain)).toBe(true)
      expect(decimalEq(newRes.transcendPointGain, oldRes.transcendPointGain)).toBe(true)
      expect(decimalEq(newRes.reincarnationPointGain, oldRes.reincarnationPointGain)).toBe(true)
    })
  }
})
