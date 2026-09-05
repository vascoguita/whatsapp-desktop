use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    AppHandle, Manager,
};

use crate::settings;

pub fn setup_tray(app: &AppHandle) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &quit])?;
    let icon = app.default_window_icon().unwrap().clone();

    let tray_icon = TrayIconBuilder::with_id("main")
        .icon(icon)
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => {
                log::debug!("tray: show requested");
                if let Some(window) = app.get_webview_window("main") {
                    if let Err(err) = window.show() {
                        log::warn!("failed to show main window: {err}");
                    }
                    if let Err(err) = window.set_focus() {
                        log::warn!("failed to focus main window: {err}");
                    }
                }
            }
            "quit" => {
                log::info!("quitting via tray menu");
                app.exit(0);
            }
            _ => {}
        })
        .build(app)?;

    tray_icon.set_visible(settings::get(app, "tray_enabled", true))?;

    Ok(())
}
