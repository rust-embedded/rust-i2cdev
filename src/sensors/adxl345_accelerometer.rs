// Copyright 2015, Paul Osborne <osbpau@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/license/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option.  This file may not be copied, modified, or distributed
// except according to those terms.

use sensors::{Accelerometer, AccelerometerSample};
use core::{I2CDevice, I2CResult, I2CError};
use smbus::I2CSMBus;
use std::path::Path;
use std::io::Cursor;
use std::io::prelude::*;
use byteorder::{ByteOrder, LittleEndian, ReadBytesExt};

// TODO: read/write data format (for now, assumed 0x00)

pub const SLAVE_ADDR_PRIMARY: u16 = 0x1D;
pub const SLAVE_ADDR_ALT: u16 = 0x53;

const REGISTER_ADDR_DATA_FORMAT: u8 = 0x31;
const REGISTER_ADDR_X0: u8 = 0x32;

pub struct ADXL345Accelerometer {
    i2cdev: I2CDevice,
}

impl ADXL345Accelerometer {
    /// Create a new accelerometer handle for the given path/addr
    ///
    /// The `SLAVE_ADDR_*` constants from this module should be
    /// used to select either the primary or alternative slave
    /// address (dependent on `ALT ADDRESS` pin)
    pub fn new<P: AsRef<Path>>(path: P, slave_addr: u16)
                               -> I2CResult<ADXL345Accelerometer> {
        let i2cdev = try!(I2CDevice::new(path, slave_addr));
        Ok(ADXL345Accelerometer {
            i2cdev: i2cdev,
        })
    }
}

const ACCEL_RANGE: f32 = 2.0;  // +- 2G (with defaults)
const ACCEL_BITS: u8 = 10;  // 10-bit resolution

impl Accelerometer for ADXL345Accelerometer {
    fn accelerometer_sample(&mut self) -> I2CResult<AccelerometerSample> {
        // datasheet recommends multi-byte read to avoid reading
        // an inconsistent set of data
        let mut buf: [u8; 6] = [0u8; 6];
        try!(self.i2cdev.write(&[REGISTER_ADDR_X0])
             .or_else(|e| Err(I2CError::from(e))));
        try!(self.i2cdev.read(&mut buf)
             .or_else(|e| Err(I2CError::from(e))));

        let mut rdr = Cursor::new(&buf[..]);
        let scalar: f32 = (1 << ACCEL_BITS) as f32 * ACCEL_RANGE;
        Ok(AccelerometerSample {
            x: rdr.read_i16::<LittleEndian>().unwrap() as f32 * scalar,
            y: rdr.read_i16::<LittleEndian>().unwrap() as f32 * scalar,
            z: rdr.read_i16::<LittleEndian>().unwrap() as f32 * scalar,
        })
    }
}
