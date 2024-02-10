#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
mod wifi;

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::spi::{Config, Spi};
use embassy_stm32::time::Hertz;

use crate::wifi::Wifi;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Starting peripheral!");
    let p = embassy_stm32::init(Default::default());
    let mut spi_config = Config::default();
    spi_config.frequency = Hertz(1_000_000);

    let spi = Spi::new(
        p.SPI3, p.PC10, p.PC12, p.PC11, p.DMA2_CH2, p.DMA2_CH1, spi_config,
    );

    let mut wifi = Wifi::new(p.PE8, p.PE0, p.PE1, spi);

    //"C1=SSID\r"                         --> set the SSID of the network
    //"C2=PASSWORD\r"                     --> enter the network password
    //"C3=SECURITY-MODE\r"                --> set the security mode
    //"C4=DHCP YES OR NOT\r"               --> activate or not DHCP mode for the IP address
    //"C0\r"                              --> join the network
    let commands = ["C1=Deny\r", "C2=DACCARVALHO\r", "C3=3\r", "C4=1\r", "C0\r"];

    let mut cursor = [0x0Au8; 10];
    let mut answer = [0x0Au8; 10];
    // reading the cursor
    wifi.read_data(&mut cursor).await;
    for cmd in commands {
        wifi.send_command(cmd).await;
        wifi.read_data(&mut answer).await;
        info!("answer for command : {:x}", answer);
    }

    info!("Sending");
    loop {}
}
