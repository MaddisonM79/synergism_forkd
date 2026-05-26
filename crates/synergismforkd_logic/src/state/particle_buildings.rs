//! Particle-buildings state slice.
//!
//! Mirrors `ParticleBuildingsState` from the legacy TS
//! `packages/logic/src/state/schema.ts`. Five positions
//! (`first..fifth_owned_particles`) plus per-position cost caches; the
//! shared resource is `reincarnation_points`. Distinct from
//! [`crate::state::ProducerFamilyState`] — particle buildings have their
//! own cost curve (`base * 2^buyingTo` + a quadratic-in-exponent tail) and
//! no per-position "didn't buy" achievement gates.

use synergismforkd_bignum::Decimal;

/// Slice of `GameState` read/written by the particle-building-purchase
/// machinery.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ParticleBuildingsState {
    /// The spend resource (reincarnation points).
    pub reincarnation_points: Decimal,
    /// Tier-1 owned count.
    pub first_owned_particles: f64,
    /// Tier-1 next cost.
    pub first_cost_particles: Decimal,
    /// Tier-2 owned count.
    pub second_owned_particles: f64,
    /// Tier-2 next cost.
    pub second_cost_particles: Decimal,
    /// Tier-3 owned count.
    pub third_owned_particles: f64,
    /// Tier-3 next cost.
    pub third_cost_particles: Decimal,
    /// Tier-4 owned count.
    pub fourth_owned_particles: f64,
    /// Tier-4 next cost.
    pub fourth_cost_particles: Decimal,
    /// Tier-5 owned count.
    pub fifth_owned_particles: f64,
    /// Tier-5 next cost.
    pub fifth_cost_particles: Decimal,
}

impl ParticleBuildingsState {
    /// Read the owned count for tier `index` (1..=5). Debug-asserts the
    /// invariant; in release, out-of-range falls through to the fifth tier
    /// (matches the legacy fall-through default).
    #[must_use]
    pub fn owned(&self, index: u8) -> f64 {
        debug_assert!(
            matches!(index, 1..=5),
            "particle index out of range: {index}"
        );
        match index {
            1 => self.first_owned_particles,
            2 => self.second_owned_particles,
            3 => self.third_owned_particles,
            4 => self.fourth_owned_particles,
            _ => self.fifth_owned_particles,
        }
    }

    /// Read the cost cache for tier `index` (1..=5). Same out-of-range
    /// behavior as [`Self::owned`].
    #[must_use]
    pub fn cost(&self, index: u8) -> Decimal {
        debug_assert!(
            matches!(index, 1..=5),
            "particle index out of range: {index}"
        );
        match index {
            1 => self.first_cost_particles,
            2 => self.second_cost_particles,
            3 => self.third_cost_particles,
            4 => self.fourth_cost_particles,
            _ => self.fifth_cost_particles,
        }
    }

    /// Write the owned count for tier `index` (1..=5). Same out-of-range
    /// behavior as [`Self::owned`].
    pub fn set_owned(&mut self, index: u8, value: f64) {
        debug_assert!(
            matches!(index, 1..=5),
            "particle index out of range: {index}"
        );
        match index {
            1 => self.first_owned_particles = value,
            2 => self.second_owned_particles = value,
            3 => self.third_owned_particles = value,
            4 => self.fourth_owned_particles = value,
            _ => self.fifth_owned_particles = value,
        }
    }

    /// Write the cost cache for tier `index` (1..=5). Same out-of-range
    /// behavior as [`Self::owned`].
    pub fn set_cost(&mut self, index: u8, value: Decimal) {
        debug_assert!(
            matches!(index, 1..=5),
            "particle index out of range: {index}"
        );
        match index {
            1 => self.first_cost_particles = value,
            2 => self.second_cost_particles = value,
            3 => self.third_cost_particles = value,
            4 => self.fourth_cost_particles = value,
            _ => self.fifth_cost_particles = value,
        }
    }
}
