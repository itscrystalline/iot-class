mod wk3;
mod wk4;

use esp_idf_svc::{log as esp_log, sys};
use log::info;

fn main() -> anyhow::Result<()> {
    sys::link_patches();
    esp_log::EspLogger::initialize_default();
    info!("starting!");

    #[cfg(feature = "wk3")]
    wk3::main()?;
    #[cfg(feature = "wk4")]
    wk4::main()?;

    Ok(())
}
