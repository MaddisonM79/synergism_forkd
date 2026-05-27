// Per-upgrade effect formulas for shop upgrades, lifted from
// packages/web_ui/src/Shop.ts.
//
// Web_ui still owns the shopUpgrades data table (it has UI fields the logic
// tier can't see: i18next-bound name/description/effectDescription
// closures, the DOM-driven modal renderer, the buy flow, price/maxLevel/
// type/resetOnSingularity/upgradeTypes fields). Cost progression already
// lives in `shopCost` in the logic tier. This module owns the
// `effects(n, [key])` field per upgrade for all 83 upgrades.
//
// The QuarkShopUpgradeRewards type — pure data with no UI dependency — is
// re-exported from web_ui's Shop.ts so the `ShopUpgradeNames` alias and
// external imports keep compiling.
//
// Sixteen effects read state outside the logic tier. Each takes the
// player-derived value as an extra parameter; the web_ui data-table closure
// forwards it.

export type QuarkShopUpgradeRewards = {
  offeringPotion: { skipSeconds: number }
  obtainiumPotion: { skipSeconds: number }
  offeringEX: { offeringMult: number }
  offeringEX2: { offeringMult: number }
  offeringEX3: { offeringMult: number; baseOfferings: number }
  obtainiumEX: { obtainiumMult: number }
  obtainiumEX2: { obtainiumMult: number }
  obtainiumEX3: { obtainiumMult: number; immaculateObtainiuMult: number }
  offeringAuto: { autoRune: boolean; autoRuneSpeedMult: number }
  obtainiumAuto: { autoResearch: boolean; researchCostMult: number }
  cashGrab: { obtainiumMult: number; offeringMult: number }
  cashGrab2: { obtainiumMult: number; offeringMult: number }
  shopTalisman: { talismanUnlocked: boolean }
  infiniteAscent: { runeUnlocked: boolean }
  shopSadisticRune: { runeUnlocked: boolean }
  antSpeed: { antELO: number }
  instantChallenge: { unlocked: boolean; extraCompPerTick: number }
  instantChallenge2: { unlocked: boolean; extraCompPerTick: number }
  challengeExtension: { reincarnationChallengeCap: number }
  challengeTome: { c10RequirementReduction: number; c9c10ScalingReduction: number }
  challengeTome2: { c10RequirementReduction: number; c9c10ScalingReduction: number }
  challenge15Auto: { unlocked: boolean }
  seasonPass: { wowCubeMult: number; wowTesseractMult: number }
  seasonPass2: { wowHypercubeMult: number; wowPlatonicMult: number }
  seasonPass3: { wowHepteractMult: number; wowOcteractMult: number }
  seasonPassY: { globalCubeMult: number; wowOcteractMult: number }
  seasonPassZ: { globalCubeMult: number; wowOcteractMult: number }
  seasonPassLost: { wowOcteractMult: number }
  seasonPassInfinity: { globalCubeMult: number; wowOcteractMult: number }
  calculator: { addQuarkMult: number; autoAnswer: boolean; autoFill: boolean }
  calculator2: { addCodeCapacity: number; addQuarkMult: number }
  calculator3: { addRewardVarianceMultiplier: number; ascensionTimerAdd: number }
  calculator4: { addCodeIntervalMult: number; addCodeCapacity: number }
  calculator5: { importGQTimerAdd: number; addCodeCapacity: number }
  calculator6: { octeractTimerAdd: number; addCodeCapacity: number }
  calculator7: { blueberryTimerAdd: number; addCodeCapacity: number }
  chronometer: { ascensionSpeedMult: number }
  chronometer2: { ascensionSpeedMult: number }
  chronometer3: { ascensionSpeedMult: number }
  chronometerZ: { ascensionSpeedMult: number }
  shopChronometerS: { ascensionSpeedMult: number; globalSpeedMult: number }
  chronometerInfinity: { ascensionSpeedMult: number; exponentSpread: number }
  improveQuarkHept: { quarkHeptExponent: number }
  improveQuarkHept2: { quarkHeptExponent: number }
  improveQuarkHept3: { quarkHeptExponent: number }
  improveQuarkHept4: { quarkHeptExponent: number }
  improveQuarkHept5: { quarkHeptExponent: number }
  cubeToQuark: { cubeQuarkMult: number }
  tesseractToQuark: { tesseractQuarkMult: number }
  hypercubeToQuark: { hypercubeQuarkMult: number }
  cubeToQuarkAll: { quarkMult: number }
  shopImprovedDaily: { dailyCodeQuarkMult: number }
  shopImprovedDaily2: { freeSingularityUpgrades: number; dailyCodeGoldenQuarkMult: number }
  shopImprovedDaily3: { freeSingularityUpgrades: number; dailyCodeGoldenQuarkMult: number }
  shopImprovedDaily4: { freeSingularityUpgrades: number; dailyCodeGoldenQuarkMult: number }
  constantEX: { maxPercentIncrease: number }
  powderEX: { orbToPowderConversionMult: number }
  powderAuto: { automaticPowderFraction: number }
  autoWarp: { unlocked: boolean }
  extraWarp: { additionalWarps: number }
  shopAmbrosiaGeneration1: { ambrosiaGenerationMult: number }
  shopAmbrosiaGeneration2: { ambrosiaGenerationMult: number }
  shopAmbrosiaGeneration3: { ambrosiaGenerationMult: number }
  shopAmbrosiaGeneration4: { ambrosiaGenerationMult: number }
  shopAmbrosiaAccelerator: { ambrosiaPointRequirementMult: number }
  shopAmbrosiaLuck1: { ambrosiaLuck: number }
  shopAmbrosiaLuck2: { ambrosiaLuck: number }
  shopAmbrosiaLuck3: { ambrosiaLuck: number }
  shopAmbrosiaLuck4: { ambrosiaLuck: number }
  shopAmbrosiaLuckMultiplier4: { additiveAmbrosiaLuckMult: number }
  shopOcteractAmbrosiaLuck: { ambrosiaLuck: number }
  shopAmbrosiaUltra: { ambrosiaLuck: number }
  shopRedLuck1: { redLuck: number; luckConversionRatio: number }
  shopRedLuck2: { redLuck: number; luckConversionRatio: number }
  shopRedLuck3: { redLuck: number; luckConversionRatio: number }
  shopHorseShoe: { bonusHorseLevels: number; singularityPenaltyMult: number }
  shopInfiniteShopUpgrades: { infiniteVouchers: number }
  shopSingularityPenaltyDebuff: { singularityPenaltyReducers: number }
  shopCashGrabUltra: { ambrosiaGenerationMult: number; cubesMult: number; quarkMult: number }
  shopEXUltra: { offeringMult: number; obtainiumMult: number; cubeMult: number }
  shopSingularitySpeedup: { singularityUpgradeSpeedMult: number }
  shopSingularityPotency: { freeUpgradeMult: number }
  shopPanthema: {
    offeringMult: number
    obtainiumMult: number
    cubeMult: number
    quarkMult: number
    ascensionSpeedMult: number
    ambrosiaGenerationMult: number
    ambrosiaLuck: number
    redLuck: number
    infinityMetaBoost: number
  }
}

export type ShopUpgradeNames = keyof QuarkShopUpgradeRewards

// shopPanthema reads bonusLevels() across seven shopUpgradeTypeInfo groups.
// The caller pre-computes each group's bonus levels. Keys match the
// ShopUpgradeGroups enum's role names (lowercased here for portability).
export interface ShopPanthemaBonusLevels {
  offering: number
  obtainium: number
  cubes: number
  speed: number
  quark: number
  ambrosiaLuck: number
  redAmbrosiaLuck: number
  ambrosiaGeneration: number
  infinityUpgrades: number
}

// ─── Constants / shared sub-formulas ──────────────────────────────────────

// offeringEX / obtainiumEX share a +6%/level multiplier with a stair-step
// ×1.08 every 10 levels.
const exMult = (n: number): number => (1 + 0.06 * n) * Math.pow(1.08, Math.floor(n / 10))

// cubeToQuark / tesseractToQuark / hypercubeToQuark share the same
// piecewise: 1 below n=1, then 1.5 + 0.5 * (1 - 0.9^(n-1)) once unlocked.
const cubeQuarkConversion = (n: number): number => {
  if (n >= 1) {
    return 1.5 + 0.5 * (1 - Math.pow(0.9, n - 1))
  }
  return 1
}

// challengeTome / challengeTome2 share the same shape (different costs/
// maxLevels, same formula).
const challengeTomeBody = <K extends 'c10RequirementReduction' | 'c9c10ScalingReduction'>(
  n: number,
  key: K
): number => {
  if (key === 'c10RequirementReduction') {
    return 2e7 * n
  }
  return -n / 100
}

// shopImprovedDaily2/3/4: same shape, different multipliers.
const shopImprovedDailyHelper = (n: number, key: string, mult: number): number => {
  if (key === 'freeSingularityUpgrades') return n
  return 1 + mult * n
}

// ─── Per-upgrade effect functions ──────────────────────────────────────────

export function offeringPotionEffect (_n: number): QuarkShopUpgradeRewards['offeringPotion']['skipSeconds'] {
  return 7200
}

export function obtainiumPotionEffect (_n: number): QuarkShopUpgradeRewards['obtainiumPotion']['skipSeconds'] {
  return 7200
}

export function offeringEXEffect (n: number): QuarkShopUpgradeRewards['offeringEX']['offeringMult'] {
  return exMult(n)
}

// offeringEX2 reads player.singularityCount.
export function offeringEX2Effect (
  n: number,
  singularityCount: number
): QuarkShopUpgradeRewards['offeringEX2']['offeringMult'] {
  return 1 + 0.01 * n * singularityCount
}

export function offeringEX3Effect<K extends keyof QuarkShopUpgradeRewards['offeringEX3']> (
  n: number,
  key: K
): QuarkShopUpgradeRewards['offeringEX3'][K] {
  if (key === 'offeringMult') {
    return Math.pow(1.012, n) as QuarkShopUpgradeRewards['offeringEX3'][K]
  }
  // baseOfferings
  return Math.floor(n / 25) as QuarkShopUpgradeRewards['offeringEX3'][K]
}

export function obtainiumEXEffect (n: number): QuarkShopUpgradeRewards['obtainiumEX']['obtainiumMult'] {
  return exMult(n)
}

// obtainiumEX2 reads player.singularityCount.
export function obtainiumEX2Effect (
  n: number,
  singularityCount: number
): QuarkShopUpgradeRewards['obtainiumEX2']['obtainiumMult'] {
  return 1 + 0.01 * n * singularityCount
}

export function obtainiumEX3Effect<K extends keyof QuarkShopUpgradeRewards['obtainiumEX3']> (
  n: number,
  key: K
): QuarkShopUpgradeRewards['obtainiumEX3'][K] {
  if (key === 'obtainiumMult') {
    return Math.pow(1.012, n) as QuarkShopUpgradeRewards['obtainiumEX3'][K]
  }
  // immaculateObtainiuMult
  return Math.pow(1.06, Math.floor(n / 25)) as QuarkShopUpgradeRewards['obtainiumEX3'][K]
}

export function offeringAutoEffect<K extends keyof QuarkShopUpgradeRewards['offeringAuto']> (
  n: number,
  key: K
): QuarkShopUpgradeRewards['offeringAuto'][K] {
  if (key === 'autoRune') {
    return (n > 0) as QuarkShopUpgradeRewards['offeringAuto'][K]
  }
  // autoRuneSpeedMult
  return (1 + 0.01 * n) as QuarkShopUpgradeRewards['offeringAuto'][K]
}

export function obtainiumAutoEffect<K extends keyof QuarkShopUpgradeRewards['obtainiumAuto']> (
  n: number,
  key: K
): QuarkShopUpgradeRewards['obtainiumAuto'][K] {
  if (key === 'autoResearch') {
    return (n > 0) as QuarkShopUpgradeRewards['obtainiumAuto'][K]
  }
  // researchCostMult
  return (1 - 0.001 * n) as QuarkShopUpgradeRewards['obtainiumAuto'][K]
}

export function cashGrabEffect (n: number): number {
  return 1 + 0.01 * n
}

export function cashGrab2Effect (n: number): number {
  return 1 + 0.005 * n
}

// shopTalisman / infiniteAscent / shopSadisticRune are boolean unlocks.
// shopTalisman and infiniteAscent are also unlocked by paid PCoin
// upgrades — caller passes the unlock flag.

export function shopTalismanEffect (
  n: number,
  pcoinInstantUnlock1: boolean
): QuarkShopUpgradeRewards['shopTalisman']['talismanUnlocked'] {
  return n > 0 || pcoinInstantUnlock1
}

export function infiniteAscentEffect (
  n: number,
  pcoinInstantUnlock2: boolean
): QuarkShopUpgradeRewards['infiniteAscent']['runeUnlocked'] {
  return n > 0 || pcoinInstantUnlock2
}

export function shopSadisticRuneEffect (n: number): QuarkShopUpgradeRewards['shopSadisticRune']['runeUnlocked'] {
  return n > 0
}

export function antSpeedEffect (n: number): QuarkShopUpgradeRewards['antSpeed']['antELO'] {
  return 4 * n
}

export function instantChallengeEffect<K extends keyof QuarkShopUpgradeRewards['instantChallenge']> (
  n: number,
  key: K
): QuarkShopUpgradeRewards['instantChallenge'][K] {
  if (key === 'unlocked') {
    return (n > 0) as QuarkShopUpgradeRewards['instantChallenge'][K]
  }
  return (10 * n) as QuarkShopUpgradeRewards['instantChallenge'][K]
}

// instantChallenge2's `extraCompPerTick` scales with player.highestSingularityCount.
export function instantChallenge2Effect<K extends keyof QuarkShopUpgradeRewards['instantChallenge2']> (
  n: number,
  key: K,
  highestSingularityCount: number
): QuarkShopUpgradeRewards['instantChallenge2'][K] {
  if (key === 'unlocked') {
    return (n > 0) as QuarkShopUpgradeRewards['instantChallenge2'][K]
  }
  return (n * highestSingularityCount) as QuarkShopUpgradeRewards['instantChallenge2'][K]
}

export function challengeExtensionEffect (
  n: number
): QuarkShopUpgradeRewards['challengeExtension']['reincarnationChallengeCap'] {
  return 2 * n
}

export function challengeTomeEffect<K extends keyof QuarkShopUpgradeRewards['challengeTome']> (
  n: number,
  key: K
): QuarkShopUpgradeRewards['challengeTome'][K] {
  return challengeTomeBody(n, key) as QuarkShopUpgradeRewards['challengeTome'][K]
}

export function challengeTome2Effect<K extends keyof QuarkShopUpgradeRewards['challengeTome2']> (
  n: number,
  key: K
): QuarkShopUpgradeRewards['challengeTome2'][K] {
  return challengeTomeBody(n, key) as QuarkShopUpgradeRewards['challengeTome2'][K]
}

export function challenge15AutoEffect (n: number): QuarkShopUpgradeRewards['challenge15Auto']['unlocked'] {
  return n > 0
}

export function seasonPassEffect (n: number): number {
  return 1 + 0.0225 * n
}

export function seasonPass2Effect (n: number): number {
  return 1 + 0.015 * n
}

export function seasonPass3Effect (n: number): number {
  return 1 + 0.015 * n
}

export function seasonPassYEffect (n: number): number {
  return 1 + 0.0075 * n
}

// seasonPassZ reads player.singularityCount.
export function seasonPassZEffect (n: number, singularityCount: number): number {
  return 1 + 0.01 * n * singularityCount
}

export function seasonPassLostEffect (n: number): QuarkShopUpgradeRewards['seasonPassLost']['wowOcteractMult'] {
  return 1 + 0.001 * n
}

export function seasonPassInfinityEffect<K extends keyof QuarkShopUpgradeRewards['seasonPassInfinity']> (
  n: number,
  key: K
): QuarkShopUpgradeRewards['seasonPassInfinity'][K] {
  if (key === 'globalCubeMult') {
    return Math.pow(1.012, n) as QuarkShopUpgradeRewards['seasonPassInfinity'][K]
  }
  // wowOcteractMult
  return Math.pow(1.012, n * 1.25) as QuarkShopUpgradeRewards['seasonPassInfinity'][K]
}

// ─── Calculator family ────────────────────────────────────────────────────

export function calculatorEffect<K extends keyof QuarkShopUpgradeRewards['calculator']> (
  n: number,
  key: K
): QuarkShopUpgradeRewards['calculator'][K] {
  if (key === 'autoAnswer') {
    return (n > 0) as QuarkShopUpgradeRewards['calculator'][K]
  } else if (key === 'addQuarkMult') {
    return (1 + 0.14 * n) as QuarkShopUpgradeRewards['calculator'][K]
  }
  // autoFill
  return (n === 5) as QuarkShopUpgradeRewards['calculator'][K]
}

export function calculator2Effect<K extends keyof QuarkShopUpgradeRewards['calculator2']> (
  n: number,
  key: K
): QuarkShopUpgradeRewards['calculator2'][K] {
  if (key === 'addCodeCapacity') {
    return (2 * n) as QuarkShopUpgradeRewards['calculator2'][K]
  }
  // addQuarkMult: only at n === 12.
  return (n === 12 ? 1.25 : 1) as QuarkShopUpgradeRewards['calculator2'][K]
}

export function calculator3Effect<K extends keyof QuarkShopUpgradeRewards['calculator3']> (
  n: number,
  key: K
): QuarkShopUpgradeRewards['calculator3'][K] {
  if (key === 'addRewardVarianceMultiplier') {
    return (1 - n / 10) as QuarkShopUpgradeRewards['calculator3'][K]
  }
  // ascensionTimerAdd
  return (60 * n) as QuarkShopUpgradeRewards['calculator3'][K]
}

export function calculator4Effect<K extends keyof QuarkShopUpgradeRewards['calculator4']> (
  n: number,
  key: K
): QuarkShopUpgradeRewards['calculator4'][K] {
  if (key === 'addCodeIntervalMult') {
    return (1 - n / 25) as QuarkShopUpgradeRewards['calculator4'][K]
  }
  // addCodeCapacity: only at n === 10.
  return (n === 10 ? 32 : 0) as QuarkShopUpgradeRewards['calculator4'][K]
}

export function calculator5Effect<K extends keyof QuarkShopUpgradeRewards['calculator5']> (
  n: number,
  key: K
): QuarkShopUpgradeRewards['calculator5'][K] {
  if (key === 'importGQTimerAdd') {
    return (6 * n) as QuarkShopUpgradeRewards['calculator5'][K]
  }
  // addCodeCapacity: floor(n/10) plus a +6 bump at n===100.
  return (Math.floor(n / 10) + (n === 100 ? 6 : 0)) as QuarkShopUpgradeRewards['calculator5'][K]
}

export function calculator6Effect<K extends keyof QuarkShopUpgradeRewards['calculator6']> (
  n: number,
  key: K
): QuarkShopUpgradeRewards['calculator6'][K] {
  if (key === 'octeractTimerAdd') {
    return n as QuarkShopUpgradeRewards['calculator6'][K]
  }
  // addCodeCapacity: only at n === 100.
  return (n === 100 ? 24 : 0) as QuarkShopUpgradeRewards['calculator6'][K]
}

export function calculator7Effect<K extends keyof QuarkShopUpgradeRewards['calculator7']> (
  n: number,
  key: K
): QuarkShopUpgradeRewards['calculator7'][K] {
  if (key === 'blueberryTimerAdd') {
    return n as QuarkShopUpgradeRewards['calculator7'][K]
  }
  // addCodeCapacity: only at n === 50.
  return (n === 50 ? 48 : 0) as QuarkShopUpgradeRewards['calculator7'][K]
}

// ─── Chronometer family ───────────────────────────────────────────────────

export function chronometerEffect (n: number): QuarkShopUpgradeRewards['chronometer']['ascensionSpeedMult'] {
  return 1 + 0.012 * n
}

export function chronometer2Effect (n: number): QuarkShopUpgradeRewards['chronometer2']['ascensionSpeedMult'] {
  return 1 + 0.006 * n
}

export function chronometer3Effect (n: number): QuarkShopUpgradeRewards['chronometer3']['ascensionSpeedMult'] {
  return 1 + 0.015 * n
}

// chronometerZ reads player.singularityCount.
export function chronometerZEffect (
  n: number,
  singularityCount: number
): QuarkShopUpgradeRewards['chronometerZ']['ascensionSpeedMult'] {
  return 1 + 0.001 * n * singularityCount
}

// shopChronometerS reads player.singularityCount, with a 200-singularity
// floor. Same value for both reward keys (ascensionSpeedMult, globalSpeedMult).
export function shopChronometerSEffect (
  n: number,
  singularityCount: number
): number {
  return Math.pow(1.01, n * Math.max(0, singularityCount - 200))
}

export function chronometerInfinityEffect<K extends keyof QuarkShopUpgradeRewards['chronometerInfinity']> (
  n: number,
  key: K
): QuarkShopUpgradeRewards['chronometerInfinity'][K] {
  if (key === 'ascensionSpeedMult') {
    return Math.pow(1.006, n) as QuarkShopUpgradeRewards['chronometerInfinity'][K]
  }
  // exponentSpread
  return (0.001 * Math.floor(n / 40)) as QuarkShopUpgradeRewards['chronometerInfinity'][K]
}

// ─── Improved quark hept family ────────────────────────────────────────────

export function improveQuarkHeptEffect (n: number): number {
  return 0.01 * n
}
export function improveQuarkHept2Effect (n: number): number {
  return 0.01 * n
}
export function improveQuarkHept3Effect (n: number): number {
  return 0.01 * n
}
export function improveQuarkHept4Effect (n: number): number {
  return 0.01 * n
}
export function improveQuarkHept5Effect (n: number): number {
  return 0.0001 * n
}

// ─── Cube/tesseract/hypercube → quark conversion family ────────────────────

export function cubeToQuarkEffect (n: number): QuarkShopUpgradeRewards['cubeToQuark']['cubeQuarkMult'] {
  return cubeQuarkConversion(n)
}
export function tesseractToQuarkEffect (n: number): QuarkShopUpgradeRewards['tesseractToQuark']['tesseractQuarkMult'] {
  return cubeQuarkConversion(n)
}
export function hypercubeToQuarkEffect (n: number): QuarkShopUpgradeRewards['hypercubeToQuark']['hypercubeQuarkMult'] {
  return cubeQuarkConversion(n)
}
export function cubeToQuarkAllEffect (n: number): QuarkShopUpgradeRewards['cubeToQuarkAll']['quarkMult'] {
  return 1 + 0.002 * n
}

// ─── Improved daily family ────────────────────────────────────────────────

export function shopImprovedDailyEffect (
  n: number
): QuarkShopUpgradeRewards['shopImprovedDaily']['dailyCodeQuarkMult'] {
  return 1 + 0.05 * n
}

export function shopImprovedDaily2Effect<K extends keyof QuarkShopUpgradeRewards['shopImprovedDaily2']> (
  n: number,
  key: K
): QuarkShopUpgradeRewards['shopImprovedDaily2'][K] {
  return shopImprovedDailyHelper(n, key as string, 0.2) as QuarkShopUpgradeRewards['shopImprovedDaily2'][K]
}

export function shopImprovedDaily3Effect<K extends keyof QuarkShopUpgradeRewards['shopImprovedDaily3']> (
  n: number,
  key: K
): QuarkShopUpgradeRewards['shopImprovedDaily3'][K] {
  return shopImprovedDailyHelper(n, key as string, 0.15) as QuarkShopUpgradeRewards['shopImprovedDaily3'][K]
}

export function shopImprovedDaily4Effect<K extends keyof QuarkShopUpgradeRewards['shopImprovedDaily4']> (
  n: number,
  key: K
): QuarkShopUpgradeRewards['shopImprovedDaily4'][K] {
  return shopImprovedDailyHelper(n, key as string, 1) as QuarkShopUpgradeRewards['shopImprovedDaily4'][K]
}

// ─── Misc late-game pure ──────────────────────────────────────────────────

export function constantEXEffect (n: number): QuarkShopUpgradeRewards['constantEX']['maxPercentIncrease'] {
  return n
}

export function powderEXEffect (n: number): QuarkShopUpgradeRewards['powderEX']['orbToPowderConversionMult'] {
  return 1 + 0.02 * n
}

export function powderAutoEffect (n: number): QuarkShopUpgradeRewards['powderAuto']['automaticPowderFraction'] {
  return 0.01 * n
}

export function autoWarpEffect (n: number): QuarkShopUpgradeRewards['autoWarp']['unlocked'] {
  return n > 0
}

export function extraWarpEffect (n: number): QuarkShopUpgradeRewards['extraWarp']['additionalWarps'] {
  return n
}

// ─── Ambrosia generation / luck family ────────────────────────────────────

export function shopAmbrosiaGeneration1Effect (n: number): number {
  return 1 + 0.01 * n
}
export function shopAmbrosiaGeneration2Effect (n: number): number {
  return 1 + 0.01 * n
}
export function shopAmbrosiaGeneration3Effect (n: number): number {
  return 1 + 0.01 * n
}
export function shopAmbrosiaGeneration4Effect (n: number): number {
  return 1 + 0.001 * n
}

// shopAmbrosiaAccelerator reads player.singularityChallenges.noAmbrosiaUpgrades.completions.
export function shopAmbrosiaAcceleratorEffect (
  n: number,
  ex5Completions: number
): QuarkShopUpgradeRewards['shopAmbrosiaAccelerator']['ambrosiaPointRequirementMult'] {
  return 1 - 0.006 * n * ex5Completions
}

export function shopAmbrosiaLuck1Effect (n: number): number {
  return 2 * n
}
export function shopAmbrosiaLuck2Effect (n: number): number {
  return 2 * n
}
export function shopAmbrosiaLuck3Effect (n: number): number {
  return 2 * n
}
export function shopAmbrosiaLuck4Effect (n: number): number {
  return 0.6 * n
}

export function shopAmbrosiaLuckMultiplier4Effect (
  n: number
): QuarkShopUpgradeRewards['shopAmbrosiaLuckMultiplier4']['additiveAmbrosiaLuckMult'] {
  return 0.01 * n
}

// shopOcteractAmbrosiaLuck reads player.wowOcteracts.
export function shopOcteractAmbrosiaLuckEffect (
  n: number,
  wowOcteracts: number
): QuarkShopUpgradeRewards['shopOcteractAmbrosiaLuck']['ambrosiaLuck'] {
  return n * (1 + Math.floor(Math.max(0, Math.log10(wowOcteracts))))
}

// shopAmbrosiaUltra reads sumOfExaltCompletions().
export function shopAmbrosiaUltraEffect (
  n: number,
  exaltCompletionsSum: number
): QuarkShopUpgradeRewards['shopAmbrosiaUltra']['ambrosiaLuck'] {
  return 2 * n * exaltCompletionsSum
}

// ─── Red luck family ──────────────────────────────────────────────────────

const redLuckBody = <K extends 'redLuck' | 'luckConversionRatio'>(n: number, key: K, redLuckMult: number): number => {
  if (key === 'redLuck') {
    return redLuckMult * n
  }
  // luckConversionRatio
  return -0.01 * Math.floor(n / 20)
}

export function shopRedLuck1Effect<K extends keyof QuarkShopUpgradeRewards['shopRedLuck1']> (
  n: number,
  key: K
): QuarkShopUpgradeRewards['shopRedLuck1'][K] {
  return redLuckBody(n, key, 0.05) as QuarkShopUpgradeRewards['shopRedLuck1'][K]
}

export function shopRedLuck2Effect<K extends keyof QuarkShopUpgradeRewards['shopRedLuck2']> (
  n: number,
  key: K
): QuarkShopUpgradeRewards['shopRedLuck2'][K] {
  return redLuckBody(n, key, 0.075) as QuarkShopUpgradeRewards['shopRedLuck2'][K]
}

export function shopRedLuck3Effect<K extends keyof QuarkShopUpgradeRewards['shopRedLuck3']> (
  n: number,
  key: K
): QuarkShopUpgradeRewards['shopRedLuck3'][K] {
  return redLuckBody(n, key, 0.1) as QuarkShopUpgradeRewards['shopRedLuck3'][K]
}

// shopHorseShoe: bonusHorseLevels is pure (3n). singularityPenaltyMult
// scales with the horseShoe rune effective level — caller passes it.
export function shopHorseShoeEffect<K extends keyof QuarkShopUpgradeRewards['shopHorseShoe']> (
  n: number,
  key: K,
  horseShoeRuneEffectiveLevel: number
): QuarkShopUpgradeRewards['shopHorseShoe'][K] {
  if (key === 'bonusHorseLevels') {
    return (3 * n) as QuarkShopUpgradeRewards['shopHorseShoe'][K]
  }
  // singularityPenaltyMult: clamped at 300, so the cap is hit at horseShoe×n ≥ 300.
  return (1 - Math.min(300, horseShoeRuneEffectiveLevel * n) / 1000) as QuarkShopUpgradeRewards['shopHorseShoe'][K]
}

// shopInfiniteShopUpgrades reads sumOfExaltCompletions().
export function shopInfiniteShopUpgradesEffect (
  n: number,
  exaltCompletionsSum: number
): QuarkShopUpgradeRewards['shopInfiniteShopUpgrades']['infiniteVouchers'] {
  return Math.floor(0.01 * n * exaltCompletionsSum)
}

export function shopSingularityPenaltyDebuffEffect (
  n: number
): QuarkShopUpgradeRewards['shopSingularityPenaltyDebuff']['singularityPenaltyReducers'] {
  return n
}

// shopCashGrabUltra reads player.lifetimeAmbrosia, capped via cbrt at 1e7.
export function shopCashGrabUltraEffect<K extends keyof QuarkShopUpgradeRewards['shopCashGrabUltra']> (
  n: number,
  key: K,
  lifetimeAmbrosia: number
): QuarkShopUpgradeRewards['shopCashGrabUltra'][K] {
  const ratio = Math.min(1, Math.cbrt(lifetimeAmbrosia / 1e7))
  if (key === 'ambrosiaGenerationMult') {
    return (1 + 0.15 * n * ratio) as QuarkShopUpgradeRewards['shopCashGrabUltra'][K]
  } else if (key === 'cubesMult') {
    return (1 + 1.2 * n * ratio) as QuarkShopUpgradeRewards['shopCashGrabUltra'][K]
  }
  // quarkMult
  return (1 + 0.08 * n * ratio) as QuarkShopUpgradeRewards['shopCashGrabUltra'][K]
}

// shopEXUltra reads player.lifetimeAmbrosia. Same value for all three keys.
export function shopEXUltraEffect (n: number, lifetimeAmbrosia: number): number {
  const ambrosiaMult = Math.min(125 * n, lifetimeAmbrosia / 1000) / 1000
  return 1 + ambrosiaMult
}

export function shopSingularitySpeedupEffect (
  n: number
): QuarkShopUpgradeRewards['shopSingularitySpeedup']['singularityUpgradeSpeedMult'] {
  return n > 0 ? 50 : 1
}

export function shopSingularityPotencyEffect (
  n: number
): QuarkShopUpgradeRewards['shopSingularityPotency']['freeUpgradeMult'] {
  return n > 0 ? 3.66 : 1
}

// shopPanthema: every reward key scales with the bonusLevels of a different
// shop-upgrade group, plus a shared infinityBoost prefix that scales with
// the InfinityUpgrades group's bonusLevels. Caller pre-computes each group's
// bonusLevels.
export function shopPanthemaEffect<K extends keyof QuarkShopUpgradeRewards['shopPanthema']> (
  n: number,
  key: K,
  bonusLevels: ShopPanthemaBonusLevels
): QuarkShopUpgradeRewards['shopPanthema'][K] {
  const infinityBoost = 1 + 0.01 * n * bonusLevels.infinityUpgrades

  if (key === 'infinityMetaBoost') {
    return infinityBoost as QuarkShopUpgradeRewards['shopPanthema'][K]
  } else if (key === 'offeringMult') {
    return (1 + 0.01 * n * bonusLevels.offering * infinityBoost) as QuarkShopUpgradeRewards['shopPanthema'][K]
  } else if (key === 'obtainiumMult') {
    return (1 + 0.01 * n * bonusLevels.obtainium * infinityBoost) as QuarkShopUpgradeRewards['shopPanthema'][K]
  } else if (key === 'cubeMult') {
    return (1 + 0.005 * n * bonusLevels.cubes * infinityBoost) as QuarkShopUpgradeRewards['shopPanthema'][K]
  } else if (key === 'ascensionSpeedMult') {
    return (1 + 0.005 * n * bonusLevels.speed * infinityBoost) as QuarkShopUpgradeRewards['shopPanthema'][K]
  } else if (key === 'quarkMult') {
    return (1 + 0.001 * n * bonusLevels.quark * infinityBoost) as QuarkShopUpgradeRewards['shopPanthema'][K]
  } else if (key === 'ambrosiaGenerationMult') {
    return (1 + 0.001 * n * bonusLevels.ambrosiaGeneration
        * infinityBoost) as QuarkShopUpgradeRewards['shopPanthema'][K]
  } else if (key === 'ambrosiaLuck') {
    return (0.2 * n * bonusLevels.ambrosiaLuck * infinityBoost) as QuarkShopUpgradeRewards['shopPanthema'][K]
  } else if (key === 'redLuck') {
    return (0.05 * n * bonusLevels.redAmbrosiaLuck * infinityBoost) as QuarkShopUpgradeRewards['shopPanthema'][K]
  }
  throw new TypeError(`unknown effect ${String(key)}`)
}
