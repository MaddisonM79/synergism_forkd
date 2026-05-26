//! Tesseract (ascension-tier) building cost + purchase loop.
//!
//! Verbatim port of
//! `legacy_core_split/packages/logic/src/mechanics/tesseractBuildings.ts`.
//!
//! Five tiers purchased with `wow_tesseracts`. The cost of the `n`-th
//! building of tier `i` is `TESSERACT_BUILDING_COSTS[i-1] * n^3`.
//! Cumulative cost to own `n` buildings is
//! `TESSERACT_BUILDING_COSTS[i-1] * (n * (n+1) / 2)^2`. All resources
//! here are plain `f64` — `Decimal` isn't needed (buying caps out
//! long before `1e308`).

use crate::events::CoreEvent;
use crate::state::TesseractBuildingsState;

/// Per-tier base cost.
const TESSERACT_BUILDING_COSTS: [f64; 5] = [1.0, 10.0, 100.0, 1_000.0, 10_000.0];

/// Five owned-counts. `None` marks a building as not-to-be-bought
/// (used by callers that want to allocate budget across a subset of
/// tiers).
pub type TesseractBuildings = [Option<f64>; 5];

// ─── calculate_tess_buildings_in_budget ────────────────────────────────────

fn buy_tess_buildings_to_cheapest_price(
    owned_buildings: TesseractBuildings,
    cheapest_price: f64,
) -> (f64, TesseractBuildings) {
    let mut buy_to_buildings: TesseractBuildings = [None; 5];
    for (i, slot) in owned_buildings.iter().enumerate() {
        if let Some(currently_owned) = *slot {
            // this_price >= cheapest_price = COSTS[i] * (buy_to + 1)^3
            // buy_to = cuberoot(cheapest_price / cost[i]) - 1; round UP
            // so the next building's price strictly exceeds
            // cheapest_price.
            let buy_to =
                ((cheapest_price / TESSERACT_BUILDING_COSTS[i]).powf(1.0 / 3.0) - 1.0).ceil();
            // cheapest_price may be below the building's current price —
            // clamp to what we already own.
            buy_to_buildings[i] = Some(currently_owned.max(buy_to));
        }
    }

    let mut price = 0.0_f64;
    for i in 0..owned_buildings.len() {
        match (owned_buildings[i], buy_to_buildings[i]) {
            (Some(buy_from), Some(buy_to)) => {
                price += TESSERACT_BUILDING_COSTS[i]
                    * ((buy_to * (buy_to + 1.0) / 2.0).powi(2)
                        - (buy_from * (buy_from + 1.0) / 2.0).powi(2));
            }
            _ => continue,
        }
    }

    (price, buy_to_buildings)
}

/// Calculate the result of repeatedly buying the cheapest tesseract
/// building, given an initial list of owned buildings and a budget.
///
/// Pure: only depends on inputs and `TESSERACT_BUILDING_COSTS`.
///
/// Documented anchor cases (from the original implementation):
///
/// ```text
/// calculate_tess_buildings_in_budget([0,0,0,0,0], 100)         -> [3,1,0,0,0]
/// calculate_tess_buildings_in_budget([None,0,0,0,0], 100)      -> [None,2,0,0,0]
/// calculate_tess_buildings_in_budget([3,1,0,0,0], 64+80-1)     -> [4,1,0,0,0]
/// calculate_tess_buildings_in_budget([3,1,0,0,0], 64+80)       -> [4,2,0,0,0]
/// calculate_tess_buildings_in_budget([9,100,100,0,100], 1000)  -> [9,100,100,1,100]
/// calculate_tess_buildings_in_budget([9,100,100,0,100], 2000)  -> [10,100,100,1,100]
/// calculate_tess_buildings_in_budget([0,0,0,0,0], 1e46)        -> runs in <1s
/// ```
#[must_use]
pub fn calculate_tess_buildings_in_budget(
    owned_buildings: TesseractBuildings,
    budget: f64,
) -> TesseractBuildings {
    // Cheapest current next-building price. If `None`, every tier is
    // opted-out.
    let mut min_current_price: Option<f64> = None;
    for (i, slot) in owned_buildings.iter().enumerate() {
        if let Some(owned) = *slot {
            let price = TESSERACT_BUILDING_COSTS[i] * (owned + 1.0).powi(3);
            min_current_price = Some(match min_current_price {
                None => price,
                Some(m) => m.min(price),
            });
        }
    }

    let min_current_price = match min_current_price {
        Some(p) if p <= budget => p,
        _ => return owned_buildings,
    };

    // Binary search for the maximum "cheapest price" the budget can
    // reach. See the original commentary — the math relies on the fact
    // that `f(cheapest_price) = cumulative cost to buy until all
    // next-prices are >= cheapest_price` is monotone in
    // `cheapest_price`.
    let mut lo = min_current_price;
    let mut hi = lo * 2.0;
    while buy_tess_buildings_to_cheapest_price(owned_buildings, hi).0 <= budget {
        lo = hi;
        hi *= 2.0;
    }
    while hi - lo > 0.5 {
        let mid = lo + (hi - lo) / 2.0;
        // Floating-point edge: mid can equal lo or hi even when
        // hi > lo. Break to avoid an infinite loop.
        if mid == lo || mid == hi {
            break;
        }
        if buy_tess_buildings_to_cheapest_price(owned_buildings, mid).0 <= budget {
            lo = mid;
        } else {
            hi = mid;
        }
    }

    let (cost, mut buildings) = buy_tess_buildings_to_cheapest_price(owned_buildings, lo);

    // Edge case: when 2..5 tiers share the cheapest price and we can
    // only afford a subset of them. Binary search hands back a state
    // where one more building is still affordable; clean it up by
    // greedily buying the cheapest a handful of times.
    let mut remaining_budget = budget - cost;
    let mut current_prices: [Option<f64>; 5] = [None; 5];
    for (i, slot) in buildings.iter().enumerate() {
        if let Some(num) = *slot {
            current_prices[i] = Some(TESSERACT_BUILDING_COSTS[i] * (num + 1.0).powi(3));
        }
    }

    for _ in 0..5 {
        let mut minimum: Option<(f64, usize)> = None;
        for (idx, slot) in current_prices.iter().enumerate() {
            if let Some(price) = *slot {
                // <= over < to prefer higher tiers when prices tie.
                minimum = Some(match minimum {
                    None => (price, idx),
                    Some((cur_price, _)) if price <= cur_price => (price, idx),
                    Some(existing) => existing,
                });
            }
        }
        match minimum {
            Some((price, idx)) if price <= remaining_budget => {
                remaining_budget -= price;
                let updated = buildings[idx].unwrap_or(0.0) + 1.0;
                buildings[idx] = Some(updated);
                current_prices[idx] = Some(TESSERACT_BUILDING_COSTS[idx] * (updated + 1.0).powi(3));
            }
            _ => break,
        }
    }

    buildings
}

// ─── get_tesseract_cost ────────────────────────────────────────────────────

/// Inputs to [`get_tesseract_cost`].
#[derive(Debug, Clone, Copy)]
pub struct GetTesseractCostInput {
    /// Number of buildings to attempt to buy.
    pub amount: f64,
    /// Limit the purchase to what `wow_tesseracts` can afford
    /// (default `true`).
    pub check_can_afford: bool,
    /// Override starting count. `None` defaults to the state's current
    /// owned for this tier.
    pub buy_from: Option<f64>,
}

/// Compute the new owned-count and tesseracts spent for a tier
/// purchase. Returns `(new_owned, cost_spent)`.
#[must_use]
pub fn get_tesseract_cost(
    index: u8,
    input: GetTesseractCostInput,
    state: &TesseractBuildingsState,
) -> (f64, f64) {
    debug_assert!(
        matches!(index, 1..=5),
        "tesseract index out of range: {index}"
    );
    let int_cost = TESSERACT_BUILDING_COSTS[usize::from(index - 1)];
    let buy_from = input
        .buy_from
        .unwrap_or_else(|| state.building(index).owned);
    let sub_cost = int_cost * (buy_from * (buy_from + 1.0) / 2.0).powi(2);

    let actual_buy = if input.check_can_afford {
        // Inverse of cumulative cost: solve
        // cost(buy_to) = wow_tesseracts + sub_cost.
        // cost(n) = int_cost * (n(n+1)/2)^2
        // → n = (-1 + sqrt(1 + 8 * sqrt(C/int_cost))) / 2
        let buy_to = (-1.0 / 2.0
            + 0.5
                * (1.0 + 8.0 * ((state.wow_tesseracts + sub_cost) / int_cost).powf(0.5)).powf(0.5))
        .floor();
        buy_to.min(buy_from + input.amount)
    } else {
        buy_from + input.amount
    };
    let actual_cost = int_cost * (actual_buy * (actual_buy + 1.0) / 2.0).powi(2) - sub_cost;
    (actual_buy, actual_cost)
}

// ─── buy_tesseract_building ────────────────────────────────────────────────

/// Inputs to [`buy_tesseract_building`].
#[derive(Debug, Clone, Copy)]
pub struct BuyTesseractBuildingInput {
    /// Which tier to buy (1..=5).
    pub index: u8,
    /// How many to buy (caller usually passes
    /// `player.tesseractbuyamount`).
    pub amount: f64,
}

/// Buy as many of the selected tesseract building as the budget
/// allows, up to `amount`. Returns the new state slice and a purchase
/// event when the count changes.
#[must_use]
pub fn buy_tesseract_building(
    state: &TesseractBuildingsState,
    input: BuyTesseractBuildingInput,
) -> (TesseractBuildingsState, Vec<CoreEvent>) {
    let mut events: Vec<CoreEvent> = Vec::new();
    let mut next = *state;
    let int_cost = TESSERACT_BUILDING_COSTS[usize::from(input.index - 1)];
    let buy_start = next.building(input.index).owned;
    let (buy_to, actual_cost) = get_tesseract_cost(
        input.index,
        GetTesseractCostInput {
            amount: input.amount,
            check_can_afford: true,
            buy_from: None,
        },
        &next,
    );

    let mut target = next.building(input.index);
    target.owned = buy_to;
    target.cost = int_cost * (1.0 + buy_to).powi(3);
    next.set_building(input.index, target);
    next.wow_tesseracts = (next.wow_tesseracts - actual_cost).max(0.0);

    if buy_to > buy_start {
        events.push(CoreEvent::TesseractBuildingsPurchased {
            index: input.index,
            before: buy_start,
            after: buy_to,
            spent: state.wow_tesseracts - next.wow_tesseracts,
        });
    }

    (next, events)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::AscendBuildingState;

    fn empty_state() -> TesseractBuildingsState {
        let zero = AscendBuildingState {
            owned: 0.0,
            cost: 1.0,
            generated: synergismforkd_bignum::Decimal::zero(),
        };
        TesseractBuildingsState {
            wow_tesseracts: 0.0,
            ascend_building_1: zero,
            ascend_building_2: zero,
            ascend_building_3: zero,
            ascend_building_4: zero,
            ascend_building_5: zero,
        }
    }

    // ─── calculate_tess_buildings_in_budget — anchor cases from TS ─────────

    #[test]
    fn anchor_all_zero_budget_100() {
        let result = calculate_tess_buildings_in_budget(
            [Some(0.0), Some(0.0), Some(0.0), Some(0.0), Some(0.0)],
            100.0,
        );
        assert_eq!(
            result,
            [Some(3.0), Some(1.0), Some(0.0), Some(0.0), Some(0.0)]
        );
    }

    #[test]
    fn anchor_tier_1_skipped_budget_100() {
        let result = calculate_tess_buildings_in_budget(
            [None, Some(0.0), Some(0.0), Some(0.0), Some(0.0)],
            100.0,
        );
        assert_eq!(result, [None, Some(2.0), Some(0.0), Some(0.0), Some(0.0)]);
    }

    #[test]
    fn anchor_3_1_0_0_0_budget_143() {
        // 64 + 80 - 1 = 143
        let result = calculate_tess_buildings_in_budget(
            [Some(3.0), Some(1.0), Some(0.0), Some(0.0), Some(0.0)],
            143.0,
        );
        assert_eq!(
            result,
            [Some(4.0), Some(1.0), Some(0.0), Some(0.0), Some(0.0)]
        );
    }

    #[test]
    fn anchor_3_1_0_0_0_budget_144() {
        // 64 + 80 = 144
        let result = calculate_tess_buildings_in_budget(
            [Some(3.0), Some(1.0), Some(0.0), Some(0.0), Some(0.0)],
            144.0,
        );
        assert_eq!(
            result,
            [Some(4.0), Some(2.0), Some(0.0), Some(0.0), Some(0.0)]
        );
    }

    #[test]
    fn anchor_high_owned_low_tier_skipped() {
        let result = calculate_tess_buildings_in_budget(
            [Some(9.0), Some(100.0), Some(100.0), Some(0.0), Some(100.0)],
            1_000.0,
        );
        assert_eq!(
            result,
            [Some(9.0), Some(100.0), Some(100.0), Some(1.0), Some(100.0)]
        );
    }

    #[test]
    fn anchor_zero_budget_returns_unchanged() {
        let owned = [Some(5.0), Some(5.0), Some(5.0), Some(5.0), Some(5.0)];
        let result = calculate_tess_buildings_in_budget(owned, 0.0);
        assert_eq!(result, owned);
    }

    // ─── buy_tesseract_building ────────────────────────────────────────────

    #[test]
    fn buy_is_noop_with_zero_tesseracts() {
        let state = empty_state();
        let (next, events) = buy_tesseract_building(
            &state,
            BuyTesseractBuildingInput {
                index: 1,
                amount: 10.0,
            },
        );
        assert_eq!(next.ascend_building_1.owned, 0.0);
        assert!(events.is_empty());
    }

    #[test]
    fn buy_purchases_when_affordable() {
        let state = TesseractBuildingsState {
            wow_tesseracts: 100.0,
            ..empty_state()
        };
        let (next, events) = buy_tesseract_building(
            &state,
            BuyTesseractBuildingInput {
                index: 1,
                amount: 100.0,
            },
        );
        assert!(next.ascend_building_1.owned > 0.0);
        assert!(next.wow_tesseracts < state.wow_tesseracts);
        assert_eq!(events.len(), 1);
        match &events[0] {
            CoreEvent::TesseractBuildingsPurchased {
                index,
                before,
                after,
                ..
            } => {
                assert_eq!(*index, 1);
                assert_eq!(*before, 0.0);
                assert_eq!(*after, next.ascend_building_1.owned);
            }
            other => panic!("expected TesseractBuildingsPurchased, got {other:?}"),
        }
    }

    #[test]
    fn buy_targets_only_the_requested_index() {
        let state = TesseractBuildingsState {
            wow_tesseracts: 100_000.0,
            ..empty_state()
        };
        let (next, _) = buy_tesseract_building(
            &state,
            BuyTesseractBuildingInput {
                index: 3,
                amount: 100.0,
            },
        );
        assert!(next.ascend_building_3.owned > 0.0);
        assert_eq!(next.ascend_building_1.owned, 0.0);
        assert_eq!(next.ascend_building_2.owned, 0.0);
        assert_eq!(next.ascend_building_4.owned, 0.0);
        assert_eq!(next.ascend_building_5.owned, 0.0);
    }

    #[test]
    fn buy_amount_caps_purchase() {
        let state = TesseractBuildingsState {
            wow_tesseracts: 1e20, // enormous budget
            ..empty_state()
        };
        let (next, _) = buy_tesseract_building(
            &state,
            BuyTesseractBuildingInput {
                index: 1,
                amount: 5.0,
            },
        );
        // Capped at amount = 5 even with huge budget.
        assert_eq!(next.ascend_building_1.owned, 5.0);
    }
}
