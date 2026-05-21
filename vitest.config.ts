import { defineConfig } from 'vitest/config'

// Save-decode + schema modules pull in DOM-touching siblings at import time
// (DOMCacheGetOrSet, document.*, getElementById in Hepteracts.ts, Talismans.ts,
// singularity.ts, etc.), so the test runner needs a DOM. jsdom is the
// lightest fit — no real browser, no headless overhead.
export default defineConfig({
  test: {
    environment: 'jsdom',
    setupFiles: ['./test/setup.ts'],
    include: ['test/**/*.test.ts'],
    testTimeout: 60_000
  }
})
