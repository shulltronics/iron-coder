//! Propmaker Featherwing Iron Coder BSP

#![no_std]

use lis3dh::{Lis3dh, Lis3dhI2C, SlaveAddr};
use lis3dh::accelerometer::Accelerometer;
use embedded_hal::blocking::i2c::{WriteRead, Write};

pub struct Accelerometer<I2C> 
    where I2C: WriteRead + Write
{
    accelerometer: Lis3dh<Lis3dhI2C<I2C>>,
}

impl<T, E> Accelerometer<T>
    where T: WriteRead<Error = E> + Write<Error = E>, E: core::fmt::Debug
{
    // Create a new PropMaker. This should probably take arguments of the
    // configured interfaces from the programmable board.
    pub fn init(i2c: T) -> Self {
        Self {
            accelerometer: Lis3dh::new_i2c(i2c, SlaveAddr::Default).unwrap(),
        }
    }

    pub fn read(&mut self) -> Result<f32, ()> {
        a.accel_norm.unwrap()[0];
    }
}