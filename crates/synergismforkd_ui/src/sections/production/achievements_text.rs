//! Achievement display text — the 509 `"Name: Description"` strings,
//! embedded from `assets/achievements_en.json` (extracted from the legacy
//! `achievements.descriptions` translation block). Shown in full on the
//! cell tooltip.

use std::sync::LazyLock;

static RAW: &str = include_str!("../../../assets/achievements_en.json");

static DESCRIPTIONS: LazyLock<Vec<String>> =
    LazyLock::new(|| serde_json::from_str(RAW).unwrap_or_default());

/// Full `"Name: Description"` text for achievement `index` (0-based), or an
/// empty string if out of range / unfilled.
#[must_use]
pub fn full(index: usize) -> &'static str {
    DESCRIPTIONS.get(index).map_or("", String::as_str)
}

#[cfg(test)]
mod tests {
    use super::*;
    use synergismforkd_logic::state::achievements::ACHIEVEMENTS_LEN;

    #[test]
    fn descriptions_parse_and_cover_every_slot() {
        assert_eq!(DESCRIPTIONS.len(), ACHIEVEMENTS_LEN);
        assert_eq!(full(1), "A Loyal Employee: Hire your first Worker.");
    }
}
