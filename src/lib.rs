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

extern crate libc;
#[macro_use]
extern crate nix;
#[macro_use]
extern crate bitflags;
extern crate byteorder;

mod i2c_ioctl;

pub use i2c_ioctl::{
    I2CBus
};
