use accelerometer::{Error, ErrorKind, RawAccelerometer};
use lsm303dlhc::Lsm303dlhc;

use crate::hal::gpio::{self, gpiob};
use crate::hal::i2c;
use crate::hal::prelude::*;
use crate::hal::rcc;
use crate::hal::stm32;

const SCALE: f32 = 4.6 / 256.0; // TODO alomejor esto hay que cambiarlo -> el acelerómetro es de 16 bits!

type I2c1 = i2c::I2c<
    stm32::I2C1,
    (
        gpiob::PB6<gpio::AlternateOD<gpio::AF4>>,
        gpiob::PB7<gpio::AlternateOD<gpio::AF4>>,
    ),
>;

pub struct Compass {
    lsm303dlhc: lsm303dlhc::Lsm303dlhc<I2c1>,
}

impl Compass {
    pub fn new(gpiob: gpiob::Parts, i2c1: stm32::I2C1, clocks: rcc::Clocks) -> Self {
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

        let i2c = i2c::I2c::new(i2c1, (scl, sda), 400.khz().into(), clocks);

        //let lsm303dlhc = lsm303dlhc::Lsm303dlhc::new(i2c);
        // Lo suyo sería ser más "delicado" con los errores
        let lsm303dlhc = Lsm303dlhc::new(i2c).unwrap();

        Self { lsm303dlhc }
    }
}

impl accelerometer::RawAccelerometer<accelerometer::vector::I16x3> for Compass {
    type Error = i2c::Error;
    fn accel_raw(
        &mut self,
    ) -> Result<accelerometer::vector::I16x3, accelerometer::Error<Self::Error>> {
        // En esta función trato los errores adecuadamente. Échale un vistazo:
        match self.lsm303dlhc.accel() {
            Ok(read) => Ok(accelerometer::vector::I16x3::new(read.x, read.y, read.z)),
            Err(err) => Err(Error::<Self::Error>::new_with_cause(ErrorKind::Device, err)),
        }
    }
}

impl accelerometer::Accelerometer for Compass {
    type Error = i2c::Error;
    fn sample_rate(&mut self) -> Result<f32, accelerometer::Error<Self::Error>> {
        // modify_accel_register es privado, no podemos usarlo
        // lo suyo sería hacer un fork de la librería del acelerómetro y ponerla bien
        // Pero bueno, de momento lo que podemos hacer es lanzar 400 (que es el valor por defecto).
        Ok(400.)
        // También podríamos lanzar un error
        //Err(Error::<Self::Error>::new(ErrorKind::Device))
        /*
        self.lsm303dlhc.modify_accel_register(CTRL_REG1_A, |r| {
            r & !(0b1111 << 4) | ((lsm303dlhc::AccelOdr as u8) << 4)
        })
        */
    }

    fn accel_norm(
        &mut self,
    ) -> Result<accelerometer::vector::F32x3, accelerometer::Error<Self::Error>> {
        let raw_acceleration: accelerometer::vector::I16x3 = self.accel_raw().unwrap();
        Ok(accelerometer::vector::F32x3::new(
            raw_acceleration.x as f32 * SCALE,
            raw_acceleration.y as f32 * SCALE,
            raw_acceleration.z as f32 * SCALE,
        ))
    }
}
