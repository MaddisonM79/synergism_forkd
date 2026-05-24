// Parity tests for the autoPotion case of addTimers.
// Old body transcribed verbatim from packages/web_ui/src/Helper.ts
// (addTimers, autoPotion case pre-migration).
//
// The legacy body calls `useConsumable(...)` inline; the logic-tier
// reformulation emits `auto-potion-fired` CoreEvents that the UI tier
// translates back into the same useConsumable calls. The oracle here
// records the equivalent event list (type, amount, fastMode) so we
// can assert event parity directly.

import { describe, expect, it } from 'vitest'
import type { CoreEvent } from '../../src/events/types'
import {
  advanceAutoPotionTimer as newAdvanceAutoPotion,
  type AdvanceAutoPotionTimerInput,
  type AdvanceAutoPotionTimerResult
} from '../../src/tick/timers'

const oldAdvanceAutoPotion = (input: AdvanceAutoPotionTimerInput): AdvanceAutoPotionTimerResult => {
  if (input.highestSingularityCount < 6) {
    return {
      autoPotionTimer: input.autoPotionTimer,
      autoPotionTimerObtainium: input.autoPotionTimerObtainium,
      events: []
    }
  }

  const events: CoreEvent[] = []

  const toggleOfferingOn = input.toggleOffering && input.offeringPotionCount > 0
  const toggleObtainiumOn = input.toggleObtainium && input.obtainiumPotionCount > 0

  let autoPotionTimer = input.autoPotionTimer + input.time * input.timeMultiplier
  let autoPotionTimerObtainium = input.autoPotionTimerObtainium + input.time * input.timeMultiplier

  const timerThreshold = (180 * Math.pow(1.03, -input.highestSingularityCount))
    / input.autoPotionSpeedMult

  const effectiveOfferingThreshold = toggleOfferingOn
    ? Math.min(1, timerThreshold) / 20
    : timerThreshold
  const effectiveObtainiumThreshold = toggleObtainiumOn
    ? Math.min(1, timerThreshold) / 20
    : timerThreshold

  if (autoPotionTimer >= effectiveOfferingThreshold) {
    const amountOfPotions = (autoPotionTimer - (autoPotionTimer % effectiveOfferingThreshold))
      / effectiveOfferingThreshold
    autoPotionTimer %= effectiveOfferingThreshold
    events.push({
      kind: 'auto-potion-fired',
      type: 'offering',
      amount: amountOfPotions,
      fastMode: toggleOfferingOn
    })
  }

  if (autoPotionTimerObtainium >= effectiveObtainiumThreshold) {
    const amountOfPotions = (autoPotionTimerObtainium - (autoPotionTimerObtainium % effectiveObtainiumThreshold))
      / effectiveObtainiumThreshold
    autoPotionTimerObtainium %= effectiveObtainiumThreshold
    events.push({
      kind: 'auto-potion-fired',
      type: 'obtainium',
      amount: amountOfPotions,
      fastMode: toggleObtainiumOn
    })
  }

  return {
    autoPotionTimer,
    autoPotionTimerObtainium,
    events
  }
}

const baseInput: AdvanceAutoPotionTimerInput = {
  time: 0.025,
  timeMultiplier: 1,
  highestSingularityCount: 10,
  autoPotionTimer: 0,
  autoPotionTimerObtainium: 0,
  toggleOffering: false,
  toggleObtainium: false,
  offeringPotionCount: 0,
  obtainiumPotionCount: 0,
  autoPotionSpeedMult: 1
}

describe('parity: advanceAutoPotionTimer', () => {
  const cases: Array<{ name: string, input: AdvanceAutoPotionTimerInput }> = [
    {
      name: 'gate off (sing < 6) — no state change',
      input: { ...baseInput, highestSingularityCount: 5, autoPotionTimer: 50, autoPotionTimerObtainium: 50 }
    },
    {
      name: 'gate off at exactly sing=5',
      input: { ...baseInput, highestSingularityCount: 5 }
    },
    {
      name: 'gate on at sing=6 (boundary)',
      input: { ...baseInput, highestSingularityCount: 6 }
    },
    {
      name: 'baseline accumulation (no toggles, no threshold crossing)',
      input: { ...baseInput, autoPotionTimer: 1, autoPotionTimerObtainium: 1 }
    },
    {
      name: 'offering timer crosses threshold (no fast mode)',
      // threshold at sing=10 mult=1 → 180 * 1.03^-10 ≈ 133.95
      input: { ...baseInput, autoPotionTimer: 135, autoPotionTimerObtainium: 50 }
    },
    {
      name: 'obtainium timer crosses threshold (no fast mode)',
      input: { ...baseInput, autoPotionTimer: 50, autoPotionTimerObtainium: 140 }
    },
    {
      name: 'both timers cross threshold simultaneously',
      input: { ...baseInput, autoPotionTimer: 200, autoPotionTimerObtainium: 200 }
    },
    {
      name: 'fast mode offering (toggle + count > 0)',
      input: {
        ...baseInput,
        toggleOffering: true,
        offeringPotionCount: 5,
        autoPotionTimer: 0.1,
        autoPotionTimerObtainium: 0
      }
    },
    {
      name: 'fast mode offering but count === 0 → fallback to slow threshold',
      input: {
        ...baseInput,
        toggleOffering: true,
        offeringPotionCount: 0,
        autoPotionTimer: 1
      }
    },
    {
      name: 'both fast modes active, both timers cross',
      input: {
        ...baseInput,
        toggleOffering: true,
        toggleObtainium: true,
        offeringPotionCount: 5,
        obtainiumPotionCount: 5,
        autoPotionTimer: 1,
        autoPotionTimerObtainium: 1
      }
    },
    {
      name: 'high singularity → very small threshold → many potions',
      input: {
        ...baseInput,
        highestSingularityCount: 200, // threshold ≈ 180 * 1.03^-200 ≈ 0.49
        autoPotionTimer: 10,
        autoPotionTimerObtainium: 10
      }
    },
    {
      name: 'autoPotionSpeedMult > 1 lowers threshold',
      input: {
        ...baseInput,
        autoPotionSpeedMult: 4,
        autoPotionTimer: 50,
        autoPotionTimerObtainium: 50
      }
    },
    {
      name: 'large time delta',
      input: { ...baseInput, time: 60, autoPotionTimer: 0, autoPotionTimerObtainium: 0 }
    },
    {
      name: 'timeMultiplier non-trivial (warp scenarios)',
      input: { ...baseInput, timeMultiplier: 5, time: 30 }
    },
    {
      name: 'exact threshold crossing — amount=1, timer resets to 0',
      // At sing=6, mult=1, threshold ≈ 180 * 1.03^-6 ≈ 150.78
      input: {
        ...baseInput,
        highestSingularityCount: 6,
        autoPotionTimer: 180 * Math.pow(1.03, -6),
        autoPotionTimerObtainium: 0
      }
    }
  ]
  for (const c of cases) {
    it(c.name, () => {
      const newR = newAdvanceAutoPotion(c.input)
      const oldR = oldAdvanceAutoPotion(c.input)
      expect(newR.autoPotionTimer).toBe(oldR.autoPotionTimer)
      expect(newR.autoPotionTimerObtainium).toBe(oldR.autoPotionTimerObtainium)
      expect(newR.events).toEqual(oldR.events)
    })
  }
})
