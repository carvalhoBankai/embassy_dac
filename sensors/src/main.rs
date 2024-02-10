#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::{Input, Level, Output, Pull, Speed};
use embassy_stm32::i2c::I2c;
use embassy_stm32::peripherals::{PB14, PC13, PC9};
use embassy_stm32::time::Hertz;
use embassy_stm32::{bind_interrupts, i2c, peripherals};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

const ADDRESS: u8 = 0x5F;
const WHOAMI: u8 = 0x0F;

bind_interrupts!(struct Irqs {
    I2C2_EV => i2c::EventInterruptHandler<peripherals::I2C2>;
    I2C2_ER => i2c::ErrorInterruptHandler<peripherals::I2C2>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    let led_yellow_blue = Output::new(p.PC9, Level::High, Speed::Low);
    let green_led = Output::new(p.PB14, Level::Low, Speed::Low);
    let button = Input::new(p.PC13, Pull::Up);
    let button = ExtiInput::new(button, p.EXTI13);
    let mut i2c = I2c::new(
        p.I2C2,
        p.PB10,
        p.PB11,
        Irqs,
        p.DMA1_CH4,
        p.DMA1_CH5,
        Hertz(100_000),
        Default::default(),
    );

    // let mut i2c = BlockingAsync::new(i2c);
    let mut data = [0u8; 1];

    spawner.spawn(blink_led(led_yellow_blue)).ok();
    spawner
        .spawn(read_button_and_set_led(button, green_led))
        .ok();

    loop {
        unwrap!(i2c.write_read(ADDRESS, &[WHOAMI], &mut data).await); // I2C cursor write to change the cursor position and read to read the who ami
        info!("Whoami: {}", data[0]);
    }
}

#[embassy_executor::task]
async fn blink_led(mut led: Output<'static, PC9>) {
    loop {
        led.set_high();
        Timer::after_millis(500).await;
        led.set_low();
        Timer::after_millis(500).await;
    }
}


// read button task and set led
#[embassy_executor::task]
async fn read_button_and_set_led(
    mut button: ExtiInput<'static, PC13>,
    mut led: Output<'static, PB14>,
) {
    loop {
        button.wait_for_falling_edge().await;
        led.set_high();
        Timer::after_secs(1).await;
        led.set_low();
    }
}