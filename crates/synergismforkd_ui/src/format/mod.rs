//! Number formatting — the fork's own modern spec (NOT a port of the legacy
//! `format()`; see `docs/systems/ui-readiness.md` history and the plan notes).
//!
//! One call path for every number on screen so precision rules can't drift
//! per call site (a legacy pain point):
//!
//! - `< 1,000`: plain, up to `max_decimals` (trailing zeros trimmed).
//! - `1e3 ..< 1e18`: short-scale suffixes `K M B T Qa`, 3 significant digits
//!   (`1.23M`, `12.3M`, `123M`).
//! - `>= 1e18`: scientific `1.23e18`; the exponent gains thousands grouping
//!   once it's itself ≥ 1e6 (`1.23e1,234,567`).
//! - `0 < x < 1`: up to 3 decimals; below `1e-3` scientific (`1.23e-5`).
//! - Negatives recurse with a `-` prefix; NaN renders `"0"` (legacy-compatible
//!   defensive default); infinities render `∞`.
//!
//! Alternative notations ([`Notation::Scientific`] / [`Notation::Engineering`])
//! replace the suffix tier for players who prefer exponents; both kick in at
//! 1e3 and keep the plain tier below.

mod time;

pub use time::format_time_short;

use serde::{Deserialize, Serialize};
use synergismforkd_bignum::Decimal;

/// Player-selected number notation (a UI preference, persisted by the host —
/// never part of `GameState`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Notation {
    /// Suffixes to `1e18`, then scientific. The default.
    #[default]
    Modern,
    /// Mantissa + exponent from `1e3` up (`1.23e4`).
    Scientific,
    /// Like scientific, but the exponent snaps to a multiple of 3
    /// (`12.3e3`).
    Engineering,
}

/// Formatting options. `max_decimals` only affects the plain (`< 1e3`) tier;
/// the suffix/scientific tiers always use 3 significant digits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FormatOpts {
    /// Notation for values at or above `1e3`.
    pub notation: Notation,
    /// Decimal cap for the plain tier (counts want 0, rates want 2).
    pub max_decimals: u8,
}

impl Default for FormatOpts {
    fn default() -> Self {
        Self {
            notation: Notation::Modern,
            max_decimals: 0,
        }
    }
}

/// Format a whole count (owned buildings, levels): plain tier shows no
/// decimals.
#[must_use]
pub fn format_count(value: Decimal, notation: Notation) -> String {
    format(
        value,
        FormatOpts {
            notation,
            max_decimals: 0,
        },
    )
}

/// Format a fractional quantity (rates, multipliers): plain tier shows up to
/// 2 decimals.
#[must_use]
pub fn format_value(value: Decimal, notation: Notation) -> String {
    format(
        value,
        FormatOpts {
            notation,
            max_decimals: 2,
        },
    )
}

/// Short-scale suffixes, one per power of 1000 starting at `1e3`.
const SUFFIXES: [&str; 5] = ["K", "M", "B", "T", "Qa"];

/// First exponent past the suffix table (Qa covers `1e15..1e18`).
const SCIENTIFIC_FLOOR_EXP: i64 = 18;

/// The single formatting entry point.
#[must_use]
pub fn format(value: Decimal, opts: FormatOpts) -> String {
    // Order matters: the degenerate checks read only the raw `mag` field —
    // calling `mantissa()` on a NaN/infinite Decimal panics inside the
    // bignum's power-of-10 table — so every later branch sees a finite
    // positive value.
    if value.mag().is_nan() {
        return "0".to_string();
    }
    if value.sign() == 0 {
        return "0".to_string();
    }
    if value.layer() == 0 && value.mag().is_infinite() {
        return if value.sign() < 0 { "-∞" } else { "∞" }.to_string();
    }
    if value.sign() < 0 {
        return format!("-{}", format(value.abs(), opts));
    }

    let approx = value.to_number();
    if approx < 1.0 {
        return format_sub_one(approx);
    }
    if approx < 1_000.0 {
        return format_plain(approx, opts.max_decimals);
    }

    let (mantissa, exponent) = mantissa_exponent(value);
    match opts.notation {
        Notation::Modern if exponent < SCIENTIFIC_FLOOR_EXP => format_suffix(approx),
        Notation::Modern | Notation::Scientific => format_scientific(mantissa, exponent),
        Notation::Engineering => format_engineering(mantissa, exponent),
    }
}

/// `0 < value < 1`: up to 3 decimals, scientific below `1e-3`.
fn format_sub_one(value: f64) -> String {
    if value >= 1e-3 {
        return trim_zeros(&format!("{value:.3}"));
    }
    // Negative-exponent scientific. Derive from f64 directly — the range is
    // tiny and layer-0 by construction.
    let exp = value.log10().floor();
    let mantissa = value / 10f64.powf(exp);
    let (mantissa, exp) = round_mantissa(mantissa, exp as i64);
    format!("{mantissa:.2}e-{}", -exp)
}

/// Plain tier: `1 <= value < 1000`, capped decimals, trailing zeros trimmed.
fn format_plain(value: f64, max_decimals: u8) -> String {
    let s = format!("{value:.*}", max_decimals as usize);
    if max_decimals == 0 {
        s
    } else {
        trim_zeros(&s)
    }
}

/// Suffix tier: `1e3 <= value < 1e18`, 3 significant digits.
///
/// f64 carries this whole range (display precision is 3 digits, well under
/// f64's ~15.9). Rounding can roll a scaled value to 1000 (`999_999` →
/// `1000K`); the loop re-normalizes so it lands as `1.00M`, and a roll past
/// `Qa` falls through to scientific.
fn format_suffix(value: f64) -> String {
    let mut group = (floored_log10(value) / 3) as usize; // 1 = K, 2 = M, …
    let mut scaled = value / 1000f64.powi(group as i32);
    let mut decimals = sig_decimals(scaled);
    let mut rounded = round_to(scaled, decimals);
    if rounded >= 1000.0 {
        group += 1;
        scaled = value / 1000f64.powi(group as i32);
        decimals = sig_decimals(scaled);
        rounded = round_to(scaled, decimals);
    }
    match SUFFIXES.get(group - 1) {
        Some(suffix) => format!("{rounded:.*}{suffix}", decimals as usize),
        // Rounding rolled past Qa (999.5e15 → 1e18): scientific takes over.
        None => format_scientific(1.0, (group as i64) * 3),
    }
}

/// Scientific tier: `m.mm e EXP`, exponent grouped once it's ≥ 1e6.
fn format_scientific(mantissa: f64, exponent: i64) -> String {
    let (mantissa, exponent) = round_mantissa(mantissa, exponent);
    format!("{mantissa:.2}e{}", group_thousands(exponent))
}

/// Engineering: exponent snapped down to a multiple of 3, mantissa
/// `1..999.9` with 3-significant-digit decimals.
fn format_engineering(mantissa: f64, exponent: i64) -> String {
    let (mantissa, exponent) = round_mantissa(mantissa, exponent);
    let shift = exponent.rem_euclid(3);
    let eng_exp = exponent - shift;
    let eng_mantissa = mantissa * 10f64.powi(shift as i32);
    let decimals = sig_decimals(eng_mantissa);
    let rounded = round_to(eng_mantissa, decimals);
    if rounded >= 1000.0 {
        // 999.6e3 → 1.00e6 (rounding crossed the next engineering band).
        return format!("1.00e{}", group_thousands(eng_exp + 3));
    }
    format!(
        "{rounded:.*}e{}",
        decimals as usize,
        group_thousands(eng_exp)
    )
}

/// Mantissa/exponent for the scientific tiers, with the legacy jitter guards
/// (`9.9999999 → 1.0e(n+1)`, `0.9999 → 1.0`) baked into [`round_mantissa`].
/// Uses the bignum's own extraction so layer-1 values (`exponent > 1.8e308`)
/// work; for the exponent range past f64 integer precision the display is
/// best-effort (the game never reaches layer 2).
fn mantissa_exponent(value: Decimal) -> (f64, i64) {
    (value.mantissa(), value.exponent() as i64)
}

/// Round to 2 decimals and re-normalize so the displayed mantissa stays in
/// `[1, 10)` — kills both the `9.999 → 10.00e5` and `0.9999 → 0.99e6`
/// flicker the legacy formatter guarded against.
fn round_mantissa(mantissa: f64, exponent: i64) -> (f64, i64) {
    let mut m = round_to(mantissa, 2);
    let mut e = exponent;
    if m >= 10.0 {
        m /= 10.0;
        e += 1;
    } else if m < 1.0 {
        m *= 10.0;
        e -= 1;
    }
    (round_to(m, 2), e)
}

/// Decimals that produce 3 significant digits for a value in `[1, 1000)`.
fn sig_decimals(scaled: f64) -> u8 {
    if scaled >= 100.0 {
        0
    } else if scaled >= 10.0 {
        1
    } else {
        2
    }
}

fn round_to(value: f64, decimals: u8) -> f64 {
    let p = 10f64.powi(decimals as i32);
    (value * p).round() / p
}

/// `log10(value).floor()` hardened against the classic `log10(1000) =
/// 2.999…` float wobble.
fn floored_log10(value: f64) -> i32 {
    let mut exp = value.log10().floor() as i32;
    if 10f64.powi(exp + 1) <= value {
        exp += 1;
    } else if 10f64.powi(exp) > value {
        exp -= 1;
    }
    exp
}

/// Thousands-group an integer with commas (`1234567` → `"1,234,567"`).
/// Locale-aware separators are an i18n-milestone concern.
fn group_thousands(n: i64) -> String {
    let digits = n.abs().to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3 + 1);
    if n < 0 {
        out.push('-');
    }
    let lead = digits.len() % 3;
    for (i, ch) in digits.chars().enumerate() {
        if i != 0 && (i + 3 - lead).is_multiple_of(3) {
            out.push(',');
        }
        out.push(ch);
    }
    out
}

fn trim_zeros(s: &str) -> String {
    if !s.contains('.') {
        return s.to_string();
    }
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(v: f64) -> Decimal {
        Decimal::from_finite(v)
    }

    fn modern(v: f64) -> String {
        format_count(d(v), Notation::Modern)
    }

    #[test]
    fn zero_infinity_and_negatives() {
        assert_eq!(modern(0.0), "0");
        // No public NaN constructor in break-eternity-rs 0.4 (the guard in
        // `format` is defense-in-depth); infinity is reachable though.
        assert_eq!(format_count(Decimal::inf(), Notation::Modern), "∞");
        assert_eq!(format_count(Decimal::neg_inf(), Notation::Modern), "-∞");
        assert_eq!(modern(-1_234_567.0), "-1.23M");
        assert_eq!(format_value(d(-1.5), Notation::Modern), "-1.5");
    }

    #[test]
    fn plain_tier_counts_and_values() {
        assert_eq!(modern(1.0), "1");
        assert_eq!(modern(999.0), "999");
        // Counts truncate to whole numbers…
        assert_eq!(modern(12.75), "13");
        // …values keep up to 2 decimals, trailing zeros trimmed.
        assert_eq!(format_value(d(12.75), Notation::Modern), "12.75");
        assert_eq!(format_value(d(12.5), Notation::Modern), "12.5");
        assert_eq!(format_value(d(12.0), Notation::Modern), "12");
    }

    #[test]
    fn sub_one_tier() {
        assert_eq!(modern(0.5), "0.5");
        assert_eq!(modern(0.125), "0.125");
        assert_eq!(modern(0.001), "0.001");
        assert_eq!(modern(0.000123), "1.23e-4");
    }

    #[test]
    fn suffix_tier_three_sig_digits() {
        assert_eq!(modern(1_000.0), "1.00K");
        assert_eq!(modern(1_234.0), "1.23K");
        assert_eq!(modern(12_340.0), "12.3K");
        assert_eq!(modern(123_400.0), "123K");
        assert_eq!(modern(1_234_000.0), "1.23M");
        assert_eq!(modern(1.234e9), "1.23B");
        assert_eq!(modern(1.234e12), "1.23T");
        assert_eq!(modern(1.234e15), "1.23Qa");
        assert_eq!(modern(999.4e15), "999Qa");
    }

    #[test]
    fn suffix_rounding_rolls_over_cleanly() {
        // 999,999 would render 1000K at 0 decimals — must bump to 1.00M.
        assert_eq!(modern(999_999.0), "1.00M");
        assert_eq!(modern(999_500.0), "1.00M");
        // Below the half-step it stays in band.
        assert_eq!(modern(999_400.0), "999K");
        // Rolling past the end of the table lands in scientific. (999.6, not
        // 999.5 — the e15-scale literal isn't exactly representable and the
        // half-step would round down.)
        assert_eq!(modern(999.6e15), "1.00e18");
    }

    #[test]
    fn scientific_from_1e18_in_modern() {
        assert_eq!(modern(1.23e18), "1.23e18");
        assert_eq!(modern(4.56e19), "4.56e19");
        let huge = Decimal::from_mantissa_exponent(1.234, 1_234_567.0);
        assert_eq!(modern_decimal(huge), "1.23e1,234,567");
    }

    fn modern_decimal(v: Decimal) -> String {
        format_count(v, Notation::Modern)
    }

    #[test]
    fn mantissa_jitter_guards() {
        // 9.999e18 at 2 decimals would print 10.00e18 — must renormalize.
        assert_eq!(modern(9.999e18), "1.00e19");
        let low = Decimal::from_mantissa_exponent(0.99999, 20.0);
        assert_eq!(modern_decimal(low), "1.00e20");
    }

    #[test]
    fn pure_scientific_kicks_in_at_1e3() {
        assert_eq!(format_count(d(999.0), Notation::Scientific), "999");
        assert_eq!(format_count(d(1_234.0), Notation::Scientific), "1.23e3");
        assert_eq!(format_count(d(1.234e7), Notation::Scientific), "1.23e7");
    }

    #[test]
    fn engineering_snaps_exponent_to_threes() {
        assert_eq!(format_count(d(1_234.0), Notation::Engineering), "1.23e3");
        assert_eq!(format_count(d(45_600.0), Notation::Engineering), "45.6e3");
        assert_eq!(format_count(d(4.56e19), Notation::Engineering), "45.6e18");
        // Rounding across the band: 999.6e3 → 1.00e6.
        assert_eq!(format_count(d(999_600.0), Notation::Engineering), "1.00e6");
    }

    #[test]
    fn beyond_f64_uses_bignum_extraction() {
        // 1e400 is past f64::MAX; the layer-1 path must still format.
        let huge = Decimal::from_mantissa_exponent(2.5, 400.0);
        assert_eq!(modern_decimal(huge), "2.50e400");
    }

    #[test]
    fn grouping() {
        assert_eq!(group_thousands(1), "1");
        assert_eq!(group_thousands(123), "123");
        assert_eq!(group_thousands(1_234), "1,234");
        assert_eq!(group_thousands(1_234_567), "1,234,567");
    }
}
