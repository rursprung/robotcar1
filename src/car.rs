use crate::servo::Servo;
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::PwmPin;
use tb6612fng::Motor;

/// Represents the robot car.
pub struct Car<ServoPwm, MAIN1, MAIN2, MAPWM>
where
    ServoPwm: PwmPin,
{
    steering: Servo<ServoPwm>,
    motor: Motor<MAIN1, MAIN2, MAPWM>,
}

impl<ServoPwm, MAIN1, MAIN2, MAPWM> Car<ServoPwm, MAIN1, MAIN2, MAPWM>
where
    ServoPwm: PwmPin<Duty = u16>,
    MAIN1: OutputPin,
    MAIN2: OutputPin,
    MAPWM: PwmPin<Duty = u16>,
{
    pub fn new(
        steering: Servo<ServoPwm>,
        motor: Motor<MAIN1, MAIN2, MAPWM>,
    ) -> Car<ServoPwm, MAIN1, MAIN2, MAPWM> {
        Car { steering, motor }
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

    pub fn drive_forward(&mut self, speed: u8) {
        // TODO: handle result
        self.motor.drive_forward(speed).expect("can drive forward");
    }

    pub fn drive_backwards(&mut self, speed: u8) {
        // TODO: handle result
        self.motor
            .drive_backwards(speed)
            .expect("can drive backwards");
    }

    pub fn halt(&mut self) {
        self.motor.brake();
    }
}
