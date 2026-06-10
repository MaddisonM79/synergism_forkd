//! Browser glue: clocks, visibility, clipboard, and UI-preferences storage.
//! Everything here touches `web-sys` / `js-sys` and is wasm-only (the module
//! is cfg-gated in `lib.rs`).

use synergismforkd_ui::bridge::UiPrefs;

/// `localStorage` key for [`UiPrefs`] (theme, notation, buy amount). UI-only
/// — deliberately separate from the save blob.
pub const PREFS_KEY: &str = "SynergismForkdPrefs";

/// Wall-clock, Unix epoch ms (stamps saves).
#[must_use]
pub fn now_ms() -> u64 {
    js_sys::Date::now() as u64
}

/// Monotonic seconds for tick dt (`performance.now()`; falls back to the
/// wall-clock if the API is missing).
#[must_use]
pub fn perf_now_s() -> f64 {
    web_sys::window()
        .and_then(|w| w.performance())
        .map_or_else(|| js_sys::Date::now() / 1000.0, |p| p.now() / 1000.0)
}

/// `document.hidden` — the loop force-saves on the visible→hidden edge
/// (tab close / switch), replacing a `beforeunload` handler.
#[must_use]
pub fn document_hidden() -> bool {
    web_sys::window()
        .and_then(|w| w.document())
        .map(|d| d.hidden())
        .unwrap_or(false)
}

/// Write text to the clipboard. Resolves `true` on success.
pub async fn clipboard_write(text: &str) -> bool {
    let Some(window) = web_sys::window() else {
        return false;
    };
    let clipboard = window.navigator().clipboard();
    wasm_bindgen_futures::JsFuture::from(clipboard.write_text(text))
        .await
        .is_ok()
}

/// Load persisted UI prefs (defaults on absence/corruption — prefs are
/// low-stakes).
#[must_use]
pub fn load_prefs() -> UiPrefs {
    let stored = web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item(PREFS_KEY).ok().flatten());
    stored
        .and_then(|json| serde_json::from_str(&json).ok())
        .unwrap_or_default()
}

/// Persist UI prefs (failures swallowed — next change retries).
pub fn save_prefs(prefs: &UiPrefs) {
    let Ok(json) = serde_json::to_string(prefs) else {
        return;
    };
    if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
        let _ = storage.set_item(PREFS_KEY, &json);
    }
}
