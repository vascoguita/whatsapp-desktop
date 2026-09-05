use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

pub fn get(app: &AppHandle, key: &str, default: bool) -> bool {
    let store = match app.store("settings.json") {
        Ok(store) => store,
        Err(err) => {
            log::warn!("failed to open settings store, using default for '{key}': {err}");
            return default;
        }
    };

    store
        .get(key)
        .and_then(|value| value.as_bool())
        .unwrap_or(default)
}

pub fn set(app: &AppHandle, key: &str, value: bool) {
    let store = match app.store("settings.json") {
        Ok(store) => store,
        Err(err) => {
            log::warn!("failed to open settings store, could not persist '{key}': {err}");
            return;
        }
    };

    store.set(key, value);
    if let Err(err) = store.save() {
        log::warn!("failed to save settings store after setting '{key}': {err}");
    }
}
