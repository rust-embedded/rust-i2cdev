// Copyright 2015, Paul Osborne <osbpau@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/license/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option.  This file may not be copied, modified, or distributed
// except according to those terms.

// Reads data from several different sensors all on the same i2c bus
// - Analog Devices ADXL345 Digital Accelerometer
// - Freescale MPL115A2 Digital Barometer/Thermometer
// - Bosch BNO055 9-Axis O Absolute Orientation Sensor

extern crate i2cdev;
extern crate docopt;

use std::thread;
use std::env::args;
use docopt::Docopt;
use i2cdev::sensors::{Thermometer, Barometer};
use i2cdev::sensors::mpl115a2_barometer::*;

const USAGE: &'static str = "
Reading Wii Nunchuck data via Linux i2cdev.

Usage:
  sensors <device>
  sensors (-h | --help)
  sensors --version

Options:
  -h --help    Show this help text.
  --version    Show version.
";


fn main() {
    let args = Docopt::new(USAGE)
        .and_then(|d| d.argv(args().into_iter()).parse())
        .unwrap_or_else(|e| e.exit());
    let device = args.get_str("<device>");
    let mut mpl115a2 = MPL115A2BarometerThermometer::new(device).unwrap();
    loop {
        println!("Temperature: {:?} C", mpl115a2.temperature_celsius());
        println!("Pressure:    {:?} kPa", mpl115a2.pressure_kpa());
        thread::sleep_ms(1000);
    }
}
