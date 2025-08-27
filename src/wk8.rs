use embedded_svc::wifi::{AuthMethod, ClientConfiguration, Configuration as WifiConfiguration};

use esp_idf_svc::hal::prelude::Peripherals;
use esp_idf_svc::ipv4::{
    ClientConfiguration as IpClientConfiguration, Configuration as IpConfiguration,
    DHCPClientSettings,
};
use esp_idf_svc::netif::{EspNetif, NetifConfiguration, NetifStack};
use esp_idf_svc::wifi::{BlockingWifi, EspWifi, WifiDriver};
use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition};
use log::info;

use crate::secrets;

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

    loop {
        std::thread::sleep(core::time::Duration::from_secs(5));
        info!("Wifi Interface info: {ip_info:?}");
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

    // wifi.wait_netif_up()?;
    // info!("Wifi netif up");

    Ok(())
}
