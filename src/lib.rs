use tauri::{Manager, WebviewUrl, WebviewWindowBuilder, WindowEvent};
use tauri_plugin_autostart::ManagerExt;

mod logging;
mod menu;
mod settings;
mod tray;

const CLIPBOARD_PASTE_FALLBACK_SCRIPT: &str = include_str!("clipboard_paste_fallback.js");

pub fn run() {
    logging::install_panic_hook();

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            log::info!("second instance launched, focusing existing window");
            if let Some(window) = app.get_webview_window("main") {
                if let Err(err) = window.show() {
                    log::warn!("failed to show main window: {err}");
                }
                if let Err(err) = window.unminimize() {
                    log::warn!("failed to unminimize main window: {err}");
                }
                if let Err(err) = window.set_focus() {
                    log::warn!("failed to focus main window: {err}");
                }
            }
        }))
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--autostart"]),
        ))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(logging::plugin())
        .setup(|app| {
            let handle = app.handle();

            let autostart_enabled = settings::get(handle, "autostart_enabled", true);
            let launched_hidden = std::env::args().any(|arg| arg == "--autostart")
                && autostart_enabled
                && settings::get(handle, "tray_enabled", true)
                && settings::get(handle, "autostart_hidden", true);

            log::info!("starting up (launched_hidden={launched_hidden})");

            WebviewWindowBuilder::new(
                handle,
                "main",
                WebviewUrl::External("https://web.whatsapp.com".parse().unwrap()),
            )
            .title(handle.config().product_name.clone().unwrap_or_default())
            .initialization_script(CLIPBOARD_PASTE_FALLBACK_SCRIPT)
            .visible(!launched_hidden)
            .build()?;

            tray::setup_tray(handle)?;
            menu::setup_menu(handle)?;
            app.on_menu_event(menu::handle_menu_event);

            let manager = app.autolaunch();
            let result = if autostart_enabled {
                manager.enable()
            } else {
                manager.disable()
            };
            if let Err(err) = result {
                log::warn!("failed to sync autostart registration: {err}");
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                if settings::get(window.app_handle(), "tray_enabled", true) {
                    api.prevent_close();
                    if let Err(err) = window.hide() {
                        log::warn!("failed to hide window on close: {err}");
                    }
                }
            }
        })
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|_app_handle, _event| {});
}
