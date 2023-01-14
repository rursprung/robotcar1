//! Abstraction layer for the TOF sensor to avoid knowing about the details of it in `Car` (there's no
//! generic `trait` for TOFs available yet unlike for other devices (e.g. displays)).
//!
/// If there were a general need for this, an `embedded-distance-sensor` crate (or similar) should
/// be created with a more elaborate abstraction for distance sensors and this should then be implemented
/// by the different drivers, so that consumers can directly interact with these traits.
/// It is however unclear how much benefit this would bring (not investigated so far).
use core::fmt::Debug;
use embedded_hal::blocking::i2c::{Read, Write, WriteRead};
use vl53l1x_uld::{Error, VL53L1X};

/// Represents a simple distance sensor.
///
/// For simplicity the error is currently not modelled here but instead the error of the actual
/// implementation will be used (it's not referenced directly anywhere in `Car`, so this is fine).
pub trait DistanceSensor<E> {
    /// Get the distance measured in millimeters (if available).
    fn get_distance_in_mm(&mut self) -> Result<u16, E>;
}

impl<I2C, E> DistanceSensor<Error<E>> for VL53L1X<I2C>
where
    E: Debug,
    I2C: Write<Error = E> + Read<Error = E> + WriteRead<Error = E>,
{
    fn get_distance_in_mm(&mut self) -> Result<u16, Error<E>> {
        self.clear_interrupt()?;
        self.get_distance()
    }
}
