use embedded_hal::PwmPin;

pub struct Servo<PWM> {
    pwm: PWM,
    min_duty: u16,
    max_duty: u16,
}

impl<PWM> Servo<PWM>
where
    PWM: PwmPin<Duty = u16>,
{
    pub fn new(mut pwm: PWM, min_duty: u16, max_duty: u16, initial_angle: u8) -> Servo<PWM> {
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
        let duty = (((self.max_duty - self.min_duty) as f32 * (angle as f32 / 180.0)) as u16
            + self.min_duty) as u16;
        defmt::info!(
            "setting steering angle to {}, resulting in duty {}",
            angle,
            duty
        );
        self.pwm.set_duty(duty);
    }
}
