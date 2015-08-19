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

use std::fs::File;

extern crate libc;
extern crate nix;

#[macro_use]
extern crate bitflags;

pub mod i2c_ioctl;

/// Struct providing access to some I2C Bus
///
/// A single bus may have multiple devices on it.  The
/// kernel exposes one device (e.g. `/dev/i2c-1`) per
/// I2C bus that the system has access (and which is
/// exposed to userspace).
///
/// The bus is the root for all operations.
pub struct I2CBus {
    devfile: File
}

