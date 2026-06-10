//! Saves: export to clipboard, import from a pasted blob, and the danger
//! zone (three reset tiers). All storage side effects go through
//! [`HostCommand`]s; the loop driver executes them and reports back via
//! toasts. Settings-only reset is pure UI (defaulting the prefs signal —
//! the host's persistence effect rewrites storage).

use dioxus::prelude::*;

use crate::bridge::{use_bridge, HostCommand, UiPrefs};
use crate::i18n::t;

#[component]
pub fn Saves() -> Element {
    let bridge = use_bridge();

    let on_import = use_callback(move |blob: Option<String>| {
        if let Some(blob) = blob {
            let trimmed = blob.trim();
            if !trimmed.is_empty() {
                bridge.dispatch_host(HostCommand::ImportSave(trimmed.to_string()));
            }
        }
    });

    rsx! {
        div { class: "sf-section-head",
            h1 { {t("settings.saves.title")} }
        }
        div { class: "sf-settings-list",
            div { class: "sf-settings-row",
                div { class: "text",
                    div { {t("settings.saves.export")} }
                    div { class: "hint", {t("settings.saves.export_hint")} }
                }
                button {
                    onclick: move |_| bridge.dispatch_host(HostCommand::ExportSave),
                    {t("settings.saves.export")}
                }
            }
            div { class: "sf-settings-row",
                div { class: "text",
                    div { {t("settings.saves.import")} }
                    div { class: "hint", {t("settings.saves.import_hint")} }
                }
                button {
                    onclick: move |_| {
                        bridge.prompt("dialogs.import.title", "dialogs.import.body", on_import);
                    },
                    {t("settings.saves.import")}
                }
            }
        }
        DangerZone {}
    }
}

/// The three reset tiers, each behind its own confirm dialog.
#[component]
fn DangerZone() -> Element {
    let bridge = use_bridge();

    let reset_state = use_callback(move |()| {
        bridge.dispatch_host(HostCommand::HardReset);
    });
    let reset_settings = use_callback(move |()| {
        let mut prefs = bridge.prefs;
        prefs.set(UiPrefs::default());
        bridge.toast_info("toasts.settings_reset");
    });
    let reset_everything = use_callback(move |()| {
        bridge.dispatch_host(HostCommand::ResetEverything);
    });

    rsx! {
        div { class: "sf-danger-zone",
            h2 { {t("settings.danger.title")} }
            div { class: "sf-settings-row",
                div { class: "text",
                    div { {t("settings.danger.reset_state")} }
                    div { class: "hint", {t("settings.danger.reset_state_hint")} }
                }
                button {
                    class: "sf-danger-btn",
                    onclick: move |_| {
                        bridge.confirm(
                            "dialogs.reset_state.title",
                            "dialogs.reset_state.body",
                            reset_state,
                        );
                    },
                    {t("settings.danger.reset")}
                }
            }
            div { class: "sf-settings-row",
                div { class: "text",
                    div { {t("settings.danger.reset_settings")} }
                    div { class: "hint", {t("settings.danger.reset_settings_hint")} }
                }
                button {
                    class: "sf-danger-btn",
                    onclick: move |_| {
                        bridge.confirm(
                            "dialogs.reset_settings.title",
                            "dialogs.reset_settings.body",
                            reset_settings,
                        );
                    },
                    {t("settings.danger.reset")}
                }
            }
            div { class: "sf-settings-row",
                div { class: "text",
                    div { {t("settings.danger.reset_all")} }
                    div { class: "hint", {t("settings.danger.reset_all_hint")} }
                }
                button {
                    class: "sf-danger-btn",
                    onclick: move |_| {
                        bridge.confirm(
                            "dialogs.reset_all.title",
                            "dialogs.reset_all.body",
                            reset_everything,
                        );
                    },
                    {t("settings.danger.reset")}
                }
            }
        }
    }
}
