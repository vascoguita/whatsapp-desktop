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

    let about = MenuItem::with_id(app, "about", "About", true, None::<&str>)?;

    let menubar = Menu::new(app)?;
    menubar.append(
        &SubmenuBuilder::new(app, "Settings")
            .item(&items.tray)
            .item(&items.autostart)
            .item(&items.autostart_hidden)
            .build()?,
    )?;
    menubar.append(&about)?;
    app.set_menu(menubar)?;
    app.manage(items);

    Ok(())
}

pub fn handle_menu_event(app: &AppHandle, event: tauri::menu::MenuEvent) {
    let Some(items) = app
        .try_state::<MenuItems>()
        .map(|state| state.inner().clone())
    else {
        return;
    };

    match event.id().as_ref() {
        "tray-enabled" => {
            let enabled = !settings::get(app, "tray_enabled", true);

            if let Some(tray) = app.tray_by_id("main") {
                let _ = tray.set_visible(enabled);
            }
            let _ = items.tray.set_checked(enabled);
            settings::set(app, "tray_enabled", enabled);

            if !enabled {
                let _ = items.autostart_hidden.set_checked(false);
                settings::set(app, "autostart_hidden", false);
            }
        }
        "autostart-enabled" => {
            let manager = app.autolaunch();
            let enabled = !manager.is_enabled().unwrap_or(false);
            let _ = if enabled {
                manager.enable()
            } else {
                manager.disable()
            };

            let _ = items.autostart.set_checked(enabled);
            let _ = items.autostart_hidden.set_enabled(enabled);
            if !enabled {
                let _ = items.autostart_hidden.set_checked(false);
            }
            if let Some(menu) = app.menu() {
                let _ = app.set_menu(menu);
            }

            settings::set(app, "autostart_enabled", enabled);
            if !enabled {
                settings::set(app, "autostart_hidden", false);
            }
        }
        "autostart-hidden" => {
            let enabled = !settings::get(app, "autostart_hidden", true);
            let _ = items.autostart_hidden.set_checked(enabled);

            if enabled {
                if let Some(tray) = app.tray_by_id("main") {
                    let _ = tray.set_visible(true);
                    let _ = items.tray.set_checked(true);
                    settings::set(app, "tray_enabled", true);
                }
            }

            settings::set(app, "autostart_hidden", enabled);
        }
        "about" => {
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
