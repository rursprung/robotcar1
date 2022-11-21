#![no_main]
#![no_std]

use panic_probe as _;

use defmt_rtt as _;

#[rtic::app(device = stm32f4xx_hal::pac, dispatchers = [EXTI1])]
mod app {
    use stm32f4xx_hal::{
        pac::{IWDG, TIM5},
        prelude::*,
        timer::MonoTimerUs,
        watchdog::IndependentWatchdog,
    };

    #[monotonic(binds = TIM5, default = true)]
    type MicrosecMono = MonoTimerUs<TIM5>;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        watchdog: IndependentWatchdog,
    }

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        let rcc = ctx.device.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(84.MHz()).freeze();
        let mono = ctx.device.TIM5.monotonic_us(&clocks);

        let watchdog = setup_watchdog(ctx.device.IWDG);

        defmt::info!("program started");

        (Shared {}, Local { watchdog }, init::Monotonics(mono))
    }

    /// Set up the independent watchdog and start the period task to feed it
    fn setup_watchdog(iwdg: IWDG) -> IndependentWatchdog {
        let mut watchdog = IndependentWatchdog::new(iwdg);
        watchdog.start(500u32.millis());
        watchdog.feed();
        feed_watchdog::spawn().ok();
        defmt::trace!("watchdog set up");
        watchdog
    }

    /// Feed the watchdog periodically to avoid hardware reset.
    #[task(priority=1, local=[watchdog])]
    fn feed_watchdog(cx: feed_watchdog::Context) {
        defmt::trace!("feeding the watchdog!");
        cx.local.watchdog.feed();
        feed_watchdog::spawn_after(100.millis()).ok();
    }
}
