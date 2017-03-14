// Copyright 2015, Paul Osborne <osbpau@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/license/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option.  This file may not be copied, modified, or distributed
// except according to those terms.

#![allow(dead_code)]

use sensors::{Thermometer, Barometer};
use std::thread;
use std::time::Duration;
use std::error::Error;
use core::I2CDevice;
use byteorder::{ByteOrder, BigEndian};

pub const BMP180_I2C_ADDR: u16 = 0x77;

pub const BMP180_REGISTER_CTL: u8 = 0xF4;
pub const BMP180_CMD_TEMP: u8 = 0x2E;
pub const BMP180_REGISTER_TEMP_MSB: u8 = 0xF6;
pub const BMP180_CMD_PRESSURE: u8 = 0x34;
pub const BMP180_REGISTER_PRESSURE_MSB: u8 = 0xF6;
pub const BMP180_REGISTER_AC1MSB: u8 = 0xaa;

#[derive(Copy,Clone)]
pub enum BMP180PressureMode {
    BMP180UltraLowPower,
    BMP180Standard,
    BMP180HighResolution,
    BMP180UltraHighResolution,
}

impl BMP180PressureMode {
    pub fn get_mode_value(self) -> u8 {
        match self {
            BMP180PressureMode::BMP180UltraLowPower => 0,
            BMP180PressureMode::BMP180Standard => 1,
            BMP180PressureMode::BMP180HighResolution => 2,
            BMP180PressureMode::BMP180UltraHighResolution => 3,
        }
    }
    pub fn mode_delay(self) -> Duration {
        match self {
            BMP180PressureMode::BMP180UltraLowPower => Duration::from_millis(5),
            BMP180PressureMode::BMP180Standard => Duration::from_millis(8),
            BMP180PressureMode::BMP180HighResolution => Duration::from_millis(14),
            BMP180PressureMode::BMP180UltraHighResolution => Duration::from_millis(26),
        }
    }
}
#[derive(Copy,Clone)]
pub struct BMP180CalibrationCoefficients {
    ac1: i16,
    ac2: i16,
    ac3: i16,
    ac4: u16,
    ac5: u16,
    ac6: u16,
    b1: i16,
    b2: i16,
    mb: i16,
    mc: i16,
    md: i16,
}

#[derive(Debug)]
pub struct BMP180RawReading {
    padc: u32, // 10-bit pressure ADC output value
    tadc: i16, // 10-bit pressure ADC output value
}
#[derive(Copy,Clone)]
pub struct BMP180BarometerThermometer<T: I2CDevice + Sized> {
    pub i2cdev: T,
    pub coeff: BMP180CalibrationCoefficients,
    pub pressure_precision: BMP180PressureMode,
}


impl<T> BMP180BarometerThermometer<T>
    where T: I2CDevice + Sized
{
    /// Create sensor accessor for MPL115A2 on the provided i2c bus path
    pub fn new(mut i2cdev: T, pressure_precision: BMP180PressureMode) -> Result<BMP180BarometerThermometer<T>, T::Error> {
        let coeff = try!(BMP180CalibrationCoefficients::new(&mut i2cdev));
        Ok(BMP180BarometerThermometer {
            i2cdev: i2cdev,
            coeff: coeff,
            pressure_precision: pressure_precision,
        })
    }
}

impl BMP180RawReading {
    /// Create a new reading from the provided I2C Device
    pub fn new<E: Error>(i2cdev: &mut I2CDevice<Error = E>, mode: BMP180PressureMode) -> Result<BMP180RawReading, E> {
        // fist we need read temp needed for further pressure calculations
        try!(i2cdev.smbus_write_byte_data(BMP180_REGISTER_CTL, BMP180_CMD_TEMP));

        // maximum conversion time is 5ms
        thread::sleep(Duration::from_millis(5));

        // i2c gets LittleEndian we need BigEndian
        let mut buf = [0_u8; 2];
        try!(i2cdev.write(&[BMP180_REGISTER_TEMP_MSB]));
        try!(i2cdev.read(&mut buf));
        // let padc: u16 = BigEndian::read_u16(&buf) >> 6;
        let tadc: i16 = BigEndian::read_i16(&buf[..]);
        // now lets get pressure
        let offset = mode.get_mode_value();
        let delay = mode.mode_delay();
        try!(i2cdev.smbus_write_byte_data(BMP180_REGISTER_CTL, BMP180_CMD_PRESSURE + offset));
        thread::sleep(delay);
        let mut p_buf = [0_u8; 3];
        try!(i2cdev.write(&[BMP180_REGISTER_PRESSURE_MSB]));
        try!(i2cdev.read(&mut p_buf));
        let padc: u32 = ((p_buf[0] as u32) << 16) + ((p_buf[1] as u32) << 8) + (p_buf[2] as u32) >> (8 - (mode as u8));
        Ok(BMP180RawReading {
            padc: padc,
            tadc: tadc,
        })
    }
}


impl BMP180CalibrationCoefficients {
    /// Convert a slice of data values of length 8 to coefficients
    ///
    /// This should be built from a read of registers 0x04-0x0B in
    /// order.  This gets the raw, unconverted value of each
    /// coefficient.
    pub fn new<E: Error>(i2cdev: &mut I2CDevice<Error = E>) -> Result<BMP180CalibrationCoefficients, E> {
        let mut buf: [u8; 22] = [0; 22];
        try!(i2cdev.write(&[BMP180_REGISTER_AC1MSB]));
        try!(i2cdev.read(&mut buf));
        // unimplemented!();
        Ok(BMP180CalibrationCoefficients {
            ac1: BigEndian::read_i16(&buf[0..2]),
            ac2: BigEndian::read_i16(&buf[2..4]),
            ac3: BigEndian::read_i16(&buf[4..6]),
            ac4: BigEndian::read_u16(&buf[6..8]),
            ac5: BigEndian::read_u16(&buf[8..10]),
            ac6: BigEndian::read_u16(&buf[10..12]),
            b1: BigEndian::read_i16(&buf[12..14]),
            b2: BigEndian::read_i16(&buf[14..16]),
            mb: BigEndian::read_i16(&buf[16..18]),
            mc: BigEndian::read_i16(&buf[18..20]),
            md: BigEndian::read_i16(&buf[20..22]),
        })
    }
    fn calculate_b5(self, raw_temp: i16) -> i32 {
        let x1 = (((raw_temp as i32) - (self.ac6 as i32)) * (self.ac5 as i32)) >> 15;
        let x2 = ((self.mc as i32) << 11) / (x1 + (self.md as i32));
        x1 + x2
    }
}

impl<T> Thermometer for BMP180BarometerThermometer<T>
    where T: I2CDevice + Sized
{
    type Error = T::Error;

    fn temperature_celsius(&mut self) -> Result<f32, T::Error> {
        let reading = try!(BMP180RawReading::new(&mut self.i2cdev, self.pressure_precision));
        let b5 = self.coeff.calculate_b5(reading.tadc);
        let t = (b5 + 8) >> 4;
        Ok((t as f32) / 10_f32)
    }
}
