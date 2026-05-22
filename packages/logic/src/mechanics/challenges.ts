// Challenge math. The full Challenges.ts in web_ui is mostly UI + automation
// state — but a handful of helpers are pure number-in-number-out functions
// that get called from many other modules (Buy.ts, Runes.ts, Statistics, ...).
// Migrate those one at a time; the UI + sweep state machine stays in web_ui.

/**
 * Effective Challenge Completions. Three piecewise linear curves keyed by
 * challenge tier, each diminishing returns past the first knee:
 *
 *   transcend:      [0..100] 1×, [100..1000] 0.05×, past 1000 0.01×
 *   reincarnation:  [0..25]  1×, [25..75]   0.5×,   past 75   0.1×
 *   ascension:      [0..10]  1×, past 10   0.5×
 *
 * Pure: depends only on its two arguments. Used everywhere — the cost
 * formulas in producers/accelerators/multipliers all read transcendECC for
 * their challenge-4 amplifier, and the rune EXP curve uses it too.
 */
export function CalcECC(
  type: 'transcend' | 'reincarnation' | 'ascension',
  completions: number
): number {
  let effective = 0
  switch (type) {
    case 'transcend':
      effective += Math.min(100, completions)
      effective += 1 / 20 * (Math.min(1000, Math.max(100, completions)) - 100)
      effective += 1 / 100 * (Math.max(1000, completions) - 1000)
      return effective
    case 'reincarnation':
      effective += Math.min(25, completions)
      effective += 1 / 2 * (Math.min(75, Math.max(25, completions)) - 25)
      effective += 1 / 10 * (Math.max(75, completions) - 75)
      return effective
    case 'ascension':
      effective += Math.min(10, completions)
      effective += 1 / 2 * (Math.max(10, completions) - 10)
      return effective
    default: {
      throw new Error(`Unhandled challenge type: ${type satisfies never}`)
    }
  }
}
