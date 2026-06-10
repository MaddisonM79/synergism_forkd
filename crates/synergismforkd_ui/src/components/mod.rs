//! Shared component primitives. Styling lives in
//! `assets/styles/components.css`; these files hold structure + behavior
//! only.

mod icon;

pub use icon::{Resource, ResourceIcon};

use dioxus::prelude::*;
use synergismforkd_bignum::Decimal;

use crate::bridge::{use_bridge, DialogKind, DialogRequest, ToastKind};
use crate::i18n::t;

/// The number leaf. Every on-screen quantity renders through this so the
/// player's notation preference applies game-wide.
#[component]
pub fn Num(value: Decimal, #[props(default = false)] rate: bool) -> Element {
    let bridge = use_bridge();
    let notation = bridge.prefs.read().notation;
    let text = if rate {
        crate::format::format_value(value, notation)
    } else {
        crate::format::format_count(value, notation)
    };
    rsx! {
        span { class: "sf-num", "{text}" }
    }
}

/// Hover/focus tooltip. Pure CSS reveal; `tip` renders into the bubble.
#[component]
pub fn Tooltip(tip: Element, #[props(default = false)] down: bool, children: Element) -> Element {
    // `down` opens the bubble below the anchor — for elements near the top
    // edge (the HUD), where an upward bubble would clip off-screen.
    let tip_cls = if down { "sf-tip sf-tip-down" } else { "sf-tip" };
    rsx! {
        span { class: "sf-tip-wrap", tabindex: "0",
            {children}
            span { class: tip_cls, {tip} }
        }
    }
}

/// A titled, collapsible section. Clicking the header toggles its body.
/// `title` is plain text; `open` sets the initial state.
#[component]
pub fn Collapsible(
    title: String,
    #[props(default = true)] open: bool,
    children: Element,
) -> Element {
    let mut is_open = use_signal(|| open);
    rsx! {
        section { class: "sf-collapsible",
            button {
                class: "sf-collapsible-head",
                "aria-expanded": "{is_open()}",
                onclick: move |_| is_open.toggle(),
                span { class: if is_open() { "sf-collapsible-chevron open" } else { "sf-collapsible-chevron" }, "▸" }
                span { class: "sf-collapsible-title", "{title}" }
            }
            if is_open() {
                div { class: "sf-collapsible-body", {children} }
            }
        }
    }
}

/// Thin progress bar; `fraction` clamped to 0..=1.
#[component]
pub fn Progress(fraction: f64) -> Element {
    let pct = (fraction.clamp(0.0, 1.0) * 100.0).round();
    rsx! {
        div { class: "sf-progress",
            div { style: "width: {pct}%" }
        }
    }
}

/// Bottom-right toast stack. Lifetime is CSS-driven (the `sf-toast-life`
/// animation); `onanimationend` removes the toast from state, so the loop
/// never has to touch the toasts signal on a timer.
#[component]
pub fn ToastStack() -> Element {
    let bridge = use_bridge();
    let toasts = bridge.toasts.read();
    rsx! {
        div { class: "sf-toast-stack",
            for toast in toasts.iter().cloned() {
                div {
                    key: "{toast.id}",
                    class: match toast.kind {
                        ToastKind::Info => "sf-toast info",
                        ToastKind::Success => "sf-toast success",
                        ToastKind::Warn => "sf-toast warn",
                        ToastKind::Achievement => "sf-toast achievement",
                    },
                    onanimationend: move |_| bridge.dismiss_toast(toast.id),
                    div { class: "sf-toast-body",
                        if let Some(title) = toast.title.clone() {
                            div { class: "sf-toast-title", "{title}" }
                        }
                        div { class: "sf-toast-text", "{toast.text}" }
                    }
                    button {
                        onclick: move |_| bridge.dismiss_toast(toast.id),
                        "✕"
                    }
                }
            }
        }
    }
}

/// The single modal layer. One dialog at a time; opening a new one replaces
/// the old (no nesting, by design).
#[component]
pub fn DialogLayer() -> Element {
    let bridge = use_bridge();
    let Some(request) = bridge.dialog.read().clone() else {
        return rsx! {};
    };
    rsx! {
        div { class: "sf-dialog-overlay",
            div { class: "sf-dialog",
                h2 { "{request.title}" }
                div { class: "sf-dialog-body", "{request.body}" }
                match request.kind {
                    DialogKind::Alert => rsx! {
                        div { class: "sf-dialog-actions",
                            button {
                                class: "primary",
                                onclick: move |_| bridge.close_dialog(),
                                {t("dialogs.ok")}
                            }
                        }
                    },
                    DialogKind::Confirm { on_result } => rsx! {
                        div { class: "sf-dialog-actions",
                            button {
                                onclick: move |_| {
                                    bridge.close_dialog();
                                    on_result.call(false);
                                },
                                {t("dialogs.cancel")}
                            }
                            button {
                                class: "primary",
                                onclick: move |_| {
                                    bridge.close_dialog();
                                    on_result.call(true);
                                },
                                {t("dialogs.ok")}
                            }
                        }
                    },
                    DialogKind::Prompt { on_submit } => rsx! {
                        PromptBody { on_submit }
                    },
                    DialogKind::Progress { progress, on_skip } => rsx! {
                        ProgressBody { progress, on_skip }
                    },
                }
            }
        }
    }
}

/// Prompt input + actions (own component so the input hook is stable).
#[component]
fn PromptBody(on_submit: Callback<Option<String>>) -> Element {
    let bridge = use_bridge();
    let mut input = use_signal(String::new);
    rsx! {
        textarea {
            value: "{input}",
            autofocus: true,
            oninput: move |evt| input.set(evt.value()),
        }
        div { class: "sf-dialog-actions",
            button {
                onclick: move |_| {
                    bridge.close_dialog();
                    on_submit.call(None);
                },
                {t("dialogs.cancel")}
            }
            button {
                class: "primary",
                onclick: move |_| {
                    bridge.close_dialog();
                    on_submit.call(Some(input.peek().clone()));
                },
                {t("dialogs.ok")}
            }
        }
    }
}

/// Live progress body (offline catch-up): bar + elapsed/total + skip.
#[component]
fn ProgressBody(
    progress: Signal<crate::bridge::DialogProgress>,
    on_skip: Option<Callback<()>>,
) -> Element {
    let p = progress.read();
    let fraction = if p.total_s > 0.0 {
        p.done_s / p.total_s
    } else {
        1.0
    };
    let done = crate::format::format_time_short(p.done_s);
    let total = crate::format::format_time_short(p.total_s);
    rsx! {
        Progress { fraction }
        div { class: "sf-dialog-body sf-num", "{done} / {total}" }
        if let Some(skip) = on_skip {
            div { class: "sf-dialog-actions",
                button {
                    class: "primary",
                    onclick: move |_| skip.call(()),
                    {t("dialogs.offline.skip")}
                }
            }
        }
    }
}

/// Convenience constructors for the common dialogs.
impl crate::bridge::GameBridge {
    /// OK-only message box.
    pub fn alert(&self, title_key: &str, body_key: &str) {
        self.open_dialog(DialogRequest {
            title: t(title_key).to_string(),
            body: t(body_key).to_string(),
            kind: DialogKind::Alert,
        });
    }

    /// OK/Cancel; `on_confirm` runs only on OK.
    pub fn confirm(&self, title_key: &str, body_key: &str, on_confirm: Callback<()>) {
        self.open_dialog(DialogRequest {
            title: t(title_key).to_string(),
            body: t(body_key).to_string(),
            kind: DialogKind::Confirm {
                on_result: Callback::new(move |ok| {
                    if ok {
                        on_confirm.call(());
                    }
                }),
            },
        });
    }

    /// Text prompt; `on_submit` gets `Some(text)` on OK.
    pub fn prompt(&self, title_key: &str, body_key: &str, on_submit: Callback<Option<String>>) {
        self.open_dialog(DialogRequest {
            title: t(title_key).to_string(),
            body: t(body_key).to_string(),
            kind: DialogKind::Prompt { on_submit },
        });
    }

    /// Shorthand toast helpers.
    pub fn toast_info(&self, key: &str) {
        self.toast(ToastKind::Info, t(key));
    }
    pub fn toast_success(&self, key: &str) {
        self.toast(ToastKind::Success, t(key));
    }
    pub fn toast_warn(&self, key: &str) {
        self.toast(ToastKind::Warn, t(key));
    }
}
