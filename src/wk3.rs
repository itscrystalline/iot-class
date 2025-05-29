use esp_idf_svc::hal::{
    adc::oneshot::{config::AdcChannelConfig, AdcChannelDriver, AdcDriver},
    gpio::PinDriver,
    prelude::*,
    sys::adc_atten_t_ADC_ATTEN_DB_12 as DB_12,
};
use log::info;
use std::{thread, time::Duration};

pub fn main() -> anyhow::Result<()> {
    let peripherials = Peripherals::take()?;
    let adc = AdcDriver::new(peripherials.adc1)?;

    let config = AdcChannelConfig {
        attenuation: DB_12,
        ..Default::default()
    };

    let mut led = PinDriver::output(peripherials.pins.gpio26)?;
    let mut analog_pin = AdcChannelDriver::new(&adc, peripherials.pins.gpio36, &config)?;

    let mut counter = 0u8;
    loop {
        thread::sleep(Duration::from_millis(50));
        let light: u16 = adc.read(&mut analog_pin)?;
        if light < 1000 {
            led.set_high()?;
        } else {
            led.set_low()?;
        }
        counter += 1;
        if counter >= 20 {
            counter = 0;
            info!("Light: {light}");
        }
    }
}
