//! The tray agent: keeps MyLife alive in the system tray.
//!
//! Closing the main window hides it instead of quitting, so reminders keep
//! firing. Every 60 seconds the agent runs the rules and shows any
//! reminders that are due. It needs the database unlocked, so after a
//! restart it waits until the app password is entered.

use chrono::Utc;
use tauri_plugin_notification::NotificationExt;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, WindowEvent,
};

use crate::rules::runner;
use crate::scheduler::reminders;
use crate::AppState;

/// Event the UI listens for to open quick capture.
pub const QUICK_CAPTURE_EVENT: &str = "quick-capture";
/// Event the UI listens for to refresh reminders and tasks.
pub const CHANGED_EVENT: &str = "data-changed";
/// Spec: check rules and reminders every 60 seconds.
pub const TICK_SECONDS: u64 = 60;

/// Starts the background loop. Runs for the life of the app.
pub fn start(app: AppHandle) {
    std::thread::spawn(move || loop {
        tick(&app);
        std::thread::sleep(std::time::Duration::from_secs(TICK_SECONDS));
    });
}

/// One pass: run rules, then show due reminders. Does nothing while locked.
pub fn tick(app: &AppHandle) {
    let state = app.state::<AppState>();
    let notices = {
        let Ok(guard) = state.conn.lock() else { return };
        let Some(conn) = guard.as_ref() else { return };
        let now = Utc::now();
        let result = runner::run(conn, now).and_then(|_| reminders::deliver(conn, now));
        match result {
            Ok(n) => n,
            Err(e) => {
                // Never log reminder text here (spec: no personal details in logs).
                eprintln!("Agent tick failed: {}", e.to_string().chars().take(80).collect::<String>());
                return;
            }
        }
    };
    for n in &notices {
        let mut b = app.notification().builder().title("MyLife").body(&n.title);
        if n.level >= 3 {
            b = b.title("MyLife: still waiting on this");
        }
        let _ = b.show();
    }
    let _ = app.emit(CHANGED_EVENT, ());
}

/// Brings the main window to the front.
pub fn show_main(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.unminimize();
        let _ = w.show();
        let _ = w.set_focus();
    }
}

/// Shows the window and asks the UI to open quick capture.
pub fn open_quick_capture(app: &AppHandle) {
    show_main(app);
    let _ = app.emit(QUICK_CAPTURE_EVENT, ());
}

pub fn setup_tray(app: &AppHandle) -> tauri::Result<()> {
    let open = MenuItem::with_id(app, "open", "Open MyLife", true, None::<&str>)?;
    let capture = MenuItem::with_id(app, "capture", "Quick capture", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open, &capture, &quit])?;

    let mut tray = TrayIconBuilder::with_id("main")
        .tooltip("MyLife")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open" => show_main(app),
            "capture" => open_quick_capture(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                show_main(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.build(app)?;

    // Close button hides to tray instead of quitting.
    if let Some(w) = app.get_webview_window("main") {
        let w2 = w.clone();
        w.on_window_event(move |e| {
            if let WindowEvent::CloseRequested { api, .. } = e {
                api.prevent_close();
                let _ = w2.hide();
            }
        });
    }
    Ok(())
}
