/// Drivers for L3G gyros.
/// This was tested with:
///   - L3GD20 on the XINY GY-89 10DOF LSM303D + L3GD20 + BMP180
///
/// Author: Thomas Deegan <tdeegan@gmail.com>
use std::thread;
use std::time::Duration;

use core::I2CDevice;
use sensors::{Gyroscope, GyroscopeSample};

const L3G_WHO_AM_I  : u8 = 0x0F;

const L3G_CTRL_REG1 : u8 = 0x20;
const L3G_CTRL_REG2 : u8 = 0x21;
const L3G_CTRL_REG3 : u8 = 0x22;
const L3G_CTRL_REG4 : u8 =  0x23;
const L3G_CTRL_REG5 : u8 =  0x24;
const L3G_REFERENCE : u8 =   0x25;
const L3G_OUTd_TEMP  : u8 =  0x26;
const L3G_STATUS_REG : u8 =  0x27;

const L3G_OUT_X_L   : u8    =  0x28;
const L3G_OUT_X_H   : u8    =  0x29;
const L3G_OUT_Y_L  : u8     =  0x2A;
const L3G_OUT_Y_H   : u8    =  0x2B;
const L3G_OUT_Z_L    : u8   =  0x2C;
const L3G_OUT_Z_H    : u8   =  0x2D;

// Conversion from Gs to degress per second according to our sample rate.
pub const G_GAIN : f32 = 0.070;
pub const GYRO_ADDRESS: u16 = 0x6B;

// TODO: Some constants missing.
pub struct L3gGyroscope<T: I2CDevice + Sized> {
    i2cdev: T,
}

impl<T> L3gGyroscope<T> where T: I2CDevice + Sized
{
    /// Create a new gyroscope handle for the given path/addr
    pub fn new(mut i2cdev: T) -> Result<L3gGyroscope<T>, T::Error> {
        // init sequence
        i2cdev.smbus_write_byte_data(L3G_CTRL_REG1, 0b00001111).unwrap();
        // Set the dps to 2k.
        i2cdev.smbus_write_byte_data(L3G_CTRL_REG4, 0b00110000).unwrap();

        thread::sleep(Duration::from_millis(200));
        Ok(L3gGyroscope { i2cdev: i2cdev })
    }

    /// Get the device id
    pub fn device_id(&mut self) -> Result<u8, T::Error> {
        self.i2cdev.smbus_read_byte_data(L3G_WHO_AM_I)
    }
}

impl<T> Gyroscope for L3gGyroscope<T> where T: I2CDevice + Sized
{
    type Error = T::Error;

    fn gyroscope_sample(&mut self) -> Result<GyroscopeSample, T::Error> {
        // datasheet recommends multi-byte read to avoid reading
        // an inconsistent set of data
        let results : Vec<u8> = self.i2cdev.smbus_read_i2c_block_data(0x80 | L3G_OUT_X_L, 6).unwrap();
        // This comes in the wrong order. WOW!
        let x : i16 = (((results[1] as u16) << 8)  | (results[0] as u16) ) as i16;
        let y : i16 = (((results[3] as u16) << 8) | (results[2] as u16)) as i16;
        let z : i16 = (((results[5] as u16) << 8) | (results[4] as u16)) as i16;
        Ok(GyroscopeSample {x: x as f32, y: y as f32, z: z as f32})
    }
}