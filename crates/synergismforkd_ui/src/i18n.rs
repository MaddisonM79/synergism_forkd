//! Minimal i18n: dot-notation keys from `assets/translations/en.json`,
//! embedded at compile time and parsed once.
//!
//! The JSON may be **nested** — objects are recursively flattened into the
//! dotted keys components use (`{ "hud": { "coins": "Coins" } }` →
//! `hud.coins`), so the file can be grouped per area/item while lookups stay
//! flat. A plain flat `"a.b": "v"` map still works (a top-level leaf).
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
    let root: serde_json::Value = serde_json::from_str(EN).unwrap_or(serde_json::Value::Null);
    let mut table = HashMap::new();
    flatten(&root, String::new(), &mut table);
    table
});

/// Recursively flatten a JSON value into dotted keys. Objects descend
/// (`prefix.child`); string leaves are inserted; non-string leaves are
/// ignored (translations are always strings).
fn flatten(value: &serde_json::Value, prefix: String, out: &mut HashMap<String, String>) {
    match value {
        serde_json::Value::Object(map) => {
            for (key, child) in map {
                let path = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{prefix}.{key}")
                };
                flatten(child, path, out);
            }
        }
        serde_json::Value::String(s) => {
            out.insert(prefix, s.clone());
        }
        _ => {}
    }
}

/// Look up a translation key. Unknown keys echo back verbatim.
#[must_use]
pub fn t(key: &str) -> &str {
    match TABLE.get(key) {
        Some(value) => value.as_str(),
        None => key,
    }
}

/// Look up a key and substitute `{{name}}` placeholders with the given values
/// (the legacy i18next interpolation, scoped to what the Upgrades tab needs).
/// An unknown key echoes back verbatim (with placeholders intact).
#[must_use]
pub fn t_args(key: &str, args: &[(&str, &str)]) -> String {
    let mut s = t(key).to_string();
    for (name, value) in args {
        let pat = ["{{", name, "}}"].concat();
        s = s.replace(&pat, value);
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_key_resolves() {
        assert_eq!(t("nav.group.production"), "Production");
        assert_eq!(t("buildings.coin.1"), "Workers");
        // Per-item nested keys (resources / runes / crystals) resolve from the
        // real, now-grouped file.
        assert_eq!(t("resources.coins.name"), "Coins");
        assert_eq!(t("runes.speed.name"), "Speed");
        assert!(t("crystals.1.desc").starts_with("Gain +1%"));
        assert_eq!(t("crystals.1.formula"), "(1 + level / 100)^(AP)");
    }

    #[test]
    fn unknown_key_echoes() {
        assert_eq!(t("definitely.not.a.key"), "definitely.not.a.key");
    }

    #[test]
    fn table_parses_and_is_nonempty() {
        assert!(TABLE.len() > 20, "en.json failed to parse");
    }

    #[test]
    fn nested_objects_flatten_to_dotted_keys() {
        let json = serde_json::json!({
            "hud": { "coins": "Coins" },
            "runes": { "name": { "speed": "Speed" } },
            "flat.key": "Flat",
        });
        let mut out = HashMap::new();
        flatten(&json, String::new(), &mut out);
        assert_eq!(out.get("hud.coins").map(String::as_str), Some("Coins"));
        assert_eq!(
            out.get("runes.name.speed").map(String::as_str),
            Some("Speed")
        );
        // A dotted top-level leaf is preserved verbatim (flat-file compat).
        assert_eq!(out.get("flat.key").map(String::as_str), Some("Flat"));
    }
}
