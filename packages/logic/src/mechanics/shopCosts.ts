// Shop upgrade cost formula, lifted from packages/web_ui/src/Shop.ts. Pure
// math given the per-upgrade snapshot — the web_ui side owns the shop data
// table and feeds the relevant fields in.
//
// Two branches:
//   - Consumables and one-shot upgrades (maxLevel === 1): cost is just
//     `price`. They have no level-based scaling.
//   - Stacked upgrades: cost is `price + priceIncrease * currentLevel` —
//     linear in current level.

export interface ShopCostInput {
  /** shopUpgrades[k].type === shopUpgradeTypes.CONSUMABLE. Consumables flat-price. */
  isConsumable: boolean
  /** shopUpgrades[k].maxLevel. Single-purchase upgrades (maxLevel === 1) flat-price. */
  maxLevel: number
  /** shopUpgrades[k].price — base cost. */
  price: number
  /** shopUpgrades[k].priceIncrease — per-level cost increment. */
  priceIncrease: number
  /** player.shopUpgrades[k] — the player's currently-owned level. */
  currentLevel: number
}

/**
 * Cost to purchase the next level of a shop upgrade. Flat `price` for
 * consumables and one-shot upgrades; linear scaling for stacked upgrades.
 */
export function shopCost (input: ShopCostInput): number {
  if (input.isConsumable || input.maxLevel === 1) {
    return input.price
  }
  return input.price + input.priceIncrease * input.currentLevel
}
