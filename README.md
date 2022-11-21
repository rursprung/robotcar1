# Robot Car
A project as part of my [BSc in Mobile Robotics at FHGR](https://fhgr.ch/mr) in the Mobile Roboticsproject 1 module.

## Prerequisites
1. [Install Rust](https://www.rust-lang.org/tools/install)
1. Optional: ensure that the rust toolchain is up-to-date: `rustup update`
1. Install `probe-run`: `cargo install probe-run`
1. Install `flip-link`: `cargo install flip-link`
1. Install the cross-compile target: `rustup target add thumbv7em-none-eabihf`
1. Optional: install the LLVM tools: `rustup component add llvm-tools-preview`
1. Install the STLink drivers

## Build & Download to Board
1. Connect the board via USB
2. Run `cargo run` (the correct chip & target is already defined in `Cargo.toml` and `.cargo/config`)
3. Enjoy your running program :)
