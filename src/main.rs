#[cfg(feature = "wk12")]
mod wk12;
#[cfg(feature = "wk3")]
mod wk3;
#[cfg(feature = "wk4")]
mod wk4;
#[cfg(feature = "wk5")]
mod wk5;
#[cfg(feature = "wk8")]
mod wk8;

mod secrets;

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
    #[cfg(feature = "wk5")]
    wk5::main()?;
    #[cfg(feature = "wk8")]
    wk8::main()?;
    #[cfg(feature = "wk12")]
    wk12::main()?;

    Ok(())
}
