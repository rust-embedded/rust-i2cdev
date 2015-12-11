// Copyright 2015, Paul Osborne <osbpau@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/license/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option.  This file may not be copied, modified, or distributed
// except according to those terms.

// Reads data from Wii Nunchuck

extern crate i2cdev;
extern crate docopt;

use i2cdev::core::*;
use i2cdev::linux::*;
use i2cdev::sensors::nunchuck::*;
use std::env::args;
use docopt::Docopt;


const USAGE: &'static str = "
Reading Wii Nunchuck data via Linux i2cdev.

Usage:
  nunchuck <device>
  nunchuck (-h | --help)
  nunchuck --version

Options:
  -h --help    Show this help text.
  --version    Show version.
";


fn main() {
    let args = Docopt::new(USAGE)
                   .and_then(|d| d.argv(args().into_iter()).parse())
                   .unwrap_or_else(|e| e.exit());
    let device = args.get_str("<device>");
    let i2cdev = LinuxI2CDevice::new(device, NUNCHUCK_SLAVE_ADDR).unwrap();
    match Nunchuck::new(i2cdev) {
        Err(err) => {
            println!("Unable to open {:?}, {:?}", device, err);
        }
        Ok(mut nunchuck) => {
            loop {
                match nunchuck.read() {
                    Ok(reading) => println!("{:?}", reading),
                    Err(err) => println!("Error: {:?}", err),
                };
            }
        }
    }
}
