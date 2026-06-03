// Golden-vector generator for the math-function parity harness.
//
// Imports the dependency-free pure-math functions straight from the
// frozen legacy TS (node 24 strips the TS types on import — no build or
// `npm install` needed) and emits `fixtures/parity_math.json`: the
// canonical TS outputs that the Rust port is checked against in
// `tests/parity.rs`.
//
// Re-run after curating inputs or when the legacy formulas change:
//   node crates/synergismforkd_testkit/parity/generate_math_vectors.mjs
//
// Only deterministic (RNG-free) functions belong here — the Rust RNG is
// Xoshiro per-purpose and intentionally diverges from the legacy
// MersenneTwister, so RNG-driven outputs can't be bit-compared.

import { mkdirSync, writeFileSync } from 'node:fs'

const LOGIC = '../../../legacy/core_split/packages/logic/src'
const { calculateSummationCubic, solveQuadratic } = await import(`${LOGIC}/math/summations.ts`)
const { calculateSigmoid, calculateSigmoidExponential } = await import(`${LOGIC}/math/sigmoid.ts`)
const { smallestInc } = await import(`${LOGIC}/math/smallestInc.ts`)

// ── Curated input sets (typical values + edge cases) ───────────────────
const cubicInputs = [0, 1, 2, 5, 10, 50, 100, 1000, 12345]
const quadInputs = [
  [1, -3, 2, true],
  [1, -3, 2, false],
  [1, 5, 6, true],
  [2, -7, 3, true],
  [1, 0, -4, true],
  [1, -1000, 1, true],
  [3, -12, 9, false]
]
const sigmoidInputs = [
  [2, 5, 10],
  [1, 0, 1],
  [3, 100, 50],
  [1.5, 50, 1000],
  [2, 5, 10000],
  [4, 250, 125]
]
const sigExpInputs = [
  [3, 0.5],
  [1, 0],
  [10, 0.01],
  [2, 2],
  [100, 0.001],
  [50, 0.1]
]
const smallestIncInputs = [0, 1, 100, 1e10, 4.5e15, 9e15, 1e16, 1e18]

const data = {
  calculate_summation_cubic: cubicInputs.map((n) => ({ n, result: calculateSummationCubic(n) })),
  solve_quadratic: quadInputs.map(([a, b, c, positive]) => ({
    a,
    b,
    c,
    positive,
    result: solveQuadratic(a, b, c, positive)
  })),
  calculate_sigmoid: sigmoidInputs.map(([constant, factor, divisor]) => ({
    constant,
    factor,
    divisor,
    result: calculateSigmoid(constant, factor, divisor)
  })),
  calculate_sigmoid_exponential: sigExpInputs.map(([constant, coefficient]) => ({
    constant,
    coefficient,
    result: calculateSigmoidExponential(constant, coefficient)
  })),
  smallest_inc: smallestIncInputs.map((x) => ({ x, result: smallestInc(x) }))
}

// Guard: a non-finite result can't round-trip through JSON (becomes
// `null`) and signals a bad input choice — fail loudly instead.
for (const [fn, cases] of Object.entries(data)) {
  for (const c of cases) {
    if (!Number.isFinite(c.result)) {
      throw new Error(`non-finite result for ${fn}: ${JSON.stringify(c)}`)
    }
  }
}

const outDir = new URL('../fixtures/', import.meta.url)
mkdirSync(outDir, { recursive: true })
writeFileSync(new URL('parity_math.json', outDir), `${JSON.stringify(data, null, 2)}\n`)

const total = Object.values(data).reduce((sum, cases) => sum + cases.length, 0)
console.log(`wrote fixtures/parity_math.json — ${total} vectors across ${Object.keys(data).length} functions`)
