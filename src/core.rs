// Copyright 2015, Paul Osborne <osbpau@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/license/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option.  This file may not be copied, modified, or distributed
// except according to those terms.

use byteorder::{ByteOrder, LittleEndian};

/// Error that occured while performing and I2C Operation
#[derive(Debug)]
pub enum I2CError {
    NotSupported,
    Other(&'static str),
}

/// Result of an I2C Operation
pub type I2CResult<T> = Result<T, I2CError>;

/// Interface to an I2C Slave Device from an I2C Master
///
/// Typical implementations will store state with references to the bus
/// in use and the address of the slave device.  The trait is based on the
/// Linux i2cdev interface.
pub trait I2CDevice {
    /// Read data from the device to fill the provided slice
    fn read(&mut self, data: &mut [u8]) -> I2CResult<()>;

    /// Write the provided buffer to the device
    fn write(&mut self, data: &[u8]) -> I2CResult<()>;

    /*
     * The following implementations are defaults that seek to match the
     * kernel implementation where possible.
     */

    /// This sends a single bit to the device, at the place of the Rd/Wr bit
    #[allow(unused_variables)]
    fn smbus_write_quick(&mut self, bit: bool) -> I2CResult<()> {
        // not supported by default; cannot be expressed with read/write
        // but I'm not aware of any real use cases
        Err(I2CError::NotSupported)
    }

    /// Read a single byte from a device, without specifying a device register
    ///
    /// Some devices are so simple that this interface is enough; for
    /// others, it is a shorthand if you want to read the same register as in
    /// the previous SMBus command.
    fn smbus_read_byte(&mut self) -> I2CResult<u8> {
        let mut buf = [0_u8];
        try!(self.read(&mut buf));
        Ok(buf[0])
    }

    /// Write a single byte to a device, without specifying a device register
    ///
    /// This is the opposite operation as smbus_read_byte.  As with read_byte,
    /// no register is specified.
    fn smbus_write_byte(&mut self, value: u8) -> I2CResult<()> {
        self.write(&mut [value])
    }

    /// Read a single byte from a device, from a designated register
    ///
    /// The register is specified through the Comm byte.
    fn smbus_read_byte_data(&mut self, register: u8) -> I2CResult<u8> {
        try!(self.smbus_write_byte(register));
        self.smbus_read_byte()
    }

    /// Write a single byte to a specific register on a device
    ///
    /// The register is specified through the Comm byte.
    fn smbus_write_byte_data(&mut self, register: u8, value: u8) -> I2CResult<()> {
        self.write(&mut [register, value])
    }

    /// Read 2 bytes form a given register on a device (lsb first)
    fn smbus_read_word_data(&mut self, register: u8) -> I2CResult<u16> {
        let mut buf: [u8; 2] = [0x00; 2];
        try!(self.smbus_write_byte(register));
        try!(self.read(&mut buf));
        Ok(LittleEndian::read_u16(&buf))
    }

    /// Write 2 bytes to a given register on a device (lsb first)
    fn smbus_write_word_data(&mut self, register: u8, value: u16) -> I2CResult<()> {
        let mut buf: [u8; 3] = [register, 0, 0];
        LittleEndian::write_u16(&mut buf[1..], value);
        self.write(&buf)
    }

    /// Select a register, send 16 bits of data to it, and read 16 bits of data
    fn smbus_process_word(&mut self, register: u8, value: u16) -> I2CResult<u16> {
        let mut buf: [u8; 2] = [0x00; 2];
        try!(self.smbus_write_word_data(register, value));
        try!(self.read(&mut buf));
        Ok(LittleEndian::read_u16(&buf))
    }

    /// Read a block of up to 32 bytes from a device
    ///
    /// The actual number of bytes available to read is returned in the count
    /// byte.  This code returns a correctly sized vector containing the
    /// count bytes read from the device.
    #[allow(unused_variables)]
    fn smbus_read_block_data(&mut self, register: u8) -> I2CResult<Vec<u8>> {
        Err(I2CError::NotSupported)
    }

    /// Write a block of up to 32 bytes to a device
    ///
    /// The opposite of the Block Read command, this writes up to 32 bytes to
    /// a device, to a designated register that is specified through the
    /// Comm byte. The amount of data is specified in the Count byte.
    #[allow(unused_variables)]
    fn smbus_write_block_data(&mut self, register: u8, values: &[u8]) -> I2CResult<()> {
        Err(I2CError::NotSupported)
    }

    /// Select a register, send 1 to 31 bytes of data to it, and reads
    /// 1 to 31 bytes of data from it.
    #[allow(unused_variables)]
    fn smbus_process_block(&mut self, register: u8, values: &[u8]) -> I2CResult<()> {
        Err(I2CError::NotSupported)
    }
}