use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

pub fn get(app: &AppHandle, key: &str, default: bool) -> bool {
    app.store("settings.json")
        .ok()
        .and_then(|store| store.get(key))
        .and_then(|value| value.as_bool())
        .unwrap_or(default)
}

pub fn set(app: &AppHandle, key: &str, value: bool) {
    if let Ok(store) = app.store("settings.json") {
        store.set(key, value);
        let _ = store.save();
    }
}
