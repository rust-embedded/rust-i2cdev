// Copyright 2015, Paul Osborne <osbpau@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/license/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option.  This file may not be copied, modified, or distributed
// except according to those terms.

#![allow(dead_code)]
#![allow(non_camel_case_types)]

use byteorder::{NativeEndian, ReadBytesExt, WriteBytesExt};
use nix;
use std::io::Cursor;
use std::mem;
use std::os::unix::prelude::*;
use std::ptr;

pub type I2CError = nix::Error;

#[repr(C)]
pub struct i2c_msg {
    /// slave address
    pub(crate) addr: u16,
    /// serialized I2CMsgFlags
    pub(crate) flags: u16,
    /// msg length
    pub(crate) len: u16,
    /// pointer to msg data
    pub(crate) buf: *const u8,
}

bitflags! {
    struct I2CFunctions: u32 {
        const I2C_FUNC_I2C = 0x0000_0001;
        const I2C_FUNC_10BIT_ADDR = 0x0000_0002;
        const I2C_FUNC_PROTOCOL_MANGLING = 0x0000_0004; /* I2C_M_IGNORE_NAK etc. */
        const I2C_FUNC_SMBUS_PEC = 0x0000_0008;
        const I2C_FUNC_NOSTART = 0x0000_0010; /* I2C_M_NOSTART */
        const I2C_FUNC_SMBUS_BLOCK_PROC_CALL = 0x0000_8000; /* SMBus 2.0 */
        const I2C_FUNC_SMBUS_QUICK = 0x0001_0000;
        const I2C_FUNC_SMBUS_READ_BYTE = 0x0002_0000;
        const I2C_FUNC_SMBUS_WRITE_BYTE = 0x0004_0000;
        const I2C_FUNC_SMBUS_READ_BYTE_DATA = 0x0008_0000;
        const I2C_FUNC_SMBUS_WRITE_BYTE_DATA = 0x0010_0000;
        const I2C_FUNC_SMBUS_READ_WORD_DATA = 0x0020_0000;
        const I2C_FUNC_SMBUS_WRITE_WORD_DATA = 0x0040_0000;
        const I2C_FUNC_SMBUS_PROC_CALL = 0x0080_0000;
        const I2C_FUNC_SMBUS_READ_BLOCK_DATA = 0x0100_0000;
        const I2C_FUNC_SMBUS_WRITE_BLOCK_DATA  = 0x0200_0000;
        const I2C_FUNC_SMBUS_READ_I2C_BLOCK = 0x0400_0000; /* I2C-like block xfer  */
        const I2C_FUNC_SMBUS_WRITE_I2C_BLOCK = 0x0800_0000; /* w/ 1-byte reg. addr. */

        const I2C_FUNC_SMBUS_BYTE = (I2CFunctions::I2C_FUNC_SMBUS_READ_BYTE.bits |
                                     I2CFunctions::I2C_FUNC_SMBUS_WRITE_BYTE.bits);
        const I2C_FUNC_SMBUS_BYTE_DATA = (I2CFunctions::I2C_FUNC_SMBUS_READ_BYTE_DATA.bits |
                                          I2CFunctions::I2C_FUNC_SMBUS_WRITE_BYTE_DATA.bits);
        const I2C_FUNC_SMBUS_WORD_DATA = (I2CFunctions::I2C_FUNC_SMBUS_READ_WORD_DATA.bits |
                                          I2CFunctions::I2C_FUNC_SMBUS_WRITE_WORD_DATA.bits);
        const I2C_FUNC_SMBUS_BLOCK_DATA = (I2CFunctions::I2C_FUNC_SMBUS_READ_BLOCK_DATA.bits |
                                           I2CFunctions::I2C_FUNC_SMBUS_WRITE_BLOCK_DATA.bits);
        const I2C_FUNC_SMBUS_I2C_BLOCK = (I2CFunctions::I2C_FUNC_SMBUS_READ_I2C_BLOCK.bits |
                                          I2CFunctions::I2C_FUNC_SMBUS_WRITE_I2C_BLOCK.bits);
        const I2C_FUNC_SMBUS_EMUL = (I2CFunctions::I2C_FUNC_SMBUS_QUICK.bits |
                                     I2CFunctions::I2C_FUNC_SMBUS_BYTE.bits |
                                     I2CFunctions::I2C_FUNC_SMBUS_BYTE_DATA.bits |
                                     I2CFunctions::I2C_FUNC_SMBUS_WORD_DATA.bits |
                                     I2CFunctions::I2C_FUNC_SMBUS_PROC_CALL.bits |
                                     I2CFunctions::I2C_FUNC_SMBUS_WRITE_BLOCK_DATA.bits |
                                     I2CFunctions::I2C_FUNC_SMBUS_I2C_BLOCK.bits |
                                     I2CFunctions::I2C_FUNC_SMBUS_PEC.bits);
    }
}

/// As specified in SMBus standard
const I2C_SMBUS_BLOCK_MAX: u8 = 32;

// In C, this is a union, but the largest item is clearly
// the largest.  Rust does not have unions at this time,
// so we improvise.  See https://github.com/rust-lang/rust/issues/5492
//
// union i2c_smbus_data {
//     __u8 byte;
//     __u16 word;
//     __u8 block[I2C_SMBUS_BLOCK_MAX + 2]; /* block[0] is used for length */
//                            /* and one more for user-space compatibility */
// };
#[repr(C)]
struct i2c_smbus_data {
    block: [u8; (I2C_SMBUS_BLOCK_MAX + 2) as usize],
}

impl i2c_smbus_data {
    fn empty() -> i2c_smbus_data {
        unsafe { mem::zeroed() }
    }
}

#[repr(u8)]
enum I2CSMBusReadWrite {
    I2C_SMBUS_READ = 1,
    I2C_SMBUS_WRITE = 0,
}

#[repr(u32)]
enum I2CSMBusSize {
    I2C_SMBUS_QUICK = 0,
    I2C_SMBUS_BYTE = 1,
    I2C_SMBUS_BYTE_DATA = 2,
    I2C_SMBUS_WORD_DATA = 3,
    I2C_SMBUS_PROC_CALL = 4,
    I2C_SMBUS_BLOCK_DATA = 5,
    I2C_SMBUS_I2C_BLOCK_BROKEN = 6,
    I2C_SMBUS_BLOCK_PROC_CALL = 7, // SMBus 2.0
    I2C_SMBUS_I2C_BLOCK_DATA = 8,
}

// from include/uapi/linux/i2c-dev.h
const I2C_RETRIES: u16 = 0x0701;
const I2C_TIMEOUT: u16 = 0x0702;
const I2C_SLAVE: u16 = 0x0703;
const I2C_SLAVE_FORCE: u16 = 0x0706;
const I2C_TENBIT: u16 = 0x0704;
const I2C_FUNCS: u16 = 0x0705;
const I2C_RDWR: u16 = 0x0707;
const I2C_PEC: u16 = 0x0708;
const I2C_SMBUS: u16 = 0x0720;
const I2C_RDRW_IOCTL_MAX_MSGS: u8 = 42;

/// This is the structure as used in the I2C_SMBUS ioctl call
#[repr(C)]
pub struct i2c_smbus_ioctl_data {
    // __u8 read_write;
    read_write: u8,
    // __u8 command;
    command: u8,
    // __u32 size;
    size: u32,
    // union i2c_smbus_data __user *data;
    data: *mut i2c_smbus_data,
}

/// This is the structure as used in the I2C_RDWR ioctl call
// see linux/i2c-dev.h
#[repr(C)]
pub struct i2c_rdwr_ioctl_data {
    // struct i2c_msg __user *msgs;
    msgs: *mut i2c_msg,
    // __u32 nmsgs;
    nmsgs: u32,
}

mod ioctl {
    pub use super::i2c_rdwr_ioctl_data;
    pub use super::i2c_smbus_ioctl_data;
    use super::{I2C_PEC, I2C_RDWR, I2C_SLAVE, I2C_SLAVE_FORCE, I2C_SMBUS};

    ioctl_write_int_bad!(set_i2c_slave_address, I2C_SLAVE);
    ioctl_write_int_bad!(set_i2c_slave_address_force, I2C_SLAVE_FORCE);
    ioctl_write_int_bad!(set_smbus_pec, I2C_PEC);
    ioctl_write_ptr_bad!(i2c_smbus, I2C_SMBUS, i2c_smbus_ioctl_data);
    ioctl_write_ptr_bad!(i2c_rdwr, I2C_RDWR, i2c_rdwr_ioctl_data);
}

pub fn i2c_set_slave_address(fd: RawFd, slave_address: u16) -> Result<(), nix::Error> {
    unsafe {
        ioctl::set_i2c_slave_address(fd, i32::from(slave_address))?;
    }
    Ok(())
}

pub fn i2c_set_slave_address_force(fd: RawFd, slave_address: u16) -> Result<(), nix::Error> {
    unsafe {
        ioctl::set_i2c_slave_address_force(fd, i32::from(slave_address))?;
    }
    Ok(())
}

pub fn i2c_set_smbus_pec(fd: RawFd, enable: bool) -> Result<(), nix::Error> {
    unsafe {
        ioctl::set_smbus_pec(fd, i32::from(enable))?;
    }
    Ok(())
}

unsafe fn i2c_smbus_access(
    fd: RawFd,
    read_write: I2CSMBusReadWrite,
    command: u8, // can be address or something else
    size: I2CSMBusSize,
    data: *mut i2c_smbus_data,
) -> Result<(), I2CError> {
    let args = i2c_smbus_ioctl_data {
        read_write: read_write as u8,
        command,
        size: size as u32,
        data,
    };

    // remove type information
    ioctl::i2c_smbus(fd, &args).map(drop)
}

#[inline]
pub fn i2c_smbus_write_quick(fd: RawFd, bit: bool) -> Result<(), I2CError> {
    let read_write = if bit {
        I2CSMBusReadWrite::I2C_SMBUS_READ
    } else {
        I2CSMBusReadWrite::I2C_SMBUS_WRITE
    };
    unsafe {
        i2c_smbus_access(
            fd,
            read_write,
            0,
            I2CSMBusSize::I2C_SMBUS_QUICK,
            ptr::null_mut(),
        )
    }
}

#[inline]
pub fn i2c_smbus_read_byte(fd: RawFd) -> Result<u8, I2CError> {
    let mut data = i2c_smbus_data::empty();
    unsafe {
        i2c_smbus_access(
            fd,
            I2CSMBusReadWrite::I2C_SMBUS_READ,
            0,
            I2CSMBusSize::I2C_SMBUS_BYTE,
            &mut data,
        )?
    }
    Ok(data.block[0])
}

#[inline]
pub fn i2c_smbus_write_byte(fd: RawFd, value: u8) -> Result<(), I2CError> {
    unsafe {
        i2c_smbus_access(
            fd,
            I2CSMBusReadWrite::I2C_SMBUS_WRITE,
            value,
            I2CSMBusSize::I2C_SMBUS_BYTE,
            ptr::null_mut(),
        )
    }
}

#[inline]
pub fn i2c_smbus_read_byte_data(fd: RawFd, register: u8) -> Result<u8, I2CError> {
    let mut data = i2c_smbus_data::empty();
    unsafe {
        i2c_smbus_access(
            fd,
            I2CSMBusReadWrite::I2C_SMBUS_READ,
            register,
            I2CSMBusSize::I2C_SMBUS_BYTE_DATA,
            &mut data,
        )?;
    }
    Ok(data.block[0])
}

#[inline]
pub fn i2c_smbus_write_byte_data(fd: RawFd, register: u8, value: u8) -> Result<(), I2CError> {
    let mut data = i2c_smbus_data::empty();
    data.block[0] = value;
    unsafe {
        i2c_smbus_access(
            fd,
            I2CSMBusReadWrite::I2C_SMBUS_WRITE,
            register,
            I2CSMBusSize::I2C_SMBUS_BYTE_DATA,
            &mut data,
        )
    }
}

#[inline]
pub fn i2c_smbus_read_word_data(fd: RawFd, register: u8) -> Result<u16, I2CError> {
    let mut data = i2c_smbus_data::empty();
    unsafe {
        i2c_smbus_access(
            fd,
            I2CSMBusReadWrite::I2C_SMBUS_READ,
            register,
            I2CSMBusSize::I2C_SMBUS_WORD_DATA,
            &mut data,
        )?;
    };

    Ok(Cursor::new(&data.block[..])
        .read_u16::<NativeEndian>()
        .unwrap())
}

#[inline]
pub fn i2c_smbus_write_word_data(fd: RawFd, register: u8, value: u16) -> Result<(), I2CError> {
    let mut data = i2c_smbus_data::empty();
    Cursor::new(&mut data.block[..])
        .write_u16::<NativeEndian>(value)
        .unwrap();

    unsafe {
        i2c_smbus_access(
            fd,
            I2CSMBusReadWrite::I2C_SMBUS_WRITE,
            register,
            I2CSMBusSize::I2C_SMBUS_WORD_DATA,
            &mut data,
        )
    }
}

#[inline]
pub fn i2c_smbus_process_call(fd: RawFd, register: u8, value: u16) -> Result<u16, I2CError> {
    let mut data = i2c_smbus_data::empty();
    Cursor::new(&mut data.block[..])
        .write_u16::<NativeEndian>(value)
        .unwrap();

    unsafe {
        i2c_smbus_access(
            fd,
            I2CSMBusReadWrite::I2C_SMBUS_WRITE,
            register,
            I2CSMBusSize::I2C_SMBUS_PROC_CALL,
            &mut data,
        )?;
    }
    Ok(Cursor::new(&data.block[..])
        .read_u16::<NativeEndian>()
        .unwrap())
}

#[inline]
pub fn i2c_smbus_read_block_data(fd: RawFd, register: u8) -> Result<Vec<u8>, I2CError> {
    let mut data = i2c_smbus_data::empty();
    unsafe {
        i2c_smbus_access(
            fd,
            I2CSMBusReadWrite::I2C_SMBUS_READ,
            register,
            I2CSMBusSize::I2C_SMBUS_BLOCK_DATA,
            &mut data,
        )?;
    }

    // create a vector from the data in the block starting at byte
    // 1 and ending after count bytes after that
    let count = data.block[0];
    Ok((&data.block[1..(count + 1) as usize]).to_vec())
}

pub fn i2c_smbus_read_i2c_block_data(
    fd: RawFd,
    register: u8,
    len: u8,
) -> Result<Vec<u8>, I2CError> {
    let mut data = i2c_smbus_data::empty();
    data.block[0] = len;
    unsafe {
        i2c_smbus_access(
            fd,
            I2CSMBusReadWrite::I2C_SMBUS_READ,
            register,
            I2CSMBusSize::I2C_SMBUS_I2C_BLOCK_DATA,
            &mut data,
        )?;
    }

    // create a vector from the data in the block starting at byte
    // 1 and ending after count bytes after that
    let count = data.block[0];
    Ok((&data.block[1..(count + 1) as usize]).to_vec())
}

#[inline]
fn copy_to_i2c_block_data(values: &[u8], max_size: usize) -> i2c_smbus_data {
    let mut data = i2c_smbus_data::empty();
    let len: usize = if values.len() > max_size {
        max_size
    } else {
        values.len()
    };
    data.block[0] = len as u8;
    data.block[1..=len].copy_from_slice(&values[..len]);
    data
}

#[inline]
pub fn i2c_smbus_write_block_data(fd: RawFd, register: u8, values: &[u8]) -> Result<(), I2CError> {
    let mut data = copy_to_i2c_block_data(values, 32);
    unsafe {
        i2c_smbus_access(
            fd,
            I2CSMBusReadWrite::I2C_SMBUS_WRITE,
            register,
            I2CSMBusSize::I2C_SMBUS_BLOCK_DATA,
            &mut data,
        )
    }
}

#[inline]
pub fn i2c_smbus_write_i2c_block_data(
    fd: RawFd,
    register: u8,
    values: &[u8],
) -> Result<(), I2CError> {
    let mut data = copy_to_i2c_block_data(values, 32);
    unsafe {
        i2c_smbus_access(
            fd,
            I2CSMBusReadWrite::I2C_SMBUS_WRITE,
            register,
            I2CSMBusSize::I2C_SMBUS_I2C_BLOCK_DATA,
            &mut data,
        )
    }
}

#[inline]
pub fn i2c_smbus_process_call_block(
    fd: RawFd,
    register: u8,
    values: &[u8],
) -> Result<Vec<u8>, I2CError> {
    let mut data = copy_to_i2c_block_data(values, 31);
    unsafe {
        i2c_smbus_access(
            fd,
            I2CSMBusReadWrite::I2C_SMBUS_WRITE,
            register,
            I2CSMBusSize::I2C_SMBUS_BLOCK_PROC_CALL,
            &mut data,
        )?;
    };

    // create a vector from the data in the block starting at byte
    // 1 and ending after count bytes after that
    let count = data.block[0];
    Ok((&data.block[1..(count + 1) as usize]).to_vec())
}

#[inline]
pub fn i2c_rdwr(fd: RawFd, values: &mut [i2c_msg]) -> Result<u32, I2CError> {
    let i2c_data = i2c_rdwr_ioctl_data {
        msgs: values.as_mut_ptr(),
        nmsgs: values.len() as u32,
    };

    let n;
    unsafe {
        n = ioctl::i2c_rdwr(fd, &i2c_data)?;
    }
    Ok(n as u32)
}
