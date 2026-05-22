// Smallest integer step that changes the value of `x` under IEEE-754
// doubles. Below 2^53 every integer is exactly representable, so the step is
// 1; above it, doubles lose precision and the step grows as a power of two.
//
// Ported verbatim from packages/web_ui/src/Utility.ts (originally attributed
// to httpsnet) — the buyAccelerator / buyMultiplier loops use it to walk the
// `acceleratorBought` / `multiplierBought` counters past the safe-integer
// boundary without getting stuck on repeated identical floats.
export const smallestInc = (x = 0): number => {
  if (x <= Number.MAX_SAFE_INTEGER) {
    return 1
  } else {
    return 2 ** Math.floor(Math.log2(x) - 52)
  }
}
