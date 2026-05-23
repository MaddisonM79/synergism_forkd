// Parity tests for the goldenQuarkUpgrades per-upgrade effect formulas,
// lifted from packages/web_ui/src/singularity.ts. Sweeps cover:
//   - every reward key × representative level grid for the 75 pure effects
//   - the 5 impure effects swept across their extra-player-input axes:
//       singOcteractPatreonBonus (quarkBonus)
//       divinePack (corruption loadout)
//       platonicDelta / platonicPhi (singularityCounter × speedMult)
//       favoriteUpgrade (count of maxed sibling upgrades)

import { describe, expect, it } from 'vitest'
import * as logic from '../../src/mechanics/goldenQuarkUpgrades'

// ─── Old implementations (verbatim from web_ui singularity.ts) ────────────

const oldGoldenQuarks1 = (n: number) => 1 + 0.1 * n
const oldGoldenQuarks2 = (n: number) => n > 250 ? 1 / Math.log2(n / 62.5) : 1 - Math.min(0.5, n / 500)
const oldGoldenQuarks3 = (n: number) => (n * (n + 1)) / 2
const oldStarterPack = (n: number, key: string): number => {
  if (key === 'obtainiumMult') return 1 + 5 * n
  if (key === 'offeringMult') return 1 + 5 * n
  return 1 + 4 * n
}
const oldBoolUnlock = (n: number): boolean => n > 0
const oldAscensions = (n: number) => (1 + (2 * n) / 100) * (1 + Math.floor(n / 10) / 100)
const oldCorruptionFifteen = (n: number) => n
const oldSingOfferings1 = (n: number) => 1 + 0.02 * n
const oldSingOfferings2 = (n: number) => 1 + 0.08 * n
const oldSingOfferings3 = (n: number) => 1 + 0.04 * n
const oldSingObtainium1 = (n: number) => 1 + 0.02 * n
const oldSingObtainium2 = (n: number) => 1 + 0.08 * n
const oldSingObtainium3 = (n: number) => 1 + 0.04 * n
const oldSingCubes1 = (n: number) => 1 + 0.006 * n
const oldSingCubes2 = (n: number) => 1 + 0.08 * n
const oldSingCubes3 = (n: number) => 1 + 0.04 * n
const oldSingCitadel = (n: number) => (1 + 0.02 * n) * (1 + Math.floor(n / 10) / 100)
const oldSingCitadel2 = (n: number, key: string): number => {
  if (key === 'citadel1FreeLevels') return n
  return (1 + 0.02 * n) * (1 + Math.floor(n / 10) / 100)
}
const oldSingOcteractPatreonBonus = (n: number, quarkBonus: number) => (n > 0) ? 1 + quarkBonus / 100 : 1
const oldIntermediatePack = (n: number, key: string): number => {
  if (key === 'globalSpeedMult') return n > 0 ? 2 : 1
  if (key === 'ascensionSpeedMult') return n > 0 ? 1.5 : 1
  return n > 0 ? 0.02 : 0
}
const oldAdvancedPack = (n: number, key: string): number => {
  if (key === 'corruptionScoreIncrease') return n > 0 ? 0.33 : 0
  return n > 0 ? 0.04 : 0
}
const oldExpertPack = (n: number, key: string): number => {
  if (key === 'addCodeAscensionTimeMult') return n > 0 ? 1.2 : 1
  if (key === 'ascensionScoreMult') return n > 0 ? 1.5 : 1
  return n > 0 ? 0.06 : 0
}
const oldMasterPack = (n: number, key: string): number => {
  if (key === 'ascensionScoreMult') return n > 0 ? 2 : 1
  return n > 0 ? 0.08 : 0
}
const oldDivinePack = (n: number, key: string, corruptionLoadout: readonly number[]): number => {
  if (key === 'octeractMult') {
    if (n === 0) return 1
    return corruptionLoadout.reduce(
      (acc, curr) => acc * (curr === 16 ? 1.4 : (curr === 15 ? 1.3 : (curr === 14 ? 1.25 : 1))),
      1
    )
  }
  return n > 0 ? 0.1 : 0
}
const oldPotionBuff = (n: number) => Math.max(1, 10 * Math.pow(n, 2))
const oldPotionBuff2 = (n: number) => Math.max(1, 2 * n)
const oldPotionBuff3 = (n: number) => Math.max(1, 1 + 0.5 * n)
const oldChallengeExtension = (n: number, key: string): number => {
  if (key === 'ascensionCapIncrease') return n
  return 2 * n
}
const oldSingQuarkImprover1 = (n: number) => 1 + n / 200
const oldSingQuarkHepteract = (n: number) => n / 100
const oldSingQuarkHepteract2 = (n: number) => n / 100
const oldSingQuarkHepteract3 = (n: number) => n / 200
const oldSingOcteractGain = (n: number) => 1 + 0.0125 * n
const oldSingOcteractGain2 = (n: number) => 1 + 0.05 * n
const oldSingOcteractGain3 = (n: number) => 1 + 0.025 * n
const oldSingOcteractGain4 = (n: number) => 1 + 0.02 * n
const oldSingOcteractGain5 = (n: number) => 1 + 0.01 * n
const oldPlatonicTau = (n: number, key: string): number | boolean => {
  if (key === 'tauPower') return n > 0 ? 1.01 : 1
  return n > 0
}
const oldPlatonicDelta = (n: number, singularityCounter: number, speedMult: number) =>
  n > 0 ? 1 + Math.min((singularityCounter + 1) * speedMult / (3600 * 24), 9) : 1
const oldPlatonicPhi = (n: number, singularityCounter: number, speedMult: number) =>
  n > 0 ? Math.floor(5 * Math.min(singularityCounter * speedMult / (3600 * 24), 10)) : 0
const oldSingFastForward = (n: number) => n
const oldSingAscensionSpeed = (n: number) => (n > 0) ? 0.03 : 0
const oldSingAscensionSpeed2 = (n: number) => 0.001 * n
const oldBlueberries = (n: number) => n
const oldSingAmbrosiaLuck = (n: number) => 4 * n
const oldSingAmbrosiaLuck2 = (n: number) => 2 * n
const oldSingAmbrosiaLuck3 = (n: number) => 3 * n
const oldSingAmbrosiaLuck4 = (n: number) => 5 * n
const oldSingAmbrosiaGeneration = (n: number) => 1 + n / 100
const oldSingAmbrosiaGeneration4 = (n: number) => 1 + (2 * n) / 100
const oldSingBonusTokens1 = (n: number) => n
const oldSingBonusTokens2 = (n: number) => 1 + n / 100
const oldSingBonusTokens3 = (n: number) => 2 * n
const oldSingBonusTokens4 = (n: number) => 5 * n
const oldSingInfiniteShopUpgrades = (n: number) => n
const oldSingTalismanBonusRunes = (n: number) => n / 100
const oldFavoriteUpgrade = (n: number, sumOfMaxedSiblings: number) => 1 + n / 5000 * (sumOfMaxedSiblings + 6)

const closeEnough = (a: number | boolean, b: number | boolean): boolean => {
  if (a === b) return true
  if (typeof a === 'number' && typeof b === 'number') {
    if (!Number.isFinite(a) || !Number.isFinite(b)) return a === b
    if (Math.abs(a) < 1 && Math.abs(b) < 1) return Math.abs(a - b) < 1e-12
    return Math.abs(a - b) / Math.max(Math.abs(a), Math.abs(b)) < 1e-10
  }
  return false
}

const levelGrid = [0, 1, 2, 5, 10, 15, 25, 50, 100, 250, 1000]

// ─── Pure single-key effects ──────────────────────────────────────────────

describe('parity: GQ pure single-key effects', () => {
  const cases: { name: string; new: (n: number) => number | boolean; old: (n: number) => number | boolean }[] = [
    { name: 'goldenQuarks1', new: logic.goldenQuarks1Effect, old: oldGoldenQuarks1 },
    { name: 'goldenQuarks3', new: logic.goldenQuarks3Effect, old: oldGoldenQuarks3 },
    { name: 'cookies', new: logic.cookiesEffect, old: oldBoolUnlock },
    { name: 'cookies2', new: logic.cookies2Effect, old: oldBoolUnlock },
    { name: 'cookies3', new: logic.cookies3Effect, old: oldBoolUnlock },
    { name: 'cookies4', new: logic.cookies4Effect, old: oldBoolUnlock },
    { name: 'cookies5', new: logic.cookies5Effect, old: oldBoolUnlock },
    { name: 'ascensions', new: logic.ascensionsEffect, old: oldAscensions },
    { name: 'corruptionFourteen', new: logic.corruptionFourteenEffect, old: oldBoolUnlock },
    { name: 'corruptionFifteen', new: logic.corruptionFifteenEffect, old: oldCorruptionFifteen },
    { name: 'singOfferings1', new: logic.singOfferings1Effect, old: oldSingOfferings1 },
    { name: 'singOfferings2', new: logic.singOfferings2Effect, old: oldSingOfferings2 },
    { name: 'singOfferings3', new: logic.singOfferings3Effect, old: oldSingOfferings3 },
    { name: 'singObtainium1', new: logic.singObtainium1Effect, old: oldSingObtainium1 },
    { name: 'singObtainium2', new: logic.singObtainium2Effect, old: oldSingObtainium2 },
    { name: 'singObtainium3', new: logic.singObtainium3Effect, old: oldSingObtainium3 },
    { name: 'singCubes1', new: logic.singCubes1Effect, old: oldSingCubes1 },
    { name: 'singCubes2', new: logic.singCubes2Effect, old: oldSingCubes2 },
    { name: 'singCubes3', new: logic.singCubes3Effect, old: oldSingCubes3 },
    { name: 'octeractUnlock', new: logic.octeractUnlockEffect, old: oldBoolUnlock },
    { name: 'offeringAutomatic', new: logic.offeringAutomaticEffect, old: oldBoolUnlock },
    { name: 'wowPass', new: logic.wowPassEffect, old: oldBoolUnlock },
    { name: 'wowPass2', new: logic.wowPass2Effect, old: oldBoolUnlock },
    { name: 'wowPass3', new: logic.wowPass3Effect, old: oldBoolUnlock },
    { name: 'wowPass4', new: logic.wowPass4Effect, old: oldBoolUnlock },
    { name: 'potionBuff', new: logic.potionBuffEffect, old: oldPotionBuff },
    { name: 'potionBuff2', new: logic.potionBuff2Effect, old: oldPotionBuff2 },
    { name: 'potionBuff3', new: logic.potionBuff3Effect, old: oldPotionBuff3 },
    { name: 'singQuarkImprover1', new: logic.singQuarkImprover1Effect, old: oldSingQuarkImprover1 },
    { name: 'singQuarkHepteract', new: logic.singQuarkHepteractEffect, old: oldSingQuarkHepteract },
    { name: 'singQuarkHepteract2', new: logic.singQuarkHepteract2Effect, old: oldSingQuarkHepteract2 },
    { name: 'singQuarkHepteract3', new: logic.singQuarkHepteract3Effect, old: oldSingQuarkHepteract3 },
    { name: 'singOcteractGain', new: logic.singOcteractGainEffect, old: oldSingOcteractGain },
    { name: 'singOcteractGain2', new: logic.singOcteractGain2Effect, old: oldSingOcteractGain2 },
    { name: 'singOcteractGain3', new: logic.singOcteractGain3Effect, old: oldSingOcteractGain3 },
    { name: 'singOcteractGain4', new: logic.singOcteractGain4Effect, old: oldSingOcteractGain4 },
    { name: 'singOcteractGain5', new: logic.singOcteractGain5Effect, old: oldSingOcteractGain5 },
    { name: 'platonicAlpha', new: logic.platonicAlphaEffect, old: oldBoolUnlock },
    { name: 'singFastForward', new: logic.singFastForwardEffect, old: oldSingFastForward },
    { name: 'singFastForward2', new: logic.singFastForward2Effect, old: oldSingFastForward },
    { name: 'singAscensionSpeed', new: logic.singAscensionSpeedEffect, old: oldSingAscensionSpeed },
    { name: 'singAscensionSpeed2', new: logic.singAscensionSpeed2Effect, old: oldSingAscensionSpeed2 },
    { name: 'ultimatePen', new: logic.ultimatePenEffect, old: oldBoolUnlock },
    { name: 'halfMind', new: logic.halfMindEffect, old: oldBoolUnlock },
    { name: 'oneMind', new: logic.oneMindEffect, old: oldBoolUnlock },
    { name: 'blueberries', new: logic.blueberriesEffect, old: oldBlueberries },
    { name: 'singAmbrosiaLuck', new: logic.singAmbrosiaLuckEffect, old: oldSingAmbrosiaLuck },
    { name: 'singAmbrosiaLuck2', new: logic.singAmbrosiaLuck2Effect, old: oldSingAmbrosiaLuck2 },
    { name: 'singAmbrosiaLuck3', new: logic.singAmbrosiaLuck3Effect, old: oldSingAmbrosiaLuck3 },
    { name: 'singAmbrosiaLuck4', new: logic.singAmbrosiaLuck4Effect, old: oldSingAmbrosiaLuck4 },
    { name: 'singAmbrosiaGeneration', new: logic.singAmbrosiaGenerationEffect, old: oldSingAmbrosiaGeneration },
    { name: 'singAmbrosiaGeneration2', new: logic.singAmbrosiaGeneration2Effect, old: oldSingAmbrosiaGeneration },
    { name: 'singAmbrosiaGeneration3', new: logic.singAmbrosiaGeneration3Effect, old: oldSingAmbrosiaGeneration },
    { name: 'singAmbrosiaGeneration4', new: logic.singAmbrosiaGeneration4Effect, old: oldSingAmbrosiaGeneration4 },
    { name: 'singBonusTokens1', new: logic.singBonusTokens1Effect, old: oldSingBonusTokens1 },
    { name: 'singBonusTokens2', new: logic.singBonusTokens2Effect, old: oldSingBonusTokens2 },
    { name: 'singBonusTokens3', new: logic.singBonusTokens3Effect, old: oldSingBonusTokens3 },
    { name: 'singBonusTokens4', new: logic.singBonusTokens4Effect, old: oldSingBonusTokens4 },
    { name: 'singInfiniteShopUpgrades', new: logic.singInfiniteShopUpgradesEffect, old: oldSingInfiniteShopUpgrades },
    { name: 'singTalismanBonusRunes1', new: logic.singTalismanBonusRunes1Effect, old: oldSingTalismanBonusRunes },
    { name: 'singTalismanBonusRunes2', new: logic.singTalismanBonusRunes2Effect, old: oldSingTalismanBonusRunes },
    { name: 'singTalismanBonusRunes3', new: logic.singTalismanBonusRunes3Effect, old: oldSingTalismanBonusRunes },
    { name: 'singTalismanBonusRunes4', new: logic.singTalismanBonusRunes4Effect, old: oldSingTalismanBonusRunes }
  ]
  for (const c of cases) {
    for (const n of levelGrid) {
      it(`${c.name} n=${n}`, () => {
        expect(closeEnough(c.new(n), c.old(n))).toBe(true)
      })
    }
  }
})

// goldenQuarks2 has a piecewise knee at 250 — sweep around it.
describe('parity: goldenQuarks2 (piecewise knee at 250)', () => {
  for (const n of [0, 1, 100, 249, 250, 251, 500, 1000]) {
    it(`n=${n}`, () => {
      expect(closeEnough(logic.goldenQuarks2Effect(n), oldGoldenQuarks2(n))).toBe(true)
    })
  }
})

// ─── Multi-key effects ────────────────────────────────────────────────────

describe('parity: starterPackEffect', () => {
  const keys = ['obtainiumMult', 'offeringMult', 'cubeMult'] as const
  for (const key of keys) {
    for (const n of [0, 1]) {
      it(`key=${key} n=${n}`, () => {
        expect(closeEnough(logic.starterPackEffect(n, key), oldStarterPack(n, key))).toBe(true)
      })
    }
  }
})

describe('parity: singCitadel (same value across all reward keys)', () => {
  const keys = ['offeringMult', 'obtainiumMult', 'cubeMult'] as const
  for (const key of keys) {
    for (const n of levelGrid) {
      it(`key=${key} n=${n}`, () => {
        expect(closeEnough(logic.singCitadelEffect(n, key), oldSingCitadel(n))).toBe(true)
      })
    }
  }
})

describe('parity: singCitadel2', () => {
  const keys = ['offeringMult', 'obtainiumMult', 'cubeMult', 'citadel1FreeLevels'] as const
  for (const key of keys) {
    for (const n of [0, 1, 10, 50, 100]) {
      it(`key=${key} n=${n}`, () => {
        expect(closeEnough(logic.singCitadel2Effect(n, key), oldSingCitadel2(n, key))).toBe(true)
      })
    }
  }
})

describe('parity: intermediatePackEffect', () => {
  const keys = ['globalSpeedMult', 'ascensionSpeedMult', 'packQuarkAdd'] as const
  for (const key of keys) {
    for (const n of [0, 1]) {
      it(`key=${key} n=${n}`, () => {
        expect(closeEnough(logic.intermediatePackEffect(n, key), oldIntermediatePack(n, key))).toBe(true)
      })
    }
  }
})

describe('parity: advancedPackEffect', () => {
  for (const key of ['packQuarkAdd', 'corruptionScoreIncrease'] as const) {
    for (const n of [0, 1]) {
      it(`key=${key} n=${n}`, () => {
        expect(closeEnough(logic.advancedPackEffect(n, key), oldAdvancedPack(n, key))).toBe(true)
      })
    }
  }
})

describe('parity: expertPackEffect', () => {
  for (const key of ['packQuarkAdd', 'ascensionScoreMult', 'addCodeAscensionTimeMult'] as const) {
    for (const n of [0, 1]) {
      it(`key=${key} n=${n}`, () => {
        expect(closeEnough(logic.expertPackEffect(n, key), oldExpertPack(n, key))).toBe(true)
      })
    }
  }
})

describe('parity: masterPackEffect', () => {
  for (const key of ['packQuarkAdd', 'ascensionScoreMult'] as const) {
    for (const n of [0, 1]) {
      it(`key=${key} n=${n}`, () => {
        expect(closeEnough(logic.masterPackEffect(n, key), oldMasterPack(n, key))).toBe(true)
      })
    }
  }
})

describe('parity: platonicTauEffect', () => {
  for (const key of ['unlocked', 'tauPower'] as const) {
    for (const n of [0, 1]) {
      it(`key=${key} n=${n}`, () => {
        expect(closeEnough(logic.platonicTauEffect(n, key), oldPlatonicTau(n, key))).toBe(true)
      })
    }
  }
})

describe('parity: singChallengeExtension family (same shape ×3)', () => {
  const fns = [
    { name: 'singChallengeExtension', new: logic.singChallengeExtensionEffect },
    { name: 'singChallengeExtension2', new: logic.singChallengeExtension2Effect },
    { name: 'singChallengeExtension3', new: logic.singChallengeExtension3Effect }
  ] as const
  for (const f of fns) {
    for (const key of ['ascensionCapIncrease', 'reincarnationCapIncrease'] as const) {
      for (const n of [0, 1, 2, 3, 4]) {
        it(`${f.name} key=${key} n=${n}`, () => {
          // eslint-disable-next-line @typescript-eslint/no-explicit-any
          expect(closeEnough((f.new as any)(n, key), oldChallengeExtension(n, key))).toBe(true)
        })
      }
    }
  }
})

// ─── Impure effects (extra player-input axes) ─────────────────────────────

describe('parity: singOcteractPatreonBonusEffect (n × quarkBonus)', () => {
  for (const n of [0, 1]) {
    for (const qBonus of [0, 1, 10, 50, 100, 1000]) {
      it(`n=${n} quarkBonus=${qBonus}`, () => {
        expect(closeEnough(
          logic.singOcteractPatreonBonusEffect(n, qBonus),
          oldSingOcteractPatreonBonus(n, qBonus)
        )).toBe(true)
      })
    }
  }
})

describe('parity: divinePackEffect (n × key × loadout)', () => {
  const loadouts: { name: string; loadout: number[] }[] = [
    { name: 'empty', loadout: [0, 0, 0, 0, 0, 0, 0, 0] },
    { name: 'one-14', loadout: [14, 0, 0, 0, 0, 0, 0, 0] },
    { name: 'one-15', loadout: [0, 15, 0, 0, 0, 0, 0, 0] },
    { name: 'one-16', loadout: [0, 0, 16, 0, 0, 0, 0, 0] },
    { name: 'mixed', loadout: [14, 15, 16, 0, 0, 0, 0, 0] },
    { name: 'all-16', loadout: [16, 16, 16, 16, 16, 16, 16, 16] }
  ]
  for (const key of ['octeractMult', 'packQuarkAdd'] as const) {
    for (const { name, loadout } of loadouts) {
      for (const n of [0, 1]) {
        it(`key=${key} loadout=${name} n=${n}`, () => {
          expect(closeEnough(
            logic.divinePackEffect(n, key, loadout),
            oldDivinePack(n, key, loadout)
          )).toBe(true)
        })
      }
    }
  }
})

describe('parity: platonicDeltaEffect (n × singularityCounter × speedMult)', () => {
  // 3600*24 = 86400 — cap kicks in around there scaled by speedMult.
  const singCounters = [0, 86400, 86400 * 5, 86400 * 10, 86400 * 100]
  const speeds = [1, 2, 10]
  for (const n of [0, 1]) {
    for (const sc of singCounters) {
      for (const sp of speeds) {
        it(`n=${n} sc=${sc} sp=${sp}`, () => {
          expect(closeEnough(
            logic.platonicDeltaEffect(n, sc, sp),
            oldPlatonicDelta(n, sc, sp)
          )).toBe(true)
        })
      }
    }
  }
})

describe('parity: platonicPhiEffect (n × singularityCounter × speedMult)', () => {
  const singCounters = [0, 86400, 86400 * 5, 86400 * 10, 86400 * 100]
  const speeds = [1, 2, 10]
  for (const n of [0, 1]) {
    for (const sc of singCounters) {
      for (const sp of speeds) {
        it(`n=${n} sc=${sc} sp=${sp}`, () => {
          expect(closeEnough(
            logic.platonicPhiEffect(n, sc, sp),
            oldPlatonicPhi(n, sc, sp)
          )).toBe(true)
        })
      }
    }
  }
})

describe('parity: favoriteUpgradeEffect (n × sumOfMaxedSiblings)', () => {
  for (const n of levelGrid) {
    for (const s of [0, 1, 3, 5, 7, 9]) {
      it(`n=${n} siblings=${s}`, () => {
        expect(closeEnough(
          logic.favoriteUpgradeEffect(n, s),
          oldFavoriteUpgrade(n, s)
        )).toBe(true)
      })
    }
  }
})
