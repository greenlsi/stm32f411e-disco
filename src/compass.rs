use accelerometer::{self, vector, ErrorKind, RawAccelerometer};
use lsm303dlhc::{AccelOdr, Lsm303dlhc, Sensitivity};

use crate::hal::gpio;
use crate::hal::i2c;
use crate::hal::prelude::*;
use crate::hal::rcc;
use crate::hal::stm32;

const I16MAX: f32 = i16::MAX as f32;

// TODO documentation

type I2c1 = i2c::I2c<
    stm32::I2C1,
    (
        gpio::gpiob::PB6<gpio::AlternateOD<gpio::AF4>>,
        gpio::gpiob::PB9<gpio::AlternateOD<gpio::AF4>>,
    ),
>;

pub struct Compass {
    lsm303dlhc: Lsm303dlhc<I2c1>,
    accel_odr: AccelOdr,
    accel_sensitivity: Sensitivity, // TODO add magnetometer and temperature stuff
}

impl Compass {
    pub fn new(
        gpiob: gpio::gpiob::Parts,
        i2c1: stm32::I2C1,
        clocks: rcc::Clocks,
    ) -> Result<Self, i2c::Error> {
        let scl = gpiob
            .pb6
            .into_alternate_af4()
            .internal_pull_up(true)
            .set_open_drain();
        let sda = gpiob
            .pb9
            .into_alternate_af4()
            .internal_pull_up(true)
            .set_open_drain();
        let i2c = i2c::I2c::new(i2c1, (scl, sda), 400.khz(), clocks);

        let lsm303dlhc = Lsm303dlhc::new(i2c)?;
        Ok(Self {
            lsm303dlhc,
            accel_odr: AccelOdr::Hz400,
            accel_sensitivity: Sensitivity::G1,
        })
    }

    pub fn set_accel_odr(&mut self, accel_odr: AccelOdr) -> Result<(), i2c::Error> {
        self.accel_odr = match accel_odr {
            // AccelOdr does not implement the Copy trait
            // We need to do this ugly thing...
            // TODO new version of the sensor library to implement the Copy trait for AccelOdr
            AccelOdr::Hz1 => AccelOdr::Hz1,
            AccelOdr::Hz10 => AccelOdr::Hz10,
            AccelOdr::Hz25 => AccelOdr::Hz25,
            AccelOdr::Hz50 => AccelOdr::Hz50,
            AccelOdr::Hz100 => AccelOdr::Hz100,
            AccelOdr::Hz200 => AccelOdr::Hz200,
            AccelOdr::Hz400 => AccelOdr::Hz400,
        };
        self.lsm303dlhc.accel_odr(accel_odr)
    }

    pub fn set_accel_sensitivity(
        &mut self,
        accel_sensitivity: Sensitivity,
    ) -> Result<(), i2c::Error> {
        self.accel_sensitivity = accel_sensitivity;
        self.lsm303dlhc.set_accel_sensitivity(accel_sensitivity)
    }

    pub fn accel_range(&self) -> f32 {
        match self.accel_sensitivity {
            Sensitivity::G1 => 2.,
            Sensitivity::G2 => 4.,
            Sensitivity::G4 => 8.,
            Sensitivity::G12 => 16.,
        }
    }
}

impl accelerometer::RawAccelerometer<accelerometer::vector::I16x3> for Compass {
    type Error = i2c::Error;
    fn accel_raw(
        &mut self,
    ) -> Result<accelerometer::vector::I16x3, accelerometer::Error<Self::Error>> {
        match self.lsm303dlhc.accel() {
            Ok(read) => Ok(accelerometer::vector::I16x3::new(read.x, read.y, read.z)),
            Err(err) => Err(accelerometer::Error::<Self::Error>::new_with_cause(
                ErrorKind::Device,
                err,
            )),
        }
    }
}

impl accelerometer::Accelerometer for Compass {
    type Error = i2c::Error;

    fn sample_rate(&mut self) -> Result<f32, accelerometer::Error<Self::Error>> {
        match self.accel_odr {
            AccelOdr::Hz1 => Ok(1.),
            AccelOdr::Hz10 => Ok(10.),
            AccelOdr::Hz25 => Ok(25.),
            AccelOdr::Hz50 => Ok(50.),
            AccelOdr::Hz100 => Ok(100.),
            AccelOdr::Hz200 => Ok(200.),
            AccelOdr::Hz400 => Ok(400.),
        }
    }

    fn accel_norm(&mut self) -> Result<vector::F32x3, accelerometer::Error<Self::Error>> {
        let raw_acceleration = self.accel_raw()?;
        let accel_range: f32 = self.accel_range();

        let x = raw_acceleration.x as f32 * accel_range / I16MAX;
        let y = raw_acceleration.y as f32 * accel_range / I16MAX;
        let z = raw_acceleration.z as f32 * accel_range / I16MAX;

        Ok(vector::F32x3::new(x, y, z))
    }
}
