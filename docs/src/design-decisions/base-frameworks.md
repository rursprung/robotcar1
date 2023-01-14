# Base Frameworks
There were several options of frameworks available to implement this project.

## Overview of the Possibilities Analysed
### No Framework
It is of course possible to just use the HAL and implement the functionality directly like this. However, when working
with interrupts, this complicates things as one has to manually take care of avoiding race conditions and managing
resource sharing.

### [RTIC](https://rtic.rs/)
RTIC is a concurrency framework to take care of resource sharing and task priorities when working with interrupts (hardware
and software tasks are supported). RTIC v1 does not support `async` Rust code.

### [Embassy](https://embassy.dev/)
Embassy brings `async` support to embedded Rust by providing an executor. It also brings its own HAL to offer a more
streamlined abstraction over different devices and to have an `async`-enabled HAL (the HAL can theoretically be used without
the executor and vice versa).

At the moment (January 2023) Embassy does not yet have released crates (git dependencies have to be used instead,
[a v0.1 release is being worked to](https://github.com/embassy-rs/embassy/issues/1050)) and requires a nightly rust compiler
(due to the usage of some language features which have not yet been stabilised).

## Decision
Due to the need for the use of interrupts not using a framework was not an option (seeing that frameworks are available).
While Embassy looks very promising, the lack of a stable release and the requirement for nightly Rust it however felt
premature to use it for this project. This led to the decision for RTIC.

Note that no board support package has been used for the Nucleo F401RE because none of the board-specific features is
being used (and also, the available [nucleo-f401re](https://crates.io/crates/nucleo-f401re) crate wouldn't offer too much
in terms of additional features besides access to the LED and button (neither of which is accessible with the board mounted
underneath the PCB)).
