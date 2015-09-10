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
use std::io::prelude::*;
use std::env::args;
use docopt::Docopt;
use std::thread;
use std::io;
use std::path::Path;

const NUNCHUCK_SLAVE_ADDR: u16 = 0x52;

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

#[derive(Debug)]
struct NunchuckReading {
    joystick_x: u8,
    joystick_y: u8,
    accel_x: u16,  // 10-bit
    accel_y: u16,  // 10-bit
    accel_z: u16,  // 10-bit
    c_button_pressed: bool,
    z_button_pressed: bool,
}

impl NunchuckReading {
    fn from_data(data: &[u8; 6]) -> NunchuckReading {
        NunchuckReading {
            joystick_x: data[0],
            joystick_y: data[1],
            accel_x: (data[2] as u16) << 2 | ((data[5] as u16 >> 6) & 0b11),
            accel_y: (data[3] as u16) << 2 | ((data[5] as u16 >> 4) & 0b11),
            accel_z: (data[4] as u16) << 2 | ((data[5] as u16 >> 2) & 0b11),
            c_button_pressed: (data[5] & 0b10) == 0,
            z_button_pressed: (data[5] & 0b01) == 0,
        }
    }
}

struct Nunchuck {
    i2cdev: I2CDevice,
}

#[derive(Debug)]
enum NunchuckOpenError {
    InitError(NunchuckInitError),
    I2CError(I2CError),
}

#[derive(Debug)]
enum NunchuckInitError {
    WriteFailed,
}

#[derive(Debug)]
enum NunchuckReadError {
    IOError(io::Error),
}

impl Nunchuck {
    /// Create a new Wii Nunchuck
    ///
    /// This method will open the provide i2c device file and will
    /// send the required init sequence in order to read data in
    /// the future.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Nunchuck, NunchuckOpenError> {
        let i2cdev = try!(I2CDevice::new(path, NUNCHUCK_SLAVE_ADDR)
                          .or_else(|e| Err(NunchuckOpenError::I2CError(e))));
        let mut nunchuck = Nunchuck { i2cdev: i2cdev };
        try!(nunchuck.init().or_else(|e| Err(NunchuckOpenError::InitError(e))));
        Ok(nunchuck)
    }

    /// Send the init sequence to the Wii Nunchuck
    pub fn init(&mut self) -> Result<(), NunchuckInitError> {
        try!(self.i2cdev.write_all(&[0xF0, 0x55])
             .or_else(|_| Err(NunchuckInitError::WriteFailed)));
        try!(self.i2cdev.write_all(&[0xFB, 0x00])
             .or_else(|_| Err(NunchuckInitError::WriteFailed)));
        thread::sleep_ms(100);
        Ok(())
    }

    pub fn read(&mut self) -> Result<NunchuckReading, NunchuckReadError> {
        let mut buf: [u8; 6] = [0; 6];
        try!(self.i2cdev.write_all(&[0x00])
             .or_else(|e| Err(NunchuckReadError::IOError(e))));
        thread::sleep_ms(10);
        match self.i2cdev.read(&mut buf) {
            Ok(_len) => Ok(NunchuckReading::from_data(&buf)),
            Err(err) => Err(NunchuckReadError::IOError(err)),
        }
    }
}

fn main() {
    let args = Docopt::new(USAGE)
        .and_then(|d| d.argv(args().into_iter()).parse())
        .unwrap_or_else(|e| e.exit());
    let device = args.get_str("<device>");
    match Nunchuck::new(device) {
        Err(err) => { println!("Unable to open {:?}, {:?}", device, err); }
        Ok(mut nunchuck) => {
            loop {
                match nunchuck.read() {
                    Ok(reading) => println!("{:?}", reading),
                    Err(err)    => println!("Error: {:?}", err),
                };
            }
        },
    }
}
