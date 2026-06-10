//! Achievement display text — the 509 `"Name: Description"` strings,
//! embedded from `assets/achievements_en.json` (extracted from the legacy
//! `achievements.descriptions` translation block). Shown in full on the
//! cell tooltip.

use std::sync::LazyLock;

static RAW: &str = include_str!("../../../assets/achievements_en.json");

static DESCRIPTIONS: LazyLock<Vec<String>> =
    LazyLock::new(|| serde_json::from_str(RAW).unwrap_or_default());

/// Full `"Name: Requirement"` text for achievement `index` (0-based), or an
/// empty string if out of range / unfilled.
#[must_use]
pub fn full(index: usize) -> &'static str {
    DESCRIPTIONS.get(index).map_or("", String::as_str)
}

/// The achievement's name — the clause before the first sentence-ending
/// punctuation. `:` is dropped (it's a label separator); `?`/`!` are kept
/// (they belong to the name, e.g. "Are you broke yet?"). Falls back to the
/// full string when there's no separator.
#[must_use]
pub fn name(index: usize) -> &'static str {
    let s = full(index);
    match s
        .char_indices()
        .find(|&(_, c)| matches!(c, ':' | '?' | '!'))
    {
        Some((i, ':')) => &s[..i],
        Some((i, c)) => &s[..i + c.len_utf8()],
        _ => s,
    }
}

/// The requirement clause — the text after the name separator. Falls back to
/// the full string when there's no separator.
#[must_use]
pub fn requirement(index: usize) -> &'static str {
    let s = full(index);
    match s
        .char_indices()
        .find(|&(_, c)| matches!(c, ':' | '?' | '!'))
    {
        Some((i, c)) => s[i + c.len_utf8()..].trim_start(),
        None => s,
    }
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

    #[test]
    fn name_and_requirement_split_on_each_separator() {
        // colon separator
        assert_eq!(name(1), "A Loyal Employee");
        assert_eq!(requirement(1), "Hire your first Worker.");
        // question-mark separator (achievement 130)
        assert_eq!(name(130), "Are you broke yet?");
        assert_eq!(requirement(130), "Complete {[Cost++]} five times.");
        // exclamation separator (achievement 210)
        assert_eq!(name(210), "Science!");
        assert_eq!(
            requirement(210),
            "Clear 'No Reincarnation' challenge thirty times."
        );
    }
}
