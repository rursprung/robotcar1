//#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

mod car;
mod remote_control;
mod servo;

use panic_probe as _;

use defmt_rtt as _;

#[rtic::app(device = stm32f4xx_hal::pac, dispatchers = [EXTI1])]
mod app {
    use crate::{car::Car, remote_control::RemoteControl, servo::Servo};
    use adafruit_bluefruit_rs::BluefruitLEUARTFriend;
    use stm32f4xx_hal::{
        dma::{traits::StreamISR, Stream2},
        gpio::{Edge, Input, PA0, PA9},
        i2c::I2c,
        pac::{DMA2, IWDG, TIM3, TIM5, USART1},
        prelude::*,
        timer::{MonoTimerUs, PwmChannel},
        watchdog::IndependentWatchdog,
    };

    #[monotonic(binds = TIM5, default = true)]
    type MicrosecMono = MonoTimerUs<TIM5>;

    #[shared]
    struct Shared {
        remote_control: RemoteControl,
        car: Car<PwmChannel<TIM3, 0>>,
    }

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
        let bt_module = BluefruitLEUARTFriend::new(
            ctx.device.USART1,
            ctx.device.DMA2,
            gpiob.pb6,
            gpioa.pa10,
            &clocks,
        );
        let remote_control = RemoteControl::new(bt_module);

        // set up servo 1 & 2
        let (servo1_pwm, _servo2_pwm) = ctx
            .device
            .TIM3
            .pwm_hz(
                (gpioa.pa6.into_alternate(), gpioc.pc7.into_alternate()),
                50.Hz(),
                &clocks,
            )
            .split();
        // TODO: this is not 0 - 180°, change the code a bit to represent this
        // TODO: try to get a bit more out of it to get the maximum & document that this was found using trial & error
        let servo1 = Servo::new(servo1_pwm, 3500, 6000, 90);

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

        let car = Car::new(servo1);

        defmt::info!("init done");

        (
            Shared {
                remote_control,
                car,
            },
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

    #[task(binds = DMA2_STREAM2, shared = [remote_control, car])]
    fn bluetooth_dma_interrupt(mut ctx: bluetooth_dma_interrupt::Context) {
        defmt::debug!("received DMA2_STREAM2 interrupt (transfer complete)");
        if Stream2::<DMA2>::get_transfer_complete_flag() {
            ctx.shared.remote_control.lock(|remote_control| {
                ctx.shared.car.lock(|car| {
                    remote_control.handle_bluetooth_message(car);
                });
            });
        }
    }

    #[task(binds = USART1, shared = [remote_control, car])]
    fn bluetooth_receive_interrupt(mut ctx: bluetooth_receive_interrupt::Context) {
        defmt::debug!("received USART1 interrupt (IDLE)");
        ctx.shared.remote_control.lock(|remote_control| {
            ctx.shared.car.lock(|car| {
                remote_control.handle_bluetooth_message(car);
            });
        });

        unsafe {
            // taken 1:1 from serial::Rx::clear_idle_interrupt (don't have access to Rx here because it's in the transfer)
            // see https://github.com/stm32-rs/stm32f4xx-hal/issues/550 which will hopefully provide a proper solution
            let _ = (*USART1::ptr()).sr.read();
            let _ = (*USART1::ptr()).dr.read();
        }
    }
}
