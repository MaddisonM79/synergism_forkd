//! Currency newtypes — compile-time prevention of cross-currency mixing.
//!
//! Per Ledger Finding 2: in TS every currency is just `number`, and
//! nothing prevents `coins + transcend_points` from compiling. In Rust
//! we can do better. Each currency is a newtype wrapping [`Decimal`];
//! the type system blocks `Coins + PrestigePoints` and `Coins * Coins`
//! (dimensional) at the call site.
//!
//! What's allowed:
//! - `Coins + Coins -> Coins` (same-currency arithmetic)
//! - `Coins - Coins -> Coins`
//! - `Coins * Decimal -> Coins` (apply a dimensionless multiplier)
//! - `Coins / Decimal -> Coins` (divide by a dimensionless divisor)
//! - `Coins::cmp(&other)` via [`PartialOrd`]
//! - Construct: `Coins::new(d)` or `Coins::from(d)`
//! - Escape: `coins.raw()` returns the inner `Decimal`
//! - Math: `coins.log10()`, `coins.to_number()` delegate to inner
//!
//! What's blocked at compile time:
//! - `Coins + PrestigePoints` — no `Add<PrestigePoints> for Coins`
//! - `Coins * Coins` — no `Mul<Coins> for Coins`
//! - `Coins + Decimal` — must explicitly wrap the Decimal as Coins
//!
//! Inter-currency conversion (`Coins -> PrestigePoints` via reset, etc.)
//! is intentionally NOT provided here — the game's reset formulas live
//! in `mechanics/reset_currency.rs` and produce the new currency
//! explicitly. Adding `From`/`Into` would re-open the mixing hazard.

use core::cmp::Ordering;
use core::ops::{Add, AddAssign, Div, Mul, Sub, SubAssign};

use serde::{Deserialize, Serialize};
use synergismforkd_bignum::Decimal;

/// Define one currency newtype with the standard arithmetic surface.
///
/// Why a macro rather than a generic `Currency<Tag>`? Two reasons:
/// 1. The error messages are clearer (`expected Coins, found PrestigePoints`
///    instead of `expected Currency<CoinTag>, found Currency<PrestigeTag>`).
/// 2. Adding per-currency methods (e.g., a `Coins::FREE_TIER_BAND` const)
///    is just a normal impl block, not a trait dance.
macro_rules! currency_newtype {
    (
        $(#[$meta:meta])*
        $name:ident
    ) => {
        $(#[$meta])*
        // `Decimal` can hold NaN/INFINITY values (the `from_finite`
        // constructor name implies non-finite values exist), so
        // reflexivity is not guaranteed and `Eq` is intentionally not
        // derived. The `clippy::derive_partial_eq_without_eq` lint
        // (clippy::nursery) flags this; the suppression makes the
        // intent explicit.
        #[allow(clippy::derive_partial_eq_without_eq)]
        #[derive(
            Debug,
            Clone,
            Copy,
            Default,
            PartialEq,
            PartialOrd,
            Serialize,
            Deserialize,
        )]
        #[serde(transparent)]
        pub struct $name(Decimal);

        impl $name {
            /// Zero — the additive identity. Convenient when initializing
            /// a fresh-state slice.
            pub fn zero() -> Self {
                Self(Decimal::zero())
            }

            /// One — useful for testing and as a multiplicative identity
            /// when wrapped values appear in formulas.
            pub fn one() -> Self {
                Self(Decimal::one())
            }

            /// Wrap a raw `Decimal` in the newtype. The caller asserts
            /// (by choice of constructor) that the magnitude represents
            /// this currency, not another.
            pub const fn new(value: Decimal) -> Self {
                Self(value)
            }

            /// Unwrap into the inner `Decimal`. Use sparingly — every
            /// `.raw()` call is a place the type system stops helping.
            pub const fn raw(self) -> Decimal {
                self.0
            }

            /// `log10` of the magnitude. Returns a dimensionless
            /// [`Decimal`] — the log of a currency value has no currency
            /// dimension itself.
            pub fn log10(self) -> Decimal {
                self.0.log10()
            }

            /// Magnitude as `f64`. Same precision caveats as
            /// [`Decimal::to_number`] — values past `f64::MAX` saturate
            /// to `f64::INFINITY`.
            pub fn to_number(self) -> f64 {
                self.0.to_number()
            }
        }

        impl From<Decimal> for $name {
            fn from(d: Decimal) -> Self {
                Self(d)
            }
        }

        impl Add for $name {
            type Output = Self;
            fn add(self, rhs: Self) -> Self {
                Self(self.0 + rhs.0)
            }
        }

        impl AddAssign for $name {
            fn add_assign(&mut self, rhs: Self) {
                self.0 = self.0 + rhs.0;
            }
        }

        impl Sub for $name {
            type Output = Self;
            fn sub(self, rhs: Self) -> Self {
                Self(self.0 - rhs.0)
            }
        }

        impl SubAssign for $name {
            fn sub_assign(&mut self, rhs: Self) {
                self.0 = self.0 - rhs.0;
            }
        }

        // Currency × dimensionless = Currency.
        impl Mul<Decimal> for $name {
            type Output = Self;
            fn mul(self, rhs: Decimal) -> Self {
                Self(self.0 * rhs)
            }
        }

        // Dimensionless × Currency = Currency (commutativity).
        impl Mul<$name> for Decimal {
            type Output = $name;
            fn mul(self, rhs: $name) -> $name {
                $name(self * rhs.0)
            }
        }

        // Currency ÷ dimensionless = Currency.
        impl Div<Decimal> for $name {
            type Output = Self;
            fn div(self, rhs: Decimal) -> Self {
                Self(self.0 / rhs)
            }
        }

        // Compare against a raw Decimal threshold (e.g., when comparing
        // against a `get_cost_*` result that's still untyped). Useful
        // until cost functions also produce newtypes.
        impl PartialEq<Decimal> for $name {
            fn eq(&self, other: &Decimal) -> bool {
                self.0.eq(other)
            }
        }

        impl PartialOrd<Decimal> for $name {
            fn partial_cmp(&self, other: &Decimal) -> Option<Ordering> {
                self.0.partial_cmp(other)
            }
        }
    };
}

currency_newtype! {
    /// The base coin currency — `player.coins`. Earned through producer
    /// production, spent on upgrades, accelerators, multipliers, and
    /// coin-tier producers.
    Coins
}

currency_newtype! {
    /// Diamonds-prestige currency — `player.prestigePoints`. Earned at
    /// prestige reset, spent on diamond producers and crystal upgrades
    /// (via `prestige_shards`).
    PrestigePoints
}

currency_newtype! {
    /// Mythos-prestige currency — `player.transcendPoints`. Earned at
    /// transcension, spent on mythos producers.
    TranscendPoints
}

currency_newtype! {
    /// Particles-prestige currency — `player.reincarnationPoints`. Earned
    /// at reincarnation, spent on particle producers and particle
    /// buildings.
    ReincarnationPoints
}

// ─── Multiplier (dimensionless) ──────────────────────────────────────────

/// A dimensionless multiplier. The composition layer between currency
/// production and consumption — `Multiplier * Coins -> Coins` is the
/// canonical "apply this multiplier to a currency" operation. (Anvil F11
/// / Ledger F2 item 3.)
///
/// What's allowed:
/// - `Multiplier + Multiplier -> Multiplier` (sum of additive
///   contributions — common when combining several multiplier sources)
/// - `Multiplier * Multiplier -> Multiplier` (chain multipliers
///   together to get a total)
/// - `Multiplier * <Currency> -> <Currency>` (apply to any currency)
/// - `<Currency> * Multiplier -> <Currency>` (commutativity)
/// - Comparison with self via [`PartialOrd`]
///
/// What's blocked at compile time:
/// - `Multiplier - Multiplier`: subtraction isn't a natural multiplier
///   operation — explicit `Multiplier::new(a.raw() - b.raw())` is
///   available if a caller really needs it.
/// - `Multiplier / Multiplier`: same — multipliers compose by
///   multiplication, not division.
/// - `Coins * Coins`: still blocked, because `Coins * Decimal` produces
///   `Coins` (via the macro) but neither side of `Coins * Coins`
///   matches.
#[derive(Debug, Clone, Copy, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Multiplier(Decimal);

impl Multiplier {
    /// Identity multiplier (`1.0`). `m * identity() == m`.
    pub fn one() -> Self {
        Self(Decimal::one())
    }

    /// Wrap a raw `Decimal` as a multiplier. The caller asserts that
    /// the value is dimensionless.
    pub const fn new(value: Decimal) -> Self {
        Self(value)
    }

    /// Unwrap into the inner `Decimal`. Use sparingly — escape hatch
    /// to raw arithmetic.
    pub const fn raw(self) -> Decimal {
        self.0
    }

    /// Magnitude as `f64`. Same precision caveats as
    /// [`Decimal::to_number`].
    pub fn to_number(self) -> f64 {
        self.0.to_number()
    }
}

impl From<Decimal> for Multiplier {
    fn from(d: Decimal) -> Self {
        Self(d)
    }
}

impl core::ops::Add for Multiplier {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl core::ops::AddAssign for Multiplier {
    fn add_assign(&mut self, rhs: Self) {
        self.0 = self.0 + rhs.0;
    }
}

impl core::ops::Mul for Multiplier {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Self(self.0 * rhs.0)
    }
}

impl core::ops::MulAssign for Multiplier {
    fn mul_assign(&mut self, rhs: Self) {
        self.0 = self.0 * rhs.0;
    }
}

macro_rules! impl_multiplier_apply {
    ($currency:ident) => {
        impl core::ops::Mul<$currency> for Multiplier {
            type Output = $currency;
            fn mul(self, rhs: $currency) -> $currency {
                $currency::new(self.0 * rhs.raw())
            }
        }

        impl core::ops::Mul<Multiplier> for $currency {
            type Output = $currency;
            fn mul(self, rhs: Multiplier) -> $currency {
                $currency::new(self.raw() * rhs.0)
            }
        }
    };
}

impl_multiplier_apply!(Coins);
impl_multiplier_apply!(PrestigePoints);
impl_multiplier_apply!(TranscendPoints);
impl_multiplier_apply!(ReincarnationPoints);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arithmetic_within_currency() {
        let a = Coins::new(Decimal::from_finite(100.0));
        let b = Coins::new(Decimal::from_finite(50.0));
        assert_eq!((a + b).raw().to_number(), 150.0);
        assert_eq!((a - b).raw().to_number(), 50.0);
    }

    #[test]
    fn multiply_by_dimensionless() {
        let a = Coins::new(Decimal::from_finite(100.0));
        let mult = Decimal::from_finite(2.5);
        assert_eq!((a * mult).raw().to_number(), 250.0);
        assert_eq!((mult * a).raw().to_number(), 250.0);
    }

    #[test]
    fn divide_by_dimensionless() {
        let a = Coins::new(Decimal::from_finite(100.0));
        let div = Decimal::from_finite(4.0);
        assert_eq!((a / div).raw().to_number(), 25.0);
    }

    #[test]
    fn assign_ops_mutate_in_place() {
        let mut a = Coins::new(Decimal::from_finite(100.0));
        a += Coins::new(Decimal::from_finite(50.0));
        assert_eq!(a.raw().to_number(), 150.0);
        a -= Coins::new(Decimal::from_finite(30.0));
        assert_eq!(a.raw().to_number(), 120.0);
    }

    #[test]
    fn compare_same_currency() {
        let small = Coins::new(Decimal::from_finite(10.0));
        let big = Coins::new(Decimal::from_finite(100.0));
        assert!(small < big);
        assert!(big > small);
        assert!(small <= small);
    }

    #[test]
    fn compare_against_raw_decimal() {
        let a = Coins::new(Decimal::from_finite(50.0));
        assert!(a >= Decimal::from_finite(40.0));
        assert!(a < Decimal::from_finite(60.0));
    }

    #[test]
    fn log10_delegates_to_inner() {
        let a = Coins::new(Decimal::from_finite(1000.0));
        assert!((a.log10().to_number() - 3.0).abs() < 1e-9);
    }

    #[test]
    fn default_is_zero() {
        let a = Coins::default();
        assert_eq!(a.raw(), Decimal::zero());
        assert_eq!(a, Coins::zero());
    }

    #[test]
    fn multiplier_applies_to_each_currency() {
        let m = Multiplier::new(Decimal::from_finite(2.0));
        let coins = Coins::new(Decimal::from_finite(50.0));
        let prestige = PrestigePoints::new(Decimal::from_finite(50.0));
        let transcend = TranscendPoints::new(Decimal::from_finite(50.0));
        let reincarn = ReincarnationPoints::new(Decimal::from_finite(50.0));
        assert_eq!((m * coins).raw().to_number(), 100.0);
        assert_eq!((coins * m).raw().to_number(), 100.0);
        assert_eq!((m * prestige).raw().to_number(), 100.0);
        assert_eq!((m * transcend).raw().to_number(), 100.0);
        assert_eq!((m * reincarn).raw().to_number(), 100.0);
    }

    #[test]
    fn multipliers_compose_via_mul() {
        let a = Multiplier::new(Decimal::from_finite(2.0));
        let b = Multiplier::new(Decimal::from_finite(3.0));
        assert_eq!((a * b).raw().to_number(), 6.0);
        let mut chain = Multiplier::one();
        chain *= a;
        chain *= b;
        assert_eq!(chain.raw().to_number(), 6.0);
    }

    #[test]
    fn multipliers_sum_via_add() {
        let a = Multiplier::new(Decimal::from_finite(1.5));
        let b = Multiplier::new(Decimal::from_finite(2.5));
        assert_eq!((a + b).raw().to_number(), 4.0);
    }

    #[test]
    fn currencies_are_distinct_types() {
        // Type-system sanity: this is a *compile* check, not a runtime
        // one. The line below would not compile:
        //     let _ = Coins::zero() + PrestigePoints::zero();
        // We can't `assert_fails_to_compile!` in stable Rust, so we
        // just exercise the constructors and trust the type system.
        let _coins = Coins::zero();
        let _prestige = PrestigePoints::zero();
        let _transcend = TranscendPoints::zero();
        let _reincarn = ReincarnationPoints::zero();
    }
}
