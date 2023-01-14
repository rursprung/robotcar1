//! This represents a steering unit on the robotcar, powered by a servo motor.

use crate::steering::Direction::{Centre, Left, Right};
use crate::steering::Error::InvalidPercentage;
use defmt::Format;
use embedded_hal::PwmPin;

/// The steering unit of the robotcar.
pub struct Steering<PWM>
where
    PWM: PwmPin,
{
    pwm: PWM,
    steering_centre: PWM::Duty,
    max_steering_side: PWM::Duty,
}

/// Defines errors which can happen while trying to steer.
#[derive(PartialEq, Eq, Debug, Copy, Clone, Format)]
pub enum Error {
    /// An invalid steering angle has been defined. The steering angle must be given as a percentage value between 0 and 100 to be valid.
    InvalidPercentage,
}

#[derive(PartialEq, Eq, Debug, Copy, Clone, Format)]
pub enum Direction {
    /// Steer centre (straight ahead)
    Centre,
    /// Steer left with the defined angle (in percentage: 0% = centre, 100% = max. left)
    Left(u8),
    /// Steer right with the defined angle (in percentage: 0% = centre, 100% = max. right)
    Right(u8),
}

impl<PWM> Steering<PWM>
where
    PWM: PwmPin<Duty = u16>,
{
    pub fn new(
        mut pwm: PWM,
        steering_centre: PWM::Duty,
        max_steering_side: PWM::Duty,
    ) -> Steering<PWM> {
        pwm.enable();
        let mut servo = Steering {
            pwm,
            steering_centre,
            max_steering_side,
        };

        servo.steer(Centre).ok(); // centre will never fail as we don't specify a percentage

        servo
    }

    /// Set the new steering direction. The direction will be kept until the next call which sets a new direction.
    pub fn steer(&mut self, direction: Direction) -> Result<(), Error> {
        let duty = match direction {
            Centre => self.steering_centre,
            Left(percentage) => {
                if percentage > 100 {
                    return Err(InvalidPercentage);
                }
                self.steering_centre - (self.max_steering_side / 100 * percentage as PWM::Duty)
            }
            Right(percentage) => {
                if percentage > 100 {
                    return Err(InvalidPercentage);
                }
                self.steering_centre + (self.max_steering_side / 100 * percentage as PWM::Duty)
            }
        };

        defmt::debug!("steering {}, resulting in duty {}", direction, duty);
        self.pwm.set_duty(duty);

        Ok(())
    }
}
