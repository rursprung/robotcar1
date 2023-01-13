# Software
The software is implemented in [Rust](https://www.rust-lang.org/) and uses [RTIC v1](https://rtic.rs/1/book/en/) to
provide RTOS-like features for interrupt/task handling.

Since Rust is not well known at FHGR (this might be the first project there using it?) some notes on it - esp. related
to embedded development pertaining to this project - have been collected in [an overview](rust-specifics.md).

The design decisions taken for the software have been listed separately, see [design decisions](./design-decisions/index.md).

## Software Architecture
As the STM32F4 is a resource-limited embedded device with a single core the application is implemented as a monolithic
single-threaded application, based primarily on hardware interrupts.

Due to the way RTIC is implemented, all RTIC tasks need to be in a single rust file which contains the
[`rtic::app`](https://rtic.rs/1/book/en/by-example/app.html). The software has been designed in such a way that (nearly)
all hardware-specific logic is either in device-specific drivers or in this main app.

The business logic in turn is largely separate from this, with clear APIs to be called on specific hardware / timer events.
This should - theoretically - allow to easily port the logic to a different microcontroller (or even a larger embedded system,
e.g. a Linux-based one) without having to change the business logic.

The logic has been split so that there's a general `Car` representation (which doesn't know how it'll be operated) and
a separate `RemoteControl` (which is aware of the car and can direct it). The `Car` API is hardware-agnostic, i.e.
its consumers do not have to be aware of the fact that its steering is implemented using a PWM-controlled servo motor.

## Drivers for Peripherals
The following drivers have been used for the peripherals:

| Peripheral                                                                                                              | Driver                                              | Comment                                                                                                                                                                                                        |
|-------------------------------------------------------------------------------------------------------------------------|-----------------------------------------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| TOF Sensor ([ST VL53L1X](https://www.st.com/en/imaging-and-photonics-solutions/vl53l1x.html))                           | [vl53l1x-uld](https://crates.io/crates/vl53l1x-uld) ||
| Display ([Adafruit 128x64 OLED Display](https://www.adafruit.com/product/326))                                          | [ssd1306](https://crates.io/crates/ssd1306)         ||
| IMU ([Adafruit MPU6050](https://learn.adafruit.com/mpu6050-6-dof-accelerometer-and-gyro))                               | [mpu6050](https://crates.io/crates/mpu6050)         | Currently unused, thus not included in the code.                                                                                                                                                               |
| BLE ([Adafruit Bluefruit LE UART Friend](https://learn.adafruit.com/introducing-the-adafruit-bluefruit-le-uart-friend)) | n/a                                                 | Uses basic UART in our use-case, thus no dedicated driver needed. Protocol support implemented as part of this project in [adafruit-bluefruit-protocol](https://crates.io/crates/adafruit-bluefruit-protocol). |
| Motor Driver ([SparkFun Motor Driver - Dual TB6612FNG](https://www.sparkfun.com/products/14450))                        | [tb6612fng](https://crates.io/crates/tb6612fng)     | Implemented as part of this project.                                                                                                                                                                           |

## Compiling & Running It
Please refer to the README located in the repository root for the necessary steps to compile & run the program on the
target device.
