//! Quark export accumulator math.
//!
//! Verbatim port of
//! `legacy_core_split/packages/logic/src/mechanics/quarks.ts`. The
//! UI side wraps this and gathers the player/octeract inputs; the
//! `QuarkHandler` class and the personal/global quark bonus state
//! stay in the UI tier (DOM + i18next + fetch).

/// Inputs to [`quark_handler`].
#[derive(Debug, Clone, Copy)]
pub struct QuarkHandlerInput {
    /// `player.researches[195]` — "Research 8x20". Each level adds
    /// `18000s` (5 hours) to the export cap. Zero means the base
    /// `90000s` window.
    pub research_195: f64,
    /// Sum of `player.researches` at the five quark-yielding slots:
    /// `researches[99] + [100] + [125] + [180] + [195]`. Added to
    /// the base `5` quarks/hour rate before the octeract multiplier.
    pub researches_sum: f64,
    /// `getOcteractUpgradeEffect('octeractExportQuarks', 'exportQuarkMult')`
    /// — multiplicative boost on the per-hour rate. Defaults to `1`
    /// if the upgrade isn't bought.
    pub export_quark_mult: f64,
    /// `player.quarkstimer` — seconds of accumulated export time.
    /// The actual quark gain is
    /// `floor(quarks_timer * per_hour / 3600)`; capped externally by
    /// the UI when the timer exceeds `max_time`.
    pub quarks_timer: f64,
    /// `calculateCubeQuarkMultiplier()` — already migrated, passed
    /// through unchanged. Kept here so callers get a single object to
    /// destructure.
    pub cube_mult: f64,
}

/// Result of [`quark_handler`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QuarkHandlerResult {
    /// Maximum accumulator window, in seconds.
    pub max_time: f64,
    /// Effective quarks/hour given all bonuses.
    pub per_hour: f64,
    /// `floor(per_hour * max_time / 3600)` — total quarks the cap
    /// can hold.
    pub capacity: f64,
    /// `floor(quarks_timer * per_hour / 3600)` — quarks gained right
    /// now.
    pub gain: f64,
    /// Pass-through of `input.cube_mult`.
    pub cube_mult: f64,
}

/// Computes the export-time / per-hour / capacity / current-gain
/// quartet used by every quark-display surface. Verbatim from the
/// legacy `quarkHandler` with the five player/G reads lifted into
/// explicit inputs.
#[must_use]
pub fn quark_handler(input: &QuarkHandlerInput) -> QuarkHandlerResult {
    let mut max_time = 90_000.0_f64;
    if input.research_195 > 0.0 {
        max_time += 18_000.0 * input.research_195;
    }

    let per_hour = (5.0 + input.researches_sum) * input.export_quark_mult;
    let capacity = (per_hour * max_time / 3_600.0).floor();
    let gain = (input.quarks_timer * per_hour / 3_600.0).floor();

    QuarkHandlerResult {
        max_time,
        per_hour,
        capacity,
        gain,
        cube_mult: input.cube_mult,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline() -> QuarkHandlerInput {
        QuarkHandlerInput {
            research_195: 0.0,
            researches_sum: 0.0,
            export_quark_mult: 1.0,
            quarks_timer: 0.0,
            cube_mult: 1.0,
        }
    }

    #[test]
    fn baseline_max_time_is_90000() {
        assert_eq!(quark_handler(&baseline()).max_time, 90_000.0);
    }

    #[test]
    fn research_195_adds_5h_per_level() {
        let input = QuarkHandlerInput {
            research_195: 2.0,
            ..baseline()
        };
        // 90000 + 18000*2 = 126000
        assert_eq!(quark_handler(&input).max_time, 126_000.0);
    }

    #[test]
    fn per_hour_uses_base_5_plus_researches_times_mult() {
        let input = QuarkHandlerInput {
            researches_sum: 15.0,
            export_quark_mult: 2.0,
            ..baseline()
        };
        // (5 + 15) * 2 = 40
        assert_eq!(quark_handler(&input).per_hour, 40.0);
    }

    #[test]
    fn capacity_is_per_hour_times_max_time_over_3600() {
        let input = QuarkHandlerInput {
            researches_sum: 31.0, // per_hour = 36
            ..baseline()
        };
        // capacity = floor(36 * 90000 / 3600) = floor(900) = 900
        assert_eq!(quark_handler(&input).capacity, 900.0);
    }

    #[test]
    fn gain_is_floored() {
        let input = QuarkHandlerInput {
            researches_sum: 0.0, // per_hour = 5
            quarks_timer: 100.0,
            ..baseline()
        };
        // gain = floor(100 * 5 / 3600) = floor(0.138...) = 0
        assert_eq!(quark_handler(&input).gain, 0.0);
    }

    #[test]
    fn cube_mult_is_passed_through() {
        let input = QuarkHandlerInput {
            cube_mult: 42.5,
            ..baseline()
        };
        assert_eq!(quark_handler(&input).cube_mult, 42.5);
    }
}
