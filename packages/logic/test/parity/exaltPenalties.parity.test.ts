// Parity tests for the Exalt 3 / 4 / 6 penalty math lifted from
// packages/web_ui/src/Calculate.ts. The `oldXxx` definitions transcribe the
// pre-migration bodies verbatim — including the private
// `oldExalt6PenaltyPerMinute` helper that was internal to Calculate.ts.

import { describe, expect, it } from 'vitest'
import {
  calculateExalt3AscensionLimit as newExalt3Limit,
  calculateExalt3Penalty as newExalt3Penalty,
  calculateExalt4EffectiveSingularityMultiplier as newExalt4Mult,
  calculateExalt6Penalty as newExalt6Penalty,
  calculateExalt6PenaltyPerSecond as newExalt6PerSecond,
  calculateExalt6TimeLimit as newExalt6TimeLimit
} from '../../src/mechanics/exaltPenalties'

// ─── Old implementations (verbatim from packages/web_ui/src/Calculate.ts) ───

const oldExalt3AscensionLimit = (comps: number): number => Math.max(15 - comps * 2, 0)

const oldExalt3Penalty = (
  limitedAscensionsEnabled: boolean,
  limitedAscensionsCompletions: number,
  ascensionCount: number
): number => {
  const ascensionLimit = oldExalt3AscensionLimit(limitedAscensionsCompletions)
  const ascensions = ascensionCount
  if (!limitedAscensionsEnabled) {
    return 1
  } else {
    return Math.pow(2, Math.max(ascensions - ascensionLimit, 0))
  }
}

const oldExalt4Mult = (comps: number, force: boolean, inExalt4: boolean): number => {
  return inExalt4 || force ? Math.pow(comps + 1, 3) : 1
}

const oldExalt6TimeLimit = (comps: number): number => {
  if (comps >= 10) {
    return 115 - 5 * (comps - 10)
  } else {
    return 600 - 60 * comps
  }
}

const oldExalt6PenaltyPerMinute = (comps: number): number => {
  let penaltyPerMinute = 10 + 3 * comps
  if (comps >= 10) {
    penaltyPerMinute = 60 + 10 * (comps - 10)
  }
  return penaltyPerMinute
}

const oldExalt6PerSecond = (comps: number): number => {
  const penaltyPerMinute = oldExalt6PenaltyPerMinute(comps)
  return Math.pow(penaltyPerMinute, 1 / 60)
}

const oldExalt6Penalty = (comps: number, time: number): number => {
  const timeLimit = oldExalt6TimeLimit(comps)
  const displacedTime = Math.max(0, time - timeLimit)
  if (displacedTime === 0) {
    return 1
  } else {
    const penaltyPerSecond = oldExalt6PerSecond(comps)
    return Math.pow(penaltyPerSecond, -displacedTime)
  }
}

const closeEnough = (a: number, b: number, rel = 1e-12): boolean => {
  if (a === b) return true
  if (Math.abs(a) < 1 && Math.abs(b) < 1) return Math.abs(a - b) < rel
  return Math.abs(a - b) / Math.max(Math.abs(a), Math.abs(b)) < rel
}

// Sweeps comps across the 10-comp switching boundary in both directions.
const compsGrid = [0, 1, 2, 5, 7, 9, 10, 11, 15, 20, 50]

// ─── Tests ─────────────────────────────────────────────────────────────────

describe('parity: calculateExalt3AscensionLimit', () => {
  it.each(compsGrid)('comps=%i', (comps) => {
    expect(newExalt3Limit(comps)).toBe(oldExalt3AscensionLimit(comps))
  })
})

describe('parity: calculateExalt3Penalty', () => {
  const enabledGrid = [true, false]
  const compsLocal = [0, 1, 5, 7, 10, 20]
  const ascensionGrid = [0, 1, 5, 10, 15, 30, 100]
  for (const enabled of enabledGrid) {
    for (const comps of compsLocal) {
      it.each(ascensionGrid)(`enabled=${enabled} comps=${comps} ascensions=%i`, (ascensions) => {
        const next = newExalt3Penalty({
          limitedAscensionsEnabled: enabled,
          limitedAscensionsCompletions: comps,
          ascensionCount: ascensions
        })
        const old = oldExalt3Penalty(enabled, comps, ascensions)
        expect(next).toBe(old)
      })
    }
  }
})

describe('parity: calculateExalt4EffectiveSingularityMultiplier', () => {
  const forceGrid = [true, false]
  const exalt4Grid = [true, false]
  for (const force of forceGrid) {
    for (const inExalt4 of exalt4Grid) {
      it.each(compsGrid)(`force=${force} inExalt4=${inExalt4} comps=%i`, (comps) => {
        const next = newExalt4Mult({ comps, force, inExalt4 })
        const old = oldExalt4Mult(comps, force, inExalt4)
        expect(next).toBe(old)
      })
    }
  }
})

describe('parity: calculateExalt6TimeLimit', () => {
  it.each(compsGrid)('comps=%i', (comps) => {
    expect(newExalt6TimeLimit(comps)).toBe(oldExalt6TimeLimit(comps))
  })
})

describe('parity: calculateExalt6PenaltyPerSecond', () => {
  it.each(compsGrid)('comps=%i', (comps) => {
    expect(closeEnough(newExalt6PerSecond(comps), oldExalt6PerSecond(comps))).toBe(true)
  })
})

describe('parity: calculateExalt6Penalty', () => {
  // Sweep time around the timeLimit boundary for each comps value.
  for (const comps of compsGrid) {
    const limit = oldExalt6TimeLimit(comps)
    const timeGrid = [0, limit - 10, limit - 1, limit, limit + 1, limit + 30, limit + 300]
    it.each(timeGrid)(`comps=${comps} time=%i`, (time) => {
      const next = newExalt6Penalty(comps, time)
      const old = oldExalt6Penalty(comps, time)
      expect(closeEnough(next, old)).toBe(true)
    })
  }
})
