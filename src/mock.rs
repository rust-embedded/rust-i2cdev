// Copyright 2015, Paul Osborne <osbpau@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/license/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option.  This file may not be copied, modified, or distributed
// except according to those terms.
use core::*;

pub struct I2CRegisterMap {
    registers: [u8; 0xFF],
    offset: usize,
}

impl I2CRegisterMap {

    pub fn new() -> I2CRegisterMap {
        I2CRegisterMap {
            registers: [0x00; 0xFF],
            offset: 0,
        }
    }

    pub fn write_regs(&mut self, offset: usize, data: &[u8]) {
        println!("WRITE | 0x{:X} : {:?}", offset, data);
        for i in 0..data.len() {
            self.registers[offset + i] = data[i];
        }
    }
}

impl I2CDevice for I2CRegisterMap {
    /// Read data from the device to fill the provided slice
    fn read(&mut self, data: &mut [u8]) -> I2CResult<()> {
        for i in 0..data.len() {
            data[i] = self.registers[self.offset];
            self.offset += 1;
        }
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

pub struct MockI2CDevice {
    pub regmap: I2CRegisterMap,
}


impl MockI2CDevice {
    pub fn new() -> MockI2CDevice {
        MockI2CDevice {
            regmap: I2CRegisterMap::new(),
        }
    }
}

impl I2CDevice for MockI2CDevice {
    /// Read data from the device to fill the provided slice
    fn read(&mut self, data: &mut [u8]) -> I2CResult<()> {
        self.regmap.read(data)
    }

    /// Write the provided buffer to the device
    fn write(&mut self, data: &[u8]) -> I2CResult<()> {
        self.regmap.write(data)
    }

    /// This sends a single bit to the device, at the place of the Rd/Wr bit
    fn smbus_write_quick(&mut self, bit: bool) -> I2CResult<()> {
        self.regmap.smbus_write_quick(bit)
    }

    /// Read a single byte from a device, without specifying a device register
    ///
    /// Some devices are so simple that this interface is enough; for
    /// others, it is a shorthand if you want to read the same register as in
    /// the previous SMBus command.
    fn smbus_read_byte(&mut self) -> I2CResult<u8> {
        self.regmap.smbus_read_byte()
    }

    /// Write a single byte to a device, without specifying a device register
    ///
    /// This is the opposite operation as smbus_read_byte.  As with read_byte,
    /// no register is specified.
    fn smbus_write_byte(&mut self, value: u8) -> I2CResult<()> {
        self.regmap.smbus_write_byte(value)
    }

    /// Read a single byte from a device, from a designated register
    ///
    /// The register is specified through the Comm byte.
    fn smbus_read_byte_data(&mut self, register: u8) -> I2CResult<u8> {
        self.regmap.smbus_read_byte_data(register)
    }

    /// Write a single byte to a specific register on a device
    ///
    /// The register is specified through the Comm byte.
    fn smbus_write_byte_data(&mut self, register: u8, value: u8) -> I2CResult<()> {
        self.regmap.smbus_write_byte_data(register, value)
    }

    /// Read 2 bytes form a given register on a device
    fn smbus_read_word_data(&mut self, register: u8) -> I2CResult<u16> {
        self.regmap.smbus_read_word_data(register)
    }

    /// Write 2 bytes to a given register on a device
    fn smbus_write_word_data(&mut self, register: u8, value: u16) -> I2CResult<()> {
        self.regmap.smbus_write_word_data(register, value)
    }

    /// Select a register, send 16 bits of data to it, and read 16 bits of data
    fn smbus_process_word(&mut self, register: u8, value: u16) -> I2CResult<u16> {
        self.regmap.smbus_process_word(register, value)
    }

    /// Read a block of up to 32 bytes from a device
    ///
    /// The actual number of bytes available to read is returned in the count
    /// byte.  This code returns a correctly sized vector containing the
    /// count bytes read from the device.
    fn smbus_read_block_data(&mut self, register: u8) -> I2CResult<Vec<u8>> {
        self.regmap.smbus_read_block_data(register)
    }

    /// Write a block of up to 32 bytes to a device
    ///
    /// The opposite of the Block Read command, this writes up to 32 bytes to
    /// a device, to a designated register that is specified through the
    /// Comm byte. The amount of data is specified in the Count byte.
    fn smbus_write_block_data(&mut self, register: u8, values: &[u8]) -> I2CResult<()> {
        self.regmap.smbus_write_block_data(register, values)
    }

    /// Select a register, send 1 to 31 bytes of data to it, and reads
    /// 1 to 31 bytes of data from it.
    fn smbus_process_block(&mut self, register: u8, values: &[u8]) -> I2CResult<()> {
        self.regmap.smbus_process_block(register, values)
    }
}
