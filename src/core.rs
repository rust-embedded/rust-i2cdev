// Copyright 2015, Paul Osborne <osbpau@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/license/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option.  This file may not be copied, modified, or distributed
// except according to those terms.

use byteorder::{ByteOrder, LittleEndian};
use std::error::Error;

/// Interface to an I2C Slave Device from an I2C Master
///
/// Typical implementations will store state with references to the bus
/// in use and the address of the slave device.  The trait is based on the
/// Linux i2cdev interface.
pub trait I2CDevice {
    type Error: Error;

    /// Read data from the device to fill the provided slice
    fn read(&mut self, data: &mut [u8]) -> Result<(), Self::Error>;

    /// Write the provided buffer to the device
    fn write(&mut self, data: &[u8]) -> Result<(), Self::Error>;

    /// This sends a single bit to the device, at the place of the Rd/Wr bit
    fn smbus_write_quick(&mut self, bit: bool) -> Result<(), Self::Error>;

    /// Read a single byte from a device, without specifying a device register
    ///
    /// Some devices are so simple that this interface is enough; for
    /// others, it is a shorthand if you want to read the same register as in
    /// the previous SMBus command.
    fn smbus_read_byte(&mut self) -> Result<u8, Self::Error> {
        let mut buf = [0_u8];
        self.read(&mut buf)?;
        Ok(buf[0])
    }

    /// Write a single byte to a device, without specifying a device register
    ///
    /// This is the opposite operation as smbus_read_byte.  As with read_byte,
    /// no register is specified.
    fn smbus_write_byte(&mut self, value: u8) -> Result<(), Self::Error> {
        self.write(&mut [value])
    }

    /// Read a single byte from a device, from a designated register
    ///
    /// The register is specified through the Comm byte.
    fn smbus_read_byte_data(&mut self, register: u8) -> Result<u8, Self::Error> {
        self.smbus_write_byte(register)?;
        self.smbus_read_byte()
    }

    /// Write a single byte to a specific register on a device
    ///
    /// The register is specified through the Comm byte.
    fn smbus_write_byte_data(&mut self, register: u8, value: u8) -> Result<(), Self::Error> {
        self.write(&mut [register, value])
    }

    /// Read 2 bytes from a given register on a device (lsb first)
    fn smbus_read_word_data(&mut self, register: u8) -> Result<u16, Self::Error> {
        let mut buf: [u8; 2] = [0x00; 2];
        self.smbus_write_byte(register)?;
        self.read(&mut buf)?;
        Ok(LittleEndian::read_u16(&buf))
    }

    /// Write 2 bytes to a given register on a device (lsb first)
    fn smbus_write_word_data(&mut self, register: u8, value: u16) -> Result<(), Self::Error> {
        let mut buf: [u8; 3] = [register, 0, 0];
        LittleEndian::write_u16(&mut buf[1..], value);
        self.write(&buf)
    }

    /// Select a register, send 16 bits of data to it, and read 16 bits of data
    fn smbus_process_word(&mut self, register: u8, value: u16) -> Result<u16, Self::Error> {
        let mut buf: [u8; 2] = [0x00; 2];
        self.smbus_write_word_data(register, value)?;
        self.read(&mut buf)?;
        Ok(LittleEndian::read_u16(&buf))
    }

    /// Read a block of up to 32 bytes from a device
    ///
    /// The actual number of bytes available to read is returned in the count
    /// byte.  This code returns a correctly sized vector containing the
    /// count bytes read from the device.
    fn smbus_read_block_data(&mut self, register: u8) -> Result<Vec<u8>, Self::Error>;

    /// Read a block of up to 32 bytes from a device
    ///
    /// Uses read_i2c_block_data instead read_block_data.
    fn smbus_read_i2c_block_data(&mut self, register: u8, len: u8) -> Result<Vec<u8>, Self::Error>;

    /// Write a block of up to 32 bytes to a device
    ///
    /// The opposite of the Block Read command, this writes up to 32 bytes to
    /// a device, to a designated register that is specified through the
    /// Comm byte. The amount of data is specified in the Count byte.
    fn smbus_write_block_data(&mut self, register: u8, values: &[u8]) -> Result<(), Self::Error>;

    /// Write a block of up to 32 bytes from a device
    ///
    /// Uses write_i2c_block_data instead write_block_data.
    fn smbus_write_i2c_block_data(&mut self, register: u8, values: &[u8]) -> Result<(), Self::Error>;

    /// Select a register, send 1 to 31 bytes of data to it, and reads
    /// 1 to 31 bytes of data from it.
    fn smbus_process_block(&mut self, register: u8, values: &[u8]) -> Result<Vec<u8>, Self::Error>;
}

/// Interface to an I2C Bus from an I2C Master
///
/// This is used when the client wants to interact directly with the bus
/// without specifying an I2C slave address up-front, either because it needs
/// to communicate with multiple addresses without creatings separate
/// I2CDevice objects, or because it wants to make used of the I2C_RDWR ioctl
/// which allows the client to send and transmit multiple sets I2C data in a
/// single operation, potentially to different I2C slave addresses.
///
/// Typical implementations will store state with references to the bus
/// in use.  The trait is based on the Linux i2cdev interface.
pub trait I2CBus {
    type Error: Error;

    // Performs multiple serially chained I2C read/write transactions.  On
    // success the return code is the number of successfully executed
    // transactions
    fn rdwr<'a>(&mut self, msgs: &mut Vec<I2CMsg<'a>>) -> Result<i32, Self::Error>;
}

bitflags! {
    pub struct I2CMsgFlags: u16 {
        /// this is a ten bit chip address
        const I2C_M_TEN = 0x0010;
        /// read data, from slave to master
        const I2C_M_RD = 0x0001;
        /// if I2C_FUNC_PROTOCOL_MANGLING
        const I2C_M_STOP = 0x8000;
        /// if I2C_FUNC_NOSTART
        const I2C_M_NOSTART = 0x4000;
        /// if I2C_FUNC_PROTOCOL_MANGLING
        const I2C_M_REV_DIR_ADDR = 0x2000;
        /// if I2C_FUNC_PROTOCOL_MANGLING
        const I2C_M_IGNORE_NAK = 0x1000;
        /// if I2C_FUNC_PROTOCOL_MANGLING
        const I2C_M_NO_RD_ACK = 0x0800;
        /// length will be first received byte
        const I2C_M_RECV_LEN = 0x0400;
    }
}

/// Rust version of i2c_msg
pub struct I2CMsg<'a> {
    /// slave address
    pub(crate) addr: u16,
    /// serialized I2CMsgFlags
    pub(crate) flags: u16,

    /// msg length comes from msg Vector length

    /// msg data to be sent/received
    pub(crate) data: &'a mut Vec<u8>,
}

impl<'a> I2CMsg<'a> {
    /// Create I2CMsg from address and data buffer
    pub fn new(addr: u16, data: &'a mut Vec<u8>) -> Self {
        I2CMsg {
            addr,
            flags: 0,
            data
        }
    }

    /// Set flags
    pub fn set_flags(&mut self, flags: u16) {
        self.flags = flags;
    }

    /// Get flags
    pub fn flags(&mut self) -> u16 {
        self.flags
    }

    /// Set addr
    pub fn set_addr(&mut self, addr: u16) {
        self.addr = addr;
    }

    /// Get addr
    pub fn addr(&mut self) -> u16 {
        self.addr
    }

    /// Set data
    pub fn set_data(&mut self, data: &'a mut Vec<u8>) {
        self.data = data;
    }

    /// Get addr
    pub fn data(&mut self) -> Vec<u8> {
        self.data.clone()
    }

    /// Sets the read flag
    pub fn set_read(&mut self) {
        self.flags |= I2CMsgFlags::I2C_M_RD.bits;
    }
}
