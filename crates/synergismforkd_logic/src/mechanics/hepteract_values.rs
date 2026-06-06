//! Hepteract effective-value and cap helpers.
//!
//! Verbatim port of
//! `legacy/core_split/packages/logic/src/mechanics/hepteractValues.ts`
//! (lifted from the legacy `packages/web_ui/src/Hepteracts.ts`).
//!
//! - [`hepteract_effective`]: applies the diminishing-returns formula
//!   past `LIMIT`. Special-cased for `quark` (which uses a custom
//!   non-polynomial formula owned by the UI tier — here we just pass
//!   `BAL` through when the caller flags it as the quark hept).
//! - [`hepteract_cap`]: `BASE_CAP * 2^TIMES_CAP_EXTENDED` — the
//!   player's expanded cap before any Exalt 3 doubling.
//! - [`hepteract_final_cap`]:
//!   `hepteract_cap * (limitedAscensions Exalt 3 reward active ? 2 :
//!   1)` — the post-Exalt cap actually used by the UI.
//!
//! Beyond the value/cap helpers, this module owns the two manual
//! actions — [`buy_hepteract_craft`] (multi-resource craft toward the
//! cap) and [`buy_hepteract_expand`] (spend a full bar to double the
//! cap). The per-craft conversion constants stay UI-tier; the caller
//! passes them in [`HepteractConversions`].

use smallvec::SmallVec;
use synergismforkd_bignum::Decimal;

use crate::events::CoreEvent;
use crate::state::{CubeBalancesState, HepteractCraft, HepteractsState};

/// Inputs to [`hepteract_effective`].
#[derive(Debug, Clone, Copy)]
pub struct HepteractEffectiveInput {
    /// `hepteracts[k].BAL` — raw accumulated value.
    pub raw_amount: f64,
    /// `hepteracts[k].LIMIT` — threshold past which DR applies.
    pub limit: f64,
    /// `hepteracts[k].DR + hepteracts[k].DR_INCREASE()` — combined
    /// diminishing-returns exponent. The UI tier evaluates
    /// `DR_INCREASE` since it can depend on upgrade state; this module
    /// gets the resolved scalar.
    pub dr_exponent: f64,
    /// `true` when this is the `quark` hept. Quark hept has a custom
    /// non-polynomial formula owned by the UI tier; logic just passes
    /// `BAL` through.
    pub is_quark: bool,
}

/// Effective hepteract value with diminishing returns past `LIMIT`.
///
/// - **quark**: just returns `raw_amount` (the UI tier owns the custom
///   formula).
/// - **`raw_amount ≤ LIMIT`**: linear, returns `raw_amount`.
/// - **`raw_amount > LIMIT`**:
///   `LIMIT * (raw_amount / LIMIT) ^ dr_exponent` — the value past
///   `LIMIT` is softened by the DR exponent (`dr_exponent < 1` for
///   most hepts, so growth past `LIMIT` is sub-linear).
#[must_use]
pub fn hepteract_effective(input: &HepteractEffectiveInput) -> f64 {
    if input.is_quark {
        return input.raw_amount;
    }
    let mut effective_value = input.raw_amount.min(input.limit);
    if input.raw_amount > input.limit {
        effective_value *= (input.raw_amount / input.limit).powf(input.dr_exponent);
    }
    effective_value
}

/// Player's expanded hepteract cap before the Exalt 3 doubling.
/// `BASE_CAP * 2^TIMES_CAP_EXTENDED` — each expansion doubles the cap.
#[must_use]
pub fn hepteract_cap(base_cap: f64, times_cap_extended: f64) -> f64 {
    2.0_f64.powf(times_cap_extended) * base_cap
}

/// The cap actually used by the UI — multiplies [`hepteract_cap`] by 2
/// if the limitedAscensions (Exalt 3) `hepteractCap` reward is active,
/// else 1.
#[must_use]
pub fn hepteract_final_cap(
    base_cap: f64,
    times_cap_extended: f64,
    exalt_3_hepteract_cap: bool,
) -> f64 {
    let special_multiplier = if exalt_3_hepteract_cap { 2.0 } else { 1.0 };
    hepteract_cap(base_cap, times_cap_extended) * special_multiplier
}

// ─── Manual buy: craft + expand ───────────────────────────────────────────

/// Selects which of the eight hepteract crafts a buy targets. Discriminants
/// match the [`CoreEvent`] `index` and the `HepteractsState` field order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum HepteractKind {
    /// `chronos` craft.
    Chronos = 0,
    /// `hyperrealism` craft.
    Hyperrealism = 1,
    /// `quark` craft.
    Quark = 2,
    /// `challenge` craft.
    Challenge = 3,
    /// `abyss` craft.
    Abyss = 4,
    /// `accelerator` craft.
    Accelerator = 5,
    /// `acceleratorBoost` craft.
    AcceleratorBoost = 6,
    /// `multiplier` craft.
    Multiplier = 7,
}

fn craft_mut(hepteracts: &mut HepteractsState, kind: HepteractKind) -> &mut HepteractCraft {
    match kind {
        HepteractKind::Chronos => &mut hepteracts.chronos,
        HepteractKind::Hyperrealism => &mut hepteracts.hyperrealism,
        HepteractKind::Quark => &mut hepteracts.quark,
        HepteractKind::Challenge => &mut hepteracts.challenge,
        HepteractKind::Abyss => &mut hepteracts.abyss,
        HepteractKind::Accelerator => &mut hepteracts.accelerator,
        HepteractKind::AcceleratorBoost => &mut hepteracts.accelerator_boost,
        HepteractKind::Multiplier => &mut hepteracts.multiplier,
    }
}

/// Floored craft units a Decimal resource affords at `per_unit_cost`. A
/// `<= 0` cost means the craft doesn't consume the resource, so it imposes no
/// limit (`+∞`).
fn resource_craft_limit(available: Decimal, per_unit_cost: f64) -> f64 {
    if per_unit_cost <= 0.0 {
        f64::INFINITY
    } else {
        (available.to_number() / per_unit_cost).floor()
    }
}

/// Subtract `amount` from a Decimal resource, clamped at zero. Skips the work
/// (and the `Decimal` allocation) when nothing is owed.
fn spend_decimal(resource: &mut Decimal, amount: f64) {
    if amount > 0.0 {
        *resource = (*resource - Decimal::from_finite(amount)).max(Decimal::zero());
    }
}

/// Per-craft conversion constants (the legacy `HEPTERACT_CONVERSION` +
/// `OTHER_CONVERSIONS`), supplied by the caller (UI-tier data table). A `0`
/// rate means the craft does not consume that resource. Every rate except
/// `worlds` is additionally scaled by the craft-cost multiplier.
#[derive(Debug, Clone, Copy, Default)]
pub struct HepteractConversions {
    /// `HEPTERACT_CONVERSION` — wow-abyssals per craft unit.
    pub hepteract: f64,
    /// `OTHER_CONVERSIONS.obtainium`.
    pub obtainium: f64,
    /// `OTHER_CONVERSIONS.offerings`.
    pub offerings: f64,
    /// `OTHER_CONVERSIONS.worlds` — the one resource spent without the
    /// craft-cost multiplier.
    pub worlds: f64,
    /// `OTHER_CONVERSIONS.wowCubes`.
    pub wow_cubes: f64,
    /// `OTHER_CONVERSIONS.wowTesseracts`.
    pub wow_tesseracts: f64,
    /// `OTHER_CONVERSIONS.wowHypercubes`.
    pub wow_hypercubes: f64,
    /// `OTHER_CONVERSIONS.wowPlatonicCubes`.
    pub wow_platonic_cubes: f64,
}

/// Inputs to [`buy_hepteract_craft`].
#[derive(Debug, Clone, Copy)]
pub struct BuyHepteractCraftInput {
    /// Which craft to top up.
    pub kind: HepteractKind,
    /// Per-craft conversion rates (UI-tier data table).
    pub conversions: HepteractConversions,
    /// `calculateSingularityDebuff('Hepteract Costs')` — multiplies every cost
    /// except `worlds`. Caller-provided (UI-tier); `1.0` at low singularity.
    pub craft_cost_multi: f64,
    /// `getSingularityChallengeEffect('limitedAscensions', 'hepteractCap')`
    /// (Exalt 3) — doubles the craft ceiling when active. `false` outside
    /// Exalt 3.
    pub exalt_3_cap: bool,
    /// Units the player asked to craft (the legacy prompt). Ignored when `max`
    /// is set.
    pub requested_amount: f64,
    /// `true` = craft-max: fill toward the cap with everything affordable.
    pub max: bool,
}

/// Craft toward a hepteract's cap by spending wow-abyssals plus the craft's
/// `OTHER_CONVERSIONS` resources — a port of the legacy `craftHepteracts`
/// core. The craftable amount is the floor-min over every spendable resource
/// (`getCraftableHepteractAmount`) capped by the remaining bar room; `max`
/// fills it, otherwise `requested_amount` bounds it. Emits
/// [`CoreEvent::HepteractCrafted`].
///
/// Faithful-at-current-state deferrals: the unlock check (`UNLOCKED()`) and
/// the integer/positive validation of the requested amount are UI-tier, so
/// the caller gates on them; the per-craft conversion table and the
/// singularity-debuff multiplier are caller-provided.
#[must_use]
pub fn buy_hepteract_craft(
    hepteracts: &mut HepteractsState,
    cube_balances: &mut CubeBalancesState,
    obtainium: &mut Decimal,
    offerings: &mut Decimal,
    worlds: &mut Decimal,
    input: BuyHepteractCraftInput,
) -> SmallVec<[CoreEvent; 4]> {
    let mut events = SmallVec::new();
    let c = input.conversions;
    let multi = input.craft_cost_multi;

    let craft = craft_mut(hepteracts, input.kind);
    let final_cap = craft.cap * if input.exalt_3_cap { 2.0 } else { 1.0 };
    let room = final_cap - craft.bal;
    if room <= 0.0 {
        return events;
    }

    // getCraftableHepteractAmount: the floor-min over each spendable resource,
    // plus the remaining cap room (unfloored, per legacy).
    let craftable = room
        .min((cube_balances.wow_abyssals / (c.hepteract * multi)).floor())
        .min(resource_craft_limit(*obtainium, c.obtainium * multi))
        .min(resource_craft_limit(*offerings, c.offerings * multi))
        .min(resource_craft_limit(*worlds, c.worlds)) // worlds: no cost multiplier
        .min(resource_craft_limit(cube_balances.wow_cubes, c.wow_cubes * multi))
        .min(resource_craft_limit(
            cube_balances.wow_tesseracts,
            c.wow_tesseracts * multi,
        ))
        .min(resource_craft_limit(
            cube_balances.wow_hypercubes,
            c.wow_hypercubes * multi,
        ))
        .min(resource_craft_limit(
            cube_balances.wow_platonic_cubes,
            c.wow_platonic_cubes * multi,
        ));

    let requested = if input.max {
        craftable
    } else {
        input.requested_amount
    };
    let amount = craftable.min(requested);
    if amount <= 0.0 {
        return events;
    }

    let before = craft.bal;
    craft.bal = final_cap.min(craft.bal + amount);

    // Spend. wow-abyssals and the Decimal `OTHER_CONVERSIONS` use the cost
    // multiplier; `worlds` is the one exception (no multiplier).
    cube_balances.wow_abyssals =
        (cube_balances.wow_abyssals - amount * c.hepteract * multi).max(0.0);
    spend_decimal(obtainium, amount * c.obtainium * multi);
    spend_decimal(offerings, amount * c.offerings * multi);
    spend_decimal(worlds, amount * c.worlds);
    spend_decimal(&mut cube_balances.wow_cubes, amount * c.wow_cubes * multi);
    spend_decimal(
        &mut cube_balances.wow_tesseracts,
        amount * c.wow_tesseracts * multi,
    );
    spend_decimal(
        &mut cube_balances.wow_hypercubes,
        amount * c.wow_hypercubes * multi,
    );
    spend_decimal(
        &mut cube_balances.wow_platonic_cubes,
        amount * c.wow_platonic_cubes * multi,
    );

    events.push(CoreEvent::HepteractCrafted {
        index: input.kind as u32,
        before,
        after: craft.bal,
        amount,
    });
    events
}

/// Inputs to [`buy_hepteract_expand`].
#[derive(Debug, Clone, Copy)]
pub struct BuyHepteractExpandInput {
    /// Which craft's cap to double.
    pub kind: HepteractKind,
}

/// Double a hepteract craft's cap by spending one full (non-Exalt) bar — a
/// port of the legacy `expandHepteracts`. Requires the bar to be full to
/// `cap`; spends `cap` of the balance and doubles `cap` (the legacy
/// `TIMES_CAP_EXTENDED += 1`, since `cap == BASE_CAP * 2^TIMES_CAP_EXTENDED`).
/// Emits [`CoreEvent::HepteractCapExpanded`]. The unlock check is UI-tier
/// (caller-gated).
#[must_use]
pub fn buy_hepteract_expand(
    hepteracts: &mut HepteractsState,
    input: BuyHepteractExpandInput,
) -> SmallVec<[CoreEvent; 4]> {
    let mut events = SmallVec::new();
    let craft = craft_mut(hepteracts, input.kind);
    if craft.cap <= 0.0 || craft.bal < craft.cap {
        return events;
    }
    craft.bal = (craft.bal - craft.cap).max(0.0);
    craft.cap *= 2.0;
    events.push(CoreEvent::HepteractCapExpanded {
        index: input.kind as u32,
        bal_after: craft.bal,
        cap_after: craft.cap,
    });
    events
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hepteract_effective_quark_passes_through() {
        let input = HepteractEffectiveInput {
            raw_amount: 1e18,
            limit: 100.0,
            dr_exponent: 0.5,
            is_quark: true,
        };
        assert_eq!(hepteract_effective(&input), 1e18);
    }

    #[test]
    fn hepteract_effective_below_limit_is_linear() {
        let input = HepteractEffectiveInput {
            raw_amount: 50.0,
            limit: 100.0,
            dr_exponent: 0.5,
            is_quark: false,
        };
        assert_eq!(hepteract_effective(&input), 50.0);
    }

    #[test]
    fn hepteract_effective_above_limit_uses_dr() {
        // raw = 400, limit = 100, dr = 0.5 → 100 * (400/100)^0.5 = 200
        let input = HepteractEffectiveInput {
            raw_amount: 400.0,
            limit: 100.0,
            dr_exponent: 0.5,
            is_quark: false,
        };
        assert!((hepteract_effective(&input) - 200.0).abs() < 1e-9);
    }

    #[test]
    fn hepteract_effective_at_exact_limit() {
        // raw = limit → returns limit (no DR branch)
        let input = HepteractEffectiveInput {
            raw_amount: 100.0,
            limit: 100.0,
            dr_exponent: 0.5,
            is_quark: false,
        };
        assert_eq!(hepteract_effective(&input), 100.0);
    }

    #[test]
    fn hepteract_cap_doubles_per_extension() {
        assert_eq!(hepteract_cap(1_000.0, 0.0), 1_000.0);
        assert_eq!(hepteract_cap(1_000.0, 1.0), 2_000.0);
        assert_eq!(hepteract_cap(1_000.0, 5.0), 32_000.0);
    }

    #[test]
    fn hepteract_final_cap_exalt_3_doubles() {
        let without = hepteract_final_cap(1_000.0, 3.0, false);
        let with_exalt = hepteract_final_cap(1_000.0, 3.0, true);
        // 1000 * 2^3 = 8000; with exalt → 16000
        assert_eq!(without, 8_000.0);
        assert_eq!(with_exalt, 16_000.0);
    }

    // ─── Manual buy: craft + expand ───────────────────────────────────────

    fn hepteracts_with_chronos(bal: f64, cap: f64) -> HepteractsState {
        let mut h = HepteractsState::default();
        h.chronos.bal = bal;
        h.chronos.cap = cap;
        h
    }

    fn abyssals(amount: f64) -> CubeBalancesState {
        CubeBalancesState {
            wow_abyssals: amount,
            ..CubeBalancesState::default()
        }
    }

    fn craft_input(
        conversions: HepteractConversions,
        requested: f64,
        max: bool,
    ) -> BuyHepteractCraftInput {
        BuyHepteractCraftInput {
            kind: HepteractKind::Chronos,
            conversions,
            craft_cost_multi: 1.0,
            exalt_3_cap: false,
            requested_amount: requested,
            max,
        }
    }

    #[test]
    fn buy_hepteract_craft_crafts_and_spends() {
        // cap 100, 50 abyssals, 1 abyssal/unit: requested 10 → craft 10.
        let mut h = hepteracts_with_chronos(0.0, 100.0);
        let mut cb = abyssals(50.0);
        let (mut ob, mut of, mut wo) = (Decimal::zero(), Decimal::zero(), Decimal::zero());
        let conv = HepteractConversions {
            hepteract: 1.0,
            ..HepteractConversions::default()
        };
        let events = buy_hepteract_craft(
            &mut h,
            &mut cb,
            &mut ob,
            &mut of,
            &mut wo,
            craft_input(conv, 10.0, false),
        );
        assert_eq!(h.chronos.bal, 10.0);
        assert!((cb.wow_abyssals - 40.0).abs() < 1e-9);
        assert!(matches!(
            events.as_slice(),
            [CoreEvent::HepteractCrafted { index: 0, amount, .. }] if (amount - 10.0).abs() < 1e-9
        ));
    }

    #[test]
    fn buy_hepteract_craft_max_fills_to_affordable() {
        // max with only 30 abyssals (1/unit) → craft 30, spend all.
        let mut h = hepteracts_with_chronos(0.0, 100.0);
        let mut cb = abyssals(30.0);
        let (mut ob, mut of, mut wo) = (Decimal::zero(), Decimal::zero(), Decimal::zero());
        let conv = HepteractConversions {
            hepteract: 1.0,
            ..HepteractConversions::default()
        };
        let events = buy_hepteract_craft(
            &mut h,
            &mut cb,
            &mut ob,
            &mut of,
            &mut wo,
            craft_input(conv, 0.0, true),
        );
        assert_eq!(h.chronos.bal, 30.0);
        assert_eq!(cb.wow_abyssals, 0.0);
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn buy_hepteract_craft_capped_at_room() {
        // bal 95 of cap 100: only 5 of room left even though 50 abyssals afford more.
        let mut h = hepteracts_with_chronos(95.0, 100.0);
        let mut cb = abyssals(50.0);
        let (mut ob, mut of, mut wo) = (Decimal::zero(), Decimal::zero(), Decimal::zero());
        let conv = HepteractConversions {
            hepteract: 1.0,
            ..HepteractConversions::default()
        };
        let events = buy_hepteract_craft(
            &mut h,
            &mut cb,
            &mut ob,
            &mut of,
            &mut wo,
            craft_input(conv, 20.0, false),
        );
        assert_eq!(h.chronos.bal, 100.0);
        assert!((cb.wow_abyssals - 45.0).abs() < 1e-9);
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn buy_hepteract_craft_multi_resource_min_binds() {
        // hepteract 1/unit (100 abyssals → 100) + obtainium 2/unit (10 → 5):
        // the obtainium limit binds, so 5 craft.
        let mut h = hepteracts_with_chronos(0.0, 100.0);
        let mut cb = abyssals(100.0);
        let (mut ob, mut of, mut wo) =
            (Decimal::from_finite(10.0), Decimal::zero(), Decimal::zero());
        let conv = HepteractConversions {
            hepteract: 1.0,
            obtainium: 2.0,
            ..HepteractConversions::default()
        };
        let events = buy_hepteract_craft(
            &mut h,
            &mut cb,
            &mut ob,
            &mut of,
            &mut wo,
            craft_input(conv, 10.0, false),
        );
        assert_eq!(h.chronos.bal, 5.0);
        assert!((cb.wow_abyssals - 95.0).abs() < 1e-9);
        assert_eq!(ob.to_number(), 0.0);
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn buy_hepteract_craft_at_cap_is_noop() {
        let mut h = hepteracts_with_chronos(100.0, 100.0);
        let mut cb = abyssals(50.0);
        let (mut ob, mut of, mut wo) = (Decimal::zero(), Decimal::zero(), Decimal::zero());
        let conv = HepteractConversions {
            hepteract: 1.0,
            ..HepteractConversions::default()
        };
        let events = buy_hepteract_craft(
            &mut h,
            &mut cb,
            &mut ob,
            &mut of,
            &mut wo,
            craft_input(conv, 10.0, false),
        );
        assert_eq!(h.chronos.bal, 100.0);
        assert_eq!(cb.wow_abyssals, 50.0);
        assert!(events.is_empty());
    }

    #[test]
    fn buy_hepteract_expand_doubles_cap() {
        let mut h = hepteracts_with_chronos(100.0, 100.0);
        let events = buy_hepteract_expand(
            &mut h,
            BuyHepteractExpandInput {
                kind: HepteractKind::Chronos,
            },
        );
        assert_eq!(h.chronos.bal, 0.0);
        assert_eq!(h.chronos.cap, 200.0);
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn buy_hepteract_expand_not_full_is_noop() {
        let mut h = hepteracts_with_chronos(50.0, 100.0);
        let events = buy_hepteract_expand(
            &mut h,
            BuyHepteractExpandInput {
                kind: HepteractKind::Chronos,
            },
        );
        assert_eq!(h.chronos.cap, 100.0);
        assert!(events.is_empty());
    }

    #[test]
    fn buy_hepteract_expand_locked_is_noop() {
        // Default (locked) craft: cap 0 → no expansion.
        let mut h = HepteractsState::default();
        let events = buy_hepteract_expand(
            &mut h,
            BuyHepteractExpandInput {
                kind: HepteractKind::Chronos,
            },
        );
        assert!(events.is_empty());
    }
}
