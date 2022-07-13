use l3gd20;

use crate::hal::gpio;
use crate::hal::prelude::*;
use crate::hal::rcc;
use crate::hal::spi;
use crate::hal::stm32;

use embedded_hal;
use embedded_hal::digital::v2::OutputPin;

type Spi1 = spi::Spi<
    stm32::SPI1,
    (
        gpioa::PA5<gpio::Alternate<gpio::AF5>>,
        gpioa::PA6<gpio::Alternate<gpio::AF5>>,
        gpioa::PA7<gpio::Alternate<gpio::AF5>>,
    ),
>;

type ChipSelect = gpioe::PE3<gpio::Output<gpio::PushPull>>;

pub struct Gyroscope {
    l3gd20: l3gd20::l3gd20<Spi1, ChipSelect>,
}

impl Gyroscope {
    pub fn new(
        gpioa: gpioa::Parts,
        gpioe: gpioe::Parts,
        spi1: stm32::SPI1,
        clocks: rcc::Clocks,
    )-> Self {
        let sck = gpioa.pa5.into_alternate_af5().internal_pull_up(false);
        let miso = gpioa.pa6.into_alternate_af5().internal_pull_up(false);
        let mosi = gpioa.pa7.into_alternate_af5().internal_pull_up(false);

        let spi_mode = spi::Mode {
            polarity: spi::Polarity::IdleLow,
            phase: spi::Phase::CaptureOnFirstTransition,
        };

        let spi = spi::Spi::spi1(spi1, (sck,miso,mosi),spi_mode, 10.mhz().into(), clocks);

        let mut chip_select = gpioe.pe3.into_push_pull_output();
        chip_select.set_high().ok();

        let config = l3gd20::Config {
            scale: l3gd20::Scale::PlusMinus8G,
            ..Default::default()
        };

        let l3gd20 = l3gd20::l3gd20::new(spi, chip_select, config);

        Self{ l3gd20 }
    }
}

