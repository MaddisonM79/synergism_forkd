// Event-buff selectors. Lifted from packages/web_ui/src/Event.ts.
//
// Two contributions stack additively per buff: the active GameEvent (if
// any) and the consumable Happy-Hour-Bell stack (scaled by queued count).
// Web_ui side handles the API fetch / consumable inventory; logic owns the
// per-buff switch logic.

/**
 * Numeric tags for each in-game buff source. Mirrors web_ui's BuffType enum
 * (Event.ts) exactly — values 0..13.
 */
export enum BuffType {
  Quark = 0,
  GoldenQuark = 1,
  Cubes = 2,
  PowderConversion = 3,
  AscensionSpeed = 4,
  GlobalSpeed = 5,
  AscensionScore = 6,
  AntSacrifice = 7,
  Offering = 8,
  Obtainium = 9,
  Octeract = 10,
  BlueberryTime = 11,
  AmbrosiaLuck = 12,
  OneMind = 13
}

/**
 * Wire shape of a server-fetched GameEvent. Mirrors web_ui's GameEvent
 * interface — keys map to BuffType values via getEventBuff.
 */
export interface GameEventBuffs {
  quark: number
  goldenQuark: number
  cubes: number
  powderConversion: number
  ascensionSpeed: number
  globalSpeed: number
  ascensionScore: number
  antSacrifice: number
  offering: number
  obtainium: number
  octeract: number
  blueberryTime: number
  ambrosiaLuck: number
  oneMind: number
}

/**
 * Per-buff value coming from the active GameEvent. Returns 0 when `event`
 * is null (no active event). OneMind also requires the GQ-upgrade unlock
 * — caller passes the boolean.
 */
export function getEventBuff (
  buff: BuffType,
  event: GameEventBuffs | null,
  oneMindUnlocked: boolean
): number {
  if (event === null) {
    return 0
  }
  switch (buff) {
    case BuffType.Quark:
      return event.quark
    case BuffType.GoldenQuark:
      return event.goldenQuark
    case BuffType.Cubes:
      return event.cubes
    case BuffType.PowderConversion:
      return event.powderConversion
    case BuffType.AscensionSpeed:
      return event.ascensionSpeed
    case BuffType.GlobalSpeed:
      return event.globalSpeed
    case BuffType.AscensionScore:
      return event.ascensionScore
    case BuffType.AntSacrifice:
      return event.antSacrifice
    case BuffType.Offering:
      return event.offering
    case BuffType.Obtainium:
      return event.obtainium
    case BuffType.Octeract:
      return event.octeract
    case BuffType.OneMind:
      return oneMindUnlocked ? event.oneMind : 0
    case BuffType.BlueberryTime:
      return event.blueberryTime
    case BuffType.AmbrosiaLuck:
      return event.ambrosiaLuck
    default: {
      throw new Error(`Unhandled BuffType: ${buff as number}`)
    }
  }
}

/**
 * Per-buff value coming from the Happy Hour Bell consumable stack. The
 * caller passes the total HAPPY_HOUR_BELL.amount; the "interval" used in
 * scaling is `amount - 1`. Returns 0 when no bells are queued.
 *
 * Only Quark / Cubes / Offering / Obtainium / BlueberryTime / AmbrosiaLuck
 * have non-trivial happy-hour contributions; the rest return 0.
 */
export function consumableEventBuff (buff: BuffType, happyHourBellAmount: number): number {
  if (happyHourBellAmount === 0) {
    return 0
  }
  const interval = happyHourBellAmount - 1
  switch (buff) {
    case BuffType.Quark:
      return 0.25 + 0.025 * interval
    case BuffType.GoldenQuark:
      return 0
    case BuffType.Cubes:
      return 0.5 + 0.05 * interval
    case BuffType.PowderConversion:
      return 0
    case BuffType.AscensionSpeed:
      return 0
    case BuffType.GlobalSpeed:
      return 0
    case BuffType.AscensionScore:
      return 0
    case BuffType.AntSacrifice:
      return 0
    case BuffType.Offering:
      return 0.5 + 0.05 * interval
    case BuffType.Obtainium:
      return 0.5 + 0.05 * interval
    case BuffType.Octeract:
      return 0
    case BuffType.OneMind:
      return 0
    case BuffType.BlueberryTime:
      return 0.1 + 0.01 * interval
    case BuffType.AmbrosiaLuck:
      return 0.1 + 0.01 * interval
    default: {
      throw new Error(`Unhandled BuffType: ${buff as number}`)
    }
  }
}

/** Sum of the event buff and the consumable buff for a given source. */
export function calculateEventSourceBuff (
  buff: BuffType,
  event: GameEventBuffs | null,
  oneMindUnlocked: boolean,
  happyHourBellAmount: number
): number {
  return getEventBuff(buff, event, oneMindUnlocked)
    + consumableEventBuff(buff, happyHourBellAmount)
}
