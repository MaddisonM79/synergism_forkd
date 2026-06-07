// Golden-vector generator for the `calculate.ts` COMBINE aggregators.
//
// The audit's central blind spot: the composition/aggregation layer is
// TS-anchored (the extracted package's own `*.parity.test.ts`) but NOT
// Rust-anchored. The existing harness only covers RNG-free *leaf* fns; this
// extends it to the pure-input aggregators in `mechanics/calculate.ts` that
// have a direct flat-input Rust counterpart in `mechanics/calculate.rs`:
//
//   - calculateGlobalSpeedMult   (the C1 global-speed multiplier — DR branches)
//   - calculateAscensionSpeedMult
//   - getReductionValue
//   - calculateOfferings         (Exalt-8 taxman cap)
//   - calculateObtainium         (c14 zero-out + illiteracy-DR exponent)
//
// These take flat input structs on BOTH sides, so the same input drives both
// and the outputs are compared directly — no GameState→input mapping needed.
// (The state-coupled assemblers — computeGlobalMultipliers' ~60-field input,
// the offering/obtainium StatLine assemblers, and the full `tack` snapshot —
// need a GameState fixture or a headless monolith oracle; that is the
// remaining P0.4 work.)
//
// Deps installed once via `cd legacy/core_split && npm install --ignore-scripts`.
// Run with tsx:
//   npx tsx crates/synergismforkd_testkit/parity/generate_aggregator_vectors.mjs
//
// Decimal-returning aggregators serialize via `Decimal.toString()`; the Rust
// test reconstructs with `Decimal::from_string` and compares in log10 space
// (values stay in the range where break_infinity.js and break-eternity-rs
// agree).

import { mkdirSync, writeFileSync } from 'node:fs'

const LOGIC = '../../../legacy/core_split/packages/logic/src'
const {
  calculateGlobalSpeedMult,
  calculateAscensionSpeedMult,
  getReductionValue,
  calculateOfferings,
  calculateObtainium
} = await import(`${LOGIC}/mechanics/calculate.ts`)
const { Decimal } = await import(`${LOGIC}/math/bignum.ts`)

const D = (x) => new Decimal(x)

// ── calculateGlobalSpeedMult ── branches: normalMult>100 (sqrt cap),
// normalMult<1 (drPower), else passthrough; then × immaculateMult.
const calculate_global_speed_mult = [
  { normal_mult: 1, immaculate_mult: 1, dr_power: 1 },
  { normal_mult: 50, immaculate_mult: 2, dr_power: 1 },
  { normal_mult: 400, immaculate_mult: 3, dr_power: 1 }, // >100 sqrt branch
  { normal_mult: 0.25, immaculate_mult: 5, dr_power: 0.9 }, // <1 DR branch
  { normal_mult: 0.5, immaculate_mult: 1, dr_power: 0.5 }, // <1 DR branch
  { normal_mult: 99.9, immaculate_mult: 1.5, dr_power: 1 }
].map((c) => ({
  ...c,
  result: calculateGlobalSpeedMult({
    normalMult: c.normal_mult,
    immaculateMult: c.immaculate_mult,
    drPower: c.dr_power
  })
}))

// ── calculateAscensionSpeedMult ── base^(1±spread) around 1.
const calculate_ascension_speed_mult = [
  { base: 1, exponent_spread: 0 },
  { base: 5, exponent_spread: 0.2 }, // base>=1 → base^(1+spread)
  { base: 0.5, exponent_spread: 0.2 }, // base<1 → base^(1-spread)
  { base: 100, exponent_spread: 0.5 },
  { base: 0.1, exponent_spread: 0.75 }
].map((c) => ({
  ...c,
  result: calculateAscensionSpeedMult({ base: c.base, exponentSpread: c.exponent_spread })
}))

// ── getReductionValue ── 1 + thrift + researches/200 + ECC(cc4)/200 + antScale.
const get_reduction_value = [
  { thrift_cost_delay: 0, researches_sum: 0, challenge_completions_4: 0, ant_building_cost_scale: 0 },
  {
    thrift_cost_delay: 0.5,
    researches_sum: 500,
    challenge_completions_4: 10,
    ant_building_cost_scale: 0.3
  },
  {
    thrift_cost_delay: 1,
    researches_sum: 1000,
    challenge_completions_4: 25,
    ant_building_cost_scale: 1
  }
].map((c) => ({
  ...c,
  result: getReductionValue({
    thriftCostDelay: c.thrift_cost_delay,
    researchesSum: c.researches_sum,
    challengeCompletions4: c.challenge_completions_4,
    antBuildingCostScale: c.ant_building_cost_scale
  })
}))

// ── calculateOfferings ── max(base, mult·time), Exalt-8 taxman cap when
// completions >= 2.
const calculate_offerings = [
  {
    base_offerings: 1,
    time_multiplier: 0,
    offering_mult: '1',
    taxman_last_stand_enabled: false,
    taxman_last_stand_completions: 0,
    current_offerings: '0'
  },
  {
    base_offerings: 12,
    time_multiplier: 5,
    offering_mult: '1000',
    taxman_last_stand_enabled: false,
    taxman_last_stand_completions: 0,
    current_offerings: '0'
  },
  {
    base_offerings: 1,
    time_multiplier: 2,
    offering_mult: '1e50',
    taxman_last_stand_enabled: false,
    taxman_last_stand_completions: 0,
    current_offerings: '0'
  },
  {
    // taxman cap active: min(5*100+1, max(1, 1e20*10)) = 501.
    base_offerings: 1,
    time_multiplier: 10,
    offering_mult: '1e20',
    taxman_last_stand_enabled: true,
    taxman_last_stand_completions: 2,
    current_offerings: '5'
  }
].map((c) => ({
  ...c,
  value: calculateOfferings({
    baseOfferings: c.base_offerings,
    timeMultiplier: c.time_multiplier,
    offeringMult: D(c.offering_mult),
    taxmanLastStandEnabled: c.taxman_last_stand_enabled,
    taxmanLastStandCompletions: c.taxman_last_stand_completions,
    currentOfferings: D(c.current_offerings)
  }).toString()
}))

// ── calculateObtainium ── immaculate·baseMults^DR·time, floored by base;
// c14 zeroes; Exalt-8 taxman cap.
const calculate_obtainium = [
  {
    base_obtainium: 1,
    immaculate: 1,
    dr: 1,
    time_multiplier: 1,
    base_mults: '1',
    in_ascension_challenge_14: false,
    taxman_last_stand_enabled: false,
    taxman_last_stand_completions: 0,
    current_obtainium: '0'
  },
  {
    base_obtainium: 5,
    immaculate: 2,
    dr: 1,
    time_multiplier: 3,
    base_mults: '1e10',
    in_ascension_challenge_14: false,
    taxman_last_stand_enabled: false,
    taxman_last_stand_completions: 0,
    current_obtainium: '0'
  },
  {
    // illiteracy DR exponent < 1 damps the headline product.
    base_obtainium: 1,
    immaculate: 1,
    dr: 0.5,
    time_multiplier: 1,
    base_mults: '1e40',
    in_ascension_challenge_14: false,
    taxman_last_stand_enabled: false,
    taxman_last_stand_completions: 0,
    current_obtainium: '0'
  },
  {
    // c14 zero-out → 0.
    base_obtainium: 100,
    immaculate: 5,
    dr: 1,
    time_multiplier: 2,
    base_mults: '1e20',
    in_ascension_challenge_14: true,
    taxman_last_stand_enabled: false,
    taxman_last_stand_completions: 0,
    current_obtainium: '0'
  }
].map((c) => ({
  ...c,
  value: calculateObtainium({
    baseObtainium: c.base_obtainium,
    immaculate: c.immaculate,
    DR: c.dr,
    timeMultiplier: c.time_multiplier,
    baseMults: D(c.base_mults),
    inAscensionChallenge14: c.in_ascension_challenge_14,
    taxmanLastStandEnabled: c.taxman_last_stand_enabled,
    taxmanLastStandCompletions: c.taxman_last_stand_completions,
    currentObtainium: D(c.current_obtainium)
  }).toString()
}))

const data = {
  calculate_global_speed_mult,
  calculate_ascension_speed_mult,
  get_reduction_value,
  calculate_offerings,
  calculate_obtainium
}

const outDir = new URL('../fixtures/', import.meta.url)
mkdirSync(outDir, { recursive: true })
writeFileSync(new URL('parity_aggregators.json', outDir), `${JSON.stringify(data, null, 2)}\n`)

const total = Object.values(data).reduce((sum, cases) => sum + cases.length, 0)
console.log(
  `wrote fixtures/parity_aggregators.json — ${total} aggregator vectors across ${
    Object.keys(data).length
  } functions`
)
