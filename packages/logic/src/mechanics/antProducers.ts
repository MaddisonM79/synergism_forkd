// Ant-producer data + cost solvers + base-production formula. Lifted from:
//   packages/web_ui/src/Features/Ants/AntProducers/data/data.ts (pure fields)
//   packages/web_ui/src/Features/Ants/AntProducers/lib/get-cost.ts
//   packages/web_ui/src/Features/Ants/AntProducers/lib/calculate-production.ts
//
// The data table is indexed 0..8 to match AntProducers enum (Workers=0
// .. HolySpirit=8). UI-only fields (additionalTexts closures) stay in
// web_ui — only the pure baseCost/costIncrease/baseProduction/color/produces
// fields move here.
//
// Cost shape differs from antUpgrades: per-producer `costIncrease` is the
// geometric base (3 for Workers, 10 for Breeders, etc.), not a log-10 exp.
// Formula: cost-to-buy-Nth = baseCost × costIncrease^(N-1).

import { Decimal } from '../math/bignum'

export interface AntProducerData {
  baseCost: Decimal
  /** Geometric multiplier per purchase: cost-to-buy-Nth = baseCost × costIncrease^(N-1). */
  costIncrease: number
  /** Per-producer baseline production rate. */
  baseProduction: Decimal
  /** UI hint color — pure string, kept here for completeness. */
  color: string
  /** Index of the producer this one generates; undefined for Workers (top of chain). */
  produces?: number
}

/** 9-entry data table, indexed 0..8. */
export const antProducerData: readonly AntProducerData[] = [
  // Workers (0) — produces no other producer (top of chain)
  {
    baseCost: Decimal.fromString('1'),
    costIncrease: 3,
    baseProduction: Decimal.fromNumber(0.01),
    color: '#AB8654'
  },
  // Breeders (1) → produces Workers
  {
    baseCost: Decimal.fromString('10'),
    costIncrease: 10,
    baseProduction: Decimal.fromNumber(1.5e-4),
    color: '#B77D48',
    produces: 0
  },
  // MetaBreeders (2) → produces Breeders
  {
    baseCost: Decimal.fromString('1e5'),
    costIncrease: 1e2,
    baseProduction: Decimal.fromNumber(5e-6),
    color: '#C2783D',
    produces: 1
  },
  // MegaBreeders (3) → produces MetaBreeders
  {
    baseCost: Decimal.fromString('1e12'),
    costIncrease: 1e4,
    baseProduction: Decimal.fromNumber(3e-5),
    color: '#CA7035',
    produces: 2
  },
  // Queens (4) → produces MegaBreeders
  {
    baseCost: Decimal.fromString('1e145'),
    costIncrease: 1e8,
    baseProduction: Decimal.fromNumber(1e-30),
    color: '#D26B2D',
    produces: 3
  },
  // LordRoyals (5) → produces Queens
  {
    baseCost: Decimal.fromString('1e700'),
    costIncrease: 1e16,
    baseProduction: Decimal.fromNumber(1e-90),
    color: '#DC6623',
    produces: 4
  },
  // Almighties (6) → produces LordRoyals
  {
    baseCost: Decimal.fromString('1e5000'),
    costIncrease: 1e32,
    baseProduction: Decimal.fromString('1e-600'),
    color: '#E76118',
    produces: 5
  },
  // Disciples (7) → produces Almighties
  {
    baseCost: Decimal.fromString('1e25000'),
    costIncrease: 1e64,
    baseProduction: Decimal.fromString('1e-3500'),
    color: '#F65D09',
    produces: 6
  },
  // HolySpirit (8) → produces Disciples
  {
    baseCost: Decimal.fromString('1e1000000'),
    costIncrease: 1e128,
    baseProduction: Decimal.fromString('1e-110000'),
    color: '#FFFFFF',
    produces: 7
  }
] as const

// ─── Cost solvers ─────────────────────────────────────────────────────────

export interface AntProducerCostInput {
  /** antProducerData[index].baseCost. */
  baseCost: Decimal
  /** antProducerData[index].costIncrease. */
  costIncrease: number
  /** player.ants.producers[index].purchased. */
  purchased: number
}

/**
 * Cost of buying the next producer. cost-to-reach-N = baseCost × costIncrease^N;
 * delta cost is nextCost - lastCost (with lastCost=0 when purchased=0).
 */
export function getCostNextAntProducer (input: AntProducerCostInput): Decimal {
  const nextCost = input.baseCost.times(
    Decimal.pow(input.costIncrease, input.purchased)
  )
  const lastCost = input.purchased > 0
    ? input.baseCost.times(Decimal.pow(input.costIncrease, input.purchased - 1))
    : Decimal.fromString('0')
  return nextCost.sub(lastCost)
}

export interface AntProducerMaxPurchasableInput {
  baseCost: Decimal
  costIncrease: number
  purchased: number
  /** player.ants.crumbs — budget to spend. */
  budget: Decimal
}

/**
 * Max producer count reachable with `budget`. Re-adds sunk cost (current
 * spend) to budget then solves the inverse:
 *   N = 1 + floor(log_{costIncrease}(realBudget / baseCost))
 * Floored at 0.
 */
export function getMaxPurchasableAntProducers (input: AntProducerMaxPurchasableInput): number {
  const sunkCost = input.purchased > 0
    ? input.baseCost.times(Decimal.pow(input.costIncrease, input.purchased - 1))
    : Decimal.fromString('0')
  const realBudget = input.budget.add(sunkCost)
  return Math.max(
    0,
    1 + Math.floor(Decimal.log(realBudget.div(input.baseCost), input.costIncrease))
  )
}

export interface AntProducerMaxCostInput {
  baseCost: Decimal
  costIncrease: number
  purchased: number
  /** Result of getMaxPurchasableAntProducers for the same inputs. */
  maxBuyable: number
}

/**
 * Total cost to buy from current `purchased` up to `maxBuyable`. Subtracts
 * the already-paid sunk cost (cost-of-current-N).
 */
export function getCostMaxAntProducers (input: AntProducerMaxCostInput): Decimal {
  const spent = input.purchased > 0
    ? Decimal.pow(input.costIncrease, input.purchased - 1).times(input.baseCost)
    : Decimal.fromString('0')
  const maxCost = Decimal.pow(input.costIncrease, input.maxBuyable - 1).times(input.baseCost)
  return maxCost.sub(spent)
}

// ─── Base production rate ─────────────────────────────────────────────────

export interface BaseAntsToBeGeneratedInput {
  /** player.ants.producers[index].generated. */
  generated: Decimal
  /** player.ants.producers[index].purchased. */
  purchased: number
  /** antProducerData[index].baseProduction. */
  baseProduction: Decimal
  /** calculateSelfSpeedFromMastery(index) — the per-producer mastery mult. */
  selfSpeedMult: Decimal
  /** Optional outer ant-speed mult (defaults to 1). */
  antSpeedMult?: Decimal
}

/**
 * Per-tick base production from this producer:
 *   (generated + purchased) × baseProduction × selfSpeedMult × antSpeedMult
 */
export function calculateBaseAntsToBeGenerated (input: BaseAntsToBeGeneratedInput): Decimal {
  const antSpeedMult = input.antSpeedMult ?? Decimal.fromString('1')
  return input.generated
    .add(input.purchased)
    .times(input.baseProduction)
    .times(input.selfSpeedMult)
    .times(antSpeedMult)
}
