#![no_main]
#![no_std]

use panic_halt as _;

use defmt_rtt as _;

use cortex_m_rt::entry;

use stm32f4xx_hal as _;

#[entry]
fn main() -> ! {
    defmt::info!("program started");
    loop {}
}
