// Copyright 2015, Paul Osborne <osbpau@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/license/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option.  This file may not be copied, modified, or distributed
// except according to those terms.

use ffi;
use core::{I2CDevice, I2CError, I2CResult};
use std::path::Path;
use std::fs::File;
use std::convert;
use nix;
use std::io;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::os::unix::prelude::*;

pub struct LinuxI2CDevice {
    devfile: File,
    slave_address: u16,
}

impl convert::From<nix::Error> for I2CError {
    #[allow(unused_variables)]
    fn from(e: nix::Error) -> I2CError {
        I2CError::Other("Uncategorized Nix Error")  // TODO
    }
}

impl convert::From<io::Error> for I2CError {
    #[allow(unused_variables)]
    fn from(e: io::Error) -> I2CError {
        I2CError::Other("Uncategorized IO Error")  // TODO
    }
}

impl AsRawFd for LinuxI2CDevice {
    fn as_raw_fd(&self) -> RawFd {
        self.devfile.as_raw_fd()
    }
}

impl LinuxI2CDevice {
    /// Create a new I2CDevice for the specified path
    pub fn new<P: AsRef<Path>>(path: P, slave_address: u16) -> I2CResult<LinuxI2CDevice> {
        let file = try!(OpenOptions::new()
                            .read(true)
                            .write(true)
                            .open(path)
                            .or_else(|e| Err(I2CError::from(e))));
        let mut device = LinuxI2CDevice {
            devfile: file,
            slave_address: 0, /* will be set later */
        };
        try!(device.set_slave_address(slave_address));
        Ok(device)
    }

    /// Set the slave address for this device
    ///
    /// Typically the address is expected to be 7-bits but 10-bit addresses
    /// may be supported by the kernel driver in some cases.  Little validation
    /// is done in Rust as the kernel is good at making sure things are valid.
    ///
    /// Note that if you have created a device using
    /// `I2Device::new(...)` it is not necesasry to call this method
    /// (it is done internally).  Calling this method is only
    /// necessary if you need to change the slave device and you do
    /// not want to create a new device.
    fn set_slave_address(&mut self, slave_address: u16) -> I2CResult<()> {
        try!(ffi::i2c_set_slave_address(self.as_raw_fd(), slave_address));
        self.slave_address = slave_address;
        Ok(())
    }

}

impl I2CDevice for LinuxI2CDevice {

    /// Read data from the device to fill the provided slice
    fn read(&mut self, data: &mut [u8]) -> I2CResult<()> {
        try!(self.devfile.read(data).or_else(|e| Err(I2CError::from(e))));
        Ok(())
    }

    /// Write the provided buffer to the device
    fn write(&mut self, data: &[u8]) -> I2CResult<()> {
        try!(self.devfile.write(data).or_else(|e| Err(I2CError::from(e))));
        Ok(())
    }

    /// This sends a single bit to the device, at the place of the Rd/Wr bit
    fn smbus_write_quick(&mut self, bit: bool) -> I2CResult<()> {
        ffi::i2c_smbus_write_quick(self.as_raw_fd(), bit)
    }

    /// Read a single byte from a device, without specifying a device register
    ///
    /// Some devices are so simple that this interface is enough; for
    /// others, it is a shorthand if you want to read the same register as in
    /// the previous SMBus command.
    fn smbus_read_byte(&mut self) -> I2CResult<u8> {
        ffi::i2c_smbus_read_byte(self.as_raw_fd())
    }

    /// Write a single byte to a sdevice, without specifying a device register
    ///
    /// This is the opposite operation as smbus_read_byte.  As with read_byte,
    /// no register is specified.
    fn smbus_write_byte(&mut self, value: u8) -> I2CResult<()> {
        ffi::i2c_smbus_write_byte(self.as_raw_fd(), value)
    }

    /// Read a single byte from a device, from a designated register
    ///
    /// The register is specified through the Comm byte.
    fn smbus_read_byte_data(&mut self, register: u8) -> I2CResult<u8> {
        ffi::i2c_smbus_read_byte_data(self.as_raw_fd(), register)
    }

    /// Write a single byte to a specific register on a device
    ///
    /// The register is specified through the Comm byte.
    fn smbus_write_byte_data(&mut self, register: u8, value: u8) -> I2CResult<()> {
        ffi::i2c_smbus_write_byte_data(self.as_raw_fd(), register, value)
    }

    /// Read 2 bytes form a given register on a device
    fn smbus_read_word_data(&mut self, register: u8) -> I2CResult<u16> {
        ffi::i2c_smbus_read_word_data(self.as_raw_fd(), register)
    }

    /// Write 2 bytes to a given register on a device
    fn smbus_write_word_data(&mut self, register: u8, value: u16) -> I2CResult<()> {
        ffi::i2c_smbus_write_word_data(self.as_raw_fd(), register, value)
    }

    /// Select a register, send 16 bits of data to it, and read 16 bits of data
    fn smbus_process_word(&mut self, register: u8, value: u16) -> I2CResult<u16> {
        ffi::i2c_smbus_process_call(self.as_raw_fd(), register, value)
    }

    /// Read a block of up to 32 bytes from a device
    ///
    /// The actual number of bytes available to read is returned in the count
    /// byte.  This code returns a correctly sized vector containing the
    /// count bytes read from the device.
    fn smbus_read_block_data(&mut self, register: u8) -> I2CResult<Vec<u8>> {
        ffi::i2c_smbus_read_block_data(self.as_raw_fd(), register)
    }

    /// Write a block of up to 32 bytes to a device
    ///
    /// The opposite of the Block Read command, this writes up to 32 bytes to
    /// a device, to a designated register that is specified through the
    /// Comm byte. The amount of data is specified in the Count byte.
    fn smbus_write_block_data(&mut self, register: u8, values: &[u8]) -> I2CResult<()> {
        ffi::i2c_smbus_write_block_data(self.as_raw_fd(), register, values)
    }

    /// Select a register, send 1 to 31 bytes of data to it, and reads
    /// 1 to 31 bytes of data from it.
    fn smbus_process_block(&mut self, register: u8, values: &[u8]) -> I2CResult<()> {
        ffi::i2c_smbus_write_i2c_block_data(self.as_raw_fd(), register, values)
    }
}
