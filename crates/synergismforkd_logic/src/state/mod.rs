//! Sliced game-state types. Each mechanic family owns a slice.
//!
//! The composed `GameState` lands once enough slices exist for it to be
//! meaningful — for now each slice stands alone, and mechanic functions
//! take only the slice they touch.

pub mod accelerator;
pub mod multiplier;
pub mod producer;

pub use accelerator::AcceleratorState;
pub use multiplier::MultiplierState;
pub use producer::{BuyAmount, ProducerFamilyState};
