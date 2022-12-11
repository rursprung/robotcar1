use core::fmt::Debug;
use embedded_hal::blocking::i2c::{Read, Write, WriteRead};
use vl53l1x_uld::{self, IOVoltage, Polarity, DEFAULT_ADDRESS, VL53L1X};

pub trait DistanceSensor<E> {
    fn get_distance_in_mm(&mut self) -> Result<u16, E>;
    fn clear_interrupt(&mut self) -> Result<(), E>;
}

pub struct TOFSensor<I2C, E>
where
    E: Debug,
    I2C: Write<Error = E> + Read<Error = E> + WriteRead<Error = E>,
{
    device: VL53L1X<I2C>,
}

impl<I2C, E> TOFSensor<I2C, E>
where
    E: Debug,
    I2C: Write<Error = E> + Read<Error = E> + WriteRead<Error = E>,
{
    pub fn new(i2c: I2C) -> Result<Self, vl53l1x_uld::Error<E>> {
        let mut device = VL53L1X::new(i2c, DEFAULT_ADDRESS);
        device.init(IOVoltage::Volt2_8)?;
        device.set_interrupt_polarity(Polarity::ActiveHigh)?;
        device.start_ranging()?;

        Ok(TOFSensor { device })
    }
}

impl<I2C, E> DistanceSensor<vl53l1x_uld::Error<E>> for TOFSensor<I2C, E>
where
    E: Debug,
    I2C: Write<Error = E> + Read<Error = E> + WriteRead<Error = E>,
{
    fn get_distance_in_mm(&mut self) -> Result<u16, vl53l1x_uld::Error<E>> {
        self.device.get_distance()
    }

    fn clear_interrupt(&mut self) -> Result<(), vl53l1x_uld::Error<E>> {
        self.device.clear_interrupt()
    }
}
