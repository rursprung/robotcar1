use crate::bt_module::BluefruitLEUARTFriend;
use crate::CarT as Car;
use adafruit_bluefruit_protocol::{
    self,
    button_event::{Button, ButtonEvent, ButtonState},
    ControllerEvent,
};
use core::cmp::{max, min};

pub struct RemoteControl {
    bt_module: BluefruitLEUARTFriend,
    current_speed: i8,
}

impl RemoteControl {
    pub fn new(bt_module: BluefruitLEUARTFriend) -> RemoteControl {
        RemoteControl {
            bt_module,
            current_speed: 0,
        }
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
            defmt::info!("received event over bluetooth: {}", &event);

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
                self.current_speed = min(self.current_speed + 25, 100);
                self.handle_speed_change(car);
            }
            (Button::Down, ButtonState::Pressed) => {
                self.current_speed = max(self.current_speed - 25, -100);
                self.handle_speed_change(car);
            }
            (Button::Button1, ButtonState::Pressed) => {
                self.current_speed = 0;
                self.handle_speed_change(car);
            }
            evt => {
                defmt::error!("unimplemented event {}", evt);
            }
        }
    }

    fn handle_speed_change(&mut self, car: &mut Car) {
        if self.current_speed > 0 {
            car.drive_forward(self.current_speed as u8);
        } else if self.current_speed < 0 {
            car.drive_backwards((-self.current_speed) as u8);
        } else {
            car.halt();
        }
    }
}
