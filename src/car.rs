use crate::app::Display;
use crate::car::CarState::{ForwardDistanceInvalid, Normal};
use crate::servo::Servo;
use crate::tof_sensor::DistanceSensor;
use core::fmt::Debug;
use core::marker::PhantomData;
use defmt::Format;
use embedded_graphics::mono_font::ascii::FONT_6X12;
use embedded_graphics::mono_font::MonoTextStyleBuilder;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::Point;
use embedded_graphics::text::Text;
use embedded_graphics::Drawable;
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::PwmPin;
use fugit::ExtU32;
use tb6612fng::{DriveError, Motor};

#[derive(PartialEq, Eq, Debug, Format)]
pub enum CarState {
    /// Normal operation mode, car can be remotely controlled.
    Normal,
    /// Triggered if the distance is too small or not present at all. Can only be overridden once the distance is large enough again.
    ForwardDistanceInvalid,
}

#[derive(PartialEq, Eq, Debug, Format)]
pub enum Error {
    NotAllowedToDrive,
    DriveError(DriveError),
}

/// The maximum amount of time for which it's acceptable to not get a TOF signal. If this timeout is exceeded the car will do an emergency brake.
const MAX_FRONT_DISTANCE_SENSOR_LAG_IN_MS: u32 = 100;

/// The minimum front distance. If the distance is less than this the car will do an emergency brake.
const MIN_FRONT_DISTANCE_IN_MM: u16 = 500;

/// Represents the robot car.
pub struct Car<ServoPwm, MAIN1, MAIN2, MAPWM, DS, DE>
where
    ServoPwm: PwmPin,
    DS: DistanceSensor<DE>,
{
    // peripherals
    steering: Servo<ServoPwm>,
    motor: Motor<MAIN1, MAIN2, MAPWM>,
    front_distance_sensor: DS,
    display: Option<Display>,

    // data
    current_state: CarState,
    /// The latest measurement of the front distance (if available)
    latest_front_distance_in_mm: Option<u16>,
    last_front_distance_update: Option<fugit::TimerInstantU32<1_000_000>>,
    /// Needed to be able to specify the `DE` type parameter
    _distance_sensor_error: PhantomData<DE>,
}

impl<ServoPwm, MAIN1, MAIN2, MAPWM, DS, DE> Car<ServoPwm, MAIN1, MAIN2, MAPWM, DS, DE>
where
    ServoPwm: PwmPin<Duty = u16>,
    MAIN1: OutputPin,
    MAIN2: OutputPin,
    MAPWM: PwmPin<Duty = u16>,
    DS: DistanceSensor<DE>,
    DE: Debug,
{
    pub fn new(
        steering: Servo<ServoPwm>,
        motor: Motor<MAIN1, MAIN2, MAPWM>,
        distance_sensor: DS,
        display: Option<Display>,
    ) -> Self {
        Car {
            steering,
            motor,
            display,
            current_state: Normal,
            front_distance_sensor: distance_sensor,
            latest_front_distance_in_mm: None,
            last_front_distance_update: None,
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

    pub fn drive_forward(&mut self, speed: u8) -> Result<(), Error> {
        if self.current_state != Normal {
            return Err(Error::NotAllowedToDrive);
        }

        self.motor.drive_forward(speed).map_err(Error::DriveError)
    }

    pub fn drive_backwards(&mut self, speed: u8) -> Result<(), Error> {
        // no need to validate `self.current_state` here as we're still allowed to drive back even if
        // it's `ForwardDistanceInvalid` (we don't have a back sensor, so we presume that driving back is safe)
        self.motor.drive_backwards(speed).map_err(Error::DriveError)
    }

    /// Return the current speed of the motor (in percentage). Note that driving forward returns a positive number
    /// while driving backwards returns a negative number and both [`DriveCommand::Brake`] and [`DriveCommand::Stop`] return 0.
    pub fn current_speed(&mut self) -> i8 {
        self.motor.current_speed()
    }

    pub fn halt(&mut self) {
        self.motor.brake();
    }

    pub fn handle_distance_sensor_interrupt(
        &mut self,
        now: fugit::TimerInstantU32<1_000_000>,
    ) -> Result<(), DE> {
        if let Err(e) = self.front_distance_sensor.clear_interrupt() {
            self.latest_front_distance_in_mm = None;
            return Err(e);
        }
        let result = match self.front_distance_sensor.get_distance_in_mm() {
            Ok(distance) => {
                defmt::debug!("Received range: {}mm", distance);
                self.latest_front_distance_in_mm = Some(distance);
                self.last_front_distance_update = Some(now);
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

        self.validate_distance(now);

        result
    }

    pub fn validate_distance(&mut self, now: fugit::TimerInstantU32<1_000_000>) {
        if let Some(last_front_distance_update) = self.last_front_distance_update {
            if last_front_distance_update + MAX_FRONT_DISTANCE_SENSOR_LAG_IN_MS.millis() < now {
                defmt::error!("took too long to get a new TOF update => enabling emergency brake!");
                self.halt();
                self.current_state = ForwardDistanceInvalid;
            } else {
                // handle the case if we have data. note that if we don't have data we don't do anything
                // and just keep the previous state until we either time out (see above) or have a distance available again.
                if let Some(distance_in_mm) = self.latest_front_distance_in_mm {
                    if distance_in_mm < MIN_FRONT_DISTANCE_IN_MM {
                        defmt::warn!("collision warning, the front distance of {}mm is less than the safe minimum of {}mm - stopping the car!", distance_in_mm, MIN_FRONT_DISTANCE_IN_MM);
                        self.halt();
                        self.current_state = ForwardDistanceInvalid;
                    } else {
                        // enough distance => allow driving forward
                        self.current_state = Normal;
                    }
                }
            }
        } else {
            // no distance data available => prevent driving forward
            self.halt();
            self.current_state = ForwardDistanceInvalid;
        }

        self.update_display();
    }

    fn update_display(&mut self) {
        if let Some(display) = self.display.as_mut() {
            display.clear();
            let text_style = MonoTextStyleBuilder::new()
                .font(&FONT_6X12)
                .text_color(BinaryColor::On)
                .build();
            if let Some(front_distance_in_mm) = self.latest_front_distance_in_mm {
                let mut buffer = itoa::Buffer::new();
                let front_distance_in_mm = buffer.format(front_distance_in_mm);
                Text::new("Front distance: ", Point::new(15, 15), text_style)
                    .draw(display)
                    .unwrap();
                Text::new(front_distance_in_mm, Point::new(15, 30), text_style)
                    .draw(display)
                    .unwrap();
            }
            display.flush().unwrap();
        }
    }
}
