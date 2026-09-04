use tauri::{Manager, WebviewUrl, WebviewWindowBuilder, WindowEvent};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_log::{RotationStrategy, Target, TargetKind};

mod menu;
mod report;
mod settings;
mod tray;

const CLIPBOARD_PASTE_FALLBACK_SCRIPT: &str = include_str!("clipboard_paste_fallback.js");

fn install_panic_hook() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        log::error!("panic: {info}");
        default_hook(info);
    }));
}

pub fn run() {
    install_panic_hook();

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.unminimize();
                let _ = window.set_focus();
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
        .plugin(
            tauri_plugin_log::Builder::new()
                .targets([
                    Target::new(TargetKind::Stdout),
                    Target::new(TargetKind::LogDir {
                        file_name: Some(report::LOG_FILE_NAME.to_string()),
                    }),
                ])
                .max_file_size(50_000)
                .rotation_strategy(RotationStrategy::KeepOne)
                .build(),
        )
        .invoke_handler(tauri::generate_handler![report::submit_report])
        .setup(|app| {
            let handle = app.handle();

            let autostart_enabled = settings::get(handle, "autostart_enabled", true);
            let launched_hidden = std::env::args().any(|arg| arg == "--autostart")
                && autostart_enabled
                && settings::get(handle, "tray_enabled", true)
                && settings::get(handle, "autostart_hidden", true);

            WebviewWindowBuilder::new(
                handle,
                "main",
                WebviewUrl::External("https://web.whatsapp.com".parse().unwrap()),
            )
            .title("WhatsApp Desktop")
            .initialization_script(CLIPBOARD_PASTE_FALLBACK_SCRIPT)
            .visible(!launched_hidden)
            .build()?;

            tray::setup_tray(handle)?;
            menu::setup_menu(handle)?;
            app.on_menu_event(menu::handle_menu_event);

            let manager = app.autolaunch();
            let _ = if autostart_enabled {
                manager.enable()
            } else {
                manager.disable()
            };

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main"
                    && settings::get(window.app_handle(), "tray_enabled", true)
                {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|_app_handle, _event| {});
}
