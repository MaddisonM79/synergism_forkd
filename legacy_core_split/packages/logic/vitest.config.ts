import { defineConfig } from 'vitest/config'

// packages/logic must remain DOM-free, so its tests run in pure Node — no
// jsdom, no window, no document. If a test ever needs those, the function
// under test belongs in packages/web_ui, not here.
//
// `test.include` matches only *.test.ts so .bench.ts files are excluded
// from regular runs; `benchmark.include` matches only *.bench.ts so the
// bench command stays scoped to performance work. Budget assertion tests
// live as ordinary *.budget.test.ts files inside the regular suite —
// they're correctness tests that happen to measure time.
export default defineConfig({
  test: {
    environment: 'node',
    include: ['test/**/*.test.ts'],
    testTimeout: 30_000,
    passWithNoTests: true
  },
  benchmark: {
    include: ['test/**/*.bench.ts']
  }
})
