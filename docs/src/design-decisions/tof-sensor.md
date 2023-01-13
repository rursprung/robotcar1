# TOF Sensor
The standard TOF sensor provided for the project was the [ST VL53L3CX](https://www.st.com/en/imaging-and-photonics-solutions/vl53l3cx.html)
(specifically it was provided in the format of the [ST VL53L3CX-SATEL](https://www.st.com/en/evaluation-tools/vl53l3cx-satel.html)
breakout board).
There was no existing Rust driver available for this when starting the project. Rewriting the driver in Rust was not feasible
in the timeframe as the original C driver from ST contains thousands and thousands of lines of code with a lot of logic
(most of the logic for this sensor is not running on the sensor chip but is instead in the driver).

## Design Of C-based ST TOF Drivers
The C implementation of drivers for the TOF sensors provided by ST follow a common pattern:
* Platform-independent logic is implemented by them
* You have a platform-specific implementation which instantiates the driver and has a `struct` (which is then passed to the driver)
  which can afterwards be used to handle communication
* They provide a platform-specific header file with APIs which they call, and you have to implement it (e.g. I2C communication;
  the aforementioned custom, platform-specific `struct` is used for this)

## Using FFI (Foreign Function Include) To Use The C Driver
My first approach was to use [FFI](https://doc.rust-lang.org/book/ch19-01-unsafe-rust.html?highlight=ffi#using-extern-functions-to-call-external-code)
to be able to use the ST-provided driver and call the C code from Rust (to interact with it) and call Rust from C (to
handle the I2C communication).

While I was able to get the Rust to C and C to Rust calls working, the driver wouldn't start up completely due to wrong
data being read from the sensor. Debugging the I2C calls showed that most of them returned the exact same values as when
using the Arduino C++ code (it was faster to get that one running & debugged than the C-based driver, but their logic is
the same, and they're both from ST themselves). However, certain reads (always the same ones in each run) would just return
`0x00` instead of the expected value.

## Using An Alternative TOF Sensor
ST manufactures multiple different TOF sensors and offers them on the same breakout board format (i.e. they are pin
compatible). Namely, the [ST VL53L1X](https://www.st.com/en/imaging-and-photonics-solutions/vl53l1x.html) (with the
[ST VL53L1X-SATEL](https://www.st.com/en/evaluation-tools/vl53l1x-satel.html) breakout board) offers a similar ranging
distance (most others were too short for our use-case) and already [has a Rust driver](https://crates.io/crates/vl53l1x-uld)
(which is even fully implemented in Rust - this sensor runs more logic on-chip and thus has a smaller driver).

## Decision
Due to the amount of time already invested in trying to get the VL53L3CX driver running and the cheap cost of just getting
another TOF I went for the VL53L1X and used the existing driver, which worked well.
