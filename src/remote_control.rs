use crate::servo::Servo;
use adafruit_bluefruit_rs::bluefruit_protocol::{
    Button, ButtonEvent, ButtonState, ControllerEvent,
};
use adafruit_bluefruit_rs::{bluefruit_protocol, BluefruitLEUARTFriend};
use embedded_hal::PwmPin;

pub fn handle_bluetooth_message<PWM>(
    bt_module: &mut BluefruitLEUARTFriend,
    steering: &mut Servo<PWM>,
) where
    PWM: PwmPin<Duty = u16>,
{
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
                handle_event(event, steering);
            }
            Err(err) => {
                defmt::error!("error in event parsing: {}", err);
            }
        }
    }

    // switch out the buffers
    bt_module.rx_buffer = Some(filled_buffer);
}

fn handle_event<PWM>(event: ControllerEvent, steering: &mut Servo<PWM>)
where
    PWM: PwmPin<Duty = u16>,
{
    match event {
        ControllerEvent::ButtonEvent(button_event) => handle_button_event(button_event, steering),
        evt => {
            defmt::warn!("unimplemented event {}", evt);
        }
    }
}

fn handle_button_event<PWM>(event: ButtonEvent, steering: &mut Servo<PWM>)
where
    PWM: PwmPin<Duty = u16>,
{
    match (event.button(), event.state()) {
        (Button::Left, ButtonState::Pressed) => {
            steering.steer(0);
        }
        (Button::Right, ButtonState::Pressed) => {
            steering.steer(180);
        }
        evt => {
            steering.steer(90);
            defmt::warn!("unimplemented event {}", evt);
        }
    }
}
