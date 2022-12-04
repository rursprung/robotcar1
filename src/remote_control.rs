use crate::car::Car;
use adafruit_bluefruit_rs::bluefruit_protocol::{
    Button, ButtonEvent, ButtonState, ControllerEvent,
};
use adafruit_bluefruit_rs::{bluefruit_protocol, BluefruitLEUARTFriend};
use embedded_hal::PwmPin;

pub struct RemoteControl {
    bt_module: BluefruitLEUARTFriend,
}

impl RemoteControl {
    pub fn new(bt_module: BluefruitLEUARTFriend) -> RemoteControl {
        RemoteControl { bt_module }
    }

    pub fn handle_bluetooth_message<PWM>(&mut self, car: &mut Car<PWM>)
    where
        PWM: PwmPin<Duty = u16>,
    {
        let bt_module = &mut self.bt_module;
        let (filled_buffer, _) = bt_module
            .rx_transfer
            .next_transfer(bt_module.rx_buffer.take().unwrap())
            .unwrap();
        defmt::debug!(
            "bluetooth: DMA transfer complete, received {:a}",
            filled_buffer
        );

        let events = bluefruit_protocol::parse::<4>(filled_buffer);
        for event in events {
            defmt::info!("received event over bluetooth: {}", &event);

            match event {
                Ok(event) => {
                    handle_event(event, car);
                }
                Err(err) => {
                    defmt::error!("error in event parsing: {}", err);
                }
            }
        }

        // switch out the buffers
        bt_module.rx_buffer = Some(filled_buffer);
    }
}

fn handle_event<PWM>(event: ControllerEvent, car: &mut Car<PWM>)
where
    PWM: PwmPin<Duty = u16>,
{
    match event {
        ControllerEvent::ButtonEvent(button_event) => handle_button_event(button_event, car),
        evt => {
            defmt::error!("unimplemented event {}", evt);
        }
    }
}

fn handle_button_event<PWM>(event: ButtonEvent, car: &mut Car<PWM>)
where
    PWM: PwmPin<Duty = u16>,
{
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
        evt => {
            defmt::error!("unimplemented event {}", evt);
        }
    }
}
