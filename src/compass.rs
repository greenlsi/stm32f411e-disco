use accelerometer::{self, ErrorKind, RawAccelerometer};
use lsm303dlhc::{AccelOdr, Lsm303dlhc, Sensitivity};

use crate::hal::gpio::{self, gpiob};
use crate::hal::i2c;
use crate::hal::prelude::*;
use crate::hal::rcc;
use crate::hal::stm32;

const MAX: f32 = 32786.; //Yo usaría la macro como en el ejemplo que me pasaste

type I2c1 = i2c::I2c<
    stm32::I2C1,
    (
        gpiob::PB6<gpio::AlternateOD<gpio::AF4>>,
        gpiob::PB9<gpio::AlternateOD<gpio::AF4>>, // el pin este estaba mal
    ),
>;

pub struct Compass {
    lsm303dlhc: lsm303dlhc::Lsm303dlhc<I2c1>,
    odr: lsm303dlhc::AccelOdr,
    sensitivity: lsm303dlhc::Sensitivity,
}

impl Compass {
    pub fn new(gpiob: gpiob::Parts, i2c1: stm32::I2C1, clocks: rcc::Clocks) -> Self {
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

        //let lsm303dlhc = lsm303dlhc::Lsm303dlhc::new(i2c);
        // Lo suyo sería ser más "delicado" con los errores
        let lsm303dlhc = Lsm303dlhc::new(i2c).unwrap();
        let odr = AccelOdr::Hz400;
        let sensitivity = Sensitivity::G1;

        Self {
            lsm303dlhc,
            odr,
            sensitivity,
        }
    }
    pub fn set_accel_odr(&mut self, odr: AccelOdr) -> Result<(), i2c::Error> {
        // No modificabas sensitivity!
        self.odr = match odr {
            // AccelOdr no implementa el trato Copy, así que toca hacer esta chapuza... A ver si mejoro la librería del compass
            AccelOdr::Hz1 => AccelOdr::Hz1,
            AccelOdr::Hz10 => AccelOdr::Hz10,
            AccelOdr::Hz25 => AccelOdr::Hz25,
            AccelOdr::Hz50 => AccelOdr::Hz50,
            AccelOdr::Hz100 => AccelOdr::Hz100,
            AccelOdr::Hz200 => AccelOdr::Hz200,
            AccelOdr::Hz400 => AccelOdr::Hz400,
        };
        self.lsm303dlhc.accel_odr(odr)
    }
    pub fn set_accel_sensitivity(&mut self, sensitivity: Sensitivity) -> Result<(), i2c::Error> {
        self.sensitivity = sensitivity; // No modificabas sensitivity!
        self.lsm303dlhc.set_accel_sensitivity(sensitivity)
    }
    pub fn range(&self) -> f32 {
        match self.sensitivity {
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
        match self.odr {
            AccelOdr::Hz1 => Ok(1.),
            AccelOdr::Hz10 => Ok(10.),
            AccelOdr::Hz25 => Ok(25.),
            AccelOdr::Hz50 => Ok(50.),
            AccelOdr::Hz100 => Ok(100.),
            AccelOdr::Hz200 => Ok(200.),
            AccelOdr::Hz400 => Ok(400.),
        }
    }

    fn accel_norm(
        &mut self,
    ) -> Result<accelerometer::vector::F32x3, accelerometer::Error<Self::Error>> {
        let raw_acceleration = self.accel_raw()?;
        let rango: f32 = self.range();
        let x = raw_acceleration.x as f32 * (rango / MAX);
        let y = raw_acceleration.y as f32 * (rango / MAX);
        let z = raw_acceleration.z as f32 * (rango / MAX);
        Ok(accelerometer::vector::F32x3::new(x, y, z))
    }
}
