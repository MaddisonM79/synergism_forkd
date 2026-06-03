//! Per-tick ant production. Direct port of
//! `legacy/core_split/packages/logic/src/tick/antGeneration.ts`.
//!
//! The producer loop runs from HolySpirit (8) down to Breeders (1): each
//! tier credits the tier directly below via `baseGeneration × dt`. The
//! loop has a real iteration dependency — iteration `N` writes
//! `updated_generated[produces]`, which a later (lower-tier) iteration
//! reads — so a local array is mutated in place across iterations. After
//! the loop, Workers (0) produces crumbs into the three crumb
//! accumulators. No event (this is generation, called from Phase 4).
//!
//! `activateELO` stays in the UI tier (it depends on wall-clock, the ant
//! leaderboards, and quark crediting).

use synergismforkd_bignum::Decimal;

use crate::mechanics::ant_masteries::{
    calculate_self_speed_from_mastery, SelfSpeedFromMasteryInput,
};
use crate::mechanics::ant_producers::{
    ant_producer_data, calculate_base_ants_to_be_generated, BaseAntsToBeGeneratedInput,
};
use crate::state::{PlayerAntMastery, PlayerAntProducer};

/// HolySpirit — top of the producer chain. Workers=0 .. HolySpirit=8.
const LAST_ANT_PRODUCER: usize = 8;

/// Inputs to [`generate_ants_and_crumbs`].
pub(crate) struct GenerateAntsAndCrumbsInput<'a> {
    /// Tick delta (seconds).
    pub dt: f64,
    /// Pre-evaluated `calculateActualAntSpeedMult()`.
    pub ant_speed_mult: Decimal,
    /// `player.ants.producers` (0..=8).
    pub producers: &'a [PlayerAntProducer; 9],
    /// `player.ants.masteries` (0..=8).
    pub masteries: &'a [PlayerAntMastery; 9],
    /// `player.ants.crumbs`.
    pub crumbs: Decimal,
    /// `player.ants.crumbsThisSacrifice`.
    pub crumbs_this_sacrifice: Decimal,
    /// `player.ants.crumbsEverMade`.
    pub crumbs_ever_made: Decimal,
}

/// Result of [`generate_ants_and_crumbs`].
pub(crate) struct GenerateAntsAndCrumbsResult {
    /// Updated `generated` for each tier (0..=8).
    pub producers_generated: [Decimal; 9],
    /// Updated `crumbs`.
    pub crumbs: Decimal,
    /// Updated `crumbs_this_sacrifice`.
    pub crumbs_this_sacrifice: Decimal,
    /// Updated `crumbs_ever_made`.
    pub crumbs_ever_made: Decimal,
}

/// Compute one producer's base per-tick production (the shared
/// `selfSpeed → baseAnts` chain).
fn base_generation(
    index: usize,
    generated: Decimal,
    producers: &[PlayerAntProducer; 9],
    masteries: &[PlayerAntMastery; 9],
    ant_speed_mult: Decimal,
) -> Decimal {
    let self_speed_mult = calculate_self_speed_from_mastery(&SelfSpeedFromMasteryInput {
        producer: index as u8,
        mastery_level: masteries[index].mastery,
        purchased: producers[index].purchased,
    });
    calculate_base_ants_to_be_generated(&BaseAntsToBeGeneratedInput {
        generated,
        purchased: producers[index].purchased,
        base_production: ant_producer_data(index as u8).base_production,
        self_speed_mult,
        ant_speed_mult,
    })
}

/// Per-tick ant production + crumb generation. See module docs for the
/// iteration-dependency contract.
pub(crate) fn generate_ants_and_crumbs(
    input: &GenerateAntsAndCrumbsInput,
) -> GenerateAntsAndCrumbsResult {
    let dt = Decimal::from_finite(input.dt);
    let mut updated_generated: [Decimal; 9] = std::array::from_fn(|i| input.producers[i].generated);

    // Producer loop: each higher tier produces the tier directly below.
    for ant_type in (1..=LAST_ANT_PRODUCER).rev() {
        let base = base_generation(
            ant_type,
            updated_generated[ant_type],
            input.producers,
            input.masteries,
            input.ant_speed_mult,
        );
        if let Some(produces) = ant_producer_data(ant_type as u8).produces {
            updated_generated[produces as usize] += base * dt;
        }
    }

    // Workers (0) produce crumbs from their (now-updated) generated count.
    let crumbs_to_generate = base_generation(
        0,
        updated_generated[0],
        input.producers,
        input.masteries,
        input.ant_speed_mult,
    ) * dt;

    GenerateAntsAndCrumbsResult {
        producers_generated: updated_generated,
        crumbs: input.crumbs + crumbs_to_generate,
        crumbs_this_sacrifice: input.crumbs_this_sacrifice + crumbs_to_generate,
        crumbs_ever_made: input.crumbs_ever_made + crumbs_to_generate,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn producers_with(purchased: [f64; 9]) -> [PlayerAntProducer; 9] {
        std::array::from_fn(|i| PlayerAntProducer {
            purchased: purchased[i],
            generated: Decimal::zero(),
        })
    }

    #[test]
    fn workers_generate_crumbs_from_purchases() {
        // Only Workers (0) purchased → they mint crumbs even with no
        // higher tiers.
        let mut purchased = [0.0; 9];
        purchased[0] = 100.0;
        let input = GenerateAntsAndCrumbsInput {
            dt: 1.0,
            ant_speed_mult: Decimal::one(),
            producers: &producers_with(purchased),
            masteries: &[PlayerAntMastery::default(); 9],
            crumbs: Decimal::zero(),
            crumbs_this_sacrifice: Decimal::zero(),
            crumbs_ever_made: Decimal::zero(),
        };
        let r = generate_ants_and_crumbs(&input);
        assert!(r.crumbs.to_number() > 0.0);
        // All three crumb accumulators move by the same delta.
        assert_eq!(r.crumbs, r.crumbs_this_sacrifice);
        assert_eq!(r.crumbs, r.crumbs_ever_made);
    }

    #[test]
    fn higher_tier_feeds_lower_tier_same_tick() {
        // Breeders (1) purchased → they generate Workers (0) this tick.
        let mut purchased = [0.0; 9];
        purchased[1] = 1e6;
        let input = GenerateAntsAndCrumbsInput {
            dt: 1.0,
            ant_speed_mult: Decimal::one(),
            producers: &producers_with(purchased),
            masteries: &[PlayerAntMastery::default(); 9],
            crumbs: Decimal::zero(),
            crumbs_this_sacrifice: Decimal::zero(),
            crumbs_ever_made: Decimal::zero(),
        };
        let r = generate_ants_and_crumbs(&input);
        // Workers.generated grew from the Breeders production.
        assert!(r.producers_generated[0].to_number() > 0.0);
    }

    #[test]
    fn idle_when_nothing_owned() {
        let input = GenerateAntsAndCrumbsInput {
            dt: 1.0,
            ant_speed_mult: Decimal::one(),
            producers: &producers_with([0.0; 9]),
            masteries: &[PlayerAntMastery::default(); 9],
            crumbs: Decimal::from_finite(5.0),
            crumbs_this_sacrifice: Decimal::zero(),
            crumbs_ever_made: Decimal::zero(),
        };
        let r = generate_ants_and_crumbs(&input);
        // No producers → no crumb generation; balance unchanged.
        assert_eq!(r.crumbs.to_number(), 5.0);
        assert_eq!(r.producers_generated[0].to_number(), 0.0);
    }
}
