// Single chokepoint over the big-number library.
//
// Importing Decimal from here (not from `break_infinity.js` directly) means
// that swapping the implementation — for example, when the Rust port lands a
// `num-bigfloat`-backed replacement compiled to WASM — only requires editing
// this file.
export { default as Decimal } from 'break_infinity.js'
export type { DecimalSource } from 'break_infinity.js'
