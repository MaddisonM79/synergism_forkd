//! Ascension-related formulas.
//!
//! Verbatim port of
//! `legacy_core_split/packages/logic/src/mechanics/ascensions.ts`
//! (lifted from the legacy `packages/web_ui/src/Calculate.ts`).
//! Currently just the per-reset ascension count; the legacy UI
//! collects the `ascensionCountMultStats` `StatLine` values into an
//! array and passes them in, then this module multiplies and floors.

/// Inputs to [`calculate_ascension_count`].
#[derive(Debug, Clone, Copy)]
pub struct CalculateAscensionCountInput<'a> {
    /// `player.singularityChallenges.limitedAscensions.enabled` —
    /// when `true`, the count is capped at 1 (Exalt 3 forces one
    /// ascension at a time).
    pub limited_ascensions_enabled: bool,
    /// Precomputed multiplier contributions (legacy:
    /// `ascensionCountMultStats.map(s => s.stat())`). Product is
    /// floored to give the final count.
    pub ascension_count_mults: &'a [f64],
}

/// `1` when Exalt 3 is active; otherwise
/// `floor(product(ascension_count_mults))`. Multiplier contributions
/// can include fractional and `> 1` boosts; flooring handles the
/// off-by-one rounding.
#[must_use]
pub fn calculate_ascension_count(input: &CalculateAscensionCountInput<'_>) -> f64 {
    if input.limited_ascensions_enabled {
        return 1.0;
    }
    input.ascension_count_mults.iter().product::<f64>().floor()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn limited_ascensions_caps_at_one() {
        let input = CalculateAscensionCountInput {
            limited_ascensions_enabled: true,
            ascension_count_mults: &[10.0, 20.0, 30.0],
        };
        assert_eq!(calculate_ascension_count(&input), 1.0);
    }

    #[test]
    fn empty_mults_gives_one() {
        // product([]) = 1 → floor(1) = 1
        let input = CalculateAscensionCountInput {
            limited_ascensions_enabled: false,
            ascension_count_mults: &[],
        };
        assert_eq!(calculate_ascension_count(&input), 1.0);
    }

    #[test]
    fn products_are_floored() {
        // 1.5 * 2.5 = 3.75 → floor 3
        let input = CalculateAscensionCountInput {
            limited_ascensions_enabled: false,
            ascension_count_mults: &[1.5, 2.5],
        };
        assert_eq!(calculate_ascension_count(&input), 3.0);
    }

    #[test]
    fn multiplies_all_contributions() {
        let input = CalculateAscensionCountInput {
            limited_ascensions_enabled: false,
            ascension_count_mults: &[2.0, 3.0, 4.0],
        };
        assert_eq!(calculate_ascension_count(&input), 24.0);
    }
}
