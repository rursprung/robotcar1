#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_probe as _;

use defmt_rtt as _;

#[rtic::app(device = stm32f4xx_hal::pac, dispatchers = [EXTI1])]
mod app {
    use stm32f4xx_hal::{
        dma::StreamsTuple,
        gpio::{Edge, Input, PA0, PA9},
        i2c::I2c,
        pac::{IWDG, TIM5, USART1},
        prelude::*,
        serial::{self, config::DmaConfig, Rx, Serial, Tx},
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
        button: PA9<Input>,
        tof_data_interrupt: PA0<Input>,
    }

    #[init]
    fn init(mut ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        let mut syscfg = ctx.device.SYSCFG.constrain();

        let rcc = ctx.device.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(84.MHz()).freeze();
        let mono = ctx.device.TIM5.monotonic_us(&clocks);

        let watchdog = setup_watchdog(ctx.device.IWDG);

        let gpioa = ctx.device.GPIOA.split();
        let gpiob = ctx.device.GPIOB.split();
        let gpioc = ctx.device.GPIOC.split();

        // Note: as a first step we just try to set up all peripherals here to let the compiler check if we made any mistakes.
        // This would e.g. fail when trying to use the same timer twice for two different PWMs or when using the wrong pins with the wrong PWM / I2C / etc.
        // This code will afterwards be moved / re-written when the actual functionality will be implemented.

        // set up the status LEDs
        let _led_status_ok = gpioa.pa7.into_push_pull_output();
        let _led_status_autonomous = gpioa.pa8.into_push_pull_output();

        // set up the user button
        let mut button = gpioa.pa9.into_pull_down_input();
        button.make_interrupt_source(&mut syscfg);
        button.enable_interrupt(&mut ctx.device.EXTI);
        button.trigger_on_edge(&mut ctx.device.EXTI, Edge::Falling);

        // set up I2C
        let _i2c = I2c::new(ctx.device.I2C1, (gpiob.pb8, gpiob.pb9), 400.kHz(), &clocks);

        // set up the interrupt for the TOF
        let mut tof_data_interrupt = gpioa.pa0.into_pull_down_input();
        tof_data_interrupt.make_interrupt_source(&mut syscfg);
        tof_data_interrupt.enable_interrupt(&mut ctx.device.EXTI);
        tof_data_interrupt.trigger_on_edge(&mut ctx.device.EXTI, Edge::Falling);

        // set up USART (for the bluetooth module)
        let usart1 = Serial::new(
            ctx.device.USART1,
            (gpiob.pb6.into_alternate(), gpioa.pa10.into_alternate()),
            serial::Config::default()
                .baudrate(9600.bps())
                .dma(DmaConfig::Rx),
            &clocks,
        )
        .expect("USART1 can be set up");
        let (_usart1_tx, _usart1_rx): (Tx<USART1, u8>, Rx<USART1, u8>) = usart1.split();
        let _streams = StreamsTuple::new(ctx.device.DMA2);

        // set up servo 1 & 2
        let (_servo1_pwm, _servo2_pwm) = ctx
            .device
            .TIM3
            .pwm_hz(
                (gpioa.pa6.into_alternate(), gpioc.pc7.into_alternate()),
                50.Hz(),
                &clocks,
            )
            .split();

        // set up motor 1 & 2
        let _motor1_in1 = gpiob.pb5.into_push_pull_output();
        let _motor1_in2 = gpiob.pb4.into_push_pull_output();
        let _motor2_in1 = gpioa.pa1.into_push_pull_output();
        let _motor2_in2 = gpioa.pa4.into_push_pull_output();
        let (_motor2_pwm, _motor1_pwm) = ctx
            .device
            .TIM2
            .pwm_hz(
                (gpiob.pb3.into_alternate(), gpiob.pb10.into_alternate()),
                100.kHz(),
                &clocks,
            )
            .split();

        defmt::info!("init done");

        (
            Shared {},
            Local {
                watchdog,
                button,
                tof_data_interrupt,
            },
            init::Monotonics(mono),
        )
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
    #[task(priority = 1, local = [watchdog])]
    fn feed_watchdog(cx: feed_watchdog::Context) {
        defmt::trace!("feeding the watchdog!");
        cx.local.watchdog.feed();
        feed_watchdog::spawn_after(100.millis()).ok();
    }

    // see here for why this is EXTI9_5: https://github.com/stm32-rs/stm32f4xx-hal/blob/6d0c29233a4cd1f780b2fef3e47ef091ead6cf4a/src/gpio/exti.rs#L8-L23
    /// Triggers every time the user button is pressed.
    #[task(binds=  EXTI9_5, local = [button])]
    fn button_click(ctx: button_click::Context) {
        ctx.local.button.clear_interrupt_pending_bit();

        defmt::info!("button pressed");
    }

    // see here for why this is EXTI0: https://github.com/stm32-rs/stm32f4xx-hal/blob/6d0c29233a4cd1f780b2fef3e47ef091ead6cf4a/src/gpio/exti.rs#L8-L23
    /// Triggers every time the TOF has data (= new range measurement) available to be consumed.
    #[task(binds = EXTI0, local = [tof_data_interrupt])]
    fn tof_interrupt_triggered(ctx: tof_interrupt_triggered::Context) {
        ctx.local.tof_data_interrupt.clear_interrupt_pending_bit();

        defmt::info!("TOF interrupt triggered (data ready)");
    }
}
