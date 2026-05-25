// Side-by-side bignum bench: break_infinity.js vs break_eternity.js.
//
// The two libraries share a near-identical surface (both by Patashu) but
// trade speed for range:
//   - break_infinity: faster, handles up to ~1e(9e15).
//   - break_eternity: ~2-5x slower per op, handles up to 10^^1e308 via a
//                     layered tetration representation.
//
// This bench compares the hot-path ops on both at four magnitude tiers:
//   - small      (1e3)         — fits in a JS Number
//   - medium     (1e100)       — Number overflows, both libs use layered repr
//   - large      (1e300)       — near Number.MAX_VALUE; well within both
//   - very_large (1e1e6 / etc) — magnitude-of-magnitude territory
//
// A final describe block exercises ops at break_eternity's tetration tier
// (10^^100) — break_infinity can't represent these at all.
//
// To run:    `npm --workspace @synergism/logic run bench -- bignum`
// Filter:    add `-- "lib=infinity"` style filters to narrow if needed.

import infinity from 'break_infinity.js'
import eternity from 'break_eternity.js'
import { bench, describe } from 'vitest'

type InfinityDecimal = InstanceType<typeof infinity>
type EternityDecimal = InstanceType<typeof eternity>

// ─── Magnitude tiers ───────────────────────────────────────────────────
// `power` is the decimal exponent — both libs accept fromString('1e<n>').

const TIERS = [
  { name: 'small      (1e3)', a: '1e3', b: '7e2' },
  { name: 'medium     (1e100)', a: '1e100', b: '3.14e95' },
  { name: 'large      (1e300)', a: '1e300', b: '5.5e290' },
  { name: 'very_large (1e1e6)', a: '1e1000000', b: '7e999000' }
] as const

// Pre-build operand instances per (lib, tier). Bench bodies should
// measure the op itself, not construction overhead.
interface Operands<T> {
  a: T
  b: T
}

const buildInfinity = (s: string): InfinityDecimal => new infinity(s)
const buildEternity = (s: string): EternityDecimal => new eternity(s)

const infinityOps: Record<string, Operands<InfinityDecimal>> = Object.fromEntries(
  TIERS.map((t) => [t.name, { a: buildInfinity(t.a), b: buildInfinity(t.b) }])
)
const eternityOps: Record<string, Operands<EternityDecimal>> = Object.fromEntries(
  TIERS.map((t) => [t.name, { a: buildEternity(t.a), b: buildEternity(t.b) }])
)

// ═════════════════════════════════════════════════════════════════════════
// Construction
// ═════════════════════════════════════════════════════════════════════════

// `_sink` reads results so V8 can't dead-code-eliminate the construction.
// Updated each iteration but never observed outside the bench loop.
let _sink: unknown = null

describe('construct from string', () => {
  for (const t of TIERS) {
    bench(`infinity · ${t.name}`, () => {
      _sink = new infinity(t.a)
    })
    bench(`eternity · ${t.name}`, () => {
      _sink = new eternity(t.a)
    })
  }
})

describe('construct from number', () => {
  // Use Number.MAX_SAFE_INTEGER-fitting numbers; very_large skipped here
  // since 1e1000000 isn't a finite Number.
  const NUMERIC_TIERS = [
    { name: 'small  (1000)', v: 1000 },
    { name: 'medium (1e100)', v: 1e100 },
    { name: 'large  (1e300)', v: 1e300 }
  ] as const
  for (const t of NUMERIC_TIERS) {
    bench(`infinity · ${t.name}`, () => {
      _sink = new infinity(t.v)
    })
    bench(`eternity · ${t.name}`, () => {
      _sink = new eternity(t.v)
    })
  }
})

// ═════════════════════════════════════════════════════════════════════════
// Arithmetic
// ═════════════════════════════════════════════════════════════════════════

describe('add', () => {
  for (const t of TIERS) {
    const ops = { i: infinityOps[t.name], e: eternityOps[t.name] }
    bench(`infinity · ${t.name}`, () => {
      ops.i.a.add(ops.i.b)
    })
    bench(`eternity · ${t.name}`, () => {
      ops.e.a.add(ops.e.b)
    })
  }
})

describe('mul', () => {
  for (const t of TIERS) {
    const ops = { i: infinityOps[t.name], e: eternityOps[t.name] }
    bench(`infinity · ${t.name}`, () => {
      ops.i.a.mul(ops.i.b)
    })
    bench(`eternity · ${t.name}`, () => {
      ops.e.a.mul(ops.e.b)
    })
  }
})

describe('div', () => {
  for (const t of TIERS) {
    const ops = { i: infinityOps[t.name], e: eternityOps[t.name] }
    bench(`infinity · ${t.name}`, () => {
      ops.i.a.div(ops.i.b)
    })
    bench(`eternity · ${t.name}`, () => {
      ops.e.a.div(ops.e.b)
    })
  }
})

describe('pow (exponent = 2.5)', () => {
  for (const t of TIERS) {
    const ops = { i: infinityOps[t.name], e: eternityOps[t.name] }
    bench(`infinity · ${t.name}`, () => {
      ops.i.a.pow(2.5)
    })
    bench(`eternity · ${t.name}`, () => {
      ops.e.a.pow(2.5)
    })
  }
})

// ═════════════════════════════════════════════════════════════════════════
// Transcendentals
// ═════════════════════════════════════════════════════════════════════════

describe('log10', () => {
  for (const t of TIERS) {
    const ops = { i: infinityOps[t.name], e: eternityOps[t.name] }
    bench(`infinity · ${t.name}`, () => {
      ops.i.a.log10()
    })
    bench(`eternity · ${t.name}`, () => {
      ops.e.a.log10()
    })
  }
})

// exp on large values overflows even break_infinity; restrict to small/medium.
describe('exp (small/medium only — overflows otherwise)', () => {
  const SAFE_TIERS = TIERS.slice(0, 2)
  for (const t of SAFE_TIERS) {
    const ops = { i: infinityOps[t.name], e: eternityOps[t.name] }
    bench(`infinity · ${t.name}`, () => {
      ops.i.a.exp()
    })
    bench(`eternity · ${t.name}`, () => {
      ops.e.a.exp()
    })
  }
})

// ═════════════════════════════════════════════════════════════════════════
// Comparison
// ═════════════════════════════════════════════════════════════════════════

describe('eq', () => {
  for (const t of TIERS) {
    const ops = { i: infinityOps[t.name], e: eternityOps[t.name] }
    bench(`infinity · ${t.name}`, () => {
      ops.i.a.eq(ops.i.b)
    })
    bench(`eternity · ${t.name}`, () => {
      ops.e.a.eq(ops.e.b)
    })
  }
})

describe('gt', () => {
  for (const t of TIERS) {
    const ops = { i: infinityOps[t.name], e: eternityOps[t.name] }
    bench(`infinity · ${t.name}`, () => {
      ops.i.a.gt(ops.i.b)
    })
    bench(`eternity · ${t.name}`, () => {
      ops.e.a.gt(ops.e.b)
    })
  }
})

// ═════════════════════════════════════════════════════════════════════════
// Serialization
// ═════════════════════════════════════════════════════════════════════════

describe('toString', () => {
  for (const t of TIERS) {
    const ops = { i: infinityOps[t.name], e: eternityOps[t.name] }
    bench(`infinity · ${t.name}`, () => {
      ops.i.a.toString()
    })
    bench(`eternity · ${t.name}`, () => {
      ops.e.a.toString()
    })
  }
})

// ═════════════════════════════════════════════════════════════════════════
// break_eternity only — tetration tier
// ═════════════════════════════════════════════════════════════════════════
// 10^^100 (a power tower of 100 tens) is well past anything
// break_infinity can represent. These benches show what eternity costs
// at its native magnitude tier.

describe('break_eternity only — tetration tier (10^^100)', () => {
  const tetA = eternity.tetrate(10, 100)
  const tetB = eternity.tetrate(10, 99)

  bench('eternity · add', () => {
    tetA.add(tetB)
  })
  bench('eternity · mul', () => {
    tetA.mul(tetB)
  })
  bench('eternity · log10', () => {
    tetA.log10()
  })
  bench('eternity · gt', () => {
    tetA.gt(tetB)
  })
  bench('eternity · toString', () => {
    tetA.toString()
  })
})
