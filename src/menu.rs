use tauri::{
    menu::{CheckMenuItem, Menu, MenuItem, SubmenuBuilder},
    AppHandle, Manager, Wry,
};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};

use crate::settings;

#[derive(Clone)]
struct MenuItems {
    tray: CheckMenuItem<Wry>,
    autostart: CheckMenuItem<Wry>,
    autostart_hidden: CheckMenuItem<Wry>,
}

pub fn setup_menu(app: &AppHandle) -> tauri::Result<()> {
    let tray_enabled = settings::get(app, "tray_enabled", true);
    let autostart_enabled = settings::get(app, "autostart_enabled", true);
    let autostart_hidden =
        autostart_enabled && tray_enabled && settings::get(app, "autostart_hidden", true);

    let items = MenuItems {
        tray: CheckMenuItem::with_id(
            app,
            "tray-enabled",
            "Show tray icon",
            true,
            tray_enabled,
            None::<&str>,
        )?,
        autostart: CheckMenuItem::with_id(
            app,
            "autostart-enabled",
            "Launch on startup",
            true,
            autostart_enabled,
            None::<&str>,
        )?,
        autostart_hidden: CheckMenuItem::with_id(
            app,
            "autostart-hidden",
            "Launch hidden in tray icon on startup",
            autostart_enabled,
            autostart_hidden,
            None::<&str>,
        )?,
    };

    let reload = MenuItem::with_id(app, "reload", "Reload", true, Some("CmdOrCtrl+R"))?;
    let about = MenuItem::with_id(app, "about", "About", true, None::<&str>)?;

    let menubar = Menu::new(app)?;
    menubar.append(
        &SubmenuBuilder::new(app, "Settings")
            .item(&items.tray)
            .item(&items.autostart)
            .item(&items.autostart_hidden)
            .build()?,
    )?;
    menubar.append(&reload)?;
    menubar.append(&about)?;
    app.set_menu(menubar)?;
    app.manage(items);

    Ok(())
}

pub fn handle_menu_event(app: &AppHandle, event: tauri::menu::MenuEvent) {
    let id = event.id().as_ref();

    let Some(items) = app
        .try_state::<MenuItems>()
        .map(|state| state.inner().clone())
    else {
        log::warn!("menu event '{id}' received before menu items were initialized");
        return;
    };

    match id {
        "tray-enabled" => {
            let enabled = !settings::get(app, "tray_enabled", true);
            log::info!("tray icon {}", if enabled { "enabled" } else { "disabled" });

            if let Some(tray) = app.tray_by_id("main") {
                if let Err(err) = tray.set_visible(enabled) {
                    log::warn!("failed to set tray visibility: {err}");
                }
            }
            if let Err(err) = items.tray.set_checked(enabled) {
                log::warn!("failed to update tray menu checkbox: {err}");
            }
            settings::set(app, "tray_enabled", enabled);

            if !enabled {
                if let Err(err) = items.autostart_hidden.set_checked(false) {
                    log::warn!("failed to update autostart-hidden menu checkbox: {err}");
                }
                settings::set(app, "autostart_hidden", false);
            }
        }
        "autostart-enabled" => {
            let manager = app.autolaunch();
            let enabled = !manager.is_enabled().unwrap_or(false);
            log::info!(
                "launch on startup {}",
                if enabled { "enabled" } else { "disabled" }
            );

            let result = if enabled {
                manager.enable()
            } else {
                manager.disable()
            };
            if let Err(err) = result {
                log::warn!("failed to update autostart registration: {err}");
            }

            if let Err(err) = items.autostart.set_checked(enabled) {
                log::warn!("failed to update autostart menu checkbox: {err}");
            }
            if let Err(err) = items.autostart_hidden.set_enabled(enabled) {
                log::warn!("failed to update autostart-hidden menu item: {err}");
            }
            if !enabled {
                if let Err(err) = items.autostart_hidden.set_checked(false) {
                    log::warn!("failed to update autostart-hidden menu checkbox: {err}");
                }
            }
            if let Some(menu) = app.menu() {
                if let Err(err) = app.set_menu(menu) {
                    log::warn!("failed to refresh menu: {err}");
                }
            }

            settings::set(app, "autostart_enabled", enabled);
            if !enabled {
                settings::set(app, "autostart_hidden", false);
            }
        }
        "autostart-hidden" => {
            let enabled = !settings::get(app, "autostart_hidden", true);
            log::info!(
                "launch hidden in tray {}",
                if enabled { "enabled" } else { "disabled" }
            );
            if let Err(err) = items.autostart_hidden.set_checked(enabled) {
                log::warn!("failed to update autostart-hidden menu checkbox: {err}");
            }

            if enabled {
                if let Some(tray) = app.tray_by_id("main") {
                    if let Err(err) = tray.set_visible(true) {
                        log::warn!("failed to set tray visibility: {err}");
                    }
                    if let Err(err) = items.tray.set_checked(true) {
                        log::warn!("failed to update tray menu checkbox: {err}");
                    }
                    settings::set(app, "tray_enabled", true);
                }
            }

            settings::set(app, "autostart_hidden", enabled);
        }
        "reload" => {
            log::info!("reloading main window");
            if let Some(window) = app.get_webview_window("main") {
                if let Err(err) = window.reload() {
                    log::warn!("failed to reload main window: {err}");
                }
            }
        }
        "about" => {
            log::debug!("showing about dialog");
            let message = format!(
                "{description}\n\nVersion: {version}\nAuthor: {authors}\nLicense: {license}\nRepository: {repository}",
                description = env!("CARGO_PKG_DESCRIPTION"),
                version = env!("CARGO_PKG_VERSION"),
                authors = env!("CARGO_PKG_AUTHORS"),
                license = env!("CARGO_PKG_LICENSE"),
                repository = env!("CARGO_PKG_REPOSITORY"),
            );

            app.dialog()
                .message(message)
                .title("About WhatsApp Desktop")
                .kind(MessageDialogKind::Info)
                .show(|_| {});
        }
        _ => {}
    }
}
