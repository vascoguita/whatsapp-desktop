use tauri::{plugin::TauriPlugin, Runtime};
use tauri_plugin_log::RotationStrategy;

pub fn install_panic_hook() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        log::error!("panic: {info}");
        default_hook(info);
    }));
}

pub fn plugin<R: Runtime>() -> TauriPlugin<R> {
    tauri_plugin_log::Builder::new()
        .max_file_size(50_000)
        .rotation_strategy(RotationStrategy::KeepSome(10))
        .build()
}
