// Rune level-bonus + OOM-increase aggregators, lifted from
// packages/web_ui/src/Runes.ts. Each formula sums a small fixed set of
// player/upgrade contributions. Web_ui pre-extracts each contribution from
// `player.*` / `getTalismanEffects(...)` / `getAmbrosiaUpgradeEffects(...)`
// / `getLevelMilestone(...)` / `CalcECC(...)` and passes the bundled inputs.
//
// Functions migrated:
//   - firstFiveFreeLevels — small constant + cap
//   - bonusRuneLevelsSpeed / -Duplication / -InfiniteAscent — non-trivial
//     coin-based or singularity-perk math; the four other bonusRuneLevels*
//     are 1-line pass-throughs (just talisman bonus) and stay in web_ui.
//   - speedRuneOOMIncrease / -Duplication / -Prism / -Thrift / -SI —
//     the five "real" OOM aggregators. InfiniteAscent/Antiquities/HorseShoe
//     OOMIncrease are 1-line pass-throughs and stay in web_ui.
//
// All formulas are pure number math (no Decimal except for the Decimal.log
// branches in coin-based bonuses, where the caller pre-evaluates the log to
// a plain number).

// ─── firstFiveFreeLevels ───────────────────────────────────────────────────

export interface FirstFiveFreeLevelsInput {
  /** getAntUpgradeEffect(AntUpgrades.FreeRunes).freeRuneLevel */
  freeRunesAntUpgrade: number
  /** player.constantUpgrades[7] — capped at 1000, ×7. */
  constantUpgrade7: number
}

export function firstFiveFreeLevels (input: FirstFiveFreeLevelsInput): number {
  return input.freeRunesAntUpgrade + 7 * Math.min(input.constantUpgrade7, 1000)
}

// ─── bonusRuneLevels (non-trivial: speed, duplication, infiniteAscent) ────

export interface BonusRuneLevelsSpeedInput {
  /** getRuneBonusFromAllTalismans('speed'). */
  talismanBonus: number
  /** player.upgrades[27] — the upgrade level, multiplied by the coin-log split. */
  upgrade27: number
  /** Math.floor(Decimal.log(player.coins.add(1), 1e10)) — pre-evaluated. */
  coinLog1e10Floor: number
  /** Math.floor(Decimal.log(player.coins.add(1), 1e50)) — pre-evaluated. */
  coinLog1e50Floor: number
  /** player.upgrades[29] — coin-count-based bonus. */
  upgrade29: number
  /** firstOwnedCoin + secondOwnedCoin + thirdOwnedCoin + fourthOwnedCoin + fifthOwnedCoin. */
  totalOwnedCoinsFirstFive: number
}

export function bonusRuneLevelsSpeed (input: BonusRuneLevelsSpeedInput): number {
  // upgrade27 contribution: caps the 1e10-log at 50, then ADDS a second
  // 1e50-log term that's offset by -10 and clamped to [0, 50]. The two terms
  // are summed before multiplying by upgrade27.
  const upgrade27Term = input.upgrade27 * (
    Math.min(50, input.coinLog1e10Floor)
    + Math.max(0, Math.min(50, input.coinLog1e50Floor - 10))
  )
  const upgrade29Term = input.upgrade29 * Math.floor(
    Math.min(100, input.totalOwnedCoinsFirstFive / 400)
  )
  return input.talismanBonus + upgrade27Term + upgrade29Term
}

export interface BonusRuneLevelsDuplicationInput {
  /** getRuneBonusFromAllTalismans('duplication'). */
  talismanBonus: number
  /** player.upgrades[28] — coin-count-based bonus. */
  upgrade28: number
  /** firstOwnedCoin + ... + fifthOwnedCoin. */
  totalOwnedCoinsFirstFive: number
  /** player.upgrades[30] — coin-log-based bonus. */
  upgrade30: number
  /** Math.floor(Decimal.log(player.coins.add(1), 1e30)). */
  coinLog1e30Floor: number
  /** Math.floor(Decimal.log(player.coins.add(1), 1e300)). */
  coinLog1e300Floor: number
}

export function bonusRuneLevelsDuplication (input: BonusRuneLevelsDuplicationInput): number {
  const upgrade28Term = input.upgrade28 * Math.min(
    100,
    Math.floor(input.totalOwnedCoinsFirstFive / 400)
  )
  // upgrade30: sum of two log-caps (different bases), each ceiling-50.
  const upgrade30Term = input.upgrade30 * (
    Math.min(50, input.coinLog1e30Floor)
    + Math.min(50, input.coinLog1e300Floor)
  )
  return input.talismanBonus + upgrade28Term + upgrade30Term
}

export interface BonusRuneLevelsInfiniteAscentInput {
  /** PCoinUpgradeEffects.INSTANT_UNLOCK_2 ? 6 : 0 — pre-evaluated to the number. */
  instantUnlock2Bonus: number
  /** player.cubeUpgrades[73]. */
  cubeUpgrade73: number
  /** player.campaigns.bonusRune6. */
  campaignBonusRune6: number
  /** getRuneBonusFromAllTalismans('infiniteAscent'). */
  talismanBonus: number
  /** getRuneEffects('finiteDescent', 'infiniteAscentFreeLevel'). */
  finiteDescentBonus: number
}

export function bonusRuneLevelsInfiniteAscent (input: BonusRuneLevelsInfiniteAscentInput): number {
  return input.instantUnlock2Bonus
    + input.cubeUpgrade73
    + input.campaignBonusRune6
    + input.talismanBonus
    + input.finiteDescentBonus
}

// ─── runeOOMIncrease (speed, duplication, prism, thrift, SI) ──────────────
//
// All five share a common ascension-challenge term:
//   `CalcECC('ascension', c11) + 1.5 * CalcECC('ascension', c14)`
// plus per-rune research/cube/talisman/ambrosia/milestone contributions.
// Web_ui passes the already-evaluated CalcECC + ambrosia + milestone values.

export interface SpeedRuneOOMIncreaseInput {
  /** player.upgrades[66] × 2. */
  upgrade66: number
  /** player.researches[78]. */
  research78: number
  /** player.researches[111]. */
  research111: number
  /** CalcECC('ascension', player.challengecompletions[11]). */
  c11AscensionECC: number
  /** CalcECC('ascension', player.challengecompletions[14]) — multiplied by 1.5. */
  c14AscensionECC: number
  /** player.cubeUpgrades[16]. */
  cubeUpgrade16: number
  /** getTalismanEffects('chronos').speedOOMBonus. */
  chronosSpeedOOMBonus: number
  /** getAmbrosiaUpgradeEffects('ambrosiaRuneOOMBonus', 'runeOOMBonus'). */
  ambrosiaRuneOOMBonus: number
  /** getLevelMilestone('speedRune'). */
  speedRuneLevelMilestone: number
}

export function speedRuneOOMIncrease (input: SpeedRuneOOMIncreaseInput): number {
  return input.upgrade66 * 2
    + input.research78
    + input.research111
    + input.c11AscensionECC
    + 1.5 * input.c14AscensionECC
    + input.cubeUpgrade16
    + input.chronosSpeedOOMBonus
    + input.ambrosiaRuneOOMBonus
    + input.speedRuneLevelMilestone
}

export interface DuplicationRuneOOMIncreaseInput {
  /** CalcECC('transcend', player.challengecompletions[1]) — multiplied by 0.75. */
  c1TranscendECC: number
  /** player.upgrades[66] × 2. */
  upgrade66: number
  /** player.researches[90]. */
  research90: number
  /** player.researches[112]. */
  research112: number
  /** CalcECC('ascension', player.challengecompletions[11]). */
  c11AscensionECC: number
  /** CalcECC('ascension', player.challengecompletions[14]) — multiplied by 1.5. */
  c14AscensionECC: number
  /** getTalismanEffects('exemption').duplicationOOMBonus. */
  exemptionDuplicationOOMBonus: number
  /** getAmbrosiaUpgradeEffects('ambrosiaRuneOOMBonus', 'runeOOMBonus'). */
  ambrosiaRuneOOMBonus: number
  /** getLevelMilestone('duplicationRune'). */
  duplicationRuneLevelMilestone: number
}

export function duplicationRuneOOMIncrease (input: DuplicationRuneOOMIncreaseInput): number {
  return 0.75 * input.c1TranscendECC
    + input.upgrade66 * 2
    + input.research90
    + input.research112
    + input.c11AscensionECC
    + 1.5 * input.c14AscensionECC
    + input.exemptionDuplicationOOMBonus
    + input.ambrosiaRuneOOMBonus
    + input.duplicationRuneLevelMilestone
}

export interface PrismRuneOOMIncreaseInput {
  upgrade66: number
  /** player.researches[79]. */
  research79: number
  /** player.researches[113]. */
  research113: number
  c11AscensionECC: number
  c14AscensionECC: number
  cubeUpgrade16: number
  /** getTalismanEffects('mortuus').prismOOMBonus. */
  mortuusPrismOOMBonus: number
  ambrosiaRuneOOMBonus: number
  /** getLevelMilestone('prismRune'). */
  prismRuneLevelMilestone: number
}

export function prismRuneOOMIncrease (input: PrismRuneOOMIncreaseInput): number {
  return input.upgrade66 * 2
    + input.research79
    + input.research113
    + input.c11AscensionECC
    + 1.5 * input.c14AscensionECC
    + input.cubeUpgrade16
    + input.mortuusPrismOOMBonus
    + input.ambrosiaRuneOOMBonus
    + input.prismRuneLevelMilestone
}

export interface ThriftRuneOOMIncreaseInput {
  upgrade66: number
  /** player.researches[77]. */
  research77: number
  /** player.researches[114]. */
  research114: number
  c11AscensionECC: number
  c14AscensionECC: number
  /** player.cubeUpgrades[37]. */
  cubeUpgrade37: number
  /** getTalismanEffects('midas').thriftOOMBonus. */
  midasThriftOOMBonus: number
  ambrosiaRuneOOMBonus: number
  /** getLevelMilestone('thriftRune'). */
  thriftRuneLevelMilestone: number
}

export function thriftRuneOOMIncrease (input: ThriftRuneOOMIncreaseInput): number {
  return input.upgrade66 * 2
    + input.research77
    + input.research114
    + input.c11AscensionECC
    + 1.5 * input.c14AscensionECC
    + input.cubeUpgrade37
    + input.midasThriftOOMBonus
    + input.ambrosiaRuneOOMBonus
    + input.thriftRuneLevelMilestone
}

export interface SuperiorIntellectRuneOOMIncreaseInput {
  upgrade66: number
  /** player.researches[115]. */
  research115: number
  c11AscensionECC: number
  c14AscensionECC: number
  /** player.cubeUpgrades[37]. */
  cubeUpgrade37: number
  /** getTalismanEffects('polymath').SIOOMBonus. */
  polymathSIOOMBonus: number
  ambrosiaRuneOOMBonus: number
  /** getLevelMilestone('SIRune'). */
  siRuneLevelMilestone: number
}

export function superiorIntellectRuneOOMIncrease (input: SuperiorIntellectRuneOOMIncreaseInput): number {
  return input.upgrade66 * 2
    + input.research115
    + input.c11AscensionECC
    + 1.5 * input.c14AscensionECC
    + input.cubeUpgrade37
    + input.polymathSIOOMBonus
    + input.ambrosiaRuneOOMBonus
    + input.siRuneLevelMilestone
}
