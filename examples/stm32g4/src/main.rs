#![no_std]
#![no_main]

extern crate alloc;

use alloc::boxed::Box;

use defmt::{error, info, unwrap};
use embassy_executor::Spawner;
use embassy_stm32::{bind_interrupts, Config, i2c};
use embassy_stm32::i2c::{Error, I2c};
use embassy_stm32::mode::Async;
use embassy_stm32::peripherals;
use embassy_stm32::time::Hertz;
use embassy_time::{Delay, Timer};
use futures::StreamExt;
use uom::fmt::DisplayStyle::Abbreviation;
use uom::si::electric_potential::millivolt;
use uom::si::f32::ElectricPotential;

use mcp3424::{Channel, ConversionTime, MCP3424, MultiShotMode};

use crate::alloc::string::ToString;

#[allow(unused_imports)]
use defmt_rtt as _;

#[allow(unused_imports)]
use panic_probe as _;

#[global_allocator]
pub static HEAP: embedded_alloc::Heap = embedded_alloc::Heap::empty();

type Adc = MCP3424<I2c<'static, peripherals::I2C1, Async>, Error, Delay, MultiShotMode<4>>;

bind_interrupts!(struct Irqs {
    I2C1_EV => i2c::EventInterruptHandler<peripherals::I2C1>;
    I2C1_ER => i2c::ErrorInterruptHandler<peripherals::I2C1>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {

    {
        use core::mem::MaybeUninit;
        const HEAP_SIZE: usize = 8192;
        static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe { HEAP.init(HEAP_MEM.as_ptr() as usize, HEAP_SIZE) }
    }

    info!("Configring.");

    let config = Config::default();

    let peripherals = embassy_stm32::init(config);

    info!("Configuring I2C.");

    let i2c = I2c::new(
        peripherals.I2C1,
        peripherals.PA15,
        peripherals.PB7,
        Irqs,
        peripherals.DMA1_CH6,
        peripherals.DMA1_CH1,
        Hertz(100_000),
        Default::default(),
    );

    info!("Configured I2C.");

    info!("Configuring MCP3424.");

    let conversion_time_offset = ConversionTime::Offset(400);
    let adc = MCP3424::new(i2c, 0x68, Delay, MultiShotMode::new(&[
        mcp3424::Configuration::default()
            .with_channel(Channel::Channel1)
            .with_conversion_time(conversion_time_offset),
        mcp3424::Configuration::default()
            .with_channel(Channel::Channel2)
            .with_conversion_time(conversion_time_offset),
        mcp3424::Configuration::default()
            .with_channel(Channel::Channel3)
            .with_conversion_time(conversion_time_offset),
        mcp3424::Configuration::default()
            .with_channel(Channel::Channel4)
            .with_conversion_time(conversion_time_offset),
    ]));

    info!("Configured MCP3424.");

    info!("Configuration completed.");

    unwrap!(spawner.spawn(measure(adc)));

    info!("Going into main loop.");

    loop {
        Timer::after_millis(500).await;
    }
}

#[embassy_executor::task()]
async fn measure(mut adc: Adc) {

    info!("Starting measuring task.");

    let fmt_voltage = ElectricPotential::format_args(millivolt, Abbreviation);

    let mut values = Box::pin(unwrap!(adc.measure_stream().await))
        .map(|result| result
            .map(|values| values
                .map(|value| fmt_voltage.with(value).to_string()).join(", ")
            )
        );

    loop {
        while let Some(values) = values.next().await {
            match values {
                Ok(values) => info!("Measured values: {=str}", values),
                Err(error) => error!("Failed to measure, due to error {}", error),
            }
        }
    }
}
