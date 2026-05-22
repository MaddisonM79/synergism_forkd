import type { Decimal } from '../math/bignum'

// Discriminated union of events the logic core emits for the UI tier to
// react to. UI subscribers translate these into user-facing effects
// (notifications, sounds, animations, achievement awards, etc.).
//
// The union grows as mechanics migrate. Each event should carry enough data
// for the UI to render its effect without re-reading GameState — that keeps
// the boundary one-way.
export type CoreEvent =
  | {
      kind: 'accelerators-purchased'
      before: number
      after: number
      spent: Decimal
    }
  | {
      kind: 'multipliers-purchased'
      before: number
      after: number
      spent: Decimal
    }
  | {
      kind: 'particle-buildings-purchased'
      /** Which of the five particle buildings was purchased (1 = first, 5 = fifth). */
      index: 1 | 2 | 3 | 4 | 5
      before: number
      after: number
      spent: Decimal
    }
