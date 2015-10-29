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
use std::error::Error;
use core::I2CDevice;
use byteorder::{ByteOrder, BigEndian};

pub const MPL115A2_I2C_ADDR: u16 = 0x60; // appears to always be this

const REGISTER_ADDR_PADC: u8 = 0x00;
const REGISTER_ADDR_TADC: u8 = 0x02;
const REGISTER_ADDR_A0: u8 = 0x04; // other coefficients follow
const REGISTER_ADDR_START_CONVERSION: u8 = 0x12;

/// Provides access to the MPL115A2 Temperature and Pressure Sensor
///
/// http://cache.freescale.com/files/sensors/doc/data_sheet/MPL115A2.pdf
pub struct MPL115A2BarometerThermometer<T: I2CDevice + Sized> {
    pub i2cdev: T,
    pub coeff: MPL115A2Coefficients,
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
#[derive(Debug)]
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
#[derive(Debug)]
pub struct MPL115A2Coefficients {
    a0: f32, // 16 bits, 1 sign, 12 int, 3 fractional, 0 dec pt 0 pad
    b1: f32, // 16 bits, 1 sign, 2 int, 13 fractional, 0 dec pt 0 pad
    b2: f32, // 16 bits, 1 sign, 1 int, 14 fractional, 0 dec pt 0 pad
    c12: f32, // 16 bits, 1 sign, 0 int, 13 fractional, 9 dec pt 0 pad
}

fn calc_coefficient(msb: u8,
                    lsb: u8,
                    integer_bits: i32,
                    fractional_bits: i32,
                    dec_pt_zero_pad: i32)
                    -> f32 {
    // If values are less than 16 bytes, need to adjust
    let extrabits = 16 - integer_bits - fractional_bits - 1;
    let rawval: i16 = BigEndian::read_i16(&[msb, lsb]);
    let adj = (rawval as f32 / 2_f32.powi(fractional_bits + extrabits)) /
              10_f32.powi(dec_pt_zero_pad);
    adj
}

impl MPL115A2Coefficients {
    /// Convert a slice of data values of length 8 to coefficients
    ///
    /// This should be built from a read of registers 0x04-0x0B in
    /// order.  This gets the raw, unconverted value of each
    /// coefficient.
    pub fn new<E: Error>(i2cdev: &mut I2CDevice<Error=E>) -> Result<MPL115A2Coefficients, E> {
        let mut buf: [u8; 8] = [0; 8];
        try!(i2cdev.write(&[REGISTER_ADDR_A0]));
        try!(i2cdev.read(&mut buf));
        Ok(MPL115A2Coefficients {
            a0: calc_coefficient(buf[0], buf[1], 12, 3, 0),
            b1: calc_coefficient(buf[2], buf[3], 2, 13, 0),
            b2: calc_coefficient(buf[4], buf[5], 1, 14, 0),
            c12: calc_coefficient(buf[6], buf[7], 0, 13, 9),
        })
    }
}


impl MPL115A2RawReading {
    /// Create a new reading from the provided I2C Device
    pub fn new<E: Error>(i2cdev: &mut I2CDevice<Error=E>) -> Result<MPL115A2RawReading, E> {
        // tell the chip to do an ADC read so we can get updated values
        try!(i2cdev.smbus_write_byte_data(REGISTER_ADDR_START_CONVERSION, 0x00));

        // maximum conversion time is 3ms
        thread::sleep_ms(3);

        // The SMBus functions read word values as little endian but that is not
        // what we want
        let mut buf = [0_u8; 4];
        try!(i2cdev.write(&[REGISTER_ADDR_PADC]));
        try!(i2cdev.read(&mut buf));
        let padc: u16 = BigEndian::read_u16(&buf) >> 6;
        let tadc: u16 = BigEndian::read_u16(&buf[2..]) >> 6;
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
    pub fn pressure_kpa(&self, coeff: &MPL115A2Coefficients) -> f32 {
        // Pcomp = a0 + (b1 + c12 * Tadc) * Padc + b2 * Tadc
        // Pkpa = Pcomp * ((115 - 50) / 1023) + 50
        let pcomp: f32 = coeff.a0 + (coeff.b1 + coeff.c12 * self.tadc as f32) * self.padc as f32 +
                         (coeff.b2 * self.tadc as f32);

        // scale has 1023 bits of range from 50 kPa to 115 kPa
        let pkpa: f32 = pcomp * ((115.0 - 50.0) / 1023.0) + 50.0;
        pkpa
    }
}


impl<T> MPL115A2BarometerThermometer<T> where T: I2CDevice + Sized
{
    /// Create sensor accessor for MPL115A2 on the provided i2c bus path
    pub fn new(mut i2cdev: T) -> Result<MPL115A2BarometerThermometer<T>, T::Error> {
        let coeff = try!(MPL115A2Coefficients::new(&mut i2cdev));
        Ok(MPL115A2BarometerThermometer {
            i2cdev: i2cdev,
            coeff: coeff,
        })
    }
}

impl<T> Barometer for MPL115A2BarometerThermometer<T> where T: I2CDevice + Sized
{
    type Error = T::Error;

    fn pressure_kpa(&mut self) -> Result<f32, T::Error> {
        let reading = try!(MPL115A2RawReading::new(&mut self.i2cdev));
        Ok(reading.pressure_kpa(&self.coeff))
    }
}

impl<T> Thermometer for MPL115A2BarometerThermometer<T> where T: I2CDevice + Sized
{
    type Error = T::Error;

    fn temperature_celsius(&mut self) -> Result<f32, T::Error> {
        let reading = try!(MPL115A2RawReading::new(&mut self.i2cdev));
        Ok(reading.temperature_celsius())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use super::calc_coefficient;
    use sensors::*;
    use mock::MockI2CDevice;

    macro_rules! assert_almost_eq {
        ($left:expr, $right:expr) => ({
            match (&($left), &($right)) {
                (left_val, right_val) => {
                    if (*left_val - *right_val).abs() > 0.0001 {
                        panic!("assertion failed: ({:?} != {:?})", *left_val, *right_val);
                    }
                }
            }
        })
    }

    fn make_dev(mut i2cdev: MockI2CDevice) -> MPL115A2BarometerThermometer<MockI2CDevice> {
        (&mut i2cdev.regmap).write_regs(0x04,
                                        &[74, 98 /* A0 */, 165, 150 /* B1 */, 182,
                                          106 /* B2 */, 63, 232]); // C12
        MPL115A2BarometerThermometer::new(i2cdev).unwrap()
    }

    #[test]
    fn test_calc_coefficient() {
        // unsigned simple
        assert_almost_eq!(calc_coefficient(0x00, 0b1000, 12, 3, 0), 1.0);
        // signed simple
        assert_almost_eq!(calc_coefficient(0xFF, 0xF8, 12, 3, 0), -1.0);
        // pure integer (negative)
        assert_almost_eq!(calc_coefficient(0x80, 0x00, 15, 0, 0), -32_768_f32);
        // no integer part, zero padding, negative
        assert_almost_eq!(calc_coefficient(0x00, 0x01, 15, 0, 10), 0.000_000_000_1);
    }

    #[test]
    fn test_basic_pressure_read() {
        let mut i2cdev = MockI2CDevice::new();
        (&mut i2cdev.regmap).write_regs(0x00, &[0x6e, 0xc0, 0x81, 0x40]);

        let mut dev = make_dev(i2cdev);
        assert_almost_eq!(dev.pressure_kpa().unwrap(), 83.93877);
    }

    #[test]
    fn test_basic_temp_read() {
        let mut i2cdev = MockI2CDevice::new();
        (&mut i2cdev.regmap).write_regs(0, &[0x6e, 0xc0, 0x81, 0x40]);

        let mut dev = make_dev(i2cdev);
        assert_almost_eq!(dev.temperature_celsius().unwrap(), 21.448599);
    }

}
