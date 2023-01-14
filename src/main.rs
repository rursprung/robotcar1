#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

mod bt_module;
mod car;
mod remote_control;
mod steering;
mod tof_sensor;

use panic_probe as _;

use defmt_rtt as _;

pub use app::CarT;

#[rtic::app(device = stm32f4xx_hal::pac, dispatchers = [EXTI1])]
mod app {

    use crate::{
        bt_module::BluefruitLEUARTFriend,
        car::{Car, MAX_FRONT_DISTANCE_SENSOR_LAG_IN_MS},
        remote_control::RemoteControl,
        steering::Steering,
        tof_sensor::TOFSensor,
    };
    #[cfg(feature = "use-display")]
    use display_interface::DisplayError;
    #[cfg(feature = "use-display")]
    use ssd1306::I2CDisplayInterface;
    use ssd1306::{mode::BufferedGraphicsMode, prelude::*, Ssd1306};
    use stm32f4xx_hal::{
        dma::{traits::StreamISR, Stream2},
        gpio::{Edge, Input, PA0, PA9},
        i2c::I2c,
        pac::{DMA2, IWDG, TIM5},
        prelude::*,
        timer::MonoTimerUs,
        watchdog::IndependentWatchdog,
    };
    use stm32f4xx_hal::{
        gpio::{Output, PA8, PB4, PB5, PB8, PB9},
        i2c::{self, I2c1},
        pac::{TIM2, TIM3},
        timer::PwmChannel,
    };
    use tb6612fng::Motor;

    #[monotonic(binds = TIM5, default = true)]
    type MicrosecMono = MonoTimerUs<TIM5>;

    type I2C1 = I2c1<(PB8, PB9)>;
    type I2cProxy = shared_bus::I2cProxy<'static, shared_bus::AtomicCheckMutex<I2C1>>;
    pub type Display =
        Ssd1306<I2CInterface<I2cProxy>, DisplaySize128x64, BufferedGraphicsMode<DisplaySize128x64>>;
    pub type CarT = Car<
        PwmChannel<TIM3, 0>,
        PB5<Output>,
        PB4<Output>,
        PwmChannel<TIM2, 2>,
        TOFSensor<I2cProxy, i2c::Error>,
        vl53l1x_uld::Error<i2c::Error>,
        PA8<Output>,
    >;

    #[shared]
    struct Shared {
        remote_control: RemoteControl,
        car: crate::CarT,
    }

    #[local]
    struct Local {
        watchdog: IndependentWatchdog,
        button: PA9<Input>,
        tof_data_interrupt_pin: PA0<Input>,
    }

    #[init]
    fn init(mut ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        defmt::info!("booting system...");

        let mut syscfg = ctx.device.SYSCFG.constrain();

        let rcc = ctx.device.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(84.MHz()).freeze();
        let mono = ctx.device.TIM5.monotonic_us(&clocks);

        let gpioa = ctx.device.GPIOA.split();
        let gpiob = ctx.device.GPIOB.split();
        let gpioc = ctx.device.GPIOC.split();

        // Note: as a first step we just try to set up all peripherals here to let the compiler check if we made any mistakes.
        // This would e.g. fail when trying to use the same timer twice for two different PWMs or when using the wrong pins with the wrong PWM / I2C / etc.
        // This code will afterwards be moved / re-written when the actual functionality will be implemented.

        // set up the status LEDs
        let mut led_status_ok = gpioa.pa7.into_push_pull_output();
        let led_status_obstacle = gpioa.pa8.into_push_pull_output();

        // set up the user button
        let mut button = gpioa.pa9.into_pull_down_input();
        button.make_interrupt_source(&mut syscfg);
        button.enable_interrupt(&mut ctx.device.EXTI);
        button.trigger_on_edge(&mut ctx.device.EXTI, Edge::Falling);

        defmt::info!("LED & button setup done");

        // set up I2C
        let i2c = I2c::new(ctx.device.I2C1, (gpiob.pb8, gpiob.pb9), 400.kHz(), &clocks);
        #[cfg_attr(not(feature = "use-tof"), allow(unused))]
        let i2c = shared_bus::new_atomic_check!(I2C1 = i2c).unwrap();

        defmt::info!("I2C setup done");

        // the pin is always needed (unless we want to change the code even more to make this optional as well)
        #[cfg_attr(not(feature = "use-tof"), allow(unused_mut))]
        let mut tof_data_interrupt_pin = gpioa.pa0.into_pull_down_input();
        let tof_sensor;
        #[cfg(feature = "use-tof")]
        {
            // set up the interrupt for the TOF
            tof_data_interrupt_pin.make_interrupt_source(&mut syscfg);
            tof_data_interrupt_pin.enable_interrupt(&mut ctx.device.EXTI);
            tof_data_interrupt_pin.trigger_on_edge(&mut ctx.device.EXTI, Edge::Falling);

            tof_sensor =
                Some(TOFSensor::new(i2c.acquire_i2c()).expect("could initialise TOF sensor"));
            validate_distance::spawn_after((MAX_FRONT_DISTANCE_SENSOR_LAG_IN_MS + 1).millis()).ok();

            defmt::info!("TOF setup done");
        }
        #[cfg(not(feature = "use-tof"))]
        {
            tof_sensor = None;

            defmt::warn!("TOF setup SKIPPED (TOF not enabled)");
        }

        let display;
        #[cfg(feature = "use-display")]
        {
            display = setup_display(i2c.acquire_i2c()).map(Some).unwrap_or(None);

            defmt::info!("display setup done");
        }
        #[cfg(not(feature = "use-display"))]
        {
            display = None;

            defmt::warn!("display setup SKIPPED (display not enabled)");
        }

        // set up USART (for the bluetooth module)
        let bt_module = BluefruitLEUARTFriend::new(
            ctx.device.USART1,
            ctx.device.DMA2,
            gpiob.pb6,
            gpioa.pa10,
            &clocks,
        );
        let remote_control = RemoteControl::new(bt_module);

        defmt::info!("bluetooth setup done");

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

        defmt::info!("servo setup done");

        // set up the steering. PWM empirically determined.
        let steering_centre_pwm = 4930;
        let max_steering_side = 800;
        let steering = Steering::new(servo1_pwm, steering_centre_pwm, max_steering_side);

        defmt::info!("steering setup done");

        // set up motor A & B
        let motor_a_in1 = gpiob.pb5.into_push_pull_output();
        let motor_a_in2 = gpiob.pb4.into_push_pull_output();
        let _motor_b_in1 = gpioa.pa1.into_push_pull_output();
        let _motor_b_in2 = gpioa.pa4.into_push_pull_output();
        let (_motor_b_pwm, motor_a_pwm) = ctx
            .device
            .TIM2
            .pwm_hz(
                (gpiob.pb3.into_alternate(), gpiob.pb10.into_alternate()),
                100.kHz(),
                &clocks,
            )
            .split();
        let motor1 = Motor::new(motor_a_in1, motor_a_in2, motor_a_pwm);

        defmt::info!("motor setup done");

        let car = Car::new(steering, motor1, tof_sensor, display, led_status_obstacle);

        let watchdog = setup_watchdog(ctx.device.IWDG);

        defmt::info!("init done, watchdog started");

        // init is done, show this with the LED lighting up
        led_status_ok.set_high();

        (
            Shared {
                remote_control,
                car,
            },
            Local {
                watchdog,
                button,
                tof_data_interrupt_pin,
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

    #[cfg(feature = "use-display")]
    fn setup_display(i2c: I2cProxy) -> Result<Display, DisplayError> {
        let interface = I2CDisplayInterface::new_alternate_address(i2c); // our display runs on 0x3D, not 0x3C
        let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
            .into_buffered_graphics_mode();
        display.init()?;
        display.flush()?;
        Ok(display)
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
    #[task(binds = EXTI9_5, local = [button])]
    fn button_click(ctx: button_click::Context) {
        ctx.local.button.clear_interrupt_pending_bit();

        defmt::info!("button pressed");
    }

    // see here for why this is EXTI0: https://github.com/stm32-rs/stm32f4xx-hal/blob/6d0c29233a4cd1f780b2fef3e47ef091ead6cf4a/src/gpio/exti.rs#L8-L23
    /// Triggers every time the TOF has data (= new range measurement) available to be consumed.
    #[task(binds = EXTI0, local = [tof_data_interrupt_pin], shared = [car])]
    fn tof_interrupt_triggered(mut ctx: tof_interrupt_triggered::Context) {
        ctx.local
            .tof_data_interrupt_pin
            .clear_interrupt_pending_bit();
        ctx.shared.car.lock(|car| {
            car.handle_distance_sensor_interrupt(monotonics::now()).ok(); // error already logged in the function
        });
    }

    /// Ensure that we also react in case we don't get a new sensor value from the TOF
    #[task(priority = 1, shared = [car])]
    fn validate_distance(mut ctx: validate_distance::Context) {
        ctx.shared.car.lock(|car| {
            car.validate_distance(monotonics::now());
        });
        validate_distance::spawn_after((MAX_FRONT_DISTANCE_SENSOR_LAG_IN_MS + 1).millis()).ok();
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
    }
}
