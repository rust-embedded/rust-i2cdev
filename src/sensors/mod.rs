// Copyright 2015, Paul Osborne <osbpau@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/license/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option.  This file may not be copied, modified, or distributed
// except according to those terms.

use core::I2CResult;

pub mod adxl345_accelerometer;
pub mod mpl115a2_barometer;
pub mod nunchuck;
//pub mod bno055_orientation;

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
    /// Grab an accelerometer sample from the device
    fn accelerometer_sample(&mut self) -> I2CResult<AccelerometerSample>;
}

/// Trait for sensors that provide access to temperature readings
pub trait Thermometer {
    /// Get na temperature from the sensor in degrees celsisus
    ///
    /// Returns Some(temperature) if available, otherwise returns
    /// None
    fn temperature_celsius(&mut self) -> I2CResult<f32>;
}

/// Trait for sensors that provide access to pressure readings
pub trait Barometer {
    /// Get a pressure reading from the sensor in kPa
    ///
    /// Returns Some(temperature) if avialable, otherwise returns
    /// None
    fn pressure_kpa(&mut self) -> I2CResult<f32>;
}
