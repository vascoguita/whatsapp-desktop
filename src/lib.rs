use tauri::{
    menu::{CheckMenuItem, Menu, MenuItem, SubmenuBuilder},
    tray::TrayIconBuilder,
    AppHandle, Manager, Wry,
};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_store::StoreExt;

#[derive(Clone)]
struct MenuItems {
    tray: CheckMenuItem<Wry>,
    autostart: CheckMenuItem<Wry>,
    autostart_hidden: CheckMenuItem<Wry>,
}

fn get_setting(app: &AppHandle, key: &str, default: bool) -> bool {
    app.store("settings.json")
        .ok()
        .and_then(|store| store.get(key))
        .and_then(|val| val.as_bool())
        .unwrap_or(default)
}

fn set_setting(app: &AppHandle, key: &str, value: bool) {
    if let Ok(store) = app.store("settings.json") {
        store.set(key, value);
        let _ = store.save();
    }
}

fn setup_tray(app: &AppHandle) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &quit])?;

    let icon = app.default_window_icon().unwrap().clone();

    let tray_icon = TrayIconBuilder::with_id("main")
        .icon(icon)
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => {
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.show();
                    let _ = w.set_focus();
                }
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .build(app)?;

    let tray_enabled = get_setting(app, "tray_enabled", true);
    let _ = tray_icon.set_visible(tray_enabled);

    Ok(())
}

fn setup_menu(app: &AppHandle) -> tauri::Result<()> {
    let tray_enabled = get_setting(app, "tray_enabled", true);
    let autostart_enabled = get_setting(app, "autostart_enabled", true);
    let autostart_hidden = if autostart_enabled && tray_enabled {
        get_setting(app, "autostart_hidden", true)
    } else {
        false
    };

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

fn handle_menu_event(app: &AppHandle, event: tauri::menu::MenuEvent) {
    let items = match app.try_state::<MenuItems>() {
        Some(s) => s.inner().clone(),
        None => return,
    };

    match event.id().as_ref() {
        "tray-enabled" => {
            let enable = !get_setting(app, "tray_enabled", true);

            if let Some(tray) = app.tray_by_id("main") {
                let _ = tray.set_visible(enable);
            }

            let _ = items.tray.set_checked(enable);
            set_setting(app, "tray_enabled", enable);

            if !enable {
                let _ = items.autostart_hidden.set_checked(false);
                set_setting(app, "autostart_hidden", false);
            }
        }
        "autostart-enabled" => {
            let manager = app.autolaunch();
            let enable = !manager.is_enabled().unwrap_or(false);

            let _ = if enable {
                manager.enable()
            } else {
                manager.disable()
            };

            let _ = items.autostart.set_checked(enable);
            let _ = items.autostart_hidden.set_enabled(enable);

            if !enable {
                let _ = items.autostart_hidden.set_checked(false);
            }

            if let Some(m) = app.menu() {
                let _ = app.set_menu(m);
            }

            set_setting(app, "autostart_enabled", enable);
            if !enable {
                set_setting(app, "autostart_hidden", false);
            }
        }
        "autostart-hidden" => {
            let enable = !get_setting(app, "autostart_hidden", true);
            let _ = items.autostart_hidden.set_checked(enable);

            if enable {
                if let Some(tray) = app.tray_by_id("main") {
                    let _ = tray.set_visible(true);
                    let _ = items.tray.set_checked(true);
                    set_setting(app, "tray_enabled", true);
                }
            }

            set_setting(app, "autostart_hidden", enable);
        }
        _ => {}
    }
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.show();
                let _ = w.unminimize();
                let _ = w.set_focus();
            }
        }))
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--autostart"]),
        ))
        .plugin(tauri_plugin_store::Builder::new().build())
        .setup(|app| {
            let handle = app.handle();

            setup_tray(handle)?;
            setup_menu(handle)?;

            app.on_menu_event(handle_menu_event);

            let autostart_enabled = get_setting(handle, "autostart_enabled", true);
            let manager = app.autolaunch();
            let _ = if autostart_enabled {
                manager.enable()
            } else {
                manager.disable()
            };

            let is_autostart = std::env::args().any(|arg| arg == "--autostart");
            let tray_enabled = get_setting(handle, "tray_enabled", true);
            let autostart_hidden = get_setting(handle, "autostart_hidden", true);

            if autostart_enabled && tray_enabled && autostart_hidden && is_autostart {
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.hide();
                }
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if get_setting(window.app_handle(), "tray_enabled", true) {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|_app_handle, _event| {});
}
