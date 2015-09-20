// Copyright 2015, Paul Osborne <osbpau@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/license/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option.  This file may not be copied, modified, or distributed
// except according to those terms.
use std::io::Cursor;
use std::io::prelude::*;
use std::io::SeekFrom;
use core::*;


pub struct MockI2CDevice {
    pub regmap: [u8; 0xFF],
    pub reads: Vec<Vec<u8>>,
    pub writes: Vec<Vec<u8>>,
}


impl MockI2CDevice {
    pub fn new() -> MockI2CDevice {
        MockI2CDevice {
            regmap: [0; 0xFF],
            reads: Vec::new(),
            writes: Vec::new(),
        }
    }

    pub fn queue_read(&mut self, data: &[u8]) {
        let mut vec: Vec<u8> = Vec::new();
        vec.extend(data);
        self.reads.push(vec);
    }
}

impl I2CDevice for MockI2CDevice {
    /// Read data from the device to fill the provided slice
    fn read(&mut self, data: &mut [u8]) -> I2CResult<()> {
        match self.reads.pop() {
            Some(queued_data) => {
                if data.len() != queued_data.len() {
                    Err(I2CError::Other("Queued data has wrong length"))
                } else {
                    for i in 0..data.len() {
                        data[i] = queued_data[i]
                    }
                    Ok(())
                }
            },
            None => Err(I2CError::Other("No read queued")),
        }
    }

    /// Write the provided buffer to the device
    fn write(&mut self, data: &[u8]) -> I2CResult<()> {
        let mut vec: Vec<u8> = Vec::new();
        vec.extend(data);
        self.writes.push(vec);
        Ok(())
    }

    /// This sends a single bit to the device, at the place of the Rd/Wr bit
    fn smbus_write_quick(&mut self, bit: bool) -> I2CResult<()> {
        Err(I2CError::Other("Not Implemented - smbus_write_quick"))
    }

    /// Read a single byte from a device, without specifying a device register
    ///
    /// Some devices are so simple that this interface is enough; for
    /// others, it is a shorthand if you want to read the same register as in
    /// the previous SMBus command.
    fn smbus_read_byte(&mut self) -> I2CResult<u8> {
        Err(I2CError::Other("Not Implemented - smbus_read_byte"))
    }

    /// Write a single byte to a device, without specifying a device register
    ///
    /// This is the opposite operation as smbus_read_byte.  As with read_byte,
    /// no register is specified.
    fn smbus_write_byte(&mut self, value: u8) -> I2CResult<()> {
        Err(I2CError::Other("Not Implemented - smbus_write_byte"))
    }

    /// Read a single byte from a device, from a designated register
    ///
    /// The register is specified through the Comm byte.
    fn smbus_read_byte_data(&mut self, register: u8) -> I2CResult<u8> {
        Err(I2CError::Other("Not Implemented - smbus_read_byte_data"))
    }

    /// Write a single byte to a specific register on a device
    ///
    /// The register is specified through the Comm byte.
    fn smbus_write_byte_data(&mut self, register: u8, value: u8) -> I2CResult<()> {
        self.regmap[register as usize] = value;
        Ok(())
    }

    /// Read 2 bytes form a given register on a device
    fn smbus_read_word_data(&mut self, register: u8) -> I2CResult<u16> {
        let lsb = self.regmap[register as usize];
        let msb = self.regmap[register as usize];
        return Ok((msb as u16) << 8 | lsb as u16);
    }

    /// Write 2 bytes to a given register on a device
    fn smbus_write_word_data(&mut self, register: u8, value: u16) -> I2CResult<()> {
        Err(I2CError::Other("Not Implemented - smbus_write_word_data"))
    }

    /// Select a register, send 16 bits of data to it, and read 16 bits of data
    fn smbus_process_word(&mut self, register: u8, value: u16) -> I2CResult<u16> {
        Err(I2CError::Other("Not Implemented - smbus_process_word"))
    }

    /// Read a block of up to 32 bytes from a device
    ///
    /// The actual number of bytes available to read is returned in the count
    /// byte.  This code returns a correctly sized vector containing the
    /// count bytes read from the device.
    fn smbus_read_block_data(&mut self, register: u8) -> I2CResult<Vec<u8>> {
        Err(I2CError::Other("Not Implemented - smbus_read_block_data"))
    }

    /// Write a block of up to 32 bytes to a device
    ///
    /// The opposite of the Block Read command, this writes up to 32 bytes to
    /// a device, to a designated register that is specified through the
    /// Comm byte. The amount of data is specified in the Count byte.
    fn smbus_write_block_data(&mut self, register: u8, values: &[u8]) -> I2CResult<()> {
        Err(I2CError::Other("Not Implemented - smbus_write_block_data"))
    }

    /// Select a register, send 1 to 31 bytes of data to it, and reads
    /// 1 to 31 bytes of data from it.
    fn smbus_process_block(&mut self, register: u8, values: &[u8]) -> I2CResult<()> {
        Err(I2CError::Other("Not Implemented"))
    }
}
