//! The web root component: boots the save, provides the [`GameBridge`],
//! spawns the game loop, persists UI prefs, and renders the
//! platform-agnostic `App`.

use std::cell::RefCell;
use std::rc::Rc;

use dioxus::prelude::*;
use synergismforkd_ui::bridge::{DialogProgress, GameBridge};
use synergismforkd_ui::App;

use crate::catch_up::CatchUpHandles;
use crate::platform;
use crate::save_host::{web::LocalStorageBackend, SaveHost, SAVE_KEY};

#[component]
pub fn WebRoot() -> Element {
    // One-time boot: load the save + prefs. The Rc<RefCell<Option<…>>>
    // cells exist so the booted values can be MOVED into the context
    // provider / loop task below (hooks re-run on render; boot must not).
    let boot = use_hook(|| {
        let prefs = platform::load_prefs();
        let (state, outcome, host) = SaveHost::boot(LocalStorageBackend::new(SAVE_KEY));
        (
            Rc::new(RefCell::new(Some((state, prefs)))),
            Rc::new(RefCell::new(Some((host, outcome)))),
        )
    });
    let (state_cell, host_cell) = boot;

    let bridge = GameBridge::provide(move || {
        state_cell
            .borrow_mut()
            .take()
            .expect("context provider runs once")
    });

    // Catch-up scaffolding the loop task can't create itself (signals and
    // callbacks are component-scope objects).
    let progress = use_signal(DialogProgress::default);
    let mut skip = use_signal(|| false);
    let on_skip = use_callback(move |()| skip.set(true));
    let handles = CatchUpHandles {
        progress,
        skip,
        on_skip,
    };

    // Spawn the loop exactly once.
    use_hook(move || {
        let (host, outcome) = host_cell.borrow_mut().take().expect("loop spawn runs once");
        spawn(async move {
            crate::loop_driver::run(bridge, host, outcome, handles).await;
        });
    });

    // Persist UI prefs whenever they change (subscribes via the read).
    use_effect(move || {
        let prefs = *bridge.prefs.read();
        platform::save_prefs(&prefs);
    });

    rsx! {
        App {}
    }
}
