// Copyright 2015, Paul Osborne <osbpau@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/license/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option.  This file may not be copied, modified, or distributed
// except according to those terms.

// Reads data from several different sensors all on the same i2c bus
// - Analog Devices ADXL345 Digital Accelerometer
// - Freescale MPL115A2 Digital Barometer/Thermometer
// - Bosch BNO055 9-Axis O Absolute Orientation Sensor

#![allow(dead_code)] // register map

extern crate byteorder;
extern crate docopt;
extern crate i2cdev;

use docopt::Docopt;
use sensors::adxl345_accelerometer::*;
use sensors::mpl115a2_barometer::*;
use sensors::{Accelerometer, Barometer, Thermometer};
use std::env::args;
use std::thread;
use std::time::Duration;

#[cfg(any(target_os = "linux", target_os = "android"))]
use i2cdev::linux::*;

mod sensors {
    use std::error::Error;

    #[derive(Debug)]
    pub struct AccelerometerSample {
        /// x-axis G's
        pub x: f32,
        /// y-axis G's
        pub y: f32,
        /// z-axis G's
        pub z: f32,
    }

    /// Trait for sensors that provide access to accelerometer readings (3-axis)
    pub trait Accelerometer {
        type Error: Error;

        /// Grab an accelerometer sample from the device
        fn accelerometer_sample(&mut self) -> Result<AccelerometerSample, Self::Error>;
    }

    /// Trait for sensors that provide access to temperature readings
    pub trait Thermometer {
        type Error: Error;

        /// Get a temperature from the sensor in degrees celsius
        ///
        /// Returns `Ok(temperature)` if available, otherwise returns
        /// `Err(Self::Error)`
        fn temperature_celsius(&mut self) -> Result<f32, Self::Error>;
    }

    /// Trait for sensors that provide access to pressure readings
    pub trait Barometer {
        type Error: Error;

        /// Get a pressure reading from the sensor in kPa
        ///
        /// Returns `Ok(temperature)` if avialable, otherwise returns
        /// `Err(Self::Error)`
        fn pressure_kpa(&mut self) -> Result<f32, Self::Error>;
    }

    /// Trait for sensors that provide access to altitude readings
    pub trait Altimeter {
        type Error: Error;

        /// Get an altitude reading from the sensor in meters, relative to the
        /// pressure in kPa at
        /// sea level
        ///
        /// Returns `Ok(altitude)` if available, otherwise returns
        /// `Err(Self::Error)`
        fn altitude_meters(&mut self, sea_level_kpa: f32) -> Result<f32, Self::Error>;
    }

    impl<T> Altimeter for T
    where
        T: Barometer,
    {
        type Error = <Self as Barometer>::Error;

        fn altitude_meters(&mut self, sea_level_kpa: f32) -> Result<f32, Self::Error> {
            let pressure = self.pressure_kpa()? * 1000.;
            let sea_level_pa = sea_level_kpa * 1000.;

            let altitude = 44330. * (1. - (pressure / sea_level_pa).powf(0.1903));
            Ok(altitude)
        }
    }

    pub mod adxl345_accelerometer {
        use super::*;
        use byteorder::{ByteOrder, LittleEndian};
        use i2cdev::core::I2CDevice;

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

        impl<T> ADXL345Accelerometer<T>
        where
            T: I2CDevice + Sized,
        {
            /// Create a new accelerometer handle for the given path/addr
            ///
            /// The `SLAVE_ADDR_*` constants from this module should be
            /// used to select either the primary or alternative slave
            /// address (dependent on `ALT ADDRESS` pin)
            pub fn new(mut i2cdev: T) -> Result<ADXL345Accelerometer<T>, T::Error> {
                // setup standy mode to configure
                i2cdev.smbus_write_byte_data(REGISTER_POWER_CTL, 0x00)?;

                // configure some defaults
                i2cdev
                    .smbus_write_byte_data(REGISTER_BW_RATE, ADXL345DataRate::RATE_1600HZ as u8)?;
                i2cdev.smbus_write_byte_data(REGISTER_DATA_FORMAT, 0x08)?;
                i2cdev.smbus_write_byte_data(REGISTER_OFSX, 0xFD)?;
                i2cdev.smbus_write_byte_data(REGISTER_OFSY, 0x03)?;
                i2cdev.smbus_write_byte_data(REGISTER_OFSZ, 0xFE)?;

                // put device in measurement mode
                i2cdev.smbus_write_byte_data(REGISTER_POWER_CTL, 0x08)?;

                Ok(ADXL345Accelerometer { i2cdev })
            }

            /// Get the device id
            pub fn device_id(&mut self) -> Result<u8, T::Error> {
                self.i2cdev.smbus_read_byte_data(REGISTER_DEVID)
            }
        }

        const ACCEL_RANGE: f32 = 2.0; // +- 2G (with defaults)
        const ACCEL_BITS: u8 = 10; // 10-bit resolution

        impl<T> Accelerometer for ADXL345Accelerometer<T>
        where
            T: I2CDevice + Sized,
        {
            type Error = T::Error;

            fn accelerometer_sample(&mut self) -> Result<AccelerometerSample, T::Error> {
                // datasheet recommends multi-byte read to avoid reading
                // an inconsistent set of data
                let mut buf: [u8; 6] = [0u8; 6];
                self.i2cdev.write(&[REGISTER_X0])?;
                self.i2cdev.read(&mut buf)?;

                let x: i16 = LittleEndian::read_i16(&[buf[0], buf[1]]);
                let y: i16 = LittleEndian::read_i16(&[buf[2], buf[3]]);
                let z: i16 = LittleEndian::read_i16(&[buf[4], buf[5]]);
                Ok(AccelerometerSample {
                    x: (f32::from(x) / 1023.0) * (ACCEL_RANGE * 2.0),
                    y: (f32::from(y) / 1023.0) * (ACCEL_RANGE * 2.0),
                    z: (f32::from(z) / 1023.0) * (ACCEL_RANGE * 2.0),
                })
            }
        }
    }

    pub mod mpl115a2_barometer {
        use super::*;
        use byteorder::{BigEndian, ByteOrder};
        use i2cdev::core::I2CDevice;
        use std::error::Error;
        use std::thread;
        use std::time::Duration;

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
            a0: f32,  // 16 bits, 1 sign, 12 int, 3 fractional, 0 dec pt 0 pad
            b1: f32,  // 16 bits, 1 sign, 2 int, 13 fractional, 0 dec pt 0 pad
            b2: f32,  // 16 bits, 1 sign, 1 int, 14 fractional, 0 dec pt 0 pad
            c12: f32, // 16 bits, 1 sign, 0 int, 13 fractional, 9 dec pt 0 pad
        }

        fn calc_coefficient(
            msb: u8,
            lsb: u8,
            integer_bits: i32,
            fractional_bits: i32,
            dec_pt_zero_pad: i32,
        ) -> f32 {
            // If values are less than 16 bytes, need to adjust
            let extrabits = 16 - integer_bits - fractional_bits - 1;
            let rawval: i16 = BigEndian::read_i16(&[msb, lsb]);
            let adj = (f32::from(rawval) / 2_f32.powi(fractional_bits + extrabits))
                / 10_f32.powi(dec_pt_zero_pad);
            adj
        }

        impl MPL115A2Coefficients {
            /// Convert a slice of data values of length 8 to coefficients
            ///
            /// This should be built from a read of registers 0x04-0x0B in
            /// order.  This gets the raw, unconverted value of each
            /// coefficient.
            pub fn new<E: Error>(
                i2cdev: &mut dyn I2CDevice<Error = E>,
            ) -> Result<MPL115A2Coefficients, E> {
                let mut buf: [u8; 8] = [0; 8];
                i2cdev.write(&[REGISTER_ADDR_A0])?;
                i2cdev.read(&mut buf)?;
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
            pub fn new<E: Error>(
                i2cdev: &mut dyn I2CDevice<Error = E>,
            ) -> Result<MPL115A2RawReading, E> {
                // tell the chip to do an ADC read so we can get updated values
                i2cdev.smbus_write_byte_data(REGISTER_ADDR_START_CONVERSION, 0x00)?;

                // maximum conversion time is 3ms
                thread::sleep(Duration::from_millis(3));

                // The SMBus functions read word values as little endian but that is not
                // what we want
                let mut buf = [0_u8; 4];
                i2cdev.write(&[REGISTER_ADDR_PADC])?;
                i2cdev.read(&mut buf)?;
                let padc: u16 = BigEndian::read_u16(&buf) >> 6;
                let tadc: u16 = BigEndian::read_u16(&buf[2..]) >> 6;
                Ok(MPL115A2RawReading { padc, tadc })
            }

            /// Calculate the temperature in centrigrade for this reading
            pub fn temperature_celsius(&self) -> f32 {
                (f32::from(self.tadc) - 498.0) / -5.35 + 25.0
            }

            /// Calculate the pressure in pascals for this reading
            pub fn pressure_kpa(&self, coeff: &MPL115A2Coefficients) -> f32 {
                // Pcomp = a0 + (b1 + c12 * Tadc) * Padc + b2 * Tadc
                // Pkpa = Pcomp * ((115 - 50) / 1023) + 50
                let pcomp: f32 = coeff.a0
                    + (coeff.b1 + coeff.c12 * f32::from(self.tadc)) * f32::from(self.padc)
                    + (coeff.b2 * f32::from(self.tadc));

                // scale has 1023 bits of range from 50 kPa to 115 kPa
                let pkpa: f32 = pcomp * ((115.0 - 50.0) / 1023.0) + 50.0;
                pkpa
            }
        }

        impl<T> MPL115A2BarometerThermometer<T>
        where
            T: I2CDevice + Sized,
        {
            /// Create sensor accessor for MPL115A2 on the provided i2c bus path
            pub fn new(mut i2cdev: T) -> Result<MPL115A2BarometerThermometer<T>, T::Error> {
                let coeff = MPL115A2Coefficients::new(&mut i2cdev)?;
                Ok(MPL115A2BarometerThermometer { i2cdev, coeff })
            }
        }

        impl<T> Barometer for MPL115A2BarometerThermometer<T>
        where
            T: I2CDevice + Sized,
        {
            type Error = T::Error;

            fn pressure_kpa(&mut self) -> Result<f32, T::Error> {
                let reading = MPL115A2RawReading::new(&mut self.i2cdev)?;
                Ok(reading.pressure_kpa(&self.coeff))
            }
        }

        impl<T> Thermometer for MPL115A2BarometerThermometer<T>
        where
            T: I2CDevice + Sized,
        {
            type Error = T::Error;

            fn temperature_celsius(&mut self) -> Result<f32, T::Error> {
                let reading = MPL115A2RawReading::new(&mut self.i2cdev)?;
                Ok(reading.temperature_celsius())
            }
        }

        #[cfg(test)]
        mod tests {
            use super::calc_coefficient;
            use super::*;
            use mock::MockI2CDevice;
            use sensors::*;

            macro_rules! assert_almost_eq {
                ($left:expr, $right:expr) => {{
                    match (&($left), &($right)) {
                        (left_val, right_val) => {
                            if (*left_val - *right_val).abs() > 0.0001 {
                                panic!("assertion failed: ({:?} != {:?})", *left_val, *right_val);
                            }
                        }
                    }
                }};
            }

            fn make_dev(mut i2cdev: MockI2CDevice) -> MPL115A2BarometerThermometer<MockI2CDevice> {
                (&mut i2cdev.regmap).write_regs(
                    0x04,
                    &[
                        74, 98, /* A0 */
                        165, 150, /* B1 */
                        182, 106, /* B2 */
                        63, 232,
                    ],
                ); // C12
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
    }
}

const USAGE: &str = "
Reading sensor data from a variety of sensors

Usage:
  sensors <device>
  sensors (-h | --help)
  sensors --version

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
    let mpl115a2_i2cdev = LinuxI2CDevice::new(device, MPL115A2_I2C_ADDR).unwrap();
    let adxl345_i2cdev = LinuxI2CDevice::new(device, 0x53).unwrap();

    let mut mpl115a2 = MPL115A2BarometerThermometer::new(mpl115a2_i2cdev).unwrap();
    let mut adxl345 = ADXL345Accelerometer::new(adxl345_i2cdev).unwrap();

    println!("== ADXL345 ID: 0x{:X} ==", adxl345.device_id().unwrap());

    loop {
        let accel = adxl345.accelerometer_sample().unwrap();
        println!(
            "Temperature: {:?} C",
            mpl115a2.temperature_celsius().unwrap()
        );
        println!("Pressure:    {:?} kPa", mpl115a2.pressure_kpa().unwrap());
        println!("Accel:       {:?}", accel);
        println!(
            "Accel Tot:   {:?}",
            (accel.x.powi(2) + accel.y.powi(2) + accel.z.powi(2)).sqrt()
        );
        thread::sleep(Duration::from_millis(1000));
    }
}
