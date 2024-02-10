use embassy_stm32::gpio::{Input, Level, Output, Pull, Speed};
use embassy_stm32::peripherals::SPI3;
use embassy_stm32::peripherals::*;
use embassy_stm32::spi::*;

pub struct Wifi {
    rst: Output<'static, PE8>,
    cs: Output<'static, PE0>,
    ready: Input<'static, PE1>,
    spi: Spi<'static, SPI3, DMA2_CH2, DMA2_CH1>,
}

impl Wifi {
    /// create the wifi module
    pub fn new(rst: PE8, cs: PE0, ready: PE1, spi: Spi<'static, SPI3, DMA2_CH2, DMA2_CH1>) -> Self {
        let mut wifi = Wifi {
            rst: Output::new(rst, Level::Low, Speed::VeryHigh),
            cs: Output::new(cs, Level::High, Speed::VeryHigh),
            ready: Input::new(ready, Pull::Up),
            spi,
        };
        cortex_m::asm::delay(100_000);
        wifi.rst.set_high();
        cortex_m::asm::delay(100_000);
        return wifi;
    }

    // wait for ready pin to be high
    fn wait_for_ready_high(&mut self) {
        while self.ready.is_low() {}
    }

    /// wait for ready pin to be low
    fn wait_for_ready_low(&mut self) {
        while self.ready.is_high() {}
    }

    /// send a command to the wifi chip
    pub async fn send_command(&mut self, command: &str) {
        // wait for command phase
        self.wait_for_ready_high();

        self.cs.set_low();
        for c in command.as_bytes().chunks(2) {
            let data = match c {
                [a, b] => [*b, *a],
                [a] => [b'\r', *a],
                _ => unreachable!(),
            };
            self.spi.write(&data).await.ok();
        }
        self.cs.set_high();
        // end of command phase
        self.wait_for_ready_low();
    }

    /// read data from the chip
    pub async fn read_data(&mut self, data: &mut [u8]) {
        self.wait_for_ready_high();
        self.cs.set_low();
        while self.ready.is_high() {
            self.spi.read(data).await.ok();
        }
        self.cs.set_high();
    }
}
