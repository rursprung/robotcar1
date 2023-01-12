# Rust (Embedded) Specifics
For a general overview of Rust please refer to the awesome [Rust learning resources](https://www.rust-lang.org/learn),
and for more details on embedded development with Rust please refer to the awesome [Rust Embedded learning resources](https://www.rust-lang.org/what/embedded).
The same also goes for further information on [RTIC](https://rtic.rs/).

## Crates
Rust libraries are called crates and are often published on [crates.io](https://crates.io/) (but can also be pulled in from
other sources, e.g. local paths or git repositories).

[Cargo](https://doc.rust-lang.org/cargo/) is the package manager and build tool of Rust which also handles the crates
being used.

## Variable Ownership & Lifetimes
Rust has a strong concept of [ownership](https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html) of data.
References to data in turn is tracked with [lifetimes](https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html) for
the reference. The combination of these two concepts lets the compiler do strict validation of data flow to prevent many
of the known implementation errors which lead to hard-to-track problems (use-after-free, multiple threads accessing
the same variable without locking, etc.).

This sometimes leads to different implementations as one might be used to from casual C/C++ code which is less strict in
this regard.

## Embedded-Specific
### `no_std`: No Standard Library
"Normal" Rust programs are compiled against with backing support of a standard library backed by an OS. This standard
library provides a variety of useful features. Since we are running on bare metal, we cannot make use of this and thus
lose access to these nice features.

However, also in `no_std` certain features are still available: anything which isn't OS-specific and doesn't need a
memory allocator to work (i.e. the size is known at compile-time) is available in [`core`](https://doc.rust-lang.org/core/)
which is always present.

Optionally, an allocator can be added (if available / implemented for the used target) and in that case [`alloc`](https://doc.rust-lang.org/stable/core/alloc/)
can be used as well. This will provide all OS-independent but dynamically allocated APIs (e.g. string handling).

### Knurling-rs: Rust Embedded Improvement Project
[Knurling-rs](https://knurling.ferrous-systems.com/) is a project by the Rust community (mainly driven by
[Ferrous Systems](https://ferrous-systems.com/)) to improve the tooling for embedded development in Rust.
This has resulted in various tools which are also being used in this project here:

#### Logging: `defmt`
Logging is implemented using [`defmt`](https://defmt.ferrous-systems.com/). This is a deferred formatting logging framework:
the source code includes the whole log message, but at compile time this is split up:
* logging calls below the selected log level are removed
* the format strings are compiled into a table of string literals which is not part of the final program loaded on the device
* the device only knows the index of the format string and sends that plus the arguments to the listener

This way, the binary size is (drastically) reduced compared to having all the string handling in the binary.

`defmt` is widely supported in the Rust Embedded ecosystem, most crates 

#### Device Connection: `probe-run`
[`probe-run`](https://crates.io/crates/probe-run) supports downloading the application to the microcontroller, abstracting
away from the specific microcontroller and connection type (JLink, etc.). It also supports showing the log messages of
`defmt` when running an application.

It can easily be integrated with Cargo to support directly running the application with the standard `cargo run` command
(and the corresponding integration in IDEs).
