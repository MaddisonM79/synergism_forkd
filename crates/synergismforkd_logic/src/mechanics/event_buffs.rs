//! Event-buff selectors.
//!
//! Verbatim port of
//! `legacy_core_split/packages/logic/src/mechanics/eventBuffs.ts`
//! (lifted from the legacy `packages/web_ui/src/Event.ts`).
//!
//! Two contributions stack additively per buff: the active
//! `GameEvent` (if any) and the consumable Happy-Hour-Bell stack
//! (scaled by queued count). The UI side handles the API fetch /
//! consumable inventory; logic owns the per-buff switch logic.

/// Numeric tags for each in-game buff source. Mirrors the legacy
/// `BuffType` enum (`Event.ts`) exactly — values `0..=13`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum BuffType {
    /// Quark gain.
    Quark = 0,
    /// Golden-quark gain.
    GoldenQuark = 1,
    /// Cube multiplier.
    Cubes = 2,
    /// Overflux powder conversion rate.
    PowderConversion = 3,
    /// Ascension-speed multiplier.
    AscensionSpeed = 4,
    /// Global-speed multiplier.
    GlobalSpeed = 5,
    /// Ascension-score multiplier.
    AscensionScore = 6,
    /// Ant-sacrifice multiplier.
    AntSacrifice = 7,
    /// Offering multiplier.
    Offering = 8,
    /// Obtainium multiplier.
    Obtainium = 9,
    /// Octeract gain multiplier.
    Octeract = 10,
    /// Blueberry-time multiplier.
    BlueberryTime = 11,
    /// Ambrosia luck additive contribution.
    AmbrosiaLuck = 12,
    /// One-mind buff (gated by GQ upgrade unlock).
    OneMind = 13,
}

/// Wire shape of a server-fetched GameEvent. Mirrors the legacy
/// `GameEvent` interface — keys map to [`BuffType`] values via
/// [`get_event_buff`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GameEventBuffs {
    /// Quark gain buff.
    pub quark: f64,
    /// Golden-quark gain buff.
    pub golden_quark: f64,
    /// Cube multiplier buff.
    pub cubes: f64,
    /// Powder conversion buff.
    pub powder_conversion: f64,
    /// Ascension speed buff.
    pub ascension_speed: f64,
    /// Global speed buff.
    pub global_speed: f64,
    /// Ascension score buff.
    pub ascension_score: f64,
    /// Ant sacrifice buff.
    pub ant_sacrifice: f64,
    /// Offering buff.
    pub offering: f64,
    /// Obtainium buff.
    pub obtainium: f64,
    /// Octeract gain buff.
    pub octeract: f64,
    /// Blueberry time buff.
    pub blueberry_time: f64,
    /// Ambrosia luck buff.
    pub ambrosia_luck: f64,
    /// One-mind buff.
    pub one_mind: f64,
}

/// Per-buff value coming from the active GameEvent. Returns `0`
/// when `event` is `None` (no active event). `OneMind` also requires
/// the GQ-upgrade unlock — caller passes the boolean.
#[must_use]
pub fn get_event_buff(
    buff: BuffType,
    event: Option<&GameEventBuffs>,
    one_mind_unlocked: bool,
) -> f64 {
    let Some(event) = event else { return 0.0 };
    match buff {
        BuffType::Quark => event.quark,
        BuffType::GoldenQuark => event.golden_quark,
        BuffType::Cubes => event.cubes,
        BuffType::PowderConversion => event.powder_conversion,
        BuffType::AscensionSpeed => event.ascension_speed,
        BuffType::GlobalSpeed => event.global_speed,
        BuffType::AscensionScore => event.ascension_score,
        BuffType::AntSacrifice => event.ant_sacrifice,
        BuffType::Offering => event.offering,
        BuffType::Obtainium => event.obtainium,
        BuffType::Octeract => event.octeract,
        BuffType::OneMind => {
            if one_mind_unlocked {
                event.one_mind
            } else {
                0.0
            }
        }
        BuffType::BlueberryTime => event.blueberry_time,
        BuffType::AmbrosiaLuck => event.ambrosia_luck,
    }
}

/// Per-buff value coming from the Happy Hour Bell consumable stack.
/// The caller passes the total `HAPPY_HOUR_BELL.amount`; the
/// "interval" used in scaling is `amount - 1`. Returns `0` when no
/// bells are queued.
///
/// Only `Quark / Cubes / Offering / Obtainium / BlueberryTime /
/// AmbrosiaLuck` have non-trivial happy-hour contributions; the rest
/// return `0`.
#[must_use]
pub fn consumable_event_buff(buff: BuffType, happy_hour_bell_amount: f64) -> f64 {
    if happy_hour_bell_amount == 0.0 {
        return 0.0;
    }
    let interval = happy_hour_bell_amount - 1.0;
    match buff {
        BuffType::Quark => 0.25 + 0.025 * interval,
        BuffType::Cubes => 0.5 + 0.05 * interval,
        BuffType::Offering | BuffType::Obtainium => 0.5 + 0.05 * interval,
        BuffType::BlueberryTime | BuffType::AmbrosiaLuck => 0.1 + 0.01 * interval,
        BuffType::GoldenQuark
        | BuffType::PowderConversion
        | BuffType::AscensionSpeed
        | BuffType::GlobalSpeed
        | BuffType::AscensionScore
        | BuffType::AntSacrifice
        | BuffType::Octeract
        | BuffType::OneMind => 0.0,
    }
}

/// Sum of the event buff and the consumable buff for a given source.
#[must_use]
pub fn calculate_event_source_buff(
    buff: BuffType,
    event: Option<&GameEventBuffs>,
    one_mind_unlocked: bool,
    happy_hour_bell_amount: f64,
) -> f64 {
    get_event_buff(buff, event, one_mind_unlocked)
        + consumable_event_buff(buff, happy_hour_bell_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_event() -> GameEventBuffs {
        GameEventBuffs {
            quark: 0.1,
            golden_quark: 0.2,
            cubes: 0.3,
            powder_conversion: 0.4,
            ascension_speed: 0.5,
            global_speed: 0.6,
            ascension_score: 0.7,
            ant_sacrifice: 0.8,
            offering: 0.9,
            obtainium: 1.0,
            octeract: 1.1,
            blueberry_time: 1.2,
            ambrosia_luck: 1.3,
            one_mind: 1.4,
        }
    }

    #[test]
    fn get_event_buff_no_event_returns_zero() {
        assert_eq!(get_event_buff(BuffType::Quark, None, false), 0.0);
    }

    #[test]
    fn get_event_buff_dispatches_to_correct_field() {
        let event = sample_event();
        assert_eq!(get_event_buff(BuffType::Cubes, Some(&event), false), 0.3);
        assert_eq!(
            get_event_buff(BuffType::AmbrosiaLuck, Some(&event), false),
            1.3
        );
    }

    #[test]
    fn one_mind_requires_unlock() {
        let event = sample_event();
        assert_eq!(get_event_buff(BuffType::OneMind, Some(&event), false), 0.0);
        assert_eq!(get_event_buff(BuffType::OneMind, Some(&event), true), 1.4);
    }

    #[test]
    fn consumable_zero_bells_returns_zero() {
        assert_eq!(consumable_event_buff(BuffType::Quark, 0.0), 0.0);
    }

    #[test]
    fn consumable_quark_one_bell_is_base_25_pct() {
        // amount = 1, interval = 0 → 0.25
        assert_eq!(consumable_event_buff(BuffType::Quark, 1.0), 0.25);
    }

    #[test]
    fn consumable_cubes_scales_per_extra_bell() {
        // amount = 5, interval = 4 → 0.5 + 0.05*4 = 0.7
        assert_eq!(consumable_event_buff(BuffType::Cubes, 5.0), 0.7);
    }

    #[test]
    fn consumable_unaffected_buff_returns_zero() {
        assert_eq!(consumable_event_buff(BuffType::GoldenQuark, 5.0), 0.0);
    }

    #[test]
    fn calculate_event_source_buff_sums_both() {
        let event = sample_event();
        // event.cubes = 0.3; happy_hour 5 bells → 0.7
        let result = calculate_event_source_buff(BuffType::Cubes, Some(&event), false, 5.0);
        assert!((result - 1.0).abs() < 1e-12);
    }
}
