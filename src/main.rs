use esp_idf_svc::{log as esp_log, sys};
use log::info;

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    sys::link_patches();
    // Bind the log crate to the ESP Logging facilities
    esp_log::EspLogger::initialize_default();

    info!("Hello, world!");
}
