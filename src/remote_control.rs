use adafruit_bluefruit_rs::{bluefruit_protocol, BluefruitLEUARTFriend};

pub fn handle_bluetooth_message(bt_module: &mut BluefruitLEUARTFriend) {
    let (filled_buffer, _) = bt_module
        .rx_transfer
        .next_transfer(bt_module.rx_buffer.take().unwrap())
        .unwrap();
    defmt::debug!(
            "bluetooth: DMA transfer complete, received {:a}",
            filled_buffer
        );

    let event = bluefruit_protocol::parse::<4>(filled_buffer);
    defmt::info!("received event(s) over bluetooth: {}", &event);

    // switch out the buffers
    bt_module.rx_buffer = Some(filled_buffer);
}
