// Vitest benchmark suite for the tackBody composition.
//
// Three benchmarks, one per fixture (D / E / F). Each iteration runs the
// full N=1000-tick fixture loop through the production `tackBody`. Vitest
// auto-iterates and reports hz (full-fixture runs per second) + mean /
// median / p99 timings.
//
// To run:    `npm --workspace @synergism/logic run bench`
// Filter:    `npm --workspace @synergism/logic run bench -- tackBody`
//
// These benches are scoped to the migrated logic bundle composition —
// they don't measure pre-tick (resourceGain / generateAntsAndCrumbs)
// since those still live in web_ui. Use the per-tick timing as a
// regression-tracking baseline: ms_per_tick ≈ (1 / hz) * 1000 / 1000.

import { bench, describe } from 'vitest'
import { BODY_FIXTURES, runFixtureWithTackBody } from '../fixtures/tackBodyFixtures'

describe('tackBody full-fixture (N=1000 ticks per run)', () => {
  for (const f of BODY_FIXTURES) {
    bench(f.name, () => {
      runFixtureWithTackBody(f)
    })
  }
})
