// Copyright 2015, Paul Osborne <osbpau@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/license/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option.  This file may not be copied, modified, or distributed
// except according to those terms.

use std::mem;
use std::ptr;
use std::io::Cursor;
use nix;
use std::os::unix::prelude::*;
use byteorder::{NativeEndian, ReadBytesExt, WriteBytesExt};

#[allow(dead_code)]
bitflags! {
    flags I2CMsgFlags: u16 {
        /// this is a ten bit chip address
        const I2C_M_TEN = 0x0010,
        /// read data, from slave to master
        const I2C_M_RD = 0x0001,
        /// if I2C_FUNC_PROTOCOL_MANGLING
        const I2C_M_STOP = 0x8000,
        /// if I2C_FUNC_NOSTART
        const I2C_M_NOSTART = 0x4000,
        /// if I2C_FUNC_PROTOCOL_MANGLING
        const I2C_M_REV_DIR_ADDR = 0x2000,
        /// if I2C_FUNC_PROTOCOL_MANGLING
        const I2C_M_IGNORE_NAK = 0x1000,
        /// if I2C_FUNC_PROTOCOL_MANGLING
        const I2C_M_NO_RD_ACK = 0x0800,
        /// length will be first received byte
        const I2C_M_RECV_LEN = 0x0400,
    }
}

#[allow(non_camel_case_types)]
#[allow(dead_code)]
#[repr(C)]
struct i2c_msg {
    /// slave address
    addr: u16,
    /// serialized I2CMsgFlags
    flags: u16,
    /// msg length
    len: u16,
    /// pointer to msg data
    buf: *mut u8,
}

#[allow(dead_code)]
bitflags! {
    flags I2CFunctions: u32 {
        const I2C_FUNC_I2C = 0x00000001,
        const I2C_FUNC_10BIT_ADDR = 0x00000002,
        const I2C_FUNC_PROTOCOL_MANGLING = 0x00000004, /* I2C_M_IGNORE_NAK etc. */
        const I2C_FUNC_SMBUS_PEC = 0x00000008,
        const I2C_FUNC_NOSTART = 0x00000010, /* I2C_M_NOSTART */
        const I2C_FUNC_SMBUS_BLOCK_PROC_CALL = 0x00008000, /* SMBus 2.0 */
        const I2C_FUNC_SMBUS_QUICK = 0x00010000,
        const I2C_FUNC_SMBUS_READ_BYTE = 0x00020000,
        const I2C_FUNC_SMBUS_WRITE_BYTE = 0x00040000,
        const I2C_FUNC_SMBUS_READ_BYTE_DATA = 0x00080000,
        const I2C_FUNC_SMBUS_WRITE_BYTE_DATA = 0x00100000,
        const I2C_FUNC_SMBUS_READ_WORD_DATA = 0x00200000,
        const I2C_FUNC_SMBUS_WRITE_WORD_DATA = 0x00400000,
        const I2C_FUNC_SMBUS_PROC_CALL = 0x00800000,
        const I2C_FUNC_SMBUS_READ_BLOCK_DATA = 0x01000000,
        const I2C_FUNC_SMBUS_WRITE_BLOCK_DATA  = 0x02000000,
        const I2C_FUNC_SMBUS_READ_I2C_BLOCK = 0x04000000, /* I2C-like block xfer  */
        const I2C_FUNC_SMBUS_WRITE_I2C_BLOCK = 0x08000000, /* w/ 1-byte reg. addr. */

        const I2C_FUNC_SMBUS_BYTE = (I2C_FUNC_SMBUS_READ_BYTE.bits |
                                     I2C_FUNC_SMBUS_WRITE_BYTE.bits),
        const I2C_FUNC_SMBUS_BYTE_DATA = (I2C_FUNC_SMBUS_READ_BYTE_DATA.bits |
                                          I2C_FUNC_SMBUS_WRITE_BYTE_DATA.bits),
        const I2C_FUNC_SMBUS_WORD_DATA = (I2C_FUNC_SMBUS_READ_WORD_DATA.bits |
                                          I2C_FUNC_SMBUS_WRITE_WORD_DATA.bits),
        const I2C_FUNC_SMBUS_BLOCK_DATA = (I2C_FUNC_SMBUS_READ_BLOCK_DATA.bits |
                                           I2C_FUNC_SMBUS_WRITE_BLOCK_DATA.bits),
        const I2C_FUNC_SMBUS_I2C_BLOCK = (I2C_FUNC_SMBUS_READ_I2C_BLOCK.bits |
                                          I2C_FUNC_SMBUS_WRITE_I2C_BLOCK.bits),
        const I2C_FUNC_SMBUS_EMUL = (I2C_FUNC_SMBUS_QUICK.bits |
                                     I2C_FUNC_SMBUS_BYTE.bits |
                                     I2C_FUNC_SMBUS_BYTE_DATA.bits |
                                     I2C_FUNC_SMBUS_WORD_DATA.bits |
                                     I2C_FUNC_SMBUS_PROC_CALL.bits |
                                     I2C_FUNC_SMBUS_WRITE_BLOCK_DATA.bits |
                                     I2C_FUNC_SMBUS_I2C_BLOCK.bits |
                                     I2C_FUNC_SMBUS_PEC.bits),
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
#[allow(non_camel_case_types)]
#[repr(C)]
struct i2c_smbus_data {
    block: [u8; (I2C_SMBUS_BLOCK_MAX + 2) as usize],
}

impl i2c_smbus_data {
    fn empty() -> i2c_smbus_data {
        unsafe { mem::zeroed() }
    }
}

#[allow(non_camel_case_types)]
#[allow(dead_code)]
#[repr(u8)]
enum I2CSMBusReadWrite {
    I2C_SMBUS_READ = 1,
    I2C_SMBUS_WRITE = 0,
}

#[allow(non_camel_case_types)]
#[allow(dead_code)]
#[repr(u32)]
enum I2CSMBusSize {
    I2C_SMBUS_QUICK = 0,
    I2C_SMBUS_BYTE = 1,
    I2C_SMBUS_BYTE_DATA = 2,
    I2C_SMBUS_WORD_DATA = 3,
    I2C_SMBUS_PROC_CALL = 4,
    I2C_SMBUS_BLOCK_DATA = 5,
    I2C_SMBUS_I2C_BLOCK_BROKEN = 6,
    I2C_SMBUS_BLOCK_PROC_CALL = 7, /* SMBus 2.0 */
    I2C_SMBUS_I2C_BLOCK_DATA = 8,
}

// from include/uapi/linux/i2c-dev.h
//const I2C_RETRIES: u16 = 0x0701;
//const I2C_TIMEOUT: u16 = 0x0702;
const I2C_SLAVE: u16 = 0x0703;
//const I2C_SLAVE_FORCE: u16 = 0x0706;
//const I2C_TENBIT: u16 = 0x0704;
//const I2C_FUNCS: u16 = 0x0705;
//const I2C_RDWR: u16 = 0x0707;
//const I2C_PEC: u16 = 0x0708;
const I2C_SMBUS: u16 = 0x0720;
//const I2C_RDRW_IOCTL_MAX_MSGS: u8 = 42;

/// This is the structure as used in the I2C_SMBUS ioctl call
#[allow(non_camel_case_types)]
#[repr(C)]
struct i2c_smbus_ioctl_data {
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
#[allow(non_camel_case_types)]
#[allow(dead_code)]
#[repr(C)]
struct i2c_rdwr_ioctl_data {
    // struct i2c_msg __user *msgs;
    msgs: *mut i2c_msg,
    // __u32 nmsgs;
    nmsgs: u32,
}

ioctl!(bad ioctl_set_i2c_slave_address with I2C_SLAVE);
ioctl!(bad ioctl_i2c_smbus with I2C_SMBUS);

pub fn i2c_set_slave_address(fd: RawFd, slave_address: u16) -> Result<(), nix::Error> {
    try!(unsafe {
        // NOTE: the generated ioctl call expected as pointer to a u8 but
        // we just want to provide the u8 directly, so we just cast to a pointer.
        // This is correct behavior.
        ioctl_set_i2c_slave_address(fd, slave_address as *mut u8)
    });
    Ok(())
}

unsafe fn i2c_smbus_access(fd: RawFd,
                           read_write: I2CSMBusReadWrite,
                           command: u8, // can be address or something else
                           size: I2CSMBusSize,
                           data: *mut i2c_smbus_data)
                           -> Result<(), nix::Error> {
    let mut args = i2c_smbus_ioctl_data {
        read_write: read_write as u8,
        command: command,
        size: size as u32,
        data: data,
    };

    // remove type information
    let p_args: *mut u8 = mem::transmute(&mut args);
    try!(ioctl_i2c_smbus(fd, p_args));
    Ok(())
}

#[inline]
pub fn i2c_smbus_write_quick(fd: RawFd, bit: bool) -> Result<(), nix::Error> {
    let read_write = match bit {
        true => I2CSMBusReadWrite::I2C_SMBUS_READ,
        false => I2CSMBusReadWrite::I2C_SMBUS_WRITE,
    };
    unsafe {
        i2c_smbus_access(fd, read_write, 0, I2CSMBusSize::I2C_SMBUS_QUICK, ptr::null_mut())
    }
}

#[inline]
pub fn i2c_smbus_read_byte(fd: RawFd) -> Result<u8, nix::Error> {
    let mut data = i2c_smbus_data::empty();
    try!(unsafe {
        i2c_smbus_access(fd,
                         I2CSMBusReadWrite::I2C_SMBUS_READ,
                         0,
                         I2CSMBusSize::I2C_SMBUS_BYTE,
                         &mut data)
    });
    Ok(data.block[0])
}

#[inline]
pub fn i2c_smbus_write_byte(fd: RawFd, value: u8) -> Result<(), nix::Error> {
    unsafe {
        i2c_smbus_access(fd,
                         I2CSMBusReadWrite::I2C_SMBUS_WRITE,
                         value,
                         I2CSMBusSize::I2C_SMBUS_BYTE,
                         ptr::null_mut())
    }
}

#[inline]
pub fn i2c_smbus_read_byte_data(fd: RawFd, register: u8) -> Result<u8, nix::Error> {
    let mut data = i2c_smbus_data::empty();
    try!(unsafe {
            i2c_smbus_access(fd,
                             I2CSMBusReadWrite::I2C_SMBUS_READ,
                             register,
                             I2CSMBusSize::I2C_SMBUS_BYTE_DATA,
                             &mut data)
    });
    Ok(data.block[0])
}

#[inline]
pub fn i2c_smbus_write_byte_data(fd: RawFd, register: u8, value: u8) -> Result<(), nix::Error> {
    let mut data =  i2c_smbus_data::empty();
    data.block[0] = value;
    try!(unsafe {
        i2c_smbus_access(fd,
                         I2CSMBusReadWrite::I2C_SMBUS_WRITE,
                         register,
                         I2CSMBusSize::I2C_SMBUS_BYTE_DATA,
                         &mut data)
    });
    Ok(())
}

#[inline]
pub fn i2c_smbus_read_word_data(fd: RawFd, register: u8) -> Result<u16, nix::Error> {
    let mut data = i2c_smbus_data::empty();
    try!(unsafe {
            i2c_smbus_access(fd,
                             I2CSMBusReadWrite::I2C_SMBUS_READ,
                             register,
                             I2CSMBusSize::I2C_SMBUS_WORD_DATA,
                             &mut data)
    });

    Ok(Cursor::new(&data.block[..])
        .read_u16::<NativeEndian>()
        .unwrap())
}


#[inline]
pub fn i2c_smbus_write_word_data(fd: RawFd, register: u8, value: u16) -> Result<(), nix::Error> {
    let mut data = i2c_smbus_data::empty();
    Cursor::new(&mut data.block[..])
        .write_u16::<NativeEndian>(value)
        .unwrap();

    try!(unsafe {
        i2c_smbus_access(fd,
                         I2CSMBusReadWrite::I2C_SMBUS_WRITE,
                         register,
                         I2CSMBusSize::I2C_SMBUS_WORD_DATA,
                         &mut data)
    });
    Ok(())
}

#[inline]
pub fn i2c_smbus_process_call(fd: RawFd, register: u8, value: u16) -> Result<u16, nix::Error> {
    let mut data = i2c_smbus_data::empty();
    Cursor::new(&mut data.block[..])
        .write_u16::<NativeEndian>(value)
        .unwrap();

    try!(unsafe {
        i2c_smbus_access(fd,
                         I2CSMBusReadWrite::I2C_SMBUS_WRITE,
                         register,
                         I2CSMBusSize::I2C_SMBUS_PROC_CALL,
                         &mut data)
    });
    Ok(Cursor::new(&data.block[..])
        .read_u16::<NativeEndian>()
        .unwrap())
}

#[inline]
pub fn i2c_smbus_read_block_data(fd: RawFd, register: u8) -> Result<Vec<u8>, nix::Error> {
    let mut data = i2c_smbus_data::empty();
    try!(unsafe {
        i2c_smbus_access(fd,
                         I2CSMBusReadWrite::I2C_SMBUS_READ,
                         register,
                         I2CSMBusSize::I2C_SMBUS_BLOCK_DATA,
                         &mut data)
    });

    // create a vector from the data in the block starting at byte
    // 1 and ending after count bytes after that
    let count = data.block[0];
    Ok((&data.block[1..(count + 1) as usize]).to_vec())
}

#[inline]
pub fn i2c_smbus_write_block_data(fd: RawFd, register: u8, values: &[u8])
                                  -> Result<(), nix::Error> {
    let mut data = i2c_smbus_data::empty();
    let len: usize = if values.len() > 32 { 32 } else { values.len() };
    data.block[0] = len as u8;
    for i in 1..(len + 1) {
        data.block[i] = values[i - 1];
    }
    try!(unsafe {
        i2c_smbus_access(fd,
                         I2CSMBusReadWrite::I2C_SMBUS_WRITE,
                         register,
                         I2CSMBusSize::I2C_SMBUS_BLOCK_DATA,
                         &mut data)
    });
    Ok(())
}

#[inline]
pub fn i2c_smbus_write_i2c_block_data(fd: RawFd, register: u8, values: &[u8])
                                      -> Result<(), nix::Error> {
    let mut data = i2c_smbus_data::empty();
    let len: usize = if values.len() > 32 { 32 } else { values.len() };
    data.block[0] = len as u8;
    for i in 1..(len + 1) {
        data.block[i] = values[i - 1];
    }
    try!(unsafe {
        i2c_smbus_access(fd,
                         I2CSMBusReadWrite::I2C_SMBUS_WRITE,
                         register,
                         I2CSMBusSize::I2C_SMBUS_I2C_BLOCK_DATA,
                         &mut data)
    });
    Ok(())
}
