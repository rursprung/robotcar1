# Software
The software is implemented in [Rust](https://www.rust-lang.org/) and uses [RTIC v1](https://rtic.rs/1/book/en/) to
provide RTOS-like features for interrupt/task handling.

Since Rust is not well known at FHGR (this might be the first project there using it?) some notes on it - esp. related
to embedded development pertaining to this project - have been collected in [an overview](rust-specifics.md).

## Software Architecture
As the STM32F4 is a resource-limited embedded device with a single core the application is implemented as a monolithic
single-threaded application, based primarily on hardware interrupts.

Due to the way RTIC is implemented, all RTIC tasks need to be in a single rust file which contains the
[`rtic::app`](https://rtic.rs/1/book/en/by-example/app.html). The software has been designed in such a way that (nearly)
all hardware-specific logic is either in device-specific drivers or in this main app.

The business logic in turn is largely separate from this, with clear APIs to be called on specific hardware / timer events.
This should - theoretically - allow to easily port the logic to a different microcontroller (or even a larger embedded system,
e.g. a Linux-based one) without having to change the business logic.