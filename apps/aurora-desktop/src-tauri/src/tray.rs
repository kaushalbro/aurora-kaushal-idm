use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{Emitter, Manager, Runtime};

pub fn setup_tray<R: Runtime>(app: &tauri::App<R>) -> Result<(), Box<dyn std::error::Error>> {
    let handle = app.handle();

    // Menu items
    let show_item = MenuItem::with_id(handle, "show", "Show AURORA IDM", true, None::<&str>)?;
    let hide_item = MenuItem::with_id(handle, "hide", "Hide AURORA IDM", true, None::<&str>)?;
    let add_item = MenuItem::with_id(handle, "add_url", "➕ Add Download URL...", true, None::<&str>)?;
    let sep1 = PredefinedMenuItem::separator(handle)?;
    let pause_all = MenuItem::with_id(handle, "pause_all", "⏸️ Pause All Downloads", true, None::<&str>)?;
    let resume_all = MenuItem::with_id(handle, "resume_all", "▶️ Resume All Downloads", true, None::<&str>)?;
    let sep2 = PredefinedMenuItem::separator(handle)?;
    let autostart_enabled = crate::autostart::is_autostart_enabled();
    let autostart_item = CheckMenuItem::with_id(
        handle,
        "toggle_autostart",
        "🚀 Launch on System Startup",
        true,
        autostart_enabled,
        None::<&str>,
    )?;
    let settings_item = MenuItem::with_id(handle, "settings", "⚙️ Settings...", true, None::<&str>)?;
    let sep3 = PredefinedMenuItem::separator(handle)?;
    let quit_item = MenuItem::with_id(handle, "quit", "❌ Quit AURORA IDM", true, None::<&str>)?;

    let menu = Menu::with_items(
        handle,
        &[
            &show_item,
            &hide_item,
            &add_item,
            &sep1,
            &pause_all,
            &resume_all,
            &sep2,
            &autostart_item,
            &settings_item,
            &sep3,
            &quit_item,
        ],
    )?;

    let mut builder = TrayIconBuilder::with_id("aurora-tray")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .tooltip("AURORA Kaushal IDM - Accelerated Multi-Stream Download Manager");

    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }

    builder
        .on_menu_event(|app, event| {
            match event.id.as_ref() {
                "show" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.unminimize();
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "hide" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.hide();
                    }
                }
                "add_url" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.unminimize();
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                    let _ = app.emit("open-add-modal", ());
                }
                "pause_all" => {
                    let _ = app.emit("tray-pause-all", ());
                }
                "resume_all" => {
                    let _ = app.emit("tray-resume-all", ());
                }
                "toggle_autostart" => {
                    let current = crate::autostart::is_autostart_enabled();
                    let _ = crate::autostart::set_autostart(!current);
                    let _ = app.emit("autostart-changed", !current);
                }
                "settings" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.unminimize();
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                    let _ = app.emit("open-settings-modal", ());
                }
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    if window.is_visible().unwrap_or(false) {
                        let _ = window.hide();
                    } else {
                        let _ = window.unminimize();
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }
        })
        .build(app)?;

    Ok(())
}
