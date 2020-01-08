// Copyright 2015, Paul Osborne <osbpau@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/license/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option.  This file may not be copied, modified, or distributed
// except according to those terms.

//! # i2cdev
//!
//! The `i2cdev` crate provides a safe interface for interface
//! with i2c devices under Linux.  The API wraps the Linux
//! kernel interface for interacting with i2c in userspace:
//! https://www.kernel.org/doc/Documentation/i2c/dev-interface
//! ```rust,no_run
//! extern crate i2cdev;
//!
//! use std::thread;
//! use std::time::Duration;
//!
//! use i2cdev::core::*;
//! use i2cdev::linux::{LinuxI2CDevice, LinuxI2CError};
//!
//! const NUNCHUCK_SLAVE_ADDR: u16 = 0x52;
//!
//! // real code should probably not use unwrap()
//! fn i2cfun() -> Result<(), LinuxI2CError> {
//!     let mut dev = LinuxI2CDevice::new("/dev/i2c-1", NUNCHUCK_SLAVE_ADDR)?;
//!
//!     // init sequence
//!     dev.smbus_write_byte_data(0xF0, 0x55)?;
//!     dev.smbus_write_byte_data(0xFB, 0x00)?;
//!     thread::sleep(Duration::from_millis(100));
//!
//!     loop {
//!         let mut buf: [u8; 6] = [0; 6];
//!         dev.smbus_write_byte(0x00).unwrap();
//!         thread::sleep(Duration::from_millis(10));
//!         dev.read(&mut buf).unwrap();
//!         println!("Reading: {:?}", buf);
//!     }
//! }
//! ```
//!
//! ```rust,no_run
//! extern crate i2cdev;
//!
//! use std::thread;
//! use std::time::Duration;
//!
//! use i2cdev::core::*;
//! use i2cdev::linux::{LinuxI2CDevice, LinuxI2CError, LinuxI2CMessage};
//!
//! const SLAVE_ADDR: u16 = 0x57;
//!
//! fn write_read_transaction() -> Result<(), LinuxI2CError> {
//!     let mut dev = LinuxI2CDevice::new("/dev/i2c-1", SLAVE_ADDR)?;
//!
//!     let mut read_data = [0; 2];
//!     let mut msgs = [
//!         LinuxI2CMessage::write(&[0x01]),
//!         LinuxI2CMessage::read(&mut read_data)
//!     ];
//!     dev.transfer(&mut msgs)?;
//!
//!     println!("Reading: {:?}", read_data);
//!     Ok(())
//! }
//! ```
//!
//! ```rust,no_run
//! extern crate i2cdev;
//!
//! use std::thread;
//! use std::time::Duration;
//!
//! use i2cdev::core::*;
//! use i2cdev::linux::{LinuxI2CBus, LinuxI2CError, LinuxI2CMessage};
//!
//! const SLAVE_ADDR: u16 = 0x57;
//!
//! fn write_read_transaction_using_bus() -> Result<(), LinuxI2CError> {
//!     let mut dev = LinuxI2CBus::new("/dev/i2c-1")?;
//!
//!     let mut read_data = [0; 2];
//!     let mut msgs = [
//!         LinuxI2CMessage::write(&[0x01]).with_address(SLAVE_ADDR),
//!         LinuxI2CMessage::read(&mut read_data).with_address(SLAVE_ADDR)
//!     ];
//!     dev.transfer(&mut msgs)?;
//!
//!     println!("Reading: {:?}", read_data);
//!     Ok(())
//! }
//! ```

#![crate_name = "i2cdev"]
#![crate_type = "lib"]
#![deny(missing_docs)]

#[macro_use]
extern crate bitflags;
extern crate byteorder;
extern crate libc;
#[macro_use]
extern crate nix;

#[cfg(any(target_os = "linux", target_os = "android"))]
mod ffi;

/// Core I2C abstractions
pub mod core;

/// Linux I2C device support
#[cfg(any(target_os = "linux", target_os = "android"))]
pub mod linux;

/// Mock I2C device
pub mod mock;
