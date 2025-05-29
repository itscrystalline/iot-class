use esp_idf_svc::hal::{
    delay::{FreeRtos, TickType},
    i2c::*,
    peripherals::Peripherals,
    prelude::*,
};
use log::info;

const SI7021_ADDR: u8 = 0x40;
const SI7021_RESET: u8 = 0xFE;
// const SI7021_HUMID_HOLD: u8 = 0xE5;
const SI7021_HUMID_NOHOLD: u8 = 0xF5;
// const SI7021_TEMP_HOLD: u8 = 0xE3;
const SI7021_TEMP_NOHOLD: u8 = 0xF3;
const TICKS: TickType = TickType::new_millis(1000);

pub fn main() -> anyhow::Result<()> {
    info!("init periph");
    let peripherals = Peripherals::take()?;

    let scl = peripherals.pins.gpio22;
    let sda = peripherals.pins.gpio21;

    let config = I2cConfig::new().baudrate(400.kHz().into());
    let mut i2c = I2cDriver::new(peripherals.i2c0, sda, scl, &config)?;
    info!("init i2c");

    info!("init si7021");
    i2c.write(SI7021_ADDR, &[SI7021_RESET], TICKS.into())?;
    FreeRtos::delay_ms(50);

    loop {
        let humid = read_humidity(&mut i2c)?;
        let temp = read_temp(&mut i2c)?;
        info!("humidity: {humid}");
        info!("temp: {temp}");
        FreeRtos::delay_ms(1000);
    }
}

fn read_humidity(i2c: &mut I2cDriver<'_>) -> anyhow::Result<f32> {
    let mut buf = [0u8; 3];
    i2c.write(SI7021_ADDR, &[SI7021_HUMID_NOHOLD], TICKS.into())?;
    while i2c.read(SI7021_ADDR, &mut buf, TICKS.into()).is_err() {}
    let humidity_u16 = u16::from_be_bytes([buf[0], buf[1]]);
    // scale it
    Ok((humidity_u16 as f32 * 125.0 / 65536.0) - 6.0)
}

fn read_temp(i2c: &mut I2cDriver<'_>) -> anyhow::Result<f32> {
    let mut buf = [0u8; 3];
    i2c.write(SI7021_ADDR, &[SI7021_TEMP_NOHOLD], TICKS.into())?;
    while i2c.read(SI7021_ADDR, &mut buf, TICKS.into()).is_err() {}
    let temp_u16 = u16::from_be_bytes([buf[0], buf[1]]);
    Ok((temp_u16 as f32 * 175.72 / 65536.0) - 46.85)
}
