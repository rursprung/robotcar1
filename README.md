# Robot Car
This is my first mobile robotics project (a 3rd semester module) of the [BSc in Mobile Robotics at FHGR](https://fhgr.ch/mr).

The documentation for it (besides the one in the code) can be found [online](https://rursprung.github.io/robotcar1/).

## Prerequisites
1. [Install Rust](https://www.rust-lang.org/tools/install)
2. Optional: ensure that the rust toolchain is up-to-date: `rustup update`
3. Install `probe-run`: `cargo install probe-run`
4. Install `flip-link`: `cargo install flip-link`
5. Install the cross-compile target: `rustup target add thumbv7em-none-eabihf`
6. Optional: install the LLVM tools: `rustup component add llvm-tools-preview`
7. Install the STLink drivers

## Build & Download to Board
1. Connect the board via USB
   * Note: if the board is connected to the custom PCB you must first power it on and _then_ connect it using USB -
     otherwise the board might not start correctly (this is a limitation of the ST board and unrelated to this software).
2. Run `cargo run` (the correct chip & target is already defined in `Cargo.toml` and `.cargo/config`)
3. Enjoy your running program :)
