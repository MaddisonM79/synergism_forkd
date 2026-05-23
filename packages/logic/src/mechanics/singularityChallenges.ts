// Per-singularity-challenge effect / requirement / AP-value formulas, lifted
// from packages/web_ui/src/SingularityChallenges.ts.
//
// Web_ui still owns the singularityChallengeData table (it has UI fields
// the logic tier can't see: i18next-bound alternateDescription closures,
// HTMLTag for DOM, the scalingrewardcount / uniquerewardcount UI counts).
// This module owns the three pure-formula fields that every challenge has:
//
//   - singularityRequirement(baseReq, completions) → number
//   - achievementPointValue(n)                    → number
//   - effect(n, key)                              → reward field
//
// And the SingularityChallengeRewards type — pure data with no UI
// dependency, re-exported from web_ui's SingularityChallenges.ts so the
// `SingularityChallengeDataKeys` alias and external imports keep compiling.

export type SingularityChallengeRewards = {
  noSingularityUpgrades: {
    cubes: number
    goldenQuarks: number
    blueberries: number
    shopUpgrade: boolean
    additiveLuckMult: number
    shopUpgrade2: boolean
  }
  oneChallengeCap: {
    corrScoreIncrease: number
    blueberrySpeedMult: number
    capIncrease: number
    freeCorruptionLevel: number
    shopUpgrade: boolean
    reinCapIncrease2: number
    ascCapIncrease2: number
  }
  noOcteracts: {
    octeractPow: number
    offeringBonus: boolean
    obtainiumBonus: boolean
    shopUpgrade: boolean
  }
  limitedAscensions: {
    ascensionSpeedMult: number
    hepteractCap: boolean
    shopUpgrade: boolean
    shopUpgrade2: boolean
  }
  noAmbrosiaUpgrades: {
    bonusAmbrosia: number
    blueberries: number
    additiveLuckMult: number
    ambrosiaLuck: number
    redLuck: number
    blueberrySpeedMult: number
    redSpeedMult: number
    shopUpgrade: boolean
    shopUpgrade2: boolean
  }
  noQuarkUpgrades: {
    freeObtainiumLevels: number
    freeOfferingLevels: number
    freeSpeedLevels: number
    freeCubeLevels: number
    freeQuarkLevel: number
    freeInfinityLevels: number
    shopUpgrade: boolean
    topHatUnlock: boolean
  }
  limitedTime: {
    preserveQuarks: boolean
    quarkMult: number
    globalSpeed: number
    ascensionSpeed: number
    barRequirementMultiplier: number
    shopUpgrade: boolean
    shopUpgrade2: boolean
  }
  sadisticPrequel: {
    extraFree: number
    quarkMult: number
    freeUpgradeMult: number
    shopUpgrade: boolean
    shopUpgrade2: boolean
    shopUpgrade3: boolean
  }
  taxmanLastStand: {
    horseShoeUnlock: boolean
    shopUpgrade: boolean
    talismanUnlock: boolean
    talismanFreeLevel: number
    talismanRuneEffect: number
    antiquityOOM: number
    horseShoeOOM: number
  }
}

export type SingularityChallengeDataKeys = keyof SingularityChallengeRewards

// ─── Per-challenge singularityRequirement formulas ─────────────────────────

// All nine challenges share the (baseReq, completions) → number shape. Several
// of them have piecewise scaling past a completion threshold — those are the
// ones worth sweeping in parity tests. The trivial linear ones get their own
// arrow exports for code-locality with the matching effect functions below.

export const noSingularityUpgradesSingularityRequirement = (baseReq: number, completions: number): number => {
  // +16/completion linearly, +8 bonus once you've hit 9 completions.
  return baseReq + 16 * completions + 8 * (completions >= 9 ? 1 : 0)
}

export const oneChallengeCapSingularityRequirement = (baseReq: number, completions: number): number => {
  // +19/completion linearly, with a small -2 discount past 14 completions.
  return baseReq + 19 * completions - 2 * (completions >= 14 ? 1 : 0)
}

export const noOcteractsSingularityRequirement = (baseReq: number, completions: number): number => {
  // +13/completion below 10, +13*9 prefix then +10/completion past.
  if (completions < 10) {
    return baseReq + 13 * completions
  }
  return baseReq + 13 * 9 + 10 * (completions - 9)
}

export const limitedAscensionsSingularityRequirement = (baseReq: number, completions: number): number => {
  return baseReq + 27 * completions
}

export const noAmbrosiaUpgradesSingularityRequirement = (baseReq: number, completions: number): number => {
  // +12/completion below 10, then prefix + +4/completion past.
  if (completions < 10) {
    return baseReq + 12 * completions
  }
  return baseReq + 12 * 9 + 4 * (completions - 9)
}

export const noQuarkUpgradesSingularityRequirement = (baseReq: number, completions: number): number => {
  // Three-band piecewise: +15/comp ≤2, +70 prefix then +9/(comp-6) on 3-5,
  // +185 prefix then +8/(comp-6) past 5. The (completions-6) offsets are
  // verbatim from the legacy code — yes the middle band's offset references
  // a higher knee than its own band. Pinned by parity tests.
  if (completions > 5) {
    return baseReq + 185 + 8 * (completions - 6)
  } else if (completions > 2) {
    return baseReq + 70 + 9 * (completions - 6)
  }
  return baseReq + 15 * completions
}

export const limitedTimeSingularityRequirement = (baseReq: number, completions: number): number => {
  // +8/completion below 10, hard 277 + 2*(comp-10) past.
  if (completions > 9) {
    return 277 + 2 * (completions - 10)
  }
  return baseReq + 8 * completions
}

export const sadisticPrequelSingularityRequirement = (baseReq: number, completions: number): number => {
  return baseReq + 8 * completions
}

export const taxmanLastStandSingularityRequirement = (baseReq: number, completions: number): number => {
  return baseReq + 4 * completions
}

// ─── Per-challenge achievementPointValue formulas ──────────────────────────
//
// All trivial linear scales — kept as named exports for symmetry with the
// other two function categories and so callers can import individually.

export const noSingularityUpgradesAchievementPointValue = (n: number): number => 15 * n
export const oneChallengeCapAchievementPointValue = (n: number): number => 15 * n
export const noOcteractsAchievementPointValue = (n: number): number => 20 * n
export const limitedAscensionsAchievementPointValue = (n: number): number => 30 * n
export const noAmbrosiaUpgradesAchievementPointValue = (n: number): number => 25 * n
export const noQuarkUpgradesAchievementPointValue = (n: number): number => 20 * n
export const limitedTimeAchievementPointValue = (n: number): number => 30 * n
export const sadisticPrequelAchievementPointValue = (n: number): number => 40 * n
export const taxmanLastStandAchievementPointValue = (n: number): number => 50 * n

// ─── Per-challenge effect functions ────────────────────────────────────────
//
// Each challenge's `effect(n, key)` returns the matching field of
// SingularityChallengeRewards[challenge]. Switch-on-key cascades match the
// legacy if/else if chains verbatim — including the trailing `else` that
// returns the last-key value without checking it (legacy behavior; the
// caller is expected to only pass valid keys).

export function noSingularityUpgradesEffect<K extends keyof SingularityChallengeRewards['noSingularityUpgrades']> (
  n: number,
  key: K
): SingularityChallengeRewards['noSingularityUpgrades'][K] {
  if (key === 'cubes') {
    return (1 + n) as SingularityChallengeRewards['noSingularityUpgrades'][K]
  } else if (key === 'goldenQuarks') {
    return (1 + 0.12 * +(n > 0)) as SingularityChallengeRewards['noSingularityUpgrades'][K]
  } else if (key === 'blueberries') {
    return (+(n > 0)) as SingularityChallengeRewards['noSingularityUpgrades'][K]
  } else if (key === 'shopUpgrade') {
    return (n >= 10) as SingularityChallengeRewards['noSingularityUpgrades'][K]
  } else if (key === 'additiveLuckMult') {
    return (n >= 15 ? 0.05 : 0) as SingularityChallengeRewards['noSingularityUpgrades'][K]
  }
  // shopUpgrade2
  return (n >= 15) as SingularityChallengeRewards['noSingularityUpgrades'][K]
}

export function oneChallengeCapEffect<K extends keyof SingularityChallengeRewards['oneChallengeCap']> (
  n: number,
  key: K
): SingularityChallengeRewards['oneChallengeCap'][K] {
  if (key === 'corrScoreIncrease') {
    return (0.05 * n) as SingularityChallengeRewards['oneChallengeCap'][K]
  } else if (key === 'blueberrySpeedMult') {
    return (1 + n / 60) as SingularityChallengeRewards['oneChallengeCap'][K]
  } else if (key === 'capIncrease') {
    return (3 * +(n > 0)) as SingularityChallengeRewards['oneChallengeCap'][K]
  } else if (key === 'freeCorruptionLevel') {
    return (+(n >= 12)) as SingularityChallengeRewards['oneChallengeCap'][K]
  } else if (key === 'shopUpgrade') {
    return (n >= 12) as SingularityChallengeRewards['oneChallengeCap'][K]
  } else if (key === 'reinCapIncrease2') {
    return (7 * +(n >= 15)) as SingularityChallengeRewards['oneChallengeCap'][K]
  }
  // ascCapIncrease2
  return (2 * +(n >= 15)) as SingularityChallengeRewards['oneChallengeCap'][K]
}

export function noOcteractsEffect<K extends keyof SingularityChallengeRewards['noOcteracts']> (
  n: number,
  key: K
): SingularityChallengeRewards['noOcteracts'][K] {
  if (key === 'octeractPow') {
    // Piecewise: 0.02n below the 10-completion knee, then 0.2 + (n-10)/100 past.
    return ((n <= 10) ? 0.02 * n : 0.2 + (n - 10) / 100) as SingularityChallengeRewards['noOcteracts'][K]
  } else if (key === 'offeringBonus') {
    return (n > 0) as SingularityChallengeRewards['noOcteracts'][K]
  } else if (key === 'obtainiumBonus') {
    return (n >= 10) as SingularityChallengeRewards['noOcteracts'][K]
  }
  // shopUpgrade
  return (n >= 10) as SingularityChallengeRewards['noOcteracts'][K]
}

export function limitedAscensionsEffect<K extends keyof SingularityChallengeRewards['limitedAscensions']> (
  n: number,
  key: K
): SingularityChallengeRewards['limitedAscensions'][K] {
  if (key === 'ascensionSpeedMult') {
    return (1 + 0.25 * n / 100) as SingularityChallengeRewards['limitedAscensions'][K]
  } else if (key === 'hepteractCap') {
    return (n > 0) as SingularityChallengeRewards['limitedAscensions'][K]
  } else if (key === 'shopUpgrade') {
    return (n >= 8) as SingularityChallengeRewards['limitedAscensions'][K]
  }
  // shopUpgrade2
  return (n >= 10) as SingularityChallengeRewards['limitedAscensions'][K]
}

export function noAmbrosiaUpgradesEffect<K extends keyof SingularityChallengeRewards['noAmbrosiaUpgrades']> (
  n: number,
  key: K
): SingularityChallengeRewards['noAmbrosiaUpgrades'][K] {
  if (key === 'bonusAmbrosia') {
    return (+(n > 0)) as SingularityChallengeRewards['noAmbrosiaUpgrades'][K]
  } else if (key === 'blueberries') {
    // Floor(n/5) plus a +1 bump on first completion. Stair-steps every 5.
    return (Math.floor(n / 5) + +(n > 0)) as SingularityChallengeRewards['noAmbrosiaUpgrades'][K]
  } else if (key === 'additiveLuckMult') {
    return (n / 200) as SingularityChallengeRewards['noAmbrosiaUpgrades'][K]
  } else if (key === 'ambrosiaLuck') {
    return (20 * n) as SingularityChallengeRewards['noAmbrosiaUpgrades'][K]
  } else if (key === 'redLuck') {
    return (4 * n) as SingularityChallengeRewards['noAmbrosiaUpgrades'][K]
  } else if (key === 'blueberrySpeedMult') {
    return (1 + n / 25) as SingularityChallengeRewards['noAmbrosiaUpgrades'][K]
  } else if (key === 'redSpeedMult') {
    return (1 + 2 * n / 100) as SingularityChallengeRewards['noAmbrosiaUpgrades'][K]
  } else if (key === 'shopUpgrade') {
    return (n >= 8) as SingularityChallengeRewards['noAmbrosiaUpgrades'][K]
  }
  // shopUpgrade2
  return (n >= 10) as SingularityChallengeRewards['noAmbrosiaUpgrades'][K]
}

export function noQuarkUpgradesEffect<K extends keyof SingularityChallengeRewards['noQuarkUpgrades']> (
  n: number,
  key: K
): SingularityChallengeRewards['noQuarkUpgrades'][K] {
  if (key === 'freeObtainiumLevels') {
    return n as SingularityChallengeRewards['noQuarkUpgrades'][K]
  } else if (key === 'freeOfferingLevels') {
    return n as SingularityChallengeRewards['noQuarkUpgrades'][K]
  } else if (key === 'freeSpeedLevels') {
    return n as SingularityChallengeRewards['noQuarkUpgrades'][K]
  } else if (key === 'freeCubeLevels') {
    return n as SingularityChallengeRewards['noQuarkUpgrades'][K]
  } else if (key === 'freeQuarkLevel') {
    return (n >= 5 ? 1 : 0) as SingularityChallengeRewards['noQuarkUpgrades'][K]
  } else if (key === 'freeInfinityLevels') {
    return n as SingularityChallengeRewards['noQuarkUpgrades'][K]
  } else if (key === 'shopUpgrade') {
    return (n >= 1) as SingularityChallengeRewards['noQuarkUpgrades'][K]
  }
  // topHatUnlock
  return (n >= 10) as SingularityChallengeRewards['noQuarkUpgrades'][K]
}

export function limitedTimeEffect<K extends keyof SingularityChallengeRewards['limitedTime']> (
  n: number,
  key: K
): SingularityChallengeRewards['limitedTime'][K] {
  if (key === 'preserveQuarks') {
    return (+(n > 0)) as SingularityChallengeRewards['limitedTime'][K]
  } else if (key === 'quarkMult') {
    return (1 + 0.02 * n) as SingularityChallengeRewards['limitedTime'][K]
  } else if (key === 'globalSpeed') {
    return (1 + 0.12 * n) as SingularityChallengeRewards['limitedTime'][K]
  } else if (key === 'ascensionSpeed') {
    return (1 + 0.12 * n) as SingularityChallengeRewards['limitedTime'][K]
  } else if (key === 'barRequirementMultiplier') {
    return (1 - 0.02 * n) as SingularityChallengeRewards['limitedTime'][K]
  } else if (key === 'shopUpgrade') {
    return (n >= 5) as SingularityChallengeRewards['limitedTime'][K]
  }
  // shopUpgrade2
  return (n >= 10) as SingularityChallengeRewards['limitedTime'][K]
}

export function sadisticPrequelEffect<K extends keyof SingularityChallengeRewards['sadisticPrequel']> (
  n: number,
  key: K
): SingularityChallengeRewards['sadisticPrequel'][K] {
  if (key === 'extraFree') {
    return (50 * +(n > 0)) as SingularityChallengeRewards['sadisticPrequel'][K]
  } else if (key === 'quarkMult') {
    return (1 + 0.06 * n) as SingularityChallengeRewards['sadisticPrequel'][K]
  } else if (key === 'freeUpgradeMult') {
    return (1 + 0.06 * n) as SingularityChallengeRewards['sadisticPrequel'][K]
  } else if (key === 'shopUpgrade') {
    return (n >= 5) as SingularityChallengeRewards['sadisticPrequel'][K]
  } else if (key === 'shopUpgrade2') {
    return (n >= 10) as SingularityChallengeRewards['sadisticPrequel'][K]
  }
  // shopUpgrade3
  return (n >= 15) as SingularityChallengeRewards['sadisticPrequel'][K]
}

export function taxmanLastStandEffect<K extends keyof SingularityChallengeRewards['taxmanLastStand']> (
  n: number,
  key: K
): SingularityChallengeRewards['taxmanLastStand'][K] {
  if (key === 'horseShoeUnlock') {
    return (n > 0) as SingularityChallengeRewards['taxmanLastStand'][K]
  } else if (key === 'shopUpgrade') {
    return (n >= 5) as SingularityChallengeRewards['taxmanLastStand'][K]
  } else if (key === 'talismanUnlock') {
    return (n >= 10) as SingularityChallengeRewards['taxmanLastStand'][K]
  } else if (key === 'talismanFreeLevel') {
    return (25 * n) as SingularityChallengeRewards['taxmanLastStand'][K]
  } else if (key === 'talismanRuneEffect') {
    return (0.03 * n) as SingularityChallengeRewards['taxmanLastStand'][K]
  } else if (key === 'antiquityOOM') {
    return (1 / 50 * n / 10) as SingularityChallengeRewards['taxmanLastStand'][K]
  }
  // horseShoeOOM
  return (1 / 20 * n / 10) as SingularityChallengeRewards['taxmanLastStand'][K]
}
