use accelerometer;
use lsm303dlhc;

use crate::hal::gpio;
use crate::hal::gpio::gpiob;
use crate::hal::gpio::gpioe;
use crate::hal::i2c;
use crate::hal::prelude::*;
use crate::hal::rcc;
use crate::hal::stm32;

use embedded_hal;
use embedded_hal::digital::v2::OutputPin;

type I2c1 = i2c::I2c<
    stm32::I2C1,
    (
        gpiob::PB6<gpio::Alternate<gpio::AF4>>,
        gpiob::PB7<gpio::Alternate<gpio::AF4>>,
    ),
>;

pub struct Compass {
    lsm303dlhc: lsm303dlhc::lsm303dlhc<I2c1>,
}

impl Compass {
    pub fn new(
        gpiob: gpiob::Parts,
        gpioe: gpioe::Parts,
        i2c1: stm32::I2C1,
        clocks: rcc::Clocks,
    ) -> Self {
        let scl = gpiob
            .pb6
            .into_alternate_af4()
            .internal_pull_up(true)
            .set_open_drain();
        let sda = gpiob
            .pb7
            .into_alternate_af4()
            .internal_pull_up(true)
            .set_open_drain();

        let i2c_mode = i2c::Mode {
            polarity: i2c::Polarity::IdleLow,
            phase: i2c::Phase::CaptureOnFirstTransition,
        };

        let i2c = i2c::I2c::i2c1(i2c1, (scl, sda), i2c_mode, 400.khz().into(), clocks);

        let config = lsm303dlhc::Config {
            scale: lsm303dlhc::Scale::PlusMinus8G,
            ..Default::default()
        };
        let lsm303dlhc = lsm303dlhc::lsm303dlhc::new(i2c, config);

        Self { lsm303dlhc }
    }
}

impl compass::RawAccelerometer<accelerometer::vector::I8x3> for lsm303dlhc {
    type Error = i2c::Error;
    fn accel_raw(
        &mut self,
    ) -> Result<accelerometer::vector::I8x3, accelerometer::Error<Self::Error>> {
        let x = self.read_accel_registers(accel::Registers::OUT_X_L_A);
        let y = self.read_accel_registers(accel::Registers::OUT_Y_L_A);
        let z = self.read_accel_registers(accel::Registers::OUT_Z_L_A);
        Ok(accelerometer::vector::I8x3::new(
            i8::from_le_bytes([x]),
            i8::from_le_bytes([y]),
            i8::from_le_bytes([z]),
        ))
    }
}

impl compass::Accelerometer for lsm303dlhc {
    type Error = spi::Error;
    fn sample_rate(&mut self) -> Result<f32, accelerometer::Error<Self::Error>> {
        self.modify_accel_register(accel::Register::CTRL_REG1_A, |r| {
            r & !(0b1111 << 4) | ((odr as u8) << 4)
        })
    }

    fn accel_norm(
        &mut self,
    ) -> Result<accelerometer::vector::F32x3, accelerometer::Error<Self::Error>> {
        let raw_acceleration: accelerometer::vector::I8x3 = self.accel_raw().unwrap();
        Ok(accelerometer::vector::F32x3::new(
            raw_acceleration.x as f32 * SCALE,
            raw_acceleration.y as f32 * SCALE,
            raw_acceleration.z as f32 * SCALE,
        ))
    }
}
