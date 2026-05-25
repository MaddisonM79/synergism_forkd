//! Synergism Forkd — bignum wrapper.
//!
//! Surface mirrors `break_eternity.js`. The body is a stub: every method
//! returns a placeholder or panics with `unimplemented!()`. Downstream
//! crates import [`Decimal`] so call sites compile; the real
//! implementation lands when the maintained `break-eternity` Rust fork is
//! published.

use std::cmp::Ordering;
use std::fmt;

/// Big-number type with `break_eternity` semantics: sign + layer-based
/// exponent tower. Layer 0 is `sign * mag`; layer 1 is `sign * 10^mag`;
/// layer 2 is `sign * 10^10^mag`; and so on.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Decimal {
    sign: f64,
    mag: f64,
    layer: f64,
}

impl Decimal {
    pub const ZERO: Self = Self {
        sign: 0.0,
        mag: 0.0,
        layer: 0.0,
    };
    pub const ONE: Self = Self {
        sign: 1.0,
        mag: 0.0,
        layer: 0.0,
    };

    pub fn from_components(sign: f64, layer: f64, mag: f64) -> Self {
        Self { sign, mag, layer }
    }

    pub fn from_f64(_value: f64) -> Self {
        unimplemented!("bignum stub — wire the maintained break-eternity fork")
    }

    pub fn add(&self, _rhs: &Self) -> Self {
        unimplemented!()
    }
    pub fn sub(&self, _rhs: &Self) -> Self {
        unimplemented!()
    }
    pub fn mul(&self, _rhs: &Self) -> Self {
        unimplemented!()
    }
    pub fn div(&self, _rhs: &Self) -> Self {
        unimplemented!()
    }
    pub fn neg(&self) -> Self {
        unimplemented!()
    }
    pub fn abs(&self) -> Self {
        unimplemented!()
    }
    pub fn pow(&self, _rhs: &Self) -> Self {
        unimplemented!()
    }
    pub fn log10(&self) -> Self {
        unimplemented!()
    }
    pub fn exp(&self) -> Self {
        unimplemented!()
    }
    pub fn tetrate(&self, _height: &Self) -> Self {
        unimplemented!()
    }
    pub fn iterated_exp(&self, _times: &Self) -> Self {
        unimplemented!()
    }
    pub fn iterated_log(&self, _times: &Self) -> Self {
        unimplemented!()
    }
    pub fn slog(&self, _base: &Self) -> Self {
        unimplemented!()
    }
    pub fn ssqrt(&self) -> Self {
        unimplemented!()
    }
    pub fn lambertw(&self) -> Self {
        unimplemented!()
    }
    #[allow(clippy::should_implement_trait)]
    pub fn cmp(&self, _rhs: &Self) -> Ordering {
        unimplemented!()
    }
}

impl fmt::Display for Decimal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Decimal(stub)")
    }
}

/// Anything convertible into a `Decimal`. Mirrors `DecimalSource` in BE.js.
pub trait DecimalSource {
    fn to_decimal(self) -> Decimal;
}

impl DecimalSource for Decimal {
    fn to_decimal(self) -> Decimal {
        self
    }
}

impl DecimalSource for f64 {
    fn to_decimal(self) -> Decimal {
        Decimal::from_f64(self)
    }
}
