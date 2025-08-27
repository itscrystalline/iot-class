use embedded_hal::i2c::I2c;
use embedded_hal_bus::i2c::AtomicDevice;
use embedded_hal_bus::util::AtomicCell;
use embedded_svc::http::client::Client;
use embedded_svc::utils::io;
use embedded_svc::wifi::{AuthMethod, ClientConfiguration, Configuration as WifiConfiguration};

// use bh1750::BH1750;
// use esp_idf_svc::hal::delay::Delay;
use esp_idf_svc::hal::i2c::{I2cConfig, I2cDriver};
use esp_idf_svc::hal::prelude::*;
use esp_idf_svc::http::client::EspHttpConnection;
use esp_idf_svc::io::Write;
use esp_idf_svc::ipv4::{
    ClientConfiguration as IpClientConfiguration, Configuration as IpConfiguration,
    DHCPClientSettings,
};
use esp_idf_svc::netif::{EspNetif, NetifConfiguration, NetifStack};
use esp_idf_svc::wifi::{BlockingWifi, EspWifi, WifiDriver};
use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition};
use log::{error, info};
use serde_json::{json, Value};

use crate::secrets;

const SI7021_ADDR: u8 = 0x40;
const SI7021_RESET: u8 = 0xFE;
// const SI7021_HUMID_HOLD: u8 = 0xE5;
const SI7021_HUMID_NOHOLD: u8 = 0xF5;
// const SI7021_TEMP_HOLD: u8 = 0xE3;
const SI7021_TEMP_NOHOLD: u8 = 0xF3;
const URL: &str = "http://172.16.0.167/post_data/index.php";

// #[allow(non_snake_case)]
// #[derive(Serialize)]
// struct SensorData {
//     sensorName: String,
//     Temp: f32,
//     humid: f32,
// }

pub fn main() -> anyhow::Result<()> {
    info!("init periph");
    let peripherals = Peripherals::take()?;
    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    let wifi = WifiDriver::new(peripherals.modem, sys_loop.clone(), Some(nvs))?;
    let wifi = configure_wifi(wifi)?;

    let mut wifi = BlockingWifi::wrap(wifi, sys_loop)?;
    connect_wifi(&mut wifi)?;

    let ip_info = wifi.wifi().sta_netif().get_ip_info()?;

    info!("Wifi Interface info: {ip_info:?}");

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

    // info!("init bh1750");
    // let i2c_bh1750 = AtomicDevice::new(&i2c);
    // let mut bh1750 = BH1750::new(i2c_bh1750, Delay::new(50), false);

    info!("init http client");
    let mut http = Client::wrap(EspHttpConnection::new(&Default::default())?);

    loop {
        std::thread::sleep(core::time::Duration::from_secs(1));
        let humidity = read_humidity(&mut i2c_si7021)?;
        let temp = read_temp(&mut i2c_si7021)?;
        let json = json!({
            "sensorName": "Sensor1",
            "Temp": temp,
            "humid": humidity,
        });
        send_to_server(&mut http, json)?;
    }
}

fn configure_wifi(wifi: WifiDriver) -> anyhow::Result<EspWifi> {
    let mut wifi = EspWifi::wrap_all(
        wifi,
        // Note that setting a custom hostname can be used with any network adapter, not just Wifi
        // I.e. that would work with Eth as well, because DHCP is an L3 protocol
        EspNetif::new_with_conf(&NetifConfiguration {
            ip_configuration: Some(IpConfiguration::Client(IpClientConfiguration::DHCP(
                DHCPClientSettings {
                    hostname: Some("espthirtytwo".try_into().unwrap()),
                },
            ))),
            ..NetifConfiguration::wifi_default_client()
        })?,
        EspNetif::new(NetifStack::Ap)?,
    )?;

    let wifi_configuration = WifiConfiguration::Client(ClientConfiguration {
        ssid: secrets::SSID.try_into().unwrap(),
        bssid: None,
        auth_method: AuthMethod::WPA2Personal,
        password: secrets::PASS.try_into().unwrap(),
        channel: None,
        ..Default::default()
    });
    wifi.set_configuration(&wifi_configuration)?;

    Ok(wifi)
}

fn connect_wifi(wifi: &mut BlockingWifi<EspWifi<'static>>) -> anyhow::Result<()> {
    wifi.start()?;
    info!("Wifi started");

    wifi.connect()?;
    info!("Wifi connected");

    wifi.wait_netif_up()?;
    info!("Wifi netif up");

    Ok(())
}

fn read_humidity(i2c: &mut impl I2c) -> anyhow::Result<f32> {
    let mut buf = [0u8; 3];
    i2c.write(SI7021_ADDR, &[SI7021_HUMID_NOHOLD]).unwrap();

    while i2c.read(SI7021_ADDR, &mut buf).is_err() {}
    let humidity_u16 = u16::from_be_bytes([buf[0], buf[1]]);
    // scale it
    Ok((humidity_u16 as f32 * 125.0 / 65536.0) - 6.0)
}

fn read_temp(i2c: &mut impl I2c) -> anyhow::Result<f32> {
    let mut buf = [0u8; 3];
    i2c.write(SI7021_ADDR, &[SI7021_TEMP_NOHOLD]).unwrap();
    while i2c.read(SI7021_ADDR, &mut buf).is_err() {}
    let temp_u16 = u16::from_be_bytes([buf[0], buf[1]]);
    Ok((temp_u16 as f32 * 175.72 / 65536.0) - 46.85)
}

// fn read_lux(bh1750: &mut BH1750<AtomicDevice<'_, I2cDriver<'_>>, Delay>) -> anyhow::Result<f32> {
//     bh1750
//         .get_one_time_measurement(bh1750::Resolution::High)
//         .map_err(|e| anyhow::anyhow!("failed to read light: {e:?}"))
// }

fn send_to_server(http: &mut Client<EspHttpConnection>, json: Value) -> anyhow::Result<()> {
    let payload = serde_json::to_string(&json)?;
    let content_length = payload.len();
    let headers = [
        ("content_type", "application/json"),
        ("content-length", &format!("{content_length}")),
    ];

    let mut request = http.post(URL, &headers)?;
    request.write_all(payload.as_bytes())?;
    request.flush()?;
    info!("-> POST {payload}");
    let mut response = request.submit()?;

    let status = response.status();
    info!("<- {status}");
    let mut buf = [0u8; 1024];
    let bytes_read = io::try_read_full(&mut response, &mut buf).map_err(|e| e.0)?;
    info!("Read {bytes_read} bytes");
    match std::str::from_utf8(&buf[0..bytes_read]) {
        Ok(body_string) => info!(
            "Response body (truncated to {} bytes): {body_string:?}",
            buf.len()
        ),
        Err(e) => error!("Error decoding response body: {e}"),
    };
    Ok(())
}
