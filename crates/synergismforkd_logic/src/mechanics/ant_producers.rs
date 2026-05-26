//! Ant-producer data + cost solvers + base-production formula.
//!
//! Verbatim port of
//! `legacy_core_split/packages/logic/src/mechanics/antProducers.ts`.
//! The data table is indexed `0..=8` to match the legacy
//! `AntProducers` enum (Workers=0 .. HolySpirit=8). UI-only fields
//! (additional-texts closures) stay in the UI tier — only the pure
//! `baseCost / costIncrease / baseProduction / color / produces`
//! fields move here.
//!
//! Cost shape differs from `antUpgrades`: per-producer
//! `cost_increase` is the geometric base (3 for Workers, 10 for
//! Breeders, etc.), not a log-10 exp. Formula:
//! `cost_to_buy_nth = base_cost × cost_increase^(N - 1)`.

use synergismforkd_bignum::Decimal;

/// Pure data fields for one ant producer.
#[derive(Debug, Clone, Copy)]
pub struct AntProducerData {
    /// Base cost (Workers=1, Breeders=10, etc.).
    pub base_cost: Decimal,
    /// Geometric multiplier per purchase:
    /// `cost_to_buy_nth = base_cost × cost_increase^(N - 1)`.
    pub cost_increase: f64,
    /// Per-producer baseline production rate.
    pub base_production: Decimal,
    /// UI hint color — pure string, kept here for completeness.
    pub color: &'static str,
    /// Index of the producer this one generates; `None` for
    /// Workers (top of chain).
    pub produces: Option<u8>,
}

/// 9-entry data table, indexed `0..=8`. The Decimal fields are built
/// on demand because some values (`1e145`, `1e700`, …, `1e1000000`)
/// far exceed `f64::MAX` and need
/// [`Decimal::from_mantissa_exponent`].
#[must_use]
pub fn ant_producer_data(index: u8) -> AntProducerData {
    debug_assert!(matches!(index, 0..=8), "ant producer index out of range: {index}");
    match index {
        // Workers (0) — top of chain
        0 => AntProducerData {
            base_cost: Decimal::from_finite(1.0),
            cost_increase: 3.0,
            base_production: Decimal::from_finite(0.01),
            color: "#AB8654",
            produces: None,
        },
        // Breeders (1) → produces Workers
        1 => AntProducerData {
            base_cost: Decimal::from_finite(10.0),
            cost_increase: 10.0,
            base_production: Decimal::from_finite(1.5e-4),
            color: "#B77D48",
            produces: Some(0),
        },
        // MetaBreeders (2) → produces Breeders
        2 => AntProducerData {
            base_cost: Decimal::from_finite(1e5),
            cost_increase: 1e2,
            base_production: Decimal::from_finite(5e-6),
            color: "#C2783D",
            produces: Some(1),
        },
        // MegaBreeders (3) → produces MetaBreeders
        3 => AntProducerData {
            base_cost: Decimal::from_finite(1e12),
            cost_increase: 1e4,
            base_production: Decimal::from_finite(3e-5),
            color: "#CA7035",
            produces: Some(2),
        },
        // Queens (4) → produces MegaBreeders
        4 => AntProducerData {
            base_cost: Decimal::from_mantissa_exponent(1.0, 145.0),
            cost_increase: 1e8,
            base_production: Decimal::from_mantissa_exponent(1.0, -30.0),
            color: "#D26B2D",
            produces: Some(3),
        },
        // LordRoyals (5) → produces Queens
        5 => AntProducerData {
            base_cost: Decimal::from_mantissa_exponent(1.0, 700.0),
            cost_increase: 1e16,
            base_production: Decimal::from_mantissa_exponent(1.0, -90.0),
            color: "#DC6623",
            produces: Some(4),
        },
        // Almighties (6) → produces LordRoyals
        6 => AntProducerData {
            base_cost: Decimal::from_mantissa_exponent(1.0, 5_000.0),
            cost_increase: 1e32,
            base_production: Decimal::from_mantissa_exponent(1.0, -600.0),
            color: "#E76118",
            produces: Some(5),
        },
        // Disciples (7) → produces Almighties
        7 => AntProducerData {
            base_cost: Decimal::from_mantissa_exponent(1.0, 25_000.0),
            cost_increase: 1e64,
            base_production: Decimal::from_mantissa_exponent(1.0, -3_500.0),
            color: "#F65D09",
            produces: Some(6),
        },
        // HolySpirit (8) → produces Disciples
        _ => AntProducerData {
            base_cost: Decimal::from_mantissa_exponent(1.0, 1_000_000.0),
            cost_increase: 1e128,
            base_production: Decimal::from_mantissa_exponent(1.0, -110_000.0),
            color: "#FFFFFF",
            produces: Some(7),
        },
    }
}

// ─── Cost solvers ─────────────────────────────────────────────────────────

/// Inputs to [`get_cost_next_ant_producer`].
#[derive(Debug, Clone, Copy)]
pub struct AntProducerCostInput {
    /// `ant_producer_data(index).base_cost`.
    pub base_cost: Decimal,
    /// `ant_producer_data(index).cost_increase`.
    pub cost_increase: f64,
    /// `player.ants.producers[index].purchased`.
    pub purchased: f64,
}

/// Cost of buying the next producer.
/// `cost-to-reach-N = base_cost × cost_increase^N`; delta cost is
/// `next_cost - last_cost` (with `last_cost = 0` when
/// `purchased == 0`).
#[must_use]
pub fn get_cost_next_ant_producer(input: &AntProducerCostInput) -> Decimal {
    let cost_increase = Decimal::from_finite(input.cost_increase);
    let next_cost = input.base_cost * cost_increase.pow(Decimal::from_finite(input.purchased));
    let last_cost = if input.purchased > 0.0 {
        input.base_cost * cost_increase.pow(Decimal::from_finite(input.purchased - 1.0))
    } else {
        Decimal::zero()
    };
    next_cost - last_cost
}

/// Inputs to [`get_max_purchasable_ant_producers`].
#[derive(Debug, Clone, Copy)]
pub struct AntProducerMaxPurchasableInput {
    /// `ant_producer_data(index).base_cost`.
    pub base_cost: Decimal,
    /// `ant_producer_data(index).cost_increase`.
    pub cost_increase: f64,
    /// `player.ants.producers[index].purchased`.
    pub purchased: f64,
    /// `player.ants.crumbs` — budget to spend.
    pub budget: Decimal,
}

/// Max producer count reachable with `budget`. Re-adds sunk cost
/// (current spend) to budget then solves the inverse:
///
/// ```text
/// N = 1 + floor(log_{cost_increase}(real_budget / base_cost))
/// ```
///
/// Floored at 0.
#[must_use]
pub fn get_max_purchasable_ant_producers(input: &AntProducerMaxPurchasableInput) -> f64 {
    let cost_increase = Decimal::from_finite(input.cost_increase);
    let sunk_cost = if input.purchased > 0.0 {
        input.base_cost * cost_increase.pow(Decimal::from_finite(input.purchased - 1.0))
    } else {
        Decimal::zero()
    };
    let real_budget = input.budget + sunk_cost;
    0.0_f64.max(
        1.0 + (real_budget / input.base_cost)
            .log(cost_increase)
            .to_number()
            .floor(),
    )
}

/// Inputs to [`get_cost_max_ant_producers`].
#[derive(Debug, Clone, Copy)]
pub struct AntProducerMaxCostInput {
    /// `ant_producer_data(index).base_cost`.
    pub base_cost: Decimal,
    /// `ant_producer_data(index).cost_increase`.
    pub cost_increase: f64,
    /// `player.ants.producers[index].purchased`.
    pub purchased: f64,
    /// Result of [`get_max_purchasable_ant_producers`] for the same
    /// inputs.
    pub max_buyable: f64,
}

/// Total cost to buy from current `purchased` up to `max_buyable`.
/// Subtracts the already-paid sunk cost (cost-of-current-N).
#[must_use]
pub fn get_cost_max_ant_producers(input: &AntProducerMaxCostInput) -> Decimal {
    let cost_increase = Decimal::from_finite(input.cost_increase);
    let spent = if input.purchased > 0.0 {
        cost_increase.pow(Decimal::from_finite(input.purchased - 1.0)) * input.base_cost
    } else {
        Decimal::zero()
    };
    let max_cost =
        cost_increase.pow(Decimal::from_finite(input.max_buyable - 1.0)) * input.base_cost;
    max_cost - spent
}

// ─── Base production rate ─────────────────────────────────────────────────

/// Inputs to [`calculate_base_ants_to_be_generated`].
#[derive(Debug, Clone, Copy)]
pub struct BaseAntsToBeGeneratedInput {
    /// `player.ants.producers[index].generated`.
    pub generated: Decimal,
    /// `player.ants.producers[index].purchased`.
    pub purchased: f64,
    /// `ant_producer_data(index).base_production`.
    pub base_production: Decimal,
    /// `calculate_self_speed_from_mastery(index)` — the per-producer
    /// mastery mult.
    pub self_speed_mult: Decimal,
    /// Optional outer ant-speed mult (defaults to `1`).
    pub ant_speed_mult: Option<Decimal>,
}

/// Per-tick base production from this producer:
///
/// ```text
/// (generated + purchased) × base_production × self_speed_mult × ant_speed_mult
/// ```
#[must_use]
pub fn calculate_base_ants_to_be_generated(input: &BaseAntsToBeGeneratedInput) -> Decimal {
    let ant_speed_mult = input.ant_speed_mult.unwrap_or(Decimal::one());
    (input.generated + Decimal::from_finite(input.purchased))
        * input.base_production
        * input.self_speed_mult
        * ant_speed_mult
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workers_data_is_at_index_0() {
        let data = ant_producer_data(0);
        assert_eq!(data.base_cost.to_number(), 1.0);
        assert_eq!(data.cost_increase, 3.0);
        assert_eq!(data.produces, None);
    }

    #[test]
    fn breeders_produce_workers() {
        let data = ant_producer_data(1);
        assert_eq!(data.produces, Some(0));
    }

    #[test]
    fn next_ant_producer_cost_at_zero_is_base_cost() {
        let result = get_cost_next_ant_producer(&AntProducerCostInput {
            base_cost: Decimal::from_finite(10.0),
            cost_increase: 3.0,
            purchased: 0.0,
        });
        assert_eq!(result.to_number(), 10.0);
    }

    #[test]
    fn next_ant_producer_cost_scales_geometrically() {
        // base=10, ci=3, purchased=2 → next_cost = 10*3^2 = 90; last = 10*3 = 30
        // delta = 60
        let result = get_cost_next_ant_producer(&AntProducerCostInput {
            base_cost: Decimal::from_finite(10.0),
            cost_increase: 3.0,
            purchased: 2.0,
        });
        assert_eq!(result.to_number(), 60.0);
    }

    #[test]
    fn max_purchasable_with_zero_budget_is_zero() {
        let result = get_max_purchasable_ant_producers(&AntProducerMaxPurchasableInput {
            base_cost: Decimal::from_finite(10.0),
            cost_increase: 3.0,
            purchased: 0.0,
            budget: Decimal::zero(),
        });
        assert!(result <= 0.0);
    }

    #[test]
    fn base_ants_generated_uses_all_multipliers() {
        // (10 + 5) * 0.01 * 2.0 * 3.0 = 0.9
        let result = calculate_base_ants_to_be_generated(&BaseAntsToBeGeneratedInput {
            generated: Decimal::from_finite(10.0),
            purchased: 5.0,
            base_production: Decimal::from_finite(0.01),
            self_speed_mult: Decimal::from_finite(2.0),
            ant_speed_mult: Some(Decimal::from_finite(3.0)),
        });
        assert!((result.to_number() - 0.9).abs() < 1e-12);
    }

    #[test]
    fn base_ants_generated_defaults_speed_mult_to_one() {
        let result = calculate_base_ants_to_be_generated(&BaseAntsToBeGeneratedInput {
            generated: Decimal::from_finite(10.0),
            purchased: 0.0,
            base_production: Decimal::from_finite(0.01),
            self_speed_mult: Decimal::from_finite(2.0),
            ant_speed_mult: None,
        });
        // 10 * 0.01 * 2 * 1 = 0.2
        assert!((result.to_number() - 0.2).abs() < 1e-12);
    }
}
