// Copyright 2015, Paul Osborne <osbpau@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/license/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option.  This file may not be copied, modified, or distributed
// except according to those terms.
use core::{I2CDevice, I2CMessage, I2CTransfer};
use std::io;

/// I2C mock result type
pub type I2CResult<T> = io::Result<T>;

/// Mock I2C device register map
pub struct I2CRegisterMap {
    registers: [u8; 0xFF],
    offset: usize,
}

impl Default for I2CRegisterMap {
    fn default() -> Self {
        Self::new()
    }
}

impl I2CRegisterMap {
    /// Create new mock I2C register map
    pub fn new() -> I2CRegisterMap {
        I2CRegisterMap {
            registers: [0x00; 0xFF],
            offset: 0,
        }
    }

    /// Set several registers starting at the given offset
    pub fn write_regs(&mut self, offset: usize, data: &[u8]) {
        println!("WRITE | 0x{:X} : {:?}", offset, data);
        self.registers[offset..(data.len() + offset)].clone_from_slice(data);
    }
}

impl I2CRegisterMap {
    /// Read data from the device to fill the provided slice
    fn read(&mut self, data: &mut [u8]) -> I2CResult<()> {
        let len = data.len();
        data.clone_from_slice(&self.registers[self.offset..(self.offset + len)]);
        println!("READ  | 0x{:X} : {:?}", self.offset - data.len(), data);
        Ok(())
    }

    /// Write the provided buffer to the device
    fn write(&mut self, data: &[u8]) -> I2CResult<()> {
        // ASSUMPTION: first byte sets the offset
        // ASSUMPTION: write has length of at least one (will panic)
        let offset = data[0] as usize;
        let remdata = &data[1..];
        self.write_regs(offset, remdata);
        self.offset = offset + remdata.len();
        Ok(())
    }
}

/// Mock I2C device exposing a register map
#[derive(Default)]
pub struct MockI2CDevice {
    /// I2C register map
    pub regmap: I2CRegisterMap,
}

impl MockI2CDevice {
    /// Create a new mock I2C device
    pub fn new() -> MockI2CDevice {
        MockI2CDevice {
            regmap: I2CRegisterMap::new(),
        }
    }
}

impl I2CDevice for MockI2CDevice {
    type Error = io::Error;

    fn read(&mut self, data: &mut [u8]) -> I2CResult<()> {
        self.regmap.read(data)
    }

    fn write(&mut self, data: &[u8]) -> I2CResult<()> {
        self.regmap.write(data)
    }

    fn smbus_write_quick(&mut self, _bit: bool) -> I2CResult<()> {
        unimplemented!()
    }

    fn smbus_read_block_data(&mut self, _register: u8) -> I2CResult<Vec<u8>> {
        unimplemented!()
    }

    fn smbus_write_block_data(&mut self, _register: u8, _values: &[u8]) -> I2CResult<()> {
        unimplemented!()
    }

    fn smbus_process_block(&mut self, _register: u8, _values: &[u8]) -> I2CResult<Vec<u8>> {
        unimplemented!()
    }

    fn smbus_read_i2c_block_data(&mut self, _register: u8, _len: u8) -> I2CResult<Vec<u8>> {
        unimplemented!()
    }

    fn smbus_write_i2c_block_data(&mut self, _register: u8, _values: &[u8]) -> I2CResult<()> {
        unimplemented!()
    }
}

#[derive(Debug)]
enum MessageType<'a> {
    Write(&'a [u8]),
    Read(&'a mut [u8]),
}

/// Mock I2C message
pub struct MockI2CMessage<'a> {
    msg_type: MessageType<'a>,
}

impl<'a> I2CMessage<'a> for MockI2CMessage<'a> {
    fn read(data: &'a mut [u8]) -> Self {
        Self {
            msg_type: MessageType::Read(data),
        }
    }

    /// Write data to device
    fn write(data: &'a [u8]) -> Self {
        Self {
            msg_type: MessageType::Write(data),
        }
    }
}

impl<'a> I2CTransfer<'a> for MockI2CDevice
where
    MockI2CDevice: I2CDevice,
{
    type Error = io::Error;
    type Message = MockI2CMessage<'a>;

    /// Issue the provided sequence of I2C transactions
    fn transfer(&mut self, messages: &'a mut [Self::Message]) -> Result<u32, Self::Error> {
        for msg in messages.iter_mut() {
            match &mut msg.msg_type {
                MessageType::Read(data) => self.read(data)?,
                MessageType::Write(data) => self.write(data)?,
            }
        }
        Ok(messages.len() as u32)
    }
}
