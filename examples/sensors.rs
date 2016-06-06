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
use i2cdev::sensors::{Thermometer, Barometer, Accelerometer};
use i2cdev::sensors::mpl115a2_barometer::*;
use i2cdev::sensors::adxl345_accelerometer::*;
use i2cdev::linux::*;

const USAGE: &'static str = "
Reading sensor data from a variety of sensors

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
    let mpl115a2_i2cdev = LinuxI2CDevice::new(device, MPL115A2_I2C_ADDR).unwrap();
    let adxl345_i2cdev = LinuxI2CDevice::new(device, 0x53).unwrap();

    let mut mpl115a2 = MPL115A2BarometerThermometer::new(mpl115a2_i2cdev).unwrap();
    let mut adxl345 = ADXL345Accelerometer::new(adxl345_i2cdev).unwrap();

    println!("== ADXL345 ID: 0x{:X} ==", adxl345.device_id().unwrap());

    loop {
        let accel = adxl345.accelerometer_sample().unwrap();
        println!("Temperature: {:?} C",
                 mpl115a2.temperature_celsius().unwrap());
        println!("Pressure:    {:?} kPa", mpl115a2.pressure_kpa().unwrap());
        println!("Accel:       {:?}", accel);
        println!("Accel Tot:   {:?}",
                 (accel.x.powi(2) + accel.y.powi(2) + accel.z.powi(2)).sqrt());
        thread::sleep_ms(1000);
    }
}
