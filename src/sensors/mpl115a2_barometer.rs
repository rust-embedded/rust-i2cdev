// Copyright 2015, Paul Osborne <osbpau@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/license/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option.  This file may not be copied, modified, or distributed
// except according to those terms.

use sensors::Thermometer;
use sensors::Barometer;
use std::thread;
use std::io::prelude::*;
use core::{I2CDevice, I2CResult, I2CError};
use smbus::I2CSMBus;
use std::path::Path;

const SLAVE_ADDR: u16 = 0x60; // appears to always be this
const REGISTER_ADDR_PADC: u8 = 0x00;
const REGISTER_ADDR_TADC: u8 = 0x02;
const REGISTER_ADDR_A0: u8 = 0x04; // other coefficients follow
const REGISTER_ADDR_START_CONVERSION: u8 = 0x12;

/// Provides access to the MPL115A2 Temperature and Pressure Sensor
///
/// http://cache.freescale.com/files/sensors/doc/data_sheet/MPL115A2.pdf
pub struct MPL115A2BarometerThermometer {
    i2cdev: I2CDevice,
    coeff: MPL115A2Coefficients,
}

/// In order to get either the temperature or humdity it is
/// necessary to read several different values from the chip.
///
/// These are not generally useful in and of themselves but
/// are used for calculating temperature/pressure.  The structure
/// is exposed externally as they *could* be useful for some
/// unknown use case.  Generally, you shouldn't ever need
/// to use this directly.
///
/// One use case for use of this struct directly would be for
/// getting both temperature and pressure in a single call.
pub struct MPL115A2RawReading {
    padc: u16, // 10-bit pressure ADC output value
    tadc: u16, // 10-bit pressure ADC output value
}

/// The sensors has several coefficients that must be used in order
/// to calculate a correct value for pressure/temperature.
///
/// This structure provides access to those.  It is usually only
/// necessary to read these coefficients once per interaction
/// with the acclerometer.  It does not need to be read again
/// on each sample.
pub struct MPL115A2Coefficients {
    a0: f32, // 16 bits, 1 sign, 12 int, 3 fractional, 0 dec pt 0 pad
    b1: f32, // 16 bits, 1 sign, 2 int, 13 fractional, 0 dec pt 0 pad
    b2: f32, // 16 bits, 1 sign, 1 int, 14 fractional, 0 dec pt 0 pad
    c12: f32,// 16 bits, 1 sign, 0 int, 13 fractional, 9 dec pt 0 pad
}

fn calc_coefficient(msb: u8,
                    lsb: u8,
                    fractional_bits: i32,
                    dec_pt_zero_pad: i32) -> f32 {
    // the proper adjustments can be made by multiplying the full signed
    // integral value by 2 ^ (float bits + decimal padding)
    let rawval: u16 = ((msb as u16) << 8) | lsb as u16;
    let nonsign: f32 = (rawval & !0x8000) as f32;
    let sign: f32 = if rawval & 0x8000 == 0 { 1.0 } else { -1.0 };
    (sign * nonsign / 2_f32.powi(fractional_bits)) / 10_f32.powi(dec_pt_zero_pad)
}

impl MPL115A2Coefficients {
    /// Convert a slice of data values of length 8 to coefficients
    ///
    /// This should be built from a read of registers 0x04-0x0B in
    /// order.  This gets the raw, unconverted value of each
    /// coefficient.
    pub fn new(i2cdev: &mut I2CDevice) -> I2CResult<MPL115A2Coefficients> {
        let mut buf: [u8; 8] = [0; 8];
        try!(i2cdev.write(&[REGISTER_ADDR_A0])
             .or_else(|e| Err(I2CError::from(e))));
        try!(i2cdev.write(&mut buf)
             .or_else(|e| Err(I2CError::from(e))));
        Ok(MPL115A2Coefficients {
            a0:  calc_coefficient(buf[0], buf[1], 3, 0),
            b1:  calc_coefficient(buf[2], buf[3], 13, 0),
            b2:  calc_coefficient(buf[4], buf[5], 14, 0),
            c12: calc_coefficient(buf[6], buf[7], 13, 9),
        })
    }
}


fn le_to_be(val: u16) -> u16 {
    let msb = val & 0xff;
    let lsb = (val & 0xff00) >> 8;
    (msb << 8) | lsb
}

impl MPL115A2RawReading {

    /// Create a new reading from the provided I2C Device
    pub fn new(i2cdev: &mut I2CDevice) -> I2CResult<MPL115A2RawReading> {
        // tell the chip to do an ADC read so we can get updated values
        try!(i2cdev.smbus_write_byte_data(REGISTER_ADDR_START_CONVERSION, 0x00));

        // maximum conversion time is 3ms
        thread::sleep_ms(3);

        // The SMBus functions read word values as little endian but that is not
        // what we want
        let raw_tadc: u16 = le_to_be(try!(i2cdev.smbus_read_word_data(REGISTER_ADDR_TADC)));
        let raw_padc: u16 = le_to_be(try!(i2cdev.smbus_read_word_data(REGISTER_ADDR_PADC)));
        let padc: u16 = raw_padc >> 6;
        let tadc: u16 = raw_tadc >> 6;
        Ok(MPL115A2RawReading {
            padc: padc,
            tadc: tadc,
        })
    }

    /// Calculate the temperature in centrigrade for this reading
    pub fn temperature_celsius(&self) -> f32 {
        (self.tadc as f32 - 498.0) / -5.35 + 25.0
    }

    /// Calculate the pressure in pascals for this reading
    pub fn pressure_pascals(&self, coeff: &MPL115A2Coefficients) -> i32 {
        // Pcomp = a0 + (b1 + c12 * Tadc) * Padc + b2 * Tadc
        // Pkpa = Pcomp * ((115 - 50) / 1023) + 50
        let pcomp: f32 =
            coeff.a0 +
            (coeff.b1 + coeff.c12 * self.tadc as f32) * self.padc as f32 +
            (coeff.b2 * self.tadc as f32);

        // scale has 1023 bits of range from 50 kPa to 115 kPa
        let pkpa: f32 = pcomp * (((115 -  50) / 1023) + 50) as f32;
        let pascals: f32 = pkpa * 1000.0;
        pascals as i32
    }
}


impl MPL115A2BarometerThermometer {
    /// Create sensor accessor for MPL115A2 on the provided i2c bus path
    pub fn new<P: AsRef<Path>>(path: P) -> I2CResult<MPL115A2BarometerThermometer> {
        let mut i2cdev = try!(I2CDevice::new(path, SLAVE_ADDR));
        let coeff = try!(MPL115A2Coefficients::new(&mut i2cdev));
        Ok(MPL115A2BarometerThermometer {
            i2cdev: i2cdev,
            coeff: coeff,
        })
    }
}

impl Barometer for MPL115A2BarometerThermometer {
    fn pressure_pascals(&mut self) -> I2CResult<i32> {
        let reading = try!(MPL115A2RawReading::new(&mut self.i2cdev));
        Ok(reading.pressure_pascals(&self.coeff))
    }
}

impl Thermometer for MPL115A2BarometerThermometer {
    fn temperature_celsius(&mut self) -> I2CResult<f32> {
        let reading = try!(MPL115A2RawReading::new(&mut self.i2cdev));
        Ok(reading.temperature_celsius())
    }
}

#[test]
fn test_calc_coefficient() {
    // unsigned simple
    assert_eq!(calc_coefficient(0x00, 0b1000, 3, 0), 1.0);

    // signed simple
    assert_eq!(calc_coefficient(0x80, 0b1000, 3, 0), -1.0);

    // pure integer (negative)
    assert_eq!(calc_coefficient(0xFF, 0xFF, 0, 0), -32_767_f32);

    // no integer part, zero padding, negative
    assert_eq!(calc_coefficient(0x8F, 0xFF, 15, 10), -0.000_000_000_012496948);
}
