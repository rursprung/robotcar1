# Overview
The robotcar is a small RC toy car based on the [YFRobot Steering Gear Robot](https://yfrobot.com/collections/robot-kits/products/steering-gear-robot),
using a [ST Nucleo F401RE](https://www.st.com/en/evaluation-tools/nucleo-f401re.html) board for the microcontroller,
a custom-designed PCB and a couple of add-ons (TOF sensor, bluetooth module, IMU and display).

The primary goal was to remotely control the toy car, with extended goals being that it has a simple collision avoidance
by stopping if an object blocks its path and additionally a somewhat-autonomous mode where it can try to circumvent an obstacle.
