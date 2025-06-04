use bh1750::BH1750;
use embedded_hal::i2c::I2c;
use embedded_hal_bus::i2c::AtomicDevice;
use embedded_hal_bus::util::AtomicCell;
use esp_idf_svc::hal::delay::Delay;
use esp_idf_svc::hal::gpio::PinDriver;
use esp_idf_svc::hal::{delay::FreeRtos, i2c::*, peripherals::Peripherals, prelude::*};
use log::info;

const SI7021_ADDR: u8 = 0x40;
const SI7021_RESET: u8 = 0xFE;
// const SI7021_HUMID_HOLD: u8 = 0xE5;
const SI7021_HUMID_NOHOLD: u8 = 0xF5;
// const SI7021_TEMP_HOLD: u8 = 0xE3;
// const SI7021_TEMP_NOHOLD: u8 = 0xF3;
const THRESHOLD: u8 = 100;

pub fn main() -> anyhow::Result<()> {
    info!("init periph");
    let peripherals = Peripherals::take()?;

    let scl = peripherals.pins.gpio22;
    let sda = peripherals.pins.gpio21;

    let config = I2cConfig::new().baudrate(400.kHz().into());
    info!("init i2c");
    let i2c = AtomicCell::new(I2cDriver::new(peripherals.i2c0, sda, scl, &config)?);

    info!("init si7021");
    let mut i2c_si7021 = AtomicDevice::new(&i2c);
    i2c_si7021
        .write(SI7021_ADDR, &[SI7021_RESET])
        .map_err(|e| anyhow::anyhow!("cannot write to si7021: {e:?}"))?;

    info!("init bh1750");
    let i2c_bh1750 = AtomicDevice::new(&i2c);
    let mut bh1750 = BH1750::new(i2c_bh1750, Delay::new(50), false);

    let mut led = PinDriver::output(peripherals.pins.gpio26)?;
    FreeRtos::delay_ms(50);

    loop {
        let humid = read_humidity(&mut i2c_si7021)?;
        let lux = bh1750
            .get_one_time_measurement(bh1750::Resolution::High)
            .map_err(|e| anyhow::anyhow!("failed to read light: {e:?}"))?;
        info!("humidity: {humid}");
        info!("lux: {lux}");
        if lux > THRESHOLD.into() {
            led.set_low()?;
        } else {
            led.set_high()?;
        }
        FreeRtos::delay_ms(50);
    }
}

fn read_humidity(i2c: &mut impl I2c) -> anyhow::Result<f32> {
    let mut buf = [0u8; 3];
    i2c.write(SI7021_ADDR, &[SI7021_HUMID_NOHOLD]).unwrap();

    while i2c.read(SI7021_ADDR, &mut buf).is_err() {}
    let humidity_u16 = u16::from_be_bytes([buf[0], buf[1]]);
    // scale it
    Ok((humidity_u16 as f32 * 125.0 / 65536.0) - 6.0)
}
