// Copyright 2015, Paul Osborne <osbpau@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/license/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option.  This file may not be copied, modified, or distributed
// except according to those terms.

use std::os::unix::prelude::*;
use std::io;
use std::io::prelude::*;
use std::fs::OpenOptions;
use std::fs::File;
use std::path::Path;
use std::convert;
use nix;
use ffi;

/// Error that occured while performing and I2C Operation
#[derive(Debug)]
pub enum I2CError {
    IOError(io::Error),
    NixError(nix::Error),
}

pub type I2CResult<T> = Result<T, I2CError>;

impl convert::From<nix::Error> for I2CError {
    fn from(e: nix::Error) -> I2CError {
        I2CError::NixError(e)
    }
}

impl convert::From<io::Error> for I2CError {
    fn from(e: io::Error) -> I2CError {
        I2CError::IOError(e)
    }
}

pub struct I2CDevice {
    devfile: File,
    slave_address: u16,
}

#[derive(Debug)]
pub enum I2CDeviceOpenError {
    IOError(io::Error),
    NixError(nix::Error),
}

impl I2CDevice {
    /// Create a new I2CDevice for the specified path
    pub fn new<P: AsRef<Path>>(path: P, slave_address: u16) -> I2CResult<I2CDevice> {
        let file = try!(OpenOptions::new()
                        .read(true)
                        .write(true)
                        .open(path)
                        .or_else(|e| Err(I2CError::IOError(e))));
        let mut device = I2CDevice {
            devfile: file,
            slave_address: 0, // will be set later
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

impl AsRawFd for I2CDevice {
    fn as_raw_fd(&self) -> RawFd {
        self.devfile.as_raw_fd()
    }
}

impl Read for I2CDevice {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.devfile.read(buf)
    }
}

impl Write for I2CDevice {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.devfile.write(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.devfile.flush()
    }
}

