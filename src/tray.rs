use tauri::{
    menu::{CheckMenuItem, Menu, MenuItem, SubmenuBuilder},
    tray::TrayIconBuilder,
    AppHandle, Manager, Wry,
};
use tauri_plugin_autostart::ManagerExt;

use crate::settings;

#[derive(Clone)]
struct MenuItems {
    tray: CheckMenuItem<Wry>,
    autostart: CheckMenuItem<Wry>,
    autostart_hidden: CheckMenuItem<Wry>,
}

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
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .build(app)?;

    tray_icon.set_visible(settings::get(app, "tray_enabled", true))?;

    Ok(())
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

    let menubar = Menu::new(app)?;
    menubar.append(
        &SubmenuBuilder::new(app, "Settings")
            .item(&items.tray)
            .item(&items.autostart)
            .item(&items.autostart_hidden)
            .build()?,
    )?;
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
        _ => {}
    }
}
