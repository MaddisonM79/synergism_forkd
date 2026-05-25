// Sigmoid-style curve helpers lifted from packages/web_ui/src/Calculate.ts.
// Generic 1-line formulas that return 1 at zero progress and asymptote
// toward `constant` as progress grows. Used by cube/quark multiplier code
// in web_ui.

/**
 * Doubling-half sigmoid:
 *   1 + (constant - 1) * (1 - 2 ^ (-factor / divisor))
 * Returns 1 when factor = 0 and asymptotes to `constant` as factor → ∞.
 * `divisor` controls how quickly the curve saturates.
 */
export function calculateSigmoid(constant: number, factor: number, divisor: number): number {
  return 1 + (constant - 1) * (1 - Math.pow(2, -factor / divisor))
}

/**
 * Natural-exponential sigmoid:
 *   1 + (constant - 1) * (1 - e^(-coefficient))
 * Same shape as `calculateSigmoid` but uses `e` as the base — `coefficient`
 * is the natural-log progress rather than a halving count.
 */
export function calculateSigmoidExponential(constant: number, coefficient: number): number {
  return 1 + (constant - 1) * (1 - Math.exp(-coefficient))
}
