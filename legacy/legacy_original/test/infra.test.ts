import { describe, expect, test } from 'vitest'

// Placeholder smoke test that verifies the vitest + jsdom + fast-check
// pipeline runs. The actual fuzz target for #143 — running fast-check
// against `playerUpdateVarSchema.safeParse` — is blocked on resolving the
// circular import cluster #76 (Synergism ⇄ Calculate ⇄ UpdateHTML ⇄
// everything). When the schema chain is loaded outside the live app's init
// order, leaves like Tabs.Buildings, AntSacrificeTiers.sacrifice etc.
// evaluate to undefined and the module-graph init crashes before any test
// can run. Whack-a-mole stubbing each one creates a brittle setup that
// breaks every time a new circular ref is added.
//
// Once #76 lands (or someone splits the schema definitions out of the
// DOM-touching modules), replace this placeholder with the real fuzz
// suite: assert that `playerUpdateVarSchema.safeParse(anything)` always
// returns `{ success: true | false }` and never throws.

describe('test infrastructure smoke test', () => {
  test('vitest + jsdom is wired up', () => {
    expect(typeof window).toBe('object')
    expect(typeof document).toBe('object')
    expect(typeof window.matchMedia).toBe('function')
  })

  test('fast-check is installed', async () => {
    const fc = await import('fast-check')
    expect(typeof fc.assert).toBe('function')
    expect(typeof fc.property).toBe('function')
  })
})
