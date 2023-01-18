//! Contains the logic for the remote control. This deals with the events sent by the remote control
//! app (e.g. on a smartphone) and triggers the corresponding actions on the robotcar.

use crate::bt_module::BluefruitLEUARTFriend;
use crate::CarT as Car;
use adafruit_bluefruit_protocol::{
    self,
    button_event::{Button, ButtonEvent, ButtonState},
    ControllerEvent,
};
use core::cmp::{max, min, Ordering};

/// The remote control which handles the events sent by an app.
pub struct RemoteControl {
    bt_module: BluefruitLEUARTFriend,
}

impl RemoteControl {
    /// Instantiate a new remote control to handle events.
    pub fn new(bt_module: BluefruitLEUARTFriend) -> RemoteControl {
        RemoteControl { bt_module }
    }

    /// This needs to be triggered every time a bluetooth message has been received, which is either
    /// the case if either a line idle interrupt or a DMA full interrupt occurs.
    ///
    /// It handles the DMA buffer and acts on the message received.
    pub fn handle_bluetooth_message(&mut self, car: &mut Car) {
        let (filled_buffer, _) = self
            .bt_module
            .rx_transfer
            .next_transfer(self.bt_module.rx_buffer.take().unwrap())
            .unwrap();
        defmt::trace!(
            "bluetooth: DMA transfer complete, received {:a}",
            filled_buffer
        );

        let events = adafruit_bluefruit_protocol::parse::<4>(filled_buffer);
        for event in events {
            defmt::debug!("received event over bluetooth: {}", &event);

            match event {
                Ok(event) => {
                    self.handle_event(event, car);
                }
                Err(err) => {
                    defmt::error!("error in event parsing: {}", err);
                }
            }
        }

        // switch out the buffers
        filled_buffer.fill(0);
        self.bt_module.rx_buffer = Some(filled_buffer);

        self.bt_module.rx_transfer.clear_idle_interrupt();
    }

    fn handle_event(&mut self, event: ControllerEvent, car: &mut Car) {
        match event {
            ControllerEvent::ButtonEvent(button_event) => {
                self.handle_button_event(button_event, car)
            }
        }
    }

    /// Button events are used to remotely control the car (steering, speed change, etc.).
    fn handle_button_event(&mut self, event: ButtonEvent, car: &mut Car) {
        defmt::debug!("handling {}", event);
        match (event.button(), event.state()) {
            (Button::Left, ButtonState::Pressed) => {
                car.steer_left();
            }
            (Button::Right, ButtonState::Pressed) => {
                car.steer_right();
            }
            (Button::Left | Button::Right, ButtonState::Released) => {
                car.steer_center();
            }
            (Button::Up, ButtonState::Pressed) => {
                let new_speed = min(car.current_speed() + 25, 100);
                self.handle_speed_change(car, new_speed);
            }
            (Button::Down, ButtonState::Pressed) => {
                let new_speed = max(car.current_speed() - 25, -100);
                self.handle_speed_change(car, new_speed);
            }
            (Button::Button1, ButtonState::Pressed) => {
                self.handle_speed_change(car, 0);
            }
            (Button::Up | Button::Down | Button::Button1, ButtonState::Released) => {
                defmt::trace!("button released which doesn't need any action");
            }
            evt => {
                defmt::warn!("unimplemented event {}", evt);
            }
        }
    }

    fn handle_speed_change(&mut self, car: &mut Car, new_speed: i8) {
        defmt::debug!("new speed set by remote: {}", new_speed);
        // ignore failures as we can't report back to the actual remote control. the user will see
        // whether his actions had an effect or not and can try again if he thinks that the action
        // should work in a next step.
        match new_speed.cmp(&0) {
            Ordering::Greater => {
                if let Err(err) = car.drive_forward(new_speed as u8) {
                    defmt::error!("couldn't drive forward! {}", err);
                }
            }
            Ordering::Less => {
                if let Err(err) = car.drive_backwards((-new_speed) as u8) {
                    defmt::error!("couldn't drive backwards! {}", err);
                }
            }
            Ordering::Equal => {
                car.halt();
            }
        };
    }
}
