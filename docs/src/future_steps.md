# Future Steps
This page lists things which should or could still be implemented for this robot car.

## Fix Intermittent Hangs
It currently happens that the application crashes and is stuck. It is unclear why this happens (lack of time to debug it)
and needs further investigation. Usually, power-cycling helps (which also fully resets the peripherals, compared to just
resetting the microcontroller).

## Use IMU For Braking Distance
Currently, the automatic braking collision avoidance uses a fixed distance at which it will engage. Instead of this,
the IMU could be used to acquire the current velocity (by integrating over the acceleration) which in turn could be
used to calculate the minimum safe distance needed to come to a full stop.

## Implement Simple Autonomous Mode
An alternative to just braking as a collision avoidance would be to try and circumnavigate the obstacle (presuming that
this is possible for the obstacle). As the TOF sensor is mounted in a fixed forward-facing position this would probably
need some jiggling motion of the car to occasionally turn slightly and check if the obstacle is still on its side.

A physical user button and two LEDs are available on the PCB and currently unused. These could be used to start
the autonomous mode and indicate the current status (besides showing information on the display).

## Use a Memory Allocator
Currently, the software does not use a memory allocator. It would be possible to add [`embedded-alloc`](https://crates.io/crates/embedded-alloc)
which would then e.g. allow using string formatting at runtime for the messages on the display.

Also, the [adafruit-bluefruit-protocol](https://crates.io/crates/adafruit-bluefruit-protocol) could then use the
[`alloc::Vec`](https://doc.rust-lang.org/alloc/vec/) instead of the non-alloc `Vec` from [`heapless`](https://crates.io/crates/heapless).
