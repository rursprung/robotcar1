# User Guide

## Requirements
What you'll need in order to use this robotcar:
* The robotcar
  * Make sure to have charged its batteries before use!
* A smartphone, tablet or other device with a compatible app (see [the from Adafruit](https://learn.adafruit.com/introducing-the-adafruit-bluefruit-le-uart-friend/software-resources))
* Space where you can drive the robot car around!

## Prepare The Hardware
To get started follow these steps:
1. Insert the batteries (beware the polarities! Supported are ca. 1.5V AA batteries) into the battery holder
2. Insert the battery holder in the chassis (between the axles)
3. Connect the battery holder to the car using the XD30 power connector
4. Turn on the car using the switch on its rear

## Connect With The App
1. Open the app (e.g. Adafruit Bluefruit LE Connect on your phone)
2. Connect with the app to the car
3. Open the control pad (under "Controller" on the Android app)

## Drive!
The following control commands are available:
* Steering left/right with the left/right arrow keys
* Increasing & decreasing the speed using the up/down arrow keys (increase/decrease speed in 25% steps, ranging from
  full forward to full backwards speed)
* Brake and set speed to 0 with the "1" key 

The other keys are not assigned.

The car will automatically brake when you get too close to an obstacle in front. You'll still be able to reverse and steer
at that moment, until the distance in front is large enough and you can drive forward again.
If the car has detected an obstacle a red LED will turn on to indicate this. Once the obstacle has been cleared, the LED
will turn off.

The display on the device will show the distance (in mm) to a potential obstacle in front of the car.
