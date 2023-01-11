use embedded_hal::PwmPin;

pub struct Servo<PWM>
where
    PWM: PwmPin,
{
    pwm: PWM,
    min_duty: PWM::Duty,
    max_duty: PWM::Duty,
}

impl<PWM> Servo<PWM>
where
    PWM: PwmPin<Duty = u16>,
{
    pub fn new(
        mut pwm: PWM,
        min_duty: PWM::Duty,
        max_duty: PWM::Duty,
        initial_angle: u8,
    ) -> Servo<PWM> {
        pwm.enable();
        let mut servo = Servo {
            pwm,
            min_duty,
            max_duty,
        };

        servo.steer(initial_angle);

        servo
    }

    pub fn steer(&mut self, angle: u8) {
        if angle > 180 {
            panic!("angle > 180"); // TODO: change to result!
        }
        let duty = ((self.max_duty - self.min_duty) as f32 * (angle as f32 / 180.0)) as PWM::Duty
            + self.min_duty;
        defmt::debug!(
            "setting steering angle to {}, resulting in duty {}",
            angle,
            duty
        );
        self.pwm.set_duty(duty);
    }
}
