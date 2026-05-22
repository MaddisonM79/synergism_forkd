// Per-talisman effect formulas. Pure 1-arg functions extracted from
// the `talismans.<key>.effects` fields in packages/web_ui/src/Talismans.ts.
// Each talisman has a rarity tier `n` (0..10) that indexes into a small
// per-talisman lookup array; for n >= 6 (the "signature" tier) most also
// unlock an additional effect.
//
// The 11 inscript-value arrays move here as module-level constants since
// they're pure data. The surrounding plumbing (rarity computation,
// fragments / level state, UI, cost progression) stays in web_ui.

const exemptionInscriptValues = [0, -0.2, -0.3, -0.4, -0.45, -0.5, -0.55, -0.6, -0.61, -0.62, -0.65]
const chronosInscriptValues = [1, 1.04, 1.08, 1.12, 1.16, 1.20, 1.25, 1.30, 1.325, 1.35, 1.4]
const midasInscriptValues = [1, 1.04, 1.08, 1.12, 1.16, 1.20, 1.25, 1.30, 1.325, 1.35, 1.40]
const metaphysicsInscriptValues = [1, 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 1.7, 1.8, 1.9, 2]
const polymathInscriptValues = [1, 1.04, 1.08, 1.12, 1.16, 1.20, 1.25, 1.30, 1.325, 1.35, 1.40]
const mortuusInscriptValues = [1, 1.05, 1.1, 1.15, 1.2, 1.3, 1.4, 1.5, 1.65, 1.8, 2]
const plasticInscriptValues = [1, 1.005, 1.01, 1.015, 1.02, 1.025, 1.03, 1.04, 1.045, 1.05, 1.0666]
const wowSquareInscriptValues = [1, 1.025, 1.05, 1.075, 1.1, 1.125, 1.15, 1.2, 1.225, 1.25, 1.30]
const achievementEffectInscriptValues = [0, 0.001, 0.002, 0.003, 0.004, 0.006, 0.008, .01, .015, .02, .03]
const cookieGrandmaInscriptValues = [0, 0.01, 0.02, 0.03, 0.04, 0.05, 0.06, 0.07, 0.08, 0.09, 0.10]
const horseShoeInscriptValues = [0, 0.001, 0.002, 0.003, 0.004, 0.005, 0.007, 0.01, 0.012, 0.015, 0.02]

export interface ExemptionTalismanEffects {
  taxReduction: number
  duplicationOOMBonus: number
}
export function exemptionTalismanEffects(n: number): ExemptionTalismanEffects {
  return {
    taxReduction: exemptionInscriptValues[n] ?? 0,
    duplicationOOMBonus: n >= 6 ? 12 : 0
  }
}

export interface ChronosTalismanEffects {
  globalSpeed: number
  speedOOMBonus: number
}
export function chronosTalismanEffects(n: number): ChronosTalismanEffects {
  return {
    globalSpeed: chronosInscriptValues[n] ?? 1,
    speedOOMBonus: n >= 6 ? 12 : 0
  }
}

export interface MidasTalismanEffects {
  blessingBonus: number
  thriftOOMBonus: number
}
export function midasTalismanEffects(n: number): MidasTalismanEffects {
  return {
    blessingBonus: midasInscriptValues[n] ?? 1,
    thriftOOMBonus: n >= 6 ? 12 : 0
  }
}

export interface MetaphysicsTalismanEffects {
  talismanEffect: number
  extraTalismanEffect: number
}
export function metaphysicsTalismanEffects(n: number): MetaphysicsTalismanEffects {
  return {
    talismanEffect: metaphysicsInscriptValues[n] ?? 1,
    extraTalismanEffect: n >= 6 ? 1.07 : 1
  }
}

export interface PolymathTalismanEffects {
  ascensionSpeedBonus: number
  SIOOMBonus: number
}
export function polymathTalismanEffects(n: number): PolymathTalismanEffects {
  return {
    ascensionSpeedBonus: polymathInscriptValues[n] ?? 1,
    SIOOMBonus: n >= 6 ? 12 : 0
  }
}

export interface MortuusTalismanEffects {
  antBonus: number
  prismOOMBonus: number
}
export function mortuusTalismanEffects(n: number): MortuusTalismanEffects {
  return {
    antBonus: mortuusInscriptValues[n] ?? 1,
    prismOOMBonus: n >= 6 ? 12 : 0
  }
}

export interface PlasticTalismanEffects {
  quarkBonus: number
}
export function plasticTalismanEffects(n: number): PlasticTalismanEffects {
  return { quarkBonus: plasticInscriptValues[n] ?? 1 }
}

export interface WowSquareTalismanEffects {
  evenDimBonus: number
  oddDimBonus: number
}
export function wowSquareTalismanEffects(n: number): WowSquareTalismanEffects {
  return {
    evenDimBonus: wowSquareInscriptValues[n] ?? 1,
    oddDimBonus: n >= 6 ? 1.20 : 1
  }
}

export interface AchievementTalismanEffects {
  positiveSalvageMult: number
  negativeSalvageMult: number
}
export function achievementTalismanEffects(n: number): AchievementTalismanEffects {
  return {
    positiveSalvageMult: achievementEffectInscriptValues[n] ?? 1,
    negativeSalvageMult: n >= 6 ? -0.02 : 0
  }
}

export interface CookieGrandmaTalismanEffects {
  freeCorruptionLevel: number
  cookieSix: boolean
}
export function cookieGrandmaTalismanEffects(n: number): CookieGrandmaTalismanEffects {
  return {
    freeCorruptionLevel: cookieGrandmaInscriptValues[n] ?? 0,
    cookieSix: n >= 6
  }
}

export interface HorseShoeTalismanEffects {
  luckPercentage: number
  redLuck: number
}
export function horseShoeTalismanEffects(n: number): HorseShoeTalismanEffects {
  return {
    luckPercentage: horseShoeInscriptValues[n] ?? 0,
    redLuck: n >= 6 ? 40 : 0
  }
}
