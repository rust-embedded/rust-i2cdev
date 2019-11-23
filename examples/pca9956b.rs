// Copyright 2018, Piers Finlayson <piers@piersandkatie.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/license/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option.  This file may not be copied, modified, or distributed
// except according to those terms.extern crate i2cdev;

extern crate docopt;
extern crate i2cdev;

#[cfg(any(target_os = "linux", target_os = "android"))]
use i2cdev::core::{I2CMessage, I2CTransfer};
#[cfg(any(target_os = "linux", target_os = "android"))]
use i2cdev::linux::{LinuxI2CBus, LinuxI2CMessage};

use docopt::Docopt;
use std::env::args;

const USAGE: &str = "
Reads registers from a PCA9956B IC via Linux i2cdev.

Assumes the PCA9956B is using address 0x20.

Usage:
  pca9956b <device>
  pca9956b (-h | --help)
  pca9956b --version

Options:
  -h --help    Show this help text.
  --version    Show version.
";

const ADDR: u16 = 0x20;

#[cfg(not(any(target_os = "linux", target_os = "android")))]
fn main() {}

#[cfg(any(target_os = "linux", target_os = "android"))]
fn main() {
    let args = Docopt::new(USAGE)
        .and_then(|d| d.argv(args()).parse())
        .unwrap_or_else(|e| e.exit());
    let path = args.get_str("<device>");
    let mut bus = match LinuxI2CBus::new(path) {
        Ok(bus) => bus,
        Err(_e) => {
            println!("Error opening I2C Bus {} {}", path, _e);
            return;
        }
    };
    println!("Opened I2C Bus OK: {}", path);

    // Build two I2C messages:
    // 1) Write the MODE1 register address, with top bit indcating auto-
    //    increment should be enabled
    // 2) Read 10 bytes from the current register onwards
    let mut data = [0; 10];
    let mut msgs = [
        LinuxI2CMessage::write(&[0b1000_0000]).with_address(ADDR),
        LinuxI2CMessage::read(&mut data).with_address(ADDR),
    ];

    // Send the messages to the kernel to process
    match bus.transfer(&mut msgs) {
        Ok(rc) => println!("Successful transfer call: {} messages processed", rc),
        Err(_e) => {
            println!("Error reading/writing {}", _e);
            return;
        }
    }

    // Print the data read from the device.  A recently reset PCA9956B should
    // return:
    // 0x8005000000000000ff00
    let mut output = "Result: 0x".to_string();
    for byte in &data {
        output = format!("{}{:02x}", output, byte);
    }
    println!("{}", output);
}
