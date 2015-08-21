// Copyright 2015, Paul Osborne <osbpau@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/license/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option.  This file may not be copied, modified, or distributed
// except according to those terms.

use std::fs::File;
use std::mem;
use std::ptr;
use std::io::Cursor;
use nix;
use std::os::unix::prelude::*;
use byteorder::{NativeEndian, ReadBytesExt, WriteBytesExt};

/**
 * struct i2c_msg - an I2C transaction segment beginning with START
 * @addr: Slave address, either seven or ten bits.  When this is a ten
 *     bit address, I2C_M_TEN must be set in @flags and the adapter
 *     must support I2C_FUNC_10BIT_ADDR.
 * @flags: I2C_M_RD is handled by all adapters.  No other flags may be
 *     provided unless the adapter exported the relevant I2C_FUNC_*
 *     flags through i2c_check_functionality().
 * @len: Number of data bytes in @buf being read from or written to the
 *     I2C slave address.  For read transactions where I2C_M_RECV_LEN
 *     is set, the caller guarantees that this buffer can hold up to
 *     32 bytes in addition to the initial length byte sent by the
 *     slave (plus, if used, the SMBus PEC); and this value will be
 *     incremented by the number of block data bytes received.
 * @buf: The buffer into which data is read, or from which it's written.
 *
 * An i2c_msg is the low level representation of one segment of an I2C
 * transaction.  It is visible to drivers in the @i2c_transfer() procedure,
 * to userspace from i2c-dev, and to I2C adapter drivers through the
 * @i2c_adapter.@master_xfer() method.
 *
 * Except when I2C "protocol mangling" is used, all I2C adapters implement
 * the standard rules for I2C transactions.  Each transaction begins with a
 * START.  That is followed by the slave address, and a bit encoding read
 * versus write.  Then follow all the data bytes, possibly including a byte
 * with SMBus PEC.  The transfer terminates with a NAK, or when all those
 * bytes have been transferred and ACKed.  If this is the last message in a
 * group, it is followed by a STOP.  Otherwise it is followed by the next
 * @i2c_msg transaction segment, beginning with a (repeated) START.
 *
 * Alternatively, when the adapter supports I2C_FUNC_PROTOCOL_MANGLING then
 * passing certain @flags may have changed those standard protocol behaviors.
 * Those flags are only for use with broken/nonconforming slaves, and with
 * adapters which are known to support the specific mangling options they
 * need (one or more of IGNORE_NAK, NO_RD_ACK, NOSTART, and REV_DIR_ADDR).
 */
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
    I2C_SMBUS_BLOCK_PROC_CALL = 7, /* SMBus 2.0 */
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
#[repr(C)]
struct i2c_rdwr_ioctl_data {
    // struct i2c_msg __user *msgs;
    msgs: *mut i2c_msg,
    // __u32 nmsgs;
    nmsgs: u32,
}

/// Struct providing access to some I2C Bus
///
/// A single bus may have multiple devices on it.  The
/// kernel exposes one device (e.g. `/dev/i2c-1`) per
/// I2C bus that the system has access (and which is
/// exposed to userspace).
///
/// The bus is the root for all operations and the device on
/// which this code is operating is the master.
pub struct I2CBus {
    devfile: File,
}

ioctl!(bad ioctl_set_i2c_slave_address with I2C_SLAVE);
ioctl!(bad ioctl_i2c_smbus with I2C_SMBUS);

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
fn i2c_smbus_write_quick(fd: RawFd, bit: bool) -> Result<(), nix::Error> {
    let read_write = match bit {
        true => I2CSMBusReadWrite::I2C_SMBUS_READ,
        false => I2CSMBusReadWrite::I2C_SMBUS_WRITE,
    };
    unsafe {
        i2c_smbus_access(fd, read_write, 0, I2CSMBusSize::I2C_SMBUS_QUICK, ptr::null_mut())
    }
}

#[inline]
fn i2c_smbus_read_byte(fd: RawFd) -> Result<u8, nix::Error> {
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
fn i2c_smbus_write_byte(fd: RawFd, value: u8) -> Result<(), nix::Error> {
    unsafe {
        i2c_smbus_access(fd,
                         I2CSMBusReadWrite::I2C_SMBUS_WRITE,
                         value,
                         I2CSMBusSize::I2C_SMBUS_BYTE,
                         ptr::null_mut())
    }
}

#[inline]
fn i2c_smbus_read_byte_data(fd: RawFd, register: u8) -> Result<u8, nix::Error> {
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
fn i2c_smbus_write_byte_data(fd: RawFd, register: u8, value: u8) -> Result<(), nix::Error> {
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
fn i2c_smbus_read_word_data(fd: RawFd, register: u8) -> Result<u16, nix::Error> {
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
fn i2c_smbus_write_word_data(fd: RawFd, register: u8, value: u16) -> Result<(), nix::Error> {
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
fn i2c_smbus_process_call(fd: RawFd, register: u8, value: u16) -> Result<u16, nix::Error> {
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
fn i2c_smbus_read_block_data(fd: RawFd, register: u8) -> Result<Vec<u8>, nix::Error> {
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
fn i2c_smbus_write_block_data(fd: RawFd, register: u8, values: &[u8]) -> Result<(), nix::Error> {
    Ok(())  // TODO: implement me
}

#[inline]
fn i2c_smbus_write_i2c_block_data(fd: RawFd, register: u8, values: &[u8]) -> Result<(), nix::Error> {
    Ok(()) // TODO: implement me
}


impl I2CBus {
    pub fn new(devfile: File) -> I2CBus {
        I2CBus { devfile: devfile }
    }

    /// Select the slave with the given address
    ///
    /// Typically the address is expected to be 7-bits but 10-bit addresses
    /// may be supported by the kernel driver in some cases.  Little validation
    /// is done in Rust as the kernel is good at making sure things are valid.
    pub fn set_slave_address(&self, slave_address: u16) -> Result<(), nix::Error> {
        try!(unsafe {
            // NOTE: the generated ioctl call expected as pointer to a u8 but
            // we just want to provide the u8 directly, so we just cast to a pointer.
            // This is correct behavior.
            ioctl_set_i2c_slave_address(self.devfile.as_raw_fd(), slave_address as *mut u8)
        });
        Ok(())
    }

    /// This sends a single bit to the device, at the place of the Rd/Wr bit
    pub fn smbus_write_quick(&self, bit: bool) -> Result<(), nix::Error> {
        i2c_smbus_write_quick(self.devfile.as_raw_fd(), bit)
    }

    /// Read a single byte from a device, without specifying a device register
    ///
    /// Some devices are so simple that this interface is enough; for
    /// others, it is a shorthand if you want to read the same register as in
    /// the previous SMBus command.
    pub fn smbus_read_byte(&self) -> Result<u8, nix::Error> {
        i2c_smbus_read_byte(self.devfile.as_raw_fd())
    }

    /// Write a single byte to a sdevice, without specifying a device register
    ///
    /// This is the opposite operation as smbus_read_byte.  As with read_byte,
    /// no register is specified.
    pub fn smbus_write_byte(&self, value: u8) -> Result<(), nix::Error> {
        i2c_smbus_write_byte(self.devfile.as_raw_fd(), value)
    }

    /// Read a single byte from a device, from a designated register
    ///
    /// The register is specified through the Comm byte.
    pub fn smbus_read_byte_data(&self, register: u8) -> Result<u8, nix::Error> {
        i2c_smbus_read_byte_data(self.devfile.as_raw_fd(), register)
    }

    /// Write a single byte to a specific register on a device
    ///
    /// The register is specified through the Comm byte.
    pub fn smbus_write_byte_data(&self, register: u8, value: u8) -> Result<(), nix::Error> {
        i2c_smbus_write_byte_data(self.devfile.as_raw_fd(), register, value)
    }

    /// Read 2 bytes form a given register on a device
    pub fn smbus_read_word_data(&self, register: u8) -> Result<u16, nix::Error> {
        i2c_smbus_read_word_data(self.devfile.as_raw_fd(), register)
    }

    /// Write 2 bytes to a given register on a device
    pub fn smbus_write_word_data(&self, register: u8, value: u16) -> Result<(), nix::Error> {
        i2c_smbus_write_word_data(self.devfile.as_raw_fd(), register, value)
    }

    /// Select a register, send 16 bits of data to it, and read 16 bits of data
    pub fn smbus_process_word(&self, register: u8, value: u16) -> Result<u16, nix::Error> {
        i2c_smbus_process_call(self.devfile.as_raw_fd(), register, value)
    }

    /// Read a block of up to 32 bytes from a device
    ///
    /// The actual number of bytes available to read is returned in the count
    /// byte.  This code returns a correctly sized vector containing the
    /// count bytes read from the device.
    pub fn smbus_read_block_data(&self, register: u8) -> Result<Vec<u8>, nix::Error> {
        i2c_smbus_read_block_data(self.devfile.as_raw_fd(), register)
    }

    /// Write a block of up to 32 bytes to a device
    ///
    /// The opposite of the Block Read command, this writes up to 32 bytes to
    /// a device, to a designated register that is specified through the
    /// Comm byte. The amount of data is specified in the Count byte.
    pub fn smbus_write_block_data(&self, register: u8, values: &[u8]) -> Result<(), nix::Error> {
        i2c_smbus_write_block_data(self.devfile.as_raw_fd(), register, values)
    }

    /// Select a register, send 1 to 31 bytes of data to it, and reads
    /// 1 to 31 bytes of data from it.
    pub fn smbus_process_block(&self, register: u8, values: &[u8]) -> Result<(), nix::Error> {
        i2c_smbus_write_i2c_block_data(self.devfile.as_raw_fd(), register, values)
    }

}
