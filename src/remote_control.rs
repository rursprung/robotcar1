use crate::bt_module::BluefruitLEUARTFriend;
use crate::CarT as Car;
use adafruit_bluefruit_protocol::{
    self,
    button_event::{Button, ButtonEvent, ButtonState},
    ControllerEvent,
};
use core::cmp::{max, min, Ordering};

pub struct RemoteControl {
    bt_module: BluefruitLEUARTFriend,
}

impl RemoteControl {
    pub fn new(bt_module: BluefruitLEUARTFriend) -> RemoteControl {
        RemoteControl { bt_module }
    }

    pub fn handle_bluetooth_message(&mut self, car: &mut Car) {
        let (filled_buffer, _) = self
            .bt_module
            .rx_transfer
            .next_transfer(self.bt_module.rx_buffer.take().unwrap())
            .unwrap();
        defmt::debug!(
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

    fn handle_button_event(&mut self, event: ButtonEvent, car: &mut Car) {
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
                defmt::debug!("button released which doesn't need any action");
            }
            evt => {
                defmt::warn!("unimplemented event {}", evt);
            }
        }
    }

    fn handle_speed_change(&mut self, car: &mut Car, new_speed: i8) {
        // TODO: handle result
        match new_speed.cmp(&0) {
            Ordering::Greater => car.drive_forward(new_speed as u8).expect("can drive"),
            Ordering::Less => car.drive_backwards((-new_speed) as u8).expect("can drive"),
            Ordering::Equal => car.halt(),
        }
    }
}
