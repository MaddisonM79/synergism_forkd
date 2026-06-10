//! Minimal i18n: flat dot-notation keys from `assets/translations/en.json`,
//! embedded at compile time and parsed once.
//!
//! Per CLAUDE.md, ALL user-facing text routes through here from day one —
//! components never hardcode English. Locale switching, pluralization, and
//! interpolation are a later milestone; a missing key renders as the key
//! itself so gaps are visible in the UI instead of silently blank.

use std::collections::HashMap;
use std::sync::LazyLock;

static EN: &str = include_str!("../../../assets/translations/en.json");

static TABLE: LazyLock<HashMap<String, String>> = LazyLock::new(|| {
    // A malformed translations file is a build-content bug; surfacing every
    // key as itself (empty table) is the visible-by-design failure mode.
    serde_json::from_str(EN).unwrap_or_default()
});

/// Look up a translation key. Unknown keys echo back verbatim.
#[must_use]
pub fn t(key: &str) -> &str {
    match TABLE.get(key) {
        Some(value) => value.as_str(),
        None => key,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_key_resolves() {
        assert_eq!(t("nav.group.production"), "Production");
        assert_eq!(t("buildings.coin.1"), "Workers");
    }

    #[test]
    fn unknown_key_echoes() {
        assert_eq!(t("definitely.not.a.key"), "definitely.not.a.key");
    }

    #[test]
    fn table_parses_and_is_nonempty() {
        assert!(TABLE.len() > 20, "en.json failed to parse");
    }
}
