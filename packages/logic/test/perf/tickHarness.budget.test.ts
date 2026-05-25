// Budget-assertion tests for the tackBody composition.
//
// Companion to test/perf/tickHarness.bench.ts. The bench file produces
// tracking numbers; this file fails CI if a fixture's full 1000-tick run
// exceeds a budget. Budgets are set with ~8x headroom over the local
// baselines so they catch catastrophic regressions (≥10x slowdown) but
// don't flap on slower CI runners or GC noise.
//
// Methodology: 3 warmup runs, then median of 7 timed runs. Median
// (rather than min or mean) absorbs GC pauses without rewarding cherry-
// picked best cases.
//
// Baselines were measured on the author's machine (Apple Silicon,
// 2026-05-24); update budgets if hardware assumptions shift, but
// prefer fixing the regression first.

import { describe, expect, it } from 'vitest'
import { BODY_FIXTURES, runFixtureWithTackBody } from '../fixtures/tackBodyFixtures'

// Budget per fixture (ms for the full N=1000-tick run).
const BUDGETS_MS: Record<string, number> = {
  'D — quiet early game (counters tick, recompute every tick)': 15,
  'E — active mid-game (all bundles emitting cross-tick events)': 20,
  'F — timeWarp on (tail-only, head+middle skipped)': 15
}

const WARMUP_RUNS = 3
const TIMED_RUNS = 7

const median = (xs: readonly number[]): number => {
  const sorted = [...xs].sort((a, b) => a - b)
  return sorted[Math.floor(sorted.length / 2)]
}

describe('tackBody budget — median of 7 runs after 3 warmup runs', () => {
  for (const f of BODY_FIXTURES) {
    const budget = BUDGETS_MS[f.name]
    if (budget === undefined) {
      throw new Error(`tickHarness.budget: no budget configured for fixture "${f.name}"`)
    }

    it(`${f.name} — median 1000-tick run < ${budget}ms`, () => {
      for (let i = 0; i < WARMUP_RUNS; i++) {
        runFixtureWithTackBody(f)
      }

      const samples: number[] = []
      for (let i = 0; i < TIMED_RUNS; i++) {
        const t0 = performance.now()
        runFixtureWithTackBody(f)
        samples.push(performance.now() - t0)
      }

      const med = median(samples)
      expect(med, `median of ${TIMED_RUNS} runs (samples ms: ${samples.map((s) => s.toFixed(2)).join(', ')})`)
        .toBeLessThan(budget)
    })
  }
})
