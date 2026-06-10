//! Saves: export to clipboard, import from a pasted blob, hard reset.
//! All three are host side effects — the section only queues
//! [`HostCommand`]s; the loop driver executes them and reports back via
//! toasts.

use dioxus::prelude::*;

use crate::bridge::{use_bridge, HostCommand};
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
    let on_reset = use_callback(move |()| {
        bridge.dispatch_host(HostCommand::HardReset);
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
            div { class: "sf-settings-row",
                div { class: "text",
                    div { {t("settings.saves.hard_reset")} }
                    div { class: "hint", {t("settings.saves.hard_reset_hint")} }
                }
                button {
                    style: "background: var(--danger); border-color: var(--danger); color: var(--text-inverse)",
                    onclick: move |_| {
                        bridge.confirm(
                            "dialogs.hard_reset.title",
                            "dialogs.hard_reset.body",
                            on_reset,
                        );
                    },
                    {t("settings.saves.hard_reset")}
                }
            }
        }
    }
}
