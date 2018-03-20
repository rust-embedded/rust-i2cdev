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

#![crate_name = "i2cdev"]
#![crate_type = "lib"]

extern crate libc;
extern crate byteorder;
#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate nix;

#[cfg(any(target_os = "linux", target_os = "android"))]
mod ffi;

pub mod core;

#[cfg(any(target_os = "linux", target_os = "android"))]
pub mod linux;
pub mod mock;
