#![no_main]
#![no_std]

use panic_probe as _;

use defmt_rtt as _;

#[rtic::app(device = stm32f4xx_hal::pac, dispatchers = [EXTI1])]
mod app {
    use stm32f4xx_hal::{pac::TIM5, prelude::*, timer::MonoTimerUs};

    #[monotonic(binds = TIM5, default = true)]
    type MicrosecMono = MonoTimerUs<TIM5>;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        let rcc = ctx.device.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(84.MHz()).freeze();
        let mono = ctx.device.TIM5.monotonic_us(&clocks);

        defmt::info!("program started");

        (Shared {}, Local {}, init::Monotonics(mono))
    }
}
