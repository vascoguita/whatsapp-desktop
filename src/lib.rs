use std::sync::{Arc, Mutex};

use tauri::{
    menu::{CheckMenuItem, Menu, MenuItem, SubmenuBuilder},
    tray::{TrayIcon, TrayIconBuilder},
    Manager,
};

type TrayState = Arc<Mutex<Option<(TrayIcon, bool)>>>;

fn build_tray(app: &tauri::AppHandle) -> tauri::Result<TrayIcon> {
    let show = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &quit])?;

    TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
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
        .build(app)
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .setup(|app| {
            let store =
                tauri_plugin_store::StoreBuilder::new(app.handle(), "settings.json").build()?;
            let tray_enabled = store
                .get("tray_enabled")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);

            let tray_check = CheckMenuItem::with_id(
                app,
                "tray-enabled",
                "Enable System Tray",
                true,
                tray_enabled,
                None::<&str>,
            )?;
            let menubar = Menu::new(app)?;
            menubar.append(
                &SubmenuBuilder::new(app, "Settings")
                    .item(&tray_check)
                    .build()?,
            )?;
            app.set_menu(menubar)?;

            let tray: TrayState = Arc::new(Mutex::new(None));
            if let Ok(t) = build_tray(app.handle()) {
                let _ = t.set_visible(tray_enabled);
                *tray.lock().unwrap() = Some((t, tray_enabled));
            }
            app.manage(tray);

            app.on_menu_event(|app, event| {
                if event.id().as_ref() != "tray-enabled" {
                    return;
                }

                let state = (*app.state::<TrayState>()).clone();
                let mut guard = state.lock().unwrap();

                if let Some((t, visible)) = guard.as_mut() {
                    let enable = !*visible;
                    let _ = t.set_visible(enable);
                    *visible = enable;

                    if let Some(item) = app.menu().and_then(|m| m.get("tray-enabled")) {
                        if let Some(c) = item.as_check_menuitem() {
                            let _ = c.set_checked(enable);
                        }
                    }

                    if let Ok(store) =
                        tauri_plugin_store::StoreBuilder::new(app, "settings.json").build()
                    {
                        store.set("tray_enabled", enable);
                        let _ = store.save();
                    }
                }
            });

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|app_handle, event| {
            if let tauri::RunEvent::WindowEvent {
                label,
                event: tauri::WindowEvent::CloseRequested { api, .. },
                ..
            } = event
            {
                let tray_visible = app_handle
                    .try_state::<TrayState>()
                    .and_then(|t| t.lock().unwrap().as_ref().map(|(_, visible)| *visible))
                    .unwrap_or(false);

                if tray_visible {
                    api.prevent_close();
                    if let Some(w) = app_handle.get_webview_window(&label) {
                        let _ = w.hide();
                    }
                }
            }
        });
}
