// Copyright 2015, Paul Osborne <osbpau@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/license/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option.  This file may not be copied, modified, or distributed
// except according to those terms.

use std::error::Error;

pub mod adxl345_accelerometer;
pub mod mpl115a2_barometer;
pub mod nunchuck;
// pub mod bno055_orientation;

#[derive(Debug)]
pub struct AccelerometerSample {
    /// x-axis G's
    pub x: f32,
    /// y-axis G's
    pub y: f32,
    /// z-axis G's
    pub z: f32,
}

#[derive(Debug)]
pub struct GyroscopeSample {
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

/// Trait for sensors that provide access to accelerometer readings (3-axis)
pub trait Gyroscope {
    type Error: Error;

    /// Grab a gyroscope sample from the device.
    fn gyro_sample(&mut self) -> Result<GyroscopeSample, Self::Error>;
}

/// Trait for sensors that provide access to temperature readings
pub trait Thermometer {
    type Error: Error;

    /// Get a temperature from the sensor in degrees celsius
    ///
    /// Returns `Ok(temperature)` if available, otherwise returns `Err(Self::Error)`
    fn temperature_celsius(&mut self) -> Result<f32, Self::Error>;
}

/// Trait for sensors that provide access to pressure readings
pub trait Barometer {
    type Error: Error;

    /// Get a pressure reading from the sensor in kPa
    ///
    /// Returns `Ok(temperature)` if avialable, otherwise returns `Err(Self::Error)`
    fn pressure_kpa(&mut self) -> Result<f32, Self::Error>;
}

/// Trait for sensors that provide access to altitude readings
pub trait Altimeter {
    type Error: Error;

    /// Get an altitude reading from the sensor in meters, relative to the pressure in kPa at
    /// sea level
    ///
    /// Returns `Ok(altitude)` if available, otherwise returns `Err(Self::Error)`
    fn altitude_meters(&mut self, sea_level_kpa: f32) -> Result<f32, Self::Error>;
}

impl<T> Altimeter for T
    where T: Barometer
{
    type Error = <Self as Barometer>::Error;

    fn altitude_meters(&mut self, sea_level_kpa: f32) -> Result<f32, Self::Error> {
        let pressure = try!(self.pressure_kpa()) * 1000.;
        let sea_level_pa = sea_level_kpa * 1000.;

        let altitude = 44330. * (1. - (pressure / sea_level_pa).powf(0.1903));
        Ok(altitude)
    }
}
