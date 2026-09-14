use tauri::WebviewWindow;
use webkit2gtk::{
    glib::Cast, PermissionRequestExt, SettingsExt, UserMediaPermissionRequest, WebViewExt,
};

pub fn setup(window: &WebviewWindow) -> tauri::Result<()> {
    window.with_webview(|webview| {
        let webview = webview.inner();

        if let Some(settings) = webview.settings() {
            settings.set_enable_media_stream(true);
            settings.set_enable_webrtc(true);
        }

        webview.connect_permission_request(|_, request| {
            if request
                .downcast_ref::<UserMediaPermissionRequest>()
                .is_none()
            {
                return false;
            }
            request.allow();
            true
        });
    })
}
