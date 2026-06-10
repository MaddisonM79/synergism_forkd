//! Short human-readable durations for timers and the offline-progress
//! dialog: two units max, largest first (`3h 12m`, `45s`, `2d 5h`).

/// Format a duration in seconds. Negative or non-finite inputs render `"0s"`.
#[must_use]
pub fn format_time_short(seconds: f64) -> String {
    if !seconds.is_finite() || seconds <= 0.0 {
        return "0s".to_string();
    }
    let total = seconds.floor() as u64;
    let (days, rem) = (total / 86_400, total % 86_400);
    let (hours, rem) = (rem / 3_600, rem % 3_600);
    let (mins, secs) = (rem / 60, rem % 60);

    if days > 0 {
        join(days, "d", hours, "h")
    } else if hours > 0 {
        join(hours, "h", mins, "m")
    } else if mins > 0 {
        join(mins, "m", secs, "s")
    } else {
        format!("{secs}s")
    }
}

/// `"3h 12m"`, dropping a zero minor unit (`"3h"`, not `"3h 0m"`).
fn join(major: u64, major_unit: &str, minor: u64, minor_unit: &str) -> String {
    if minor == 0 {
        format!("{major}{major_unit}")
    } else {
        format!("{major}{major_unit} {minor}{minor_unit}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_units_largest_first() {
        assert_eq!(format_time_short(45.0), "45s");
        assert_eq!(format_time_short(192.0), "3m 12s");
        assert_eq!(format_time_short(11_520.0), "3h 12m");
        assert_eq!(format_time_short(190_800.0), "2d 5h");
    }

    #[test]
    fn zero_minor_unit_dropped() {
        assert_eq!(format_time_short(3_600.0), "1h");
        assert_eq!(format_time_short(60.0), "1m");
        assert_eq!(format_time_short(86_400.0), "1d");
    }

    #[test]
    fn degenerate_inputs() {
        assert_eq!(format_time_short(0.0), "0s");
        assert_eq!(format_time_short(-5.0), "0s");
        assert_eq!(format_time_short(f64::NAN), "0s");
        assert_eq!(format_time_short(0.4), "0s");
    }
}
