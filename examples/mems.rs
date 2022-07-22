//! This example reads the onboard accelerometer and lights the LEDs which point
//! towards ground
//!
//! Additionally, the current accelleration is printed via itm.
#![no_main]
#![no_std]

use panic_itm as _;

use stm32f411e_disco as board;

use cortex_m_rt::entry;

use board::hal::prelude::*;
use board::hal::stm32;
use board::led::{LedColor, Leds};

use cortex_m::iprintln;
use cortex_m::peripheral::Peripherals;

use accelerometer::orientation::Tracker;
use accelerometer::Accelerometer;
use board::compass::Compass;

use lsm303dlhc::Sensitivity;

#[entry]
fn main() -> ! {
    let p = stm32::Peripherals::take().unwrap();
    let cp = Peripherals::take().unwrap();

    let gpiob = p.GPIOB.split();
    let gpiod = p.GPIOD.split();
    let mut itm = cp.ITM;

    // Initialize on-board LEDs
    let mut leds = Leds::new(gpiod);

    // Constrain clock registers
    let rcc = p.RCC.constrain();

    // Configure clock to 100 MHz (i.e. the maximum) and freeze it
    let clocks = rcc.cfgr.sysclk(100.mhz()).freeze();

    // Set up the compass and the accelerometer sensitivity
    let mut compass = Compass::new(gpiob, p.I2C1, clocks).unwrap();
    compass.set_accel_sensitivity(Sensitivity::G12).unwrap();

    // Tracker to know the orientation of the board
    let mut tracker = Tracker::new(0.2);

    loop {
        let acceleration = compass.accel_norm().unwrap();
        let orientation = tracker.update(acceleration);

        iprintln!(
            &mut itm.stim[0],
            "received {:?} : {}, {}, {}",
            orientation,
            acceleration.x,
            acceleration.y,
            acceleration.z,
        );

        for led in leds.iter_mut() {
            led.off();
        }

        // x+ red
        // x- green
        // y+ orange
        // y- blue

        if acceleration.y > 0.0 {
            leds[LedColor::Orange].on();
        } else {
            leds[LedColor::Blue].on();
        }

        if acceleration.x > 0.0 {
            leds[LedColor::Red].on();
        } else {
            leds[LedColor::Green].on();
        }
    }
}
