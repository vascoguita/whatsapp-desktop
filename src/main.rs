fn main() {
    let nvidia = std::path::Path::new("/sys/module/nvidia").exists();
    let wayland = std::env::var("WAYLAND_DISPLAY").is_ok();

    if nvidia && wayland {
        std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    }
    app_lib::run();
}
