// Copyright 2015, Paul Osborne <osbpau@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/license/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option.  This file may not be copied, modified, or distributed
// except according to those terms.

#![allow(dead_code)] // register map

use sensors::{Accelerometer, AccelerometerSample};
use core::I2CDevice;
use byteorder::{ByteOrder, LittleEndian, ReadBytesExt};

// TODO: read/write data format (for now, assumed 0x00)

pub const SLAVE_ADDR_PRIMARY: u16 = 0x1D;
pub const SLAVE_ADDR_ALT: u16 = 0x53;

const REGISTER_DEVID: u8 = 0x00;
const REGSITER_THRESH_TAP: u8 = 0x1D;
const REGISTER_OFSX: u8 = 0x1E;
const REGISTER_OFSY: u8 = 0x1F;
const REGISTER_OFSZ: u8 = 0x20;
const REGISTER_DUR: u8 = 0x21;
const REGISTER_LATENT: u8 = 0x22;
const REGISTER_WINDOW: u8 = 0x23;
const REGISTER_THRESH_ACT: u8 = 0x24;
const REGISTER_THRESH_INACT: u8 = 0x25;
const REGISTER_TIME_INACT: u8 = 0x26;
const REGISTER_ACT_INACT_CTL: u8 = 0x27;
const REGISTER_THRESH_FF: u8 = 0x28;
const REGISTER_TIME_FF: u8 = 0x29;
const REGISTER_TAP_AXES: u8 = 0x2A;
const REGISTER_ACT_TAP_STATUS: u8 = 0x2B;
const REGISTER_BW_RATE: u8 = 0x2C;
const REGISTER_POWER_CTL: u8 = 0x2D;
const REGISTER_INT_ENABLE: u8 = 0x2E;
const REGISTER_INT_MAP: u8 = 0x2F;
const REGISTER_INT_SOURCE: u8 = 0x30;
const REGISTER_DATA_FORMAT: u8 = 0x31;
const REGISTER_X0: u8 = 0x32;
const REGISTER_X1: u8 = 0x33;
const REGISTER_Y0: u8 = 0x34;
const REGISTER_Y1: u8 = 0x35;
const REGISTER_Z0: u8 = 0x36;
const REGISTER_Z1: u8 = 0x37;
const REGISTER_FIFO_CTL: u8 = 0x38;
const REGISTER_FIFO_STATUS: u8 = 0x39;

#[repr(u8)]
#[allow(non_camel_case_types)]
enum ADXL345DataRate {
    RATE_3200HZ = 0x0F,
    RATE_1600HZ = 0x0E,
    RATE_800HZ = 0x0D,
    RATE_400HZ = 0x0C,
    RATE_200HZ = 0x0B,
    RATE_100HZ = 0x0A,
    RATE_50HZ = 0x09,
    RATE_25HZ = 0x08,
    RATE_12HZ5 = 0x07,
    RATE_6HZ25 = 0x06,
}

pub struct ADXL345Accelerometer<T: I2CDevice + Sized> {
    i2cdev: T,
}

impl<T> ADXL345Accelerometer<T> where T: I2CDevice + Sized {
    /// Create a new accelerometer handle for the given path/addr
    ///
    /// The `SLAVE_ADDR_*` constants from this module should be
    /// used to select either the primary or alternative slave
    /// address (dependent on `ALT ADDRESS` pin)
    pub fn new(mut i2cdev: T) -> Result<ADXL345Accelerometer<T>, T::Error> {
        // setup standy mode to configure
        try!(i2cdev.smbus_write_byte_data(REGISTER_POWER_CTL, 0x00));

        // configure some defaults
        try!(i2cdev.smbus_write_byte_data(REGISTER_BW_RATE, ADXL345DataRate::RATE_1600HZ as u8));
        try!(i2cdev.smbus_write_byte_data(REGISTER_DATA_FORMAT, 0x08));
        try!(i2cdev.smbus_write_byte_data(REGISTER_OFSX, 0xFD));
        try!(i2cdev.smbus_write_byte_data(REGISTER_OFSY, 0x03));
        try!(i2cdev.smbus_write_byte_data(REGISTER_OFSZ, 0xFE));

        // put device in measurement mode
        try!(i2cdev.smbus_write_byte_data(REGISTER_POWER_CTL, 0x08));

        Ok(ADXL345Accelerometer { i2cdev: i2cdev })
    }

    /// Get the device id
    pub fn device_id(&mut self) -> Result<u8, T::Error> {
        self.i2cdev.smbus_read_byte_data(REGISTER_DEVID)
    }
}

const ACCEL_RANGE: f32 = 2.0;  // +- 2G (with defaults)
const ACCEL_BITS: u8 = 10;  // 10-bit resolution

impl<T> Accelerometer for ADXL345Accelerometer<T> where T: I2CDevice + Sized {
    type Error = T::Error;

    fn accelerometer_sample(&mut self) -> Result<AccelerometerSample, T::Error> {
        // datasheet recommends multi-byte read to avoid reading
        // an inconsistent set of data
        let mut buf: [u8; 6] = [0u8; 6];
        try!(self.i2cdev.write(&[REGISTER_X0]));
        try!(self.i2cdev.read(&mut buf));

        let x: i16 = LittleEndian::read_i16(&[buf[0], buf[1]]);
        let y: i16 = LittleEndian::read_i16(&[buf[2], buf[3]]);
        let z: i16 = LittleEndian::read_i16(&[buf[4], buf[5]]);
        Ok(AccelerometerSample {
            x: (x as f32 / 1023.0) * (ACCEL_RANGE * 2.0),
            y: (y as f32 / 1023.0) * (ACCEL_RANGE * 2.0),
            z: (z as f32 / 1023.0) * (ACCEL_RANGE * 2.0),
        })
    }
}
