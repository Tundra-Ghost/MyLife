//! The tray agent: keeps MyLife alive in the system tray.
//!
//! Closing the main window hides it instead of quitting, so reminders can
//! keep firing. The 60-second rules and reminders loop is added in the
//! next Phase 1 step. It will live here.

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, WindowEvent,
};

/// Event the UI listens for to open quick capture.
pub const QUICK_CAPTURE_EVENT: &str = "quick-capture";

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
