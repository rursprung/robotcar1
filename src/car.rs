use crate::servo::Servo;
use crate::tof_sensor::DistanceSensor;
use core::fmt::Debug;
use core::marker::PhantomData;
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::PwmPin;
use tb6612fng::Motor;

/// Represents the robot car.
pub struct Car<ServoPwm, MAIN1, MAIN2, MAPWM, D, DE>
where
    ServoPwm: PwmPin,
    D: DistanceSensor<DE>,
{
    steering: Servo<ServoPwm>,
    motor: Motor<MAIN1, MAIN2, MAPWM>,
    front_distance_sensor: D,
    /// The latest measurement of the front distance (if available)
    latest_front_distance_in_mm: Option<u16>,
    /// Needed to be able to specify the `DE` type parameter
    _distance_sensor_error: PhantomData<DE>,
}

impl<ServoPwm, MAIN1, MAIN2, MAPWM, D, DE> Car<ServoPwm, MAIN1, MAIN2, MAPWM, D, DE>
where
    ServoPwm: PwmPin<Duty = u16>,
    MAIN1: OutputPin,
    MAIN2: OutputPin,
    MAPWM: PwmPin<Duty = u16>,
    D: DistanceSensor<DE>,
    DE: Debug,
{
    pub fn new(
        steering: Servo<ServoPwm>,
        motor: Motor<MAIN1, MAIN2, MAPWM>,
        distance_sensor: D,
    ) -> Car<ServoPwm, MAIN1, MAIN2, MAPWM, D, DE> {
        Car {
            steering,
            motor,
            front_distance_sensor: distance_sensor,
            latest_front_distance_in_mm: None,
            _distance_sensor_error: PhantomData,
        }
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

    pub fn handle_distance_sensor_interrupt(&mut self) -> Result<(), DE> {
        if let Err(e) = self.front_distance_sensor.clear_interrupt() {
            self.latest_front_distance_in_mm = None;
            return Err(e);
        }
        let result = match self.front_distance_sensor.get_distance_in_mm() {
            Ok(distance) => {
                defmt::debug!("Received range: {}mm", distance);
                self.latest_front_distance_in_mm = Some(distance);
                Ok(())
            }
            Err(e) => {
                defmt::error!(
                    "Failed to get distance from TOF: {}",
                    defmt::Debug2Format(&e)
                );
                self.latest_front_distance_in_mm = None;
                Err(e)
            }
        };

        self.handle_distance_update();

        result
    }

    fn handle_distance_update(&mut self) {
        // TODO: deal with the distance
    }
}
