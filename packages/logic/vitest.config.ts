import { defineConfig } from 'vitest/config'

// packages/logic must remain DOM-free, so its tests run in pure Node — no
// jsdom, no window, no document. If a test ever needs those, the function
// under test belongs in packages/web_ui, not here.
export default defineConfig({
  test: {
    environment: 'node',
    include: ['test/**/*.test.ts'],
    testTimeout: 30_000,
    passWithNoTests: true
  }
})
