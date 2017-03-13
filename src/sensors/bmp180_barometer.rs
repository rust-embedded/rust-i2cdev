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
    padc: u16, // 10-bit pressure ADC output value
    tadc: u16, // 10-bit pressure ADC output value
}

pub struct BMP180BarometerThermometer<T: I2CDevice + Sized> {
    pub i2cdev: T,
    pub coeff: BMP180CalibrationCoefficients,
}


fn calculate_b5(raw_temp: i16, ac5: i16, ac6: i16, mc: i16, md: i16) -> i32 {
    let x1 = ((raw_temp as i32) - (ac6 as i32)) * (ac5 as i32) >> 15;
    let x2 = (mc as i32) << 11 / (x1 + (md as i32));
    x1 + x2
}


impl<T> BMP180BarometerThermometer<T>
    where T: I2CDevice + Sized
{
    /// Create sensor accessor for MPL115A2 on the provided i2c bus path
    pub fn new(mut i2cdev: T) -> Result<BMP180BarometerThermometer<T>, T::Error> {
        let coeff = try!(BMP180CalibrationCoefficients::new(&mut i2cdev));
        Ok(BMP180BarometerThermometer {
            i2cdev: i2cdev,
            coeff: coeff,
        })
    }
}

impl BMP180RawReading {
    /// Create a new reading from the provided I2C Device
    pub fn new<E: Error>(i2cdev: &mut I2CDevice<Error = E>) -> Result<BMP180RawReading, E> {
        // tell the chip to do an ADC read so we can get updated values
        // get temp first since it's needed for pressure calculation
        try!(i2cdev.smbus_write_byte_data(BMP180_REGISTER_CTL, BMP180_CMD_TEMP));

        // maximum conversion time is 5ms
        thread::sleep(Duration::from_millis(5));

        // The SMBus functions read word values as little endian but that is not
        // what we want
        let mut buf = [0_u8; 2];
        try!(i2cdev.write(&[BMP180_REGISTER_TEMP_MSB]));
        try!(i2cdev.read(&mut buf));
        // let padc: u16 = BigEndian::read_u16(&buf) >> 6;
        let tadc: u16 = BigEndian::read_u16(&buf[..]);
        unimplemented!();
        // Ok(MPL115A2RawReading {
        //     padc: padc,
        //     tadc: tadc,
        // })
    }
}

/// Calculate the temperature in centrigrade for this reading
// pub fn temperature_celsius(&self) -> f32 {
//     (self.tadc as f32 - 498.0) / -5.35 + 25.0
// }

// /// Calculate the pressure in pascals for this reading
// pub fn pressure_kpa(&self, coeff: &MPL115A2Coefficients) -> f32 {
//     // Pcomp = a0 + (b1 + c12 * Tadc) * Padc + b2 * Tadc
//     // Pkpa = Pcomp * ((115 - 50) / 1023) + 50
//     let pcomp: f32 = coeff.a0 + (coeff.b1 + coeff.c12 * self.tadc as f32) * self.padc as f32 + (coeff.b2 * self.tadc as f32);

//     // scale has 1023 bits of range from 50 kPa to 115 kPa
//     let pkpa: f32 = pcomp * ((115.0 - 50.0) / 1023.0) + 50.0;
//     pkpa
// }




impl BMP180CalibrationCoefficients {
    /// Convert a slice of data values of length 8 to coefficients
    ///
    /// This should be built from a read of registers 0x04-0x0B in
    /// order.  This gets the raw, unconverted value of each
    /// coefficient.
    pub fn new<E: Error>(i2cdev: &mut I2CDevice<Error = E>) -> Result<BMP180CalibrationCoefficients, E> {
        let mut buf: [u8; 11] = [0; 11];
        try!(i2cdev.write(&[BMP180_REGISTER_AC1MSB]));
        try!(i2cdev.read(&mut buf));
        // unimplemented!();
        Ok(BMP180CalibrationCoefficients {
            ac1: BigEndian::read_i16(&buf[0..0]),
            ac2: BigEndian::read_i16(&buf[1..1]),
            ac3: BigEndian::read_i16(&buf[2..2]),
            ac4: BigEndian::read_u16(&buf[3..3]),
            ac5: BigEndian::read_u16(&buf[4..4]),
            ac6: BigEndian::read_u16(&buf[5..5]),
            b1: BigEndian::read_i16(&buf[6..6]),
            b2: BigEndian::read_i16(&buf[7..7]),
            mb: BigEndian::read_i16(&buf[8..8]),
            mc: BigEndian::read_i16(&buf[9..9]),
            md: BigEndian::read_i16(&buf[10..10]),
        })
    }
}
