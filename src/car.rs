use crate::servo::Servo;
use embedded_hal::PwmPin;

/// Represents the robot car.
pub struct Car<ServoPwm>
where
    ServoPwm: PwmPin,
{
    steering: Servo<ServoPwm>,
}

impl<ServoPwm> Car<ServoPwm>
where
    ServoPwm: PwmPin<Duty = u16>,
{
    pub fn new(steering: Servo<ServoPwm>) -> Car<ServoPwm> {
        Car { steering }
    }

    pub fn steer_left(&mut self) {
        self.steering.steer(0);
    }

    pub fn steer_center(&mut self) {
        self.steering.steer(90);
    }

    pub fn steer_right(&mut self) {
        self.steering.steer(180);
    }

    pub fn drive_forward(&self) {
        todo!()
    }

    pub fn drive_backwards(&self) {
        todo!()
    }

    pub fn halt(&self) {
        todo!()
    }
}
