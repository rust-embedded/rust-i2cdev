// Copyright 2015, Paul Osborne <osbpau@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/license/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option.  This file may not be copied, modified, or distributed
// except according to those terms.

use core::{I2CDevice, I2CTransfer};
use ffi;
use nix;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::fs::OpenOptions;
use std::io;
use std::io::prelude::*;
use std::os::unix::prelude::*;
use std::path::Path;

// Expose these core structs from this module
pub use core::I2CMessage;

/// Concrete linux I2C device
pub struct LinuxI2CDevice {
    devfile: File,
    slave_address: u16,
    pec: bool,
}

/// Linux I2C bus
pub struct LinuxI2CBus {
    devfile: File,
}

/// Linux I2C errors
#[derive(Debug)]
pub enum LinuxI2CError {
    /// OS error
    Nix(nix::Error),
    /// Input/output error
    Io(io::Error),
}

impl From<nix::Error> for LinuxI2CError {
    fn from(e: nix::Error) -> Self {
        LinuxI2CError::Nix(e)
    }
}

impl From<io::Error> for LinuxI2CError {
    fn from(e: io::Error) -> Self {
        LinuxI2CError::Io(e)
    }
}

impl From<LinuxI2CError> for io::Error {
    fn from(e: LinuxI2CError) -> io::Error {
        match e {
            LinuxI2CError::Io(e) => e,
            LinuxI2CError::Nix(e) => e.into(),
        }
    }
}

impl fmt::Display for LinuxI2CError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            LinuxI2CError::Nix(ref e) => fmt::Display::fmt(e, f),
            LinuxI2CError::Io(ref e) => fmt::Display::fmt(e, f),
        }
    }
}

impl Error for LinuxI2CError {
    fn cause(&self) -> Option<&dyn Error> {
        match *self {
            LinuxI2CError::Io(ref e) => Some(e),
            LinuxI2CError::Nix(ref e) => Some(e),
        }
    }
}

impl AsRawFd for LinuxI2CDevice {
    fn as_raw_fd(&self) -> RawFd {
        self.devfile.as_raw_fd()
    }
}

impl AsRawFd for LinuxI2CBus {
    fn as_raw_fd(&self) -> RawFd {
        self.devfile.as_raw_fd()
    }
}

impl LinuxI2CDevice {
    /// Create a new I2CDevice for the specified path
    pub fn new<P: AsRef<Path>>(
        path: P,
        slave_address: u16,
    ) -> Result<LinuxI2CDevice, LinuxI2CError> {
        let file = OpenOptions::new().read(true).write(true).open(path)?;
        let mut device = LinuxI2CDevice {
            devfile: file,
            slave_address: 0, // will be set later
            pec: false,
        };
        device.set_slave_address(slave_address)?;
        device.set_smbus_pec(false)?;
        Ok(device)
    }

    /// Create a new I2CDevice for the specified path, without checking if the
    /// device is bound to a driver
    ///
    /// # Safety
    /// Using this can seriously confuse the original driver, and may cause all
    /// future communication to perform the wrong operations and/or return wrong results.
    pub unsafe fn force_new<P: AsRef<Path>>(
        path: P,
        slave_address: u16,
    ) -> Result<LinuxI2CDevice, LinuxI2CError> {
        let file = OpenOptions::new().read(true).write(true).open(path)?;
        let mut device = LinuxI2CDevice {
            devfile: file,
            slave_address: 0, // will be set later
            pec: false,
        };
        device.force_set_slave_address(slave_address)?;
        device.set_smbus_pec(false)?;
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
    pub fn set_slave_address(&mut self, slave_address: u16) -> Result<(), LinuxI2CError> {
        ffi::i2c_set_slave_address(self.as_raw_fd(), slave_address)?;
        self.slave_address = slave_address;
        Ok(())
    }

    /// Set the slave address for this device, even if it is already in use
    /// by a driver
    ///
    /// This is private; use `force_new` instead.
    unsafe fn force_set_slave_address(&mut self, slave_address: u16) -> Result<(), LinuxI2CError> {
        ffi::i2c_set_slave_address_force(self.as_raw_fd(), slave_address)?;
        self.slave_address = slave_address;
        Ok(())
    }

    /// Enable/Disable PEC support for this device
    ///
    /// Used only for SMBus transactions.  This request only has an effect if the
    /// the adapter has I2C_FUNC_SMBUS_PEC; it is still safe if not, it just
    /// doesn't have any effect.
    pub fn set_smbus_pec(&mut self, enable: bool) -> Result<(), LinuxI2CError> {
        ffi::i2c_set_smbus_pec(self.as_raw_fd(), enable)?;
        self.pec = enable;
        Ok(())
    }
}

impl I2CDevice for LinuxI2CDevice {
    type Error = LinuxI2CError;

    /// Read data from the device to fill the provided slice
    fn read(&mut self, data: &mut [u8]) -> Result<(), LinuxI2CError> {
        self.devfile.read(data).map_err(From::from).map(drop)
    }

    /// Write the provided buffer to the device
    fn write(&mut self, data: &[u8]) -> Result<(), LinuxI2CError> {
        self.devfile.write(data).map_err(From::from).map(drop)
    }

    /// This sends a single bit to the device, at the place of the Rd/Wr bit
    fn smbus_write_quick(&mut self, bit: bool) -> Result<(), LinuxI2CError> {
        ffi::i2c_smbus_write_quick(self.as_raw_fd(), bit).map_err(From::from)
    }

    /// Read a single byte from a device, without specifying a device register
    ///
    /// Some devices are so simple that this interface is enough; for
    /// others, it is a shorthand if you want to read the same register as in
    /// the previous SMBus command.
    fn smbus_read_byte(&mut self) -> Result<u8, LinuxI2CError> {
        ffi::i2c_smbus_read_byte(self.as_raw_fd()).map_err(From::from)
    }

    /// Write a single byte to a sdevice, without specifying a device register
    ///
    /// This is the opposite operation as smbus_read_byte.  As with read_byte,
    /// no register is specified.
    fn smbus_write_byte(&mut self, value: u8) -> Result<(), LinuxI2CError> {
        ffi::i2c_smbus_write_byte(self.as_raw_fd(), value).map_err(From::from)
    }

    /// Read a single byte from a device, from a designated register
    ///
    /// The register is specified through the Comm byte.
    fn smbus_read_byte_data(&mut self, register: u8) -> Result<u8, LinuxI2CError> {
        ffi::i2c_smbus_read_byte_data(self.as_raw_fd(), register).map_err(From::from)
    }

    /// Write a single byte to a specific register on a device
    ///
    /// The register is specified through the Comm byte.
    fn smbus_write_byte_data(&mut self, register: u8, value: u8) -> Result<(), LinuxI2CError> {
        ffi::i2c_smbus_write_byte_data(self.as_raw_fd(), register, value).map_err(From::from)
    }

    /// Read 2 bytes form a given register on a device
    fn smbus_read_word_data(&mut self, register: u8) -> Result<u16, LinuxI2CError> {
        ffi::i2c_smbus_read_word_data(self.as_raw_fd(), register).map_err(From::from)
    }

    /// Write 2 bytes to a given register on a device
    fn smbus_write_word_data(&mut self, register: u8, value: u16) -> Result<(), LinuxI2CError> {
        ffi::i2c_smbus_write_word_data(self.as_raw_fd(), register, value).map_err(From::from)
    }

    /// Select a register, send 16 bits of data to it, and read 16 bits of data
    fn smbus_process_word(&mut self, register: u8, value: u16) -> Result<u16, LinuxI2CError> {
        ffi::i2c_smbus_process_call(self.as_raw_fd(), register, value).map_err(From::from)
    }

    /// Read a block of up to 32 bytes from a device
    ///
    /// The actual number of bytes available to read is returned in the count
    /// byte.  This code returns a correctly sized vector containing the
    /// count bytes read from the device.
    fn smbus_read_block_data(&mut self, register: u8) -> Result<Vec<u8>, LinuxI2CError> {
        ffi::i2c_smbus_read_block_data(self.as_raw_fd(), register).map_err(From::from)
    }

    /// Read a block of up to 32 bytes from a device via i2c_smbus_i2c_read_block_data
    fn smbus_read_i2c_block_data(
        &mut self,
        register: u8,
        len: u8,
    ) -> Result<Vec<u8>, LinuxI2CError> {
        ffi::i2c_smbus_read_i2c_block_data(self.as_raw_fd(), register, len).map_err(From::from)
    }

    /// Write a block of up to 32 bytes to a device
    ///
    /// The opposite of the Block Read command, this writes up to 32 bytes to
    /// a device, to a designated register that is specified through the
    /// Comm byte. The amount of data is specified in the Count byte.
    fn smbus_write_block_data(&mut self, register: u8, values: &[u8]) -> Result<(), LinuxI2CError> {
        ffi::i2c_smbus_write_block_data(self.as_raw_fd(), register, values).map_err(From::from)
    }

    /// Write a block of up to 32 bytes from a device via i2c_smbus_i2c_write_block_data
    fn smbus_write_i2c_block_data(
        &mut self,
        register: u8,
        values: &[u8],
    ) -> Result<(), LinuxI2CError> {
        ffi::i2c_smbus_write_i2c_block_data(self.as_raw_fd(), register, values).map_err(From::from)
    }

    /// Select a register, send 1 to 31 bytes of data to it, and reads
    /// 1 to 31 bytes of data from it.
    fn smbus_process_block(
        &mut self,
        register: u8,
        values: &[u8],
    ) -> Result<Vec<u8>, LinuxI2CError> {
        ffi::i2c_smbus_process_call_block(self.as_raw_fd(), register, values).map_err(From::from)
    }
}

impl<'a> I2CTransfer<'a> for LinuxI2CDevice {
    type Error = LinuxI2CError;
    type Message = LinuxI2CMessage<'a>;

    /// Issue the provided sequence of I2C transactions
    fn transfer(&mut self, messages: &'a mut [Self::Message]) -> Result<u32, LinuxI2CError> {
        for msg in messages.iter_mut() {
            (*msg).addr = self.slave_address;
        }
        ffi::i2c_rdwr(self.as_raw_fd(), messages).map_err(From::from)
    }
}

impl LinuxI2CBus {
    /// Create a new LinuxI2CBus for the specified path
    pub fn new<P: AsRef<Path>>(path: P) -> Result<LinuxI2CBus, LinuxI2CError> {
        let file = OpenOptions::new().read(true).write(true).open(path)?;
        let bus = LinuxI2CBus { devfile: file };
        Ok(bus)
    }
}

/// Linux I2C message
pub type LinuxI2CMessage<'a> = ffi::i2c_msg;

impl<'a> I2CTransfer<'a> for LinuxI2CBus {
    type Error = LinuxI2CError;
    type Message = LinuxI2CMessage<'a>;

    /// Issue the provided sequence of I2C transactions
    fn transfer(&mut self, msgs: &'a mut [Self::Message]) -> Result<u32, LinuxI2CError> {
        ffi::i2c_rdwr(self.as_raw_fd(), msgs).map_err(From::from)
    }
}

bitflags! {
    /// Various flags used by the i2c_rdwr ioctl on Linux. For details, see
    /// https://www.kernel.org/doc/Documentation/i2c/i2c-protocol
    ///
    /// In general, these are for special cases and should not be needed
    pub struct I2CMessageFlags: u16 {
        /// Use ten bit addressing on this message
        const TEN_BIT_ADDRESS = 0x0010;
        /// Read data, from slave to master
        const READ = 0x0001;
        /// Force an I2C stop condition on this message
        const STOP = 0x8000;
        /// Avoid sending an I2C start condition on this message
        const NO_START = 0x4000;
        /// If you need to invert a 'read' command bit to a 'write'
        const INVERT_COMMAND = 0x2000;
        /// Force this message to ignore I2C negative acknowlegements
        const IGNORE_NACK = 0x1000;
        /// Force message to ignore acknowledgement
        const IGNORE_ACK = 0x0800;
        /// Allow the client to specify how many bytes it will send
        const USE_RECEIVE_LENGTH = 0x0400;
    }
}

impl<'a> I2CMessage<'a> for LinuxI2CMessage<'a> {
    fn read(data: &'a mut [u8]) -> LinuxI2CMessage {
        Self {
            addr: 0, // will be filled later
            flags: I2CMessageFlags::READ.bits(),
            len: data.len() as u16,
            buf: data.as_ptr(),
        }
    }

    fn write(data: &'a [u8]) -> LinuxI2CMessage {
        Self {
            addr: 0, // will be filled later
            flags: I2CMessageFlags::empty().bits(),
            len: data.len() as u16,
            buf: data.as_ptr(),
        }
    }
}

impl<'a> LinuxI2CMessage<'a> {
    /// Set the target device address for the message
    pub fn with_address(self, slave_address: u16) -> Self {
        Self {
            addr: slave_address,
            flags: self.flags,
            len: self.len,
            buf: self.buf,
        }
    }

    /// Set optional message flags
    pub fn with_flags(self, flags: I2CMessageFlags) -> Self {
        Self {
            addr: self.addr,
            flags: flags.bits(),
            len: self.len,
            buf: self.buf,
        }
    }
}
