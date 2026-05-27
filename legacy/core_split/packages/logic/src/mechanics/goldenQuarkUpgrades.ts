// Per-upgrade effect formulas for golden-quark (singularity) upgrades,
// lifted from packages/web_ui/src/singularity.ts.
//
// Web_ui still owns the goldenQuarkUpgrades data table (it has UI fields
// the logic tier can't see: i18next-bound name/description/effectDescription
// closures, the DOM-driven modal renderer, the buy flow, the
// specialCostForm dispatch). Cost lives in `gqUpgradeCost` (already in
// logic). This module owns the per-upgrade `effect(n, [key])` field for
// all 80 upgrades.
//
// And the GoldenQuarkUpgradeRewards type — pure data with no UI
// dependency, re-exported from web_ui's singularity.ts so the
// SingularityDataKeys alias and external imports keep compiling.
//
// Five effects read state outside the logic tier:
//   - singOcteractPatreonBonus: getQuarkBonus()
//   - divinePack: player.corruptions.used.loadout
//   - platonicDelta / platonicPhi: player.singularityCounter and the
//     shopSingularitySpeedup shop-upgrade effect
//   - favoriteUpgrade: 9 sibling upgrades' levels/maxLevels
// Each takes the player-derived value(s) as extra parameter(s); the
// web_ui data-table closure forwards them.

export type GoldenQuarkUpgradeRewards = {
  goldenQuarks1: { goldenQuarkMult: number }
  goldenQuarks2: { goldenQuarkCostMult: number }
  goldenQuarks3: { exportGQPerHour: number }
  starterPack: {
    obtainiumMult: number
    offeringMult: number
    cubeMult: number
  }
  cookies: { unlocked: boolean }
  cookies2: { unlocked: boolean }
  cookies3: { unlocked: boolean }
  cookies4: { unlocked: boolean }
  cookies5: { unlocked: boolean }
  ascensions: { ascensionCountMult: number }
  corruptionFourteen: { unlocked: boolean }
  corruptionFifteen: { freeCorruptionLevel: number }
  singOfferings1: { offeringMult: number }
  singOfferings2: { offeringMult: number }
  singOfferings3: { offeringMult: number }
  singObtainium1: { obtainiumMult: number }
  singObtainium2: { obtainiumMult: number }
  singObtainium3: { obtainiumMult: number }
  singCubes1: { cubeMult: number }
  singCubes2: { cubeMult: number }
  singCubes3: { cubeMult: number }
  singCitadel: {
    offeringMult: number
    obtainiumMult: number
    cubeMult: number
  }
  singCitadel2: {
    offeringMult: number
    obtainiumMult: number
    cubeMult: number
    citadel1FreeLevels: number
  }
  octeractUnlock: { unlocked: boolean }
  singOcteractPatreonBonus: { octeractMult: number }
  offeringAutomatic: { unlocked: boolean }
  intermediatePack: {
    globalSpeedMult: number
    ascensionSpeedMult: number
    packQuarkAdd: number
  }
  advancedPack: {
    packQuarkAdd: number
    corruptionScoreIncrease: number
  }
  expertPack: {
    packQuarkAdd: number
    ascensionScoreMult: number
    addCodeAscensionTimeMult: number
  }
  masterPack: {
    packQuarkAdd: number
    ascensionScoreMult: number
  }
  divinePack: {
    packQuarkAdd: number
    octeractMult: number
  }
  wowPass: { unlocked: boolean }
  wowPass2: { unlocked: boolean }
  wowPass3: { unlocked: boolean }
  wowPass4: { unlocked: boolean }
  potionBuff: { potionPowerMult: number }
  potionBuff2: { potionPowerMult: number }
  potionBuff3: { potionPowerMult: number }
  singChallengeExtension: {
    reincarnationCapIncrease: number
    ascensionCapIncrease: number
  }
  singChallengeExtension2: {
    reincarnationCapIncrease: number
    ascensionCapIncrease: number
  }
  singChallengeExtension3: {
    reincarnationCapIncrease: number
    ascensionCapIncrease: number
  }
  singQuarkImprover1: { quarkMult: number }
  singQuarkHepteract: { quarkHeptExponent: number }
  singQuarkHepteract2: { quarkHeptExponent: number }
  singQuarkHepteract3: { quarkHeptExponent: number }
  singOcteractGain: { octeractMult: number }
  singOcteractGain2: { octeractMult: number }
  singOcteractGain3: { octeractMult: number }
  singOcteractGain4: { octeractMult: number }
  singOcteractGain5: { octeractMult: number }
  platonicTau: {
    unlocked: boolean
    tauPower: number
  }
  platonicAlpha: { unlocked: boolean }
  platonicDelta: { cubeMult: number }
  platonicPhi: { dailyCodes: number }
  singFastForward: { lookahead: number }
  singFastForward2: { lookahead: number }
  singAscensionSpeed: { exponentSpread: number }
  singAscensionSpeed2: { exponentSpread: number }
  ultimatePen: { platonicPowers: boolean }
  halfMind: { unlocked: boolean }
  oneMind: { unlocked: boolean }
  blueberries: { blueberries: number }
  singAmbrosiaLuck: { ambrosiaLuck: number }
  singAmbrosiaLuck2: { ambrosiaLuck: number }
  singAmbrosiaLuck3: { ambrosiaLuck: number }
  singAmbrosiaLuck4: { ambrosiaLuck: number }
  singAmbrosiaGeneration: { ambrosiaBarSpeedMult: number }
  singAmbrosiaGeneration2: { ambrosiaBarSpeedMult: number }
  singAmbrosiaGeneration3: { ambrosiaBarSpeedMult: number }
  singAmbrosiaGeneration4: { ambrosiaBarSpeedMult: number }
  singBonusTokens1: { firstCompletionBonusTokens: number }
  singBonusTokens2: { tokenMultiplier: number }
  singBonusTokens3: { lastCompletionBonusTokens: number }
  singBonusTokens4: { initialTokenBonus: number }
  singInfiniteShopUpgrades: { infinityVouchers: number }
  singTalismanBonusRunes1: { talismanRuneEffect: number }
  singTalismanBonusRunes2: { talismanRuneEffect: number }
  singTalismanBonusRunes3: { talismanRuneEffect: number }
  singTalismanBonusRunes4: { talismanRuneEffect: number }
  favoriteUpgrade: { quarkMult: number }
}

export type SingularityDataKeys = keyof GoldenQuarkUpgradeRewards

// ─── Per-upgrade effect functions ──────────────────────────────────────────

export function goldenQuarks1Effect (n: number): GoldenQuarkUpgradeRewards['goldenQuarks1']['goldenQuarkMult'] {
  return 1 + 0.1 * n
}

// Sharp cost-reduction piecewise: gentle linear below 250 (capped at 50%),
// then 1 / log2(n / 62.5) which decays slowly past the knee. Verbatim.
export function goldenQuarks2Effect (n: number): GoldenQuarkUpgradeRewards['goldenQuarks2']['goldenQuarkCostMult'] {
  return n > 250 ? 1 / Math.log2(n / 62.5) : 1 - Math.min(0.5, n / 500)
}

export function goldenQuarks3Effect (n: number): GoldenQuarkUpgradeRewards['goldenQuarks3']['exportGQPerHour'] {
  // Triangular numbers n*(n+1)/2.
  return (n * (n + 1)) / 2
}

export function starterPackEffect<K extends keyof GoldenQuarkUpgradeRewards['starterPack']> (
  n: number,
  key: K
): GoldenQuarkUpgradeRewards['starterPack'][K] {
  if (key === 'obtainiumMult') {
    return (1 + 5 * n) as GoldenQuarkUpgradeRewards['starterPack'][K]
  }
  if (key === 'offeringMult') {
    return (1 + 5 * n) as GoldenQuarkUpgradeRewards['starterPack'][K]
  }
  // cubeMult
  return (1 + 4 * n) as GoldenQuarkUpgradeRewards['starterPack'][K]
}

// ─── Cookies family + simple boolean unlocks ──────────────────────────────
//
// All cookies upgrades follow the same shape: `(n) => n > 0`. Named
// individually for symmetry with the other effect functions.

const boolUnlock = (n: number): boolean => n > 0

export function cookiesEffect (n: number): GoldenQuarkUpgradeRewards['cookies']['unlocked'] {
  return boolUnlock(n)
}
export function cookies2Effect (n: number): GoldenQuarkUpgradeRewards['cookies2']['unlocked'] {
  return boolUnlock(n)
}
export function cookies3Effect (n: number): GoldenQuarkUpgradeRewards['cookies3']['unlocked'] {
  return boolUnlock(n)
}
export function cookies4Effect (n: number): GoldenQuarkUpgradeRewards['cookies4']['unlocked'] {
  return boolUnlock(n)
}
export function cookies5Effect (n: number): GoldenQuarkUpgradeRewards['cookies5']['unlocked'] {
  return boolUnlock(n)
}

export function ascensionsEffect (n: number): GoldenQuarkUpgradeRewards['ascensions']['ascensionCountMult'] {
  return (1 + (2 * n) / 100) * (1 + Math.floor(n / 10) / 100)
}

export function corruptionFourteenEffect (n: number): GoldenQuarkUpgradeRewards['corruptionFourteen']['unlocked'] {
  return boolUnlock(n)
}

export function corruptionFifteenEffect (
  n: number
): GoldenQuarkUpgradeRewards['corruptionFifteen']['freeCorruptionLevel'] {
  return n
}

// ─── Offering / obtainium / cubes 1-3 ─────────────────────────────────────

export function singOfferings1Effect (n: number): GoldenQuarkUpgradeRewards['singOfferings1']['offeringMult'] {
  return 1 + 0.02 * n
}
export function singOfferings2Effect (n: number): GoldenQuarkUpgradeRewards['singOfferings2']['offeringMult'] {
  return 1 + 0.08 * n
}
export function singOfferings3Effect (n: number): GoldenQuarkUpgradeRewards['singOfferings3']['offeringMult'] {
  return 1 + 0.04 * n
}
export function singObtainium1Effect (n: number): GoldenQuarkUpgradeRewards['singObtainium1']['obtainiumMult'] {
  return 1 + 0.02 * n
}
export function singObtainium2Effect (n: number): GoldenQuarkUpgradeRewards['singObtainium2']['obtainiumMult'] {
  return 1 + 0.08 * n
}
export function singObtainium3Effect (n: number): GoldenQuarkUpgradeRewards['singObtainium3']['obtainiumMult'] {
  return 1 + 0.04 * n
}
export function singCubes1Effect (n: number): GoldenQuarkUpgradeRewards['singCubes1']['cubeMult'] {
  return 1 + 0.006 * n
}
export function singCubes2Effect (n: number): GoldenQuarkUpgradeRewards['singCubes2']['cubeMult'] {
  return 1 + 0.08 * n
}
export function singCubes3Effect (n: number): GoldenQuarkUpgradeRewards['singCubes3']['cubeMult'] {
  return 1 + 0.04 * n
}

// ─── Citadel ──────────────────────────────────────────────────────────────
//
// singCitadel returns the same multiplier for all three reward keys (the
// legacy data-table effect signature accepts the key but ignores it). The
// stair-step every 10 levels adds an additional +1% past each multiple of
// 10. Verbatim.

const citadelMult = (n: number): number => (1 + 0.02 * n) * (1 + Math.floor(n / 10) / 100)

export function singCitadelEffect<K extends keyof GoldenQuarkUpgradeRewards['singCitadel']> (
  n: number,
  _key?: K
): GoldenQuarkUpgradeRewards['singCitadel'][K] {
  return citadelMult(n) as GoldenQuarkUpgradeRewards['singCitadel'][K]
}

export function singCitadel2Effect<K extends keyof GoldenQuarkUpgradeRewards['singCitadel2']> (
  n: number,
  key: K
): GoldenQuarkUpgradeRewards['singCitadel2'][K] {
  if (key === 'citadel1FreeLevels') {
    return n as GoldenQuarkUpgradeRewards['singCitadel2'][K]
  }
  // offeringMult / obtainiumMult / cubeMult all share the same formula.
  return citadelMult(n) as GoldenQuarkUpgradeRewards['singCitadel2'][K]
}

// ─── Octeract / patreon ───────────────────────────────────────────────────

export function octeractUnlockEffect (n: number): GoldenQuarkUpgradeRewards['octeractUnlock']['unlocked'] {
  return boolUnlock(n)
}

// singOcteractPatreonBonus: gated boolean that scales with the live quark
// bonus once unlocked. quarkBonus is in percent (0..100), matching the
// legacy `getQuarkBonus()` return.
export function singOcteractPatreonBonusEffect (
  n: number,
  quarkBonus: number
): GoldenQuarkUpgradeRewards['singOcteractPatreonBonus']['octeractMult'] {
  return (n > 0) ? 1 + quarkBonus / 100 : 1
}

export function offeringAutomaticEffect (n: number): GoldenQuarkUpgradeRewards['offeringAutomatic']['unlocked'] {
  return boolUnlock(n)
}

// ─── Packs (intermediate / advanced / expert / master / divine) ──────────

export function intermediatePackEffect<K extends keyof GoldenQuarkUpgradeRewards['intermediatePack']> (
  n: number,
  key: K
): GoldenQuarkUpgradeRewards['intermediatePack'][K] {
  if (key === 'globalSpeedMult') {
    return (n > 0 ? 2 : 1) as GoldenQuarkUpgradeRewards['intermediatePack'][K]
  }
  if (key === 'ascensionSpeedMult') {
    return (n > 0 ? 1.5 : 1) as GoldenQuarkUpgradeRewards['intermediatePack'][K]
  }
  // packQuarkAdd
  return (n > 0 ? 0.02 : 0) as GoldenQuarkUpgradeRewards['intermediatePack'][K]
}

export function advancedPackEffect<K extends keyof GoldenQuarkUpgradeRewards['advancedPack']> (
  n: number,
  key: K
): GoldenQuarkUpgradeRewards['advancedPack'][K] {
  if (key === 'corruptionScoreIncrease') {
    return (n > 0 ? 0.33 : 0) as GoldenQuarkUpgradeRewards['advancedPack'][K]
  }
  // packQuarkAdd
  return (n > 0 ? 0.04 : 0) as GoldenQuarkUpgradeRewards['advancedPack'][K]
}

export function expertPackEffect<K extends keyof GoldenQuarkUpgradeRewards['expertPack']> (
  n: number,
  key: K
): GoldenQuarkUpgradeRewards['expertPack'][K] {
  if (key === 'addCodeAscensionTimeMult') {
    return (n > 0 ? 1.2 : 1) as GoldenQuarkUpgradeRewards['expertPack'][K]
  }
  if (key === 'ascensionScoreMult') {
    return (n > 0 ? 1.5 : 1) as GoldenQuarkUpgradeRewards['expertPack'][K]
  }
  // packQuarkAdd
  return (n > 0 ? 0.06 : 0) as GoldenQuarkUpgradeRewards['expertPack'][K]
}

export function masterPackEffect<K extends keyof GoldenQuarkUpgradeRewards['masterPack']> (
  n: number,
  key: K
): GoldenQuarkUpgradeRewards['masterPack'][K] {
  if (key === 'ascensionScoreMult') {
    return (n > 0 ? 2 : 1) as GoldenQuarkUpgradeRewards['masterPack'][K]
  }
  // packQuarkAdd
  return (n > 0 ? 0.08 : 0) as GoldenQuarkUpgradeRewards['masterPack'][K]
}

// divinePack: octeractMult scales with the player's active corruption
// loadout. Corruption types 14 (×1.25), 15 (×1.3), and 16 (×1.4) each
// contribute their multiplier. All others contribute ×1. The caller
// passes the values of `player.corruptions.used.loadout` (an array of
// corruption type ids).
export function divinePackEffect<K extends keyof GoldenQuarkUpgradeRewards['divinePack']> (
  n: number,
  key: K,
  corruptionLoadout: readonly number[]
): GoldenQuarkUpgradeRewards['divinePack'][K] {
  if (key === 'octeractMult') {
    if (n === 0) {
      return 1 as GoldenQuarkUpgradeRewards['divinePack'][K]
    }
    const octMult = corruptionLoadout.reduce(
      (acc, curr) => acc * (curr === 16 ? 1.4 : (curr === 15 ? 1.3 : (curr === 14 ? 1.25 : 1))),
      1
    )
    return octMult as GoldenQuarkUpgradeRewards['divinePack'][K]
  }
  // packQuarkAdd
  return (n > 0 ? 0.1 : 0) as GoldenQuarkUpgradeRewards['divinePack'][K]
}

// ─── WoW passes ────────────────────────────────────────────────────────────

export function wowPassEffect (n: number): GoldenQuarkUpgradeRewards['wowPass']['unlocked'] {
  return boolUnlock(n)
}
export function wowPass2Effect (n: number): GoldenQuarkUpgradeRewards['wowPass2']['unlocked'] {
  return boolUnlock(n)
}
export function wowPass3Effect (n: number): GoldenQuarkUpgradeRewards['wowPass3']['unlocked'] {
  return boolUnlock(n)
}
export function wowPass4Effect (n: number): GoldenQuarkUpgradeRewards['wowPass4']['unlocked'] {
  return boolUnlock(n)
}

// ─── Potion buffs (multiplicative, with explicit floors) ──────────────────

// potionBuff: 10 × n² with a floor of 1. The Math.max(1, ...) preserves
// the "no upgrade" baseline.
export function potionBuffEffect (n: number): GoldenQuarkUpgradeRewards['potionBuff']['potionPowerMult'] {
  return Math.max(1, 10 * Math.pow(n, 2))
}

export function potionBuff2Effect (n: number): GoldenQuarkUpgradeRewards['potionBuff2']['potionPowerMult'] {
  return Math.max(1, 2 * n)
}

export function potionBuff3Effect (n: number): GoldenQuarkUpgradeRewards['potionBuff3']['potionPowerMult'] {
  return Math.max(1, 1 + 0.5 * n)
}

// ─── Challenge extensions (same shape ×3 with different maxLevel) ─────────

export function singChallengeExtensionEffect<K extends keyof GoldenQuarkUpgradeRewards['singChallengeExtension']> (
  n: number,
  key: K
): GoldenQuarkUpgradeRewards['singChallengeExtension'][K] {
  if (key === 'ascensionCapIncrease') {
    return n as GoldenQuarkUpgradeRewards['singChallengeExtension'][K]
  }
  return (2 * n) as GoldenQuarkUpgradeRewards['singChallengeExtension'][K]
}

export function singChallengeExtension2Effect<K extends keyof GoldenQuarkUpgradeRewards['singChallengeExtension2']> (
  n: number,
  key: K
): GoldenQuarkUpgradeRewards['singChallengeExtension2'][K] {
  if (key === 'ascensionCapIncrease') {
    return n as GoldenQuarkUpgradeRewards['singChallengeExtension2'][K]
  }
  return (2 * n) as GoldenQuarkUpgradeRewards['singChallengeExtension2'][K]
}

export function singChallengeExtension3Effect<K extends keyof GoldenQuarkUpgradeRewards['singChallengeExtension3']> (
  n: number,
  key: K
): GoldenQuarkUpgradeRewards['singChallengeExtension3'][K] {
  if (key === 'ascensionCapIncrease') {
    return n as GoldenQuarkUpgradeRewards['singChallengeExtension3'][K]
  }
  return (2 * n) as GoldenQuarkUpgradeRewards['singChallengeExtension3'][K]
}

// ─── Quark / hepteract / octeract gain family ─────────────────────────────

export function singQuarkImprover1Effect (n: number): GoldenQuarkUpgradeRewards['singQuarkImprover1']['quarkMult'] {
  return 1 + n / 200
}

export function singQuarkHepteractEffect (
  n: number
): GoldenQuarkUpgradeRewards['singQuarkHepteract']['quarkHeptExponent'] {
  return n / 100
}

export function singQuarkHepteract2Effect (
  n: number
): GoldenQuarkUpgradeRewards['singQuarkHepteract2']['quarkHeptExponent'] {
  return n / 100
}

// singQuarkHepteract3 uses /200 (note the bigger denominator).
export function singQuarkHepteract3Effect (
  n: number
): GoldenQuarkUpgradeRewards['singQuarkHepteract3']['quarkHeptExponent'] {
  return n / 200
}

export function singOcteractGainEffect (n: number): GoldenQuarkUpgradeRewards['singOcteractGain']['octeractMult'] {
  return 1 + 0.0125 * n
}
export function singOcteractGain2Effect (n: number): GoldenQuarkUpgradeRewards['singOcteractGain2']['octeractMult'] {
  return 1 + 0.05 * n
}
export function singOcteractGain3Effect (n: number): GoldenQuarkUpgradeRewards['singOcteractGain3']['octeractMult'] {
  return 1 + 0.025 * n
}
export function singOcteractGain4Effect (n: number): GoldenQuarkUpgradeRewards['singOcteractGain4']['octeractMult'] {
  return 1 + 0.02 * n
}
export function singOcteractGain5Effect (n: number): GoldenQuarkUpgradeRewards['singOcteractGain5']['octeractMult'] {
  return 1 + 0.01 * n
}

// ─── Platonic family ──────────────────────────────────────────────────────

export function platonicTauEffect<K extends keyof GoldenQuarkUpgradeRewards['platonicTau']> (
  n: number,
  key: K
): GoldenQuarkUpgradeRewards['platonicTau'][K] {
  if (key === 'tauPower') {
    return (n > 0 ? 1.01 : 1) as GoldenQuarkUpgradeRewards['platonicTau'][K]
  }
  // unlocked
  return (n > 0) as GoldenQuarkUpgradeRewards['platonicTau'][K]
}

export function platonicAlphaEffect (n: number): GoldenQuarkUpgradeRewards['platonicAlpha']['unlocked'] {
  return boolUnlock(n)
}

// platonicDelta: cubeMult scales with `min(singularityCounter * speedMult /
// (3600*24), 9)` once unlocked. Note legacy uses `(singularityCounter+1)`.
// The 3600*24 divisor converts seconds-of-singularity → days.
export function platonicDeltaEffect (
  n: number,
  singularityCounter: number,
  singularityUpgradeSpeedMult: number
): GoldenQuarkUpgradeRewards['platonicDelta']['cubeMult'] {
  if (n <= 0) return 1
  return 1 + Math.min((singularityCounter + 1) * singularityUpgradeSpeedMult / (3600 * 24), 9)
}

// platonicPhi: dailyCodes scales with `floor(5 × min(singularityCounter ×
// speedMult / (3600*24), 10))`. No +1 offset here, unlike platonicDelta.
export function platonicPhiEffect (
  n: number,
  singularityCounter: number,
  singularityUpgradeSpeedMult: number
): GoldenQuarkUpgradeRewards['platonicPhi']['dailyCodes'] {
  if (n <= 0) return 0
  return Math.floor(5 * Math.min(singularityCounter * singularityUpgradeSpeedMult / (3600 * 24), 10))
}

// ─── Fast-forward / ascension speed / mind ────────────────────────────────

export function singFastForwardEffect (n: number): GoldenQuarkUpgradeRewards['singFastForward']['lookahead'] {
  return n
}
export function singFastForward2Effect (n: number): GoldenQuarkUpgradeRewards['singFastForward2']['lookahead'] {
  return n
}

export function singAscensionSpeedEffect (
  n: number
): GoldenQuarkUpgradeRewards['singAscensionSpeed']['exponentSpread'] {
  return (n > 0) ? 0.03 : 0
}

export function singAscensionSpeed2Effect (
  n: number
): GoldenQuarkUpgradeRewards['singAscensionSpeed2']['exponentSpread'] {
  return 0.001 * n
}

export function ultimatePenEffect (n: number): GoldenQuarkUpgradeRewards['ultimatePen']['platonicPowers'] {
  return boolUnlock(n)
}

export function halfMindEffect (n: number): GoldenQuarkUpgradeRewards['halfMind']['unlocked'] {
  return boolUnlock(n)
}

export function oneMindEffect (n: number): GoldenQuarkUpgradeRewards['oneMind']['unlocked'] {
  return boolUnlock(n)
}

// ─── Blueberries / ambrosia ───────────────────────────────────────────────

export function blueberriesEffect (n: number): GoldenQuarkUpgradeRewards['blueberries']['blueberries'] {
  return n
}

export function singAmbrosiaLuckEffect (n: number): GoldenQuarkUpgradeRewards['singAmbrosiaLuck']['ambrosiaLuck'] {
  return 4 * n
}
export function singAmbrosiaLuck2Effect (n: number): GoldenQuarkUpgradeRewards['singAmbrosiaLuck2']['ambrosiaLuck'] {
  return 2 * n
}
export function singAmbrosiaLuck3Effect (n: number): GoldenQuarkUpgradeRewards['singAmbrosiaLuck3']['ambrosiaLuck'] {
  return 3 * n
}
export function singAmbrosiaLuck4Effect (n: number): GoldenQuarkUpgradeRewards['singAmbrosiaLuck4']['ambrosiaLuck'] {
  return 5 * n
}

export function singAmbrosiaGenerationEffect (
  n: number
): GoldenQuarkUpgradeRewards['singAmbrosiaGeneration']['ambrosiaBarSpeedMult'] {
  return 1 + n / 100
}
export function singAmbrosiaGeneration2Effect (
  n: number
): GoldenQuarkUpgradeRewards['singAmbrosiaGeneration2']['ambrosiaBarSpeedMult'] {
  return 1 + n / 100
}
export function singAmbrosiaGeneration3Effect (
  n: number
): GoldenQuarkUpgradeRewards['singAmbrosiaGeneration3']['ambrosiaBarSpeedMult'] {
  return 1 + n / 100
}
export function singAmbrosiaGeneration4Effect (
  n: number
): GoldenQuarkUpgradeRewards['singAmbrosiaGeneration4']['ambrosiaBarSpeedMult'] {
  return 1 + (2 * n) / 100
}

// ─── Bonus tokens ─────────────────────────────────────────────────────────

export function singBonusTokens1Effect (
  n: number
): GoldenQuarkUpgradeRewards['singBonusTokens1']['firstCompletionBonusTokens'] {
  return n
}
export function singBonusTokens2Effect (n: number): GoldenQuarkUpgradeRewards['singBonusTokens2']['tokenMultiplier'] {
  return 1 + n / 100
}
export function singBonusTokens3Effect (
  n: number
): GoldenQuarkUpgradeRewards['singBonusTokens3']['lastCompletionBonusTokens'] {
  return 2 * n
}
export function singBonusTokens4Effect (n: number): GoldenQuarkUpgradeRewards['singBonusTokens4']['initialTokenBonus'] {
  return 5 * n
}

// ─── Misc late-game ───────────────────────────────────────────────────────

export function singInfiniteShopUpgradesEffect (
  n: number
): GoldenQuarkUpgradeRewards['singInfiniteShopUpgrades']['infinityVouchers'] {
  return n
}

export function singTalismanBonusRunes1Effect (
  n: number
): GoldenQuarkUpgradeRewards['singTalismanBonusRunes1']['talismanRuneEffect'] {
  return n / 100
}
export function singTalismanBonusRunes2Effect (
  n: number
): GoldenQuarkUpgradeRewards['singTalismanBonusRunes2']['talismanRuneEffect'] {
  return n / 100
}
export function singTalismanBonusRunes3Effect (
  n: number
): GoldenQuarkUpgradeRewards['singTalismanBonusRunes3']['talismanRuneEffect'] {
  return n / 100
}
export function singTalismanBonusRunes4Effect (
  n: number
): GoldenQuarkUpgradeRewards['singTalismanBonusRunes4']['talismanRuneEffect'] {
  return n / 100
}

// favoriteUpgrade: quark multiplier scales with the count of nine specific
// sibling upgrades that have hit their maxLevel. Caller pre-computes the
// count (an integer 0..9). The +6 baseline (so even at zero sibling
// upgrades the bonus is 6/5000 per level) is verbatim from legacy.
export function favoriteUpgradeEffect (
  n: number,
  sumOfMaxedSiblingUpgrades: number
): GoldenQuarkUpgradeRewards['favoriteUpgrade']['quarkMult'] {
  return 1 + n / 5000 * (sumOfMaxedSiblingUpgrades + 6)
}
