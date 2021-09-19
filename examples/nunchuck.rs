// Copyright 2015, Paul Osborne <osbpau@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/license/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option.  This file may not be copied, modified, or distributed
// except according to those terms.

// Reads data from Wii Nunchuck
//
// NOTE: This code is provided as an example.  Driver developers are encouraged
// to use the embedded-hal traits if possible rather than coupling directly
// to this library.  An implementation of the embedded-hal I2C traits based
// on this library may be found in the embedded-hal-linux project.

extern crate docopt;
extern crate i2cdev;

#[cfg(any(target_os = "linux", target_os = "android"))]
use i2cdev::linux::*;

#[cfg(any(target_os = "linux", taret_os = "android"))]
mod nunchuck {
    use std::error::Error;
    use std::fmt;
    use std::thread;
    use std::time::Duration;

    use i2cdev::core::I2CDevice;

    pub const NUNCHUCK_SLAVE_ADDR: u16 = 0x52;

    #[derive(Debug)]
    pub enum NunchuckError<E> {
        Error(E),
        ParseError,
    }

    impl<E: Error> fmt::Display for NunchuckError<E> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match *self {
                NunchuckError::Error(ref e) => fmt::Display::fmt(e, f),
                NunchuckError::ParseError => write!(f, "Could not parse data"),
            }
        }
    }

    impl<E: Error> Error for NunchuckError<E> {
        fn cause(&self) -> Option<&dyn Error> {
            match *self {
                NunchuckError::Error(ref e) => Some(e),
                NunchuckError::ParseError => None,
            }
        }
    }

    // TODO: Move Nunchuck code out to be an actual sensor and add tests

    #[derive(Debug)]
    pub struct NunchuckReading {
        joystick_x: u8,
        joystick_y: u8,
        accel_x: u16, // 10-bit
        accel_y: u16, // 10-bit
        accel_z: u16, // 10-bit
        c_button_pressed: bool,
        z_button_pressed: bool,
    }

    impl NunchuckReading {
        pub fn from_data(data: &[u8]) -> Option<NunchuckReading> {
            if data.len() < 6 {
                None
            } else {
                Some(NunchuckReading {
                    joystick_x: data[0],
                    joystick_y: data[1],
                    accel_x: (u16::from(data[2]) << 2) | ((u16::from(data[5]) >> 6) & 0b11),
                    accel_y: (u16::from(data[3]) << 2) | ((u16::from(data[5]) >> 4) & 0b11),
                    accel_z: (u16::from(data[4]) << 2) | ((u16::from(data[5]) >> 2) & 0b11),
                    c_button_pressed: (data[5] & 0b10) == 0,
                    z_button_pressed: (data[5] & 0b01) == 0,
                })
            }
        }
    }

    pub struct Nunchuck<T: I2CDevice> {
        i2cdev: T,
    }

    impl<T> Nunchuck<T>
    where
        T: I2CDevice,
    {
        /// Create a new Wii Nunchuck
        ///
        /// This method will open the provide i2c device file and will
        /// send the required init sequence in order to read data in
        /// the future.
        pub fn new(i2cdev: T) -> Result<Nunchuck<T>, T::Error> {
            let mut nunchuck = Nunchuck { i2cdev };
            nunchuck.init()?;
            Ok(nunchuck)
        }

        #[cfg(test)]
        pub fn get_i2cdev(&mut self) -> &mut T {
            &mut self.i2cdev
        }

        /// Send the init sequence to the Wii Nunchuck
        pub fn init(&mut self) -> Result<(), T::Error> {
            // These registers must be written; the documentation is a bit
            // lacking but it appears this is some kind of handshake to
            // perform unencrypted data tranfers
            self.i2cdev.smbus_write_byte_data(0xF0, 0x55)?;
            self.i2cdev.smbus_write_byte_data(0xFB, 0x00)
        }

        pub fn read(&mut self) -> Result<NunchuckReading, NunchuckError<T::Error>> {
            let mut buf: [u8; 6] = [0; 6];

            // tell the nunchuck to prepare a sample
            self.i2cdev
                .smbus_write_byte(0x00)
                .map_err(NunchuckError::Error)?;

            // now, read it!
            thread::sleep(Duration::from_millis(10));
            self.i2cdev.read(&mut buf).map_err(NunchuckError::Error)?;
            NunchuckReading::from_data(&buf).ok_or(NunchuckError::ParseError)
        }
    }

    #[cfg(test)]
    mod test {
        use super::*;
        use i2cdev::core::I2CDevice;
        use i2cdev::mock::MockI2CDevice;

        #[test]
        fn test_intialization() {
            // write out some "bad" values to start out with so we know the
            // write happens
            let mut i2cdev = MockI2CDevice::new();
            i2cdev.smbus_write_byte_data(0xF0, 0xFF).unwrap();
            i2cdev.smbus_write_byte_data(0xFB, 0xFF).unwrap();

            // these values must be written out for things to work
            let mut dev = Nunchuck::new(i2cdev).unwrap();
            assert_eq!(dev.get_i2cdev().smbus_read_byte_data(0xF0).unwrap(), 0x55);
            assert_eq!(dev.get_i2cdev().smbus_read_byte_data(0xFB).unwrap(), 0x00);
        }

        #[test]
        fn test_read_zeroed_out() {
            let mut dev = Nunchuck::new(MockI2CDevice::new()).unwrap();
            let reading = dev.read().unwrap();
            assert_eq!(reading.joystick_x, 0);
            assert_eq!(reading.joystick_y, 0);
            assert_eq!(reading.accel_x, 0);
            assert_eq!(reading.accel_y, 0);
            assert_eq!(reading.accel_z, 0);
            assert_eq!(reading.c_button_pressed, true);
            assert_eq!(reading.z_button_pressed, true);
        }

        #[test]
        fn test_read_sample_data() {
            let mut i2cdev = MockI2CDevice::new();
            i2cdev.write(&[0, 127, 128, 191, 129, 144, 71]).unwrap();
            let mut dev = Nunchuck::new(i2cdev).unwrap();
            let reading = dev.read().unwrap();
            assert_eq!(reading.joystick_x, 127);
            assert_eq!(reading.joystick_y, 128);
            assert_eq!(reading.accel_x, 765);
            assert_eq!(reading.accel_y, 516);
            assert_eq!(reading.accel_z, 577);
            assert_eq!(reading.c_button_pressed, false);
            assert_eq!(reading.z_button_pressed, false);
        }
    }
}

#[cfg(any(target_os = "linux", target_os = "android"))]
use nunchuck::*;

use docopt::Docopt;
use std::env::args;

const USAGE: &str = "
Reading Wii Nunchuck data via Linux i2cdev.

Usage:
  nunchuck <device>
  nunchuck (-h | --help)
  nunchuck --version

Options:
  -h --help    Show this help text.
  --version    Show version.
";

#[cfg(not(any(target_os = "linux", target_os = "android")))]
fn main() {}

#[cfg(any(target_os = "linux", target_os = "android"))]
fn main() {
    let args = Docopt::new(USAGE)
        .and_then(|d| d.argv(args()).parse())
        .unwrap_or_else(|e| e.exit());
    let device = args.get_str("<device>");
    let i2cdev = LinuxI2CDevice::new(device, NUNCHUCK_SLAVE_ADDR).unwrap();
    match Nunchuck::new(i2cdev) {
        Err(err) => {
            println!("Unable to open {:?}, {:?}", device, err);
        }
        Ok(mut nunchuck) => loop {
            match nunchuck.read() {
                Ok(reading) => println!("{:?}", reading),
                Err(err) => println!("Error: {:?}", err),
            };
        },
    }
}
