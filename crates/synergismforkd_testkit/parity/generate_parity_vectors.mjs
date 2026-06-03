// Golden-vector generator for the TS↔Rust parity harness.
//
// Imports the dependency-free pure functions straight from the frozen
// legacy TS (node 24 strips the TS types on import — no build or
// `npm install` needed) and emits `fixtures/parity_vectors.json`: the
// canonical TS outputs the Rust port is checked against in `src/parity.rs`.
//
// Re-run after curating inputs or when the legacy formulas change:
//   node crates/synergismforkd_testkit/parity/generate_parity_vectors.mjs
//
// Only deterministic (RNG-free), dependency-free functions belong here.
// (The Rust RNG diverges from the legacy MersenneTwister by design, and
// Decimal-typed mechanics need `break_infinity.js` installed — a separate
// fixture once that's wired.)

import { mkdirSync, writeFileSync } from 'node:fs'

const LOGIC = '../../../legacy/core_split/packages/logic/src'
const { calculateSummationCubic, solveQuadratic, calculateCubicSumData, calculateSummationNonLinear } =
  await import(`${LOGIC}/math/summations.ts`)
const { calculateSigmoid, calculateSigmoidExponential } = await import(`${LOGIC}/math/sigmoid.ts`)
const { smallestInc } = await import(`${LOGIC}/math/smallestInc.ts`)
const sing = await import(`${LOGIC}/mechanics/singularityMilestones.ts`)

// ── Math: scalar functions ─────────────────────────────────────────────
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
const sigmoidInputs = [[2, 5, 10], [1, 0, 1], [3, 100, 50], [1.5, 50, 1000], [2, 5, 10000], [4, 250, 125]]
const sigExpInputs = [[3, 0.5], [1, 0], [10, 0.01], [2, 2], [100, 0.001], [50, 0.1]]
const smallestIncInputs = [0, 1, 100, 1e10, 4.5e15, 9e15, 1e16, 1e18]

// ── Math: struct-returning ({ levelCanBuy, cost }) ─────────────────────
const cubicSumInputs = [
  [0, 100, 1e6, 50],
  [5, 100, 1e6, 50],
  [10, 50, 1e9, 100],
  [0, 1000, 1e4, 20],
  [50, 100, 1e6, 50] // initial >= max → { max, 0 }
]
const nonLinearInputs = [
  [5, 100, 1e6, 0.2, 1000],
  [0, 100, 1e6, 0, 500], // diffPerLevel 0 → c == 0 branch
  [10, 50, 1e9, 0.5, 1e4],
  [0, 1000, 1e4, 0.1, 100],
  [100, 100, 1e8, 0.3, 1000]
]

// ── Singularity milestones (pure f64) ──────────────────────────────────
const singCountInputs = [0, 10, 50, 100, 200, 270]
const highSingInputs = [0, 30, 100, 120, 162, 200, 205, 230, 256, 300]
const baseGqInputs = [
  [0, 0, 0],
  [100, 1e6, 50],
  [5, 1e5, 5],
  [200, 1e9, 250]
]
const exaltInputs = [[], [1, 2, 3], [1, 2, 3, 4, 5, 6], [10, 10, 10]]

const data = {
  // math — scalar
  calculate_summation_cubic: cubicInputs.map((n) => ({ n, result: calculateSummationCubic(n) })),
  solve_quadratic: quadInputs.map(([a, b, c, positive]) => ({ a, b, c, positive, result: solveQuadratic(a, b, c, positive) })),
  calculate_sigmoid: sigmoidInputs.map(([constant, factor, divisor]) => ({ constant, factor, divisor, result: calculateSigmoid(constant, factor, divisor) })),
  calculate_sigmoid_exponential: sigExpInputs.map(([constant, coefficient]) => ({ constant, coefficient, result: calculateSigmoidExponential(constant, coefficient) })),
  smallest_inc: smallestIncInputs.map((x) => ({ x, result: smallestInc(x) })),
  // math — struct
  calculate_cubic_sum_data: cubicSumInputs.map(([initial_level, base_cost, amount_to_spend, max_level]) => {
    const r = calculateCubicSumData(initial_level, base_cost, amount_to_spend, max_level)
    return { initial_level, base_cost, amount_to_spend, max_level, level_can_buy: r.levelCanBuy, cost: r.cost }
  }),
  calculate_summation_non_linear: nonLinearInputs.map(([base_level, base_cost, resource_available, diff_per_level, buy_amount]) => {
    const r = calculateSummationNonLinear(base_level, base_cost, resource_available, diff_per_level, buy_amount)
    return { base_level, base_cost, resource_available, diff_per_level, buy_amount, level_can_buy: r.levelCanBuy, cost: r.cost }
  }),
  // singularity milestones
  calculate_singularity_quark_milestone_multiplier: singCountInputs.map((singularity_count) => ({ singularity_count, result: sing.calculateSingularityQuarkMilestoneMultiplier(singularity_count) })),
  calculate_base_golden_quarks: baseGqInputs.map(([singularity, quarks_this_singularity, highest_singularity_count]) => ({
    singularity,
    quarks_this_singularity,
    highest_singularity_count,
    result: sing.calculateBaseGoldenQuarks({ singularity, quarksThisSingularity: quarks_this_singularity, highestSingularityCount: highest_singularity_count })
  })),
  calculate_singularity_ambrosia_luck_milestone_bonus: highSingInputs.map((highest_singularity_count) => ({ highest_singularity_count, result: sing.calculateSingularityAmbrosiaLuckMilestoneBonus(highest_singularity_count) })),
  calculate_dilated_five_leaf_bonus: highSingInputs.map((highest_singularity_count) => ({ highest_singularity_count, result: sing.calculateDilatedFiveLeafBonus(highest_singularity_count) })),
  derpsmith_cornucopia_bonus: highSingInputs.map((highest_singularity_count) => ({ highest_singularity_count, result: sing.derpsmithCornucopiaBonus(highest_singularity_count) })),
  calculate_immaculate_alchemy_bonus: singCountInputs.map((singularity_count) => ({ singularity_count, result: sing.calculateImmaculateAlchemyBonus(singularity_count) })),
  inheritance_tokens: highSingInputs.map((highest_singularity_count) => ({ highest_singularity_count, result: sing.inheritanceTokens(highest_singularity_count) })),
  sum_of_exalt_completions: exaltInputs.map((completions) => ({ completions, result: sing.sumOfExaltCompletions(completions) })),
  singularity_bonus_token_mult: highSingInputs.map((highest_singularity_count) => ({ highest_singularity_count, result: sing.singularityBonusTokenMult(highest_singularity_count) }))
}

// Guard: a non-finite number can't round-trip through JSON (becomes
// `null`) and signals a bad input choice — fail loudly. Checks every
// numeric field (scalars, struct fields, and array elements).
const isFiniteValue = (v) =>
  typeof v === 'number' ? Number.isFinite(v) : Array.isArray(v) ? v.every((e) => typeof e !== 'number' || Number.isFinite(e)) : true
for (const [fn, cases] of Object.entries(data)) {
  for (const c of cases) {
    for (const [k, v] of Object.entries(c)) {
      if (!isFiniteValue(v)) throw new Error(`non-finite ${fn}.${k}: ${JSON.stringify(c)}`)
    }
  }
}

const outDir = new URL('../fixtures/', import.meta.url)
mkdirSync(outDir, { recursive: true })
writeFileSync(new URL('parity_vectors.json', outDir), `${JSON.stringify(data, null, 2)}\n`)

const total = Object.values(data).reduce((sum, cases) => sum + cases.length, 0)
console.log(`wrote fixtures/parity_vectors.json — ${total} vectors across ${Object.keys(data).length} functions`)
