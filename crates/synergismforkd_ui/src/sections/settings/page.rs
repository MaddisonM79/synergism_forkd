//! The consolidated Settings page: one scrolling surface with Appearance,
//! Saves, and a quarantined Danger Zone. All storage side effects go
//! through [`HostCommand`]s (executed by the loop driver); the appearance
//! controls write `prefs` directly (the host persists them).

use dioxus::prelude::*;

use crate::bridge::{use_bridge, HostCommand, UiPrefs};
use crate::i18n::t;
use crate::theme::Theme;

#[component]
pub fn Settings() -> Element {
    rsx! {
        div { class: "sf-section-head",
            h1 { {t("nav.group.settings")} }
        }
        AppearanceSection {}
        ConfirmationsSection {}
        SavesSection {}
        DangerZone {}
    }
}

/// Which reset tier a [`ConfirmRow`] toggles. Mirrors the three reset cards on
/// the Buildings page; kept local so Settings doesn't depend on that module.
#[derive(Clone, Copy, PartialEq)]
enum ConfirmTier {
    Prestige,
    Transcension,
    Reincarnation,
}

impl ConfirmTier {
    const ALL: [ConfirmTier; 3] = [Self::Prestige, Self::Transcension, Self::Reincarnation];

    /// Row label — reuses the reset-button labels.
    fn label_key(self) -> &'static str {
        match self {
            Self::Prestige => "buildings.prestige",
            Self::Transcension => "buildings.transcend",
            Self::Reincarnation => "buildings.reincarnate",
        }
    }

    fn get(self, p: &UiPrefs) -> bool {
        match self {
            Self::Prestige => p.confirm_prestige,
            Self::Transcension => p.confirm_transcension,
            Self::Reincarnation => p.confirm_reincarnation,
        }
    }

    fn set(self, p: &mut UiPrefs, on: bool) {
        match self {
            Self::Prestige => p.confirm_prestige = on,
            Self::Transcension => p.confirm_transcension = on,
            Self::Reincarnation => p.confirm_reincarnation = on,
        }
    }
}

/// Confirmations: one independent On/Off toggle per reset tier. With a toggle
/// Off, that reset fires immediately; On pops the confirm dialog.
#[component]
fn ConfirmationsSection() -> Element {
    rsx! {
        section { class: "sf-settings-block",
            h2 { {t("settings.confirmations.title")} }
            div { class: "sf-settings-subhint", {t("settings.confirmations.hint")} }
            for tier in ConfirmTier::ALL {
                ConfirmRow { key: "{tier.label_key()}", tier }
            }
        }
    }
}

/// A single reset-tier confirmation toggle row.
#[component]
fn ConfirmRow(tier: ConfirmTier) -> Element {
    let bridge = use_bridge();
    let on = tier.get(&bridge.prefs.read());
    rsx! {
        div { class: "sf-settings-row",
            div { class: "text",
                div { {t(tier.label_key())} }
            }
            div { class: "sf-seg",
                button {
                    class: if on { "active" } else { "" },
                    onclick: move |_| {
                        let mut prefs = bridge.prefs;
                        let mut w = prefs.write();
                        tier.set(&mut w, true);
                    },
                    {t("settings.on")}
                }
                button {
                    class: if on { "" } else { "active" },
                    onclick: move |_| {
                        let mut prefs = bridge.prefs;
                        let mut w = prefs.write();
                        tier.set(&mut w, false);
                    },
                    {t("settings.off")}
                }
            }
        }
    }
}

/// Appearance: theme picker (more controls land here as they're built).
#[component]
fn AppearanceSection() -> Element {
    let bridge = use_bridge();
    let current = bridge.prefs.read().theme;
    rsx! {
        section { class: "sf-settings-block",
            h2 { {t("settings.appearance.title")} }
            div { class: "sf-settings-row",
                div { class: "text",
                    div { {t("settings.themes.title")} }
                }
                div { class: "sf-seg",
                    for theme in Theme::ALL {
                        button {
                            key: "{theme.css_value()}",
                            class: if theme == current { "active" } else { "" },
                            onclick: move |_| {
                                let mut prefs = bridge.prefs;
                                prefs.write().theme = theme;
                            },
                            {t(theme.label_key())}
                        }
                    }
                }
            }
        }
    }
}

/// Saves: export to clipboard, import from a pasted blob.
#[component]
fn SavesSection() -> Element {
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
        section { class: "sf-settings-block",
            h2 { {t("settings.saves.title")} }
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
