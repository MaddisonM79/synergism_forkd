// Golden-vector generator for the Decimal-returning parity batch.
//
// Unlike the pure-math generator, these legacy mechanics import `Decimal`
// (`break_infinity.js`) and use extensionless internal imports, so this
// needs the deps installed AND a bundler-style resolver. Run with tsx:
//   (cd legacy/core_split && npm install --ignore-scripts)   # once
//   npx tsx crates/synergismforkd_testkit/parity/generate_decimal_vectors.mjs
//
// Decimals are serialized as { mantissa, exponent } (normalized m × 10^e)
// rather than a string, because break-eternity-rs (the Rust side) has no
// string parser yet — the Rust test reconstructs via
// `Decimal::from_mantissa_exponent`. Values are kept in the range where
// the legacy `break_infinity.js` and the Rust `break-eternity-rs` agree
// (normal scales; they diverge only in tetration territory).

import { mkdirSync, writeFileSync } from 'node:fs'

const LOGIC = '../../../legacy/core_split/packages/logic/src'
const { getCostAccelerator } = await import(`${LOGIC}/mechanics/accelerators.ts`)
const { getCostMultiplier } = await import(`${LOGIC}/mechanics/multipliers.ts`)

// [buying_to, cost_divisor, transcend_ecc, in_transcension_challenge_4, in_reincarnation_challenge_8]
const inputs = [
  [1, 1, 0, false, false],
  [10, 1, 0, false, false],
  [100, 1, 0, false, false],
  [1000, 1, 0, false, false],
  [1000, 10, 0, false, false],
  [1000, 1, 50, false, false],
  [500, 1, 0, true, false],
  [500, 1, 0, false, true],
  [5000, 5, 100, false, false]
]

const casesFor = (fn) =>
  inputs.map(([buying_to, cost_divisor, transcend_ecc, in_transcension_challenge_4, in_reincarnation_challenge_8]) => {
    const d = fn(buying_to, {
      costDivisor: cost_divisor,
      transcendECC: transcend_ecc,
      inTranscensionChallenge4: in_transcension_challenge_4,
      inReincarnationChallenge8: in_reincarnation_challenge_8
    })
    return {
      buying_to,
      cost_divisor,
      transcend_ecc,
      in_transcension_challenge_4,
      in_reincarnation_challenge_8,
      mantissa: d.mantissa,
      exponent: d.exponent
    }
  })

const data = {
  get_cost_accelerator: casesFor(getCostAccelerator),
  get_cost_multiplier: casesFor(getCostMultiplier)
}

for (const [fn, cases] of Object.entries(data)) {
  for (const c of cases) {
    if (!Number.isFinite(c.mantissa) || !Number.isFinite(c.exponent)) {
      throw new Error(`non-finite ${fn}: ${JSON.stringify(c)}`)
    }
  }
}

const outDir = new URL('../fixtures/', import.meta.url)
mkdirSync(outDir, { recursive: true })
writeFileSync(new URL('parity_decimal.json', outDir), `${JSON.stringify(data, null, 2)}\n`)

const total = Object.values(data).reduce((sum, cases) => sum + cases.length, 0)
console.log(`wrote fixtures/parity_decimal.json — ${total} Decimal vectors across ${Object.keys(data).length} functions`)
