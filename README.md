Rust I2cdev
===========

[![Build Status](https://travis-ci.org/rust-embedded/rust-i2cdev.svg?branch=master)](https://travis-ci.org/rust-embedded/rust-i2cdev)
[![Version](https://img.shields.io/crates/v/i2cdev.svg)](https://crates.io/crates/i2cdev)
[![License](https://img.shields.io/crates/l/i2cdev.svg)](https://github.com/rust-embedded/rust-i2cdev/blob/master/README.md#license)

[Documentation](https://docs.rs/i2cdev)

The Rust `i2cdev` crate seeks to provide full access to the Linux i2cdev
driver interface in Rust without the need to wrap any C code or directly make
low-level system calls.  The documentation for the i2cdev interace can
be found at https://www.kernel.org/doc/Documentation/i2c/dev-interface and
in the [lm-sensors projects](http://www.lm-sensors.org/).

Example/API
-----------

The source includes an example of using the library to talk to a Wii
Nunchuck (which has an i2c interface).
[Go View the Example](https://github.com/rust-embedded/rust-i2cdev/blob/master/examples/nunchuck.rs).

Here's a real quick example showing the guts of how you create
device and start talking to it...  This device only requires basic
functions (read/write) which are done via the Read/Write traits (if
you actually want to use the Wii Nunchuck you should use
[`i2cdev::sensors::nunchuck::Nunchuck`][nunchuck]:

```rust,no_run,skeptic-template
extern crate i2cdev;

use std::thread;
use std::time::Duration;

use i2cdev::core::*;
use i2cdev::linux::{LinuxI2CDevice, LinuxI2CError};

const NUNCHUCK_SLAVE_ADDR: u16 = 0x52;

// real code should probably not use unwrap()
fn i2cfun() -> Result<(), LinuxI2CError> {
    let mut dev = try!(LinuxI2CDevice::new("/dev/i2c-1", NUNCHUCK_SLAVE_ADDR));

    // init sequence
    try!(dev.smbus_write_byte_data(0xF0, 0x55));
    try!(dev.smbus_write_byte_data(0xFB, 0x00));
    thread::sleep(Duration::from_millis(100));

    loop {
        let mut buf: [u8; 6] = [0; 6];
        dev.smbus_write_byte(0x00).unwrap();
        thread::sleep(Duration::from_millis(10));
        dev.read(&mut buf).unwrap();
        println!("Reading: {:?}", buf);
    }
}
```

In addition to the Read/Write traits, the following methods are
available via the [I2CDevice trait](https://rust-embedded.github.io/rust-i2cdev/i2cdev/core/trait.I2CDevice.html).

[nunchuck]: http://rust-embedded.github.io/rust-i2cdev/i2cdev/sensors/nunchuck/struct.Nunchuck.html

Features
--------

The following features are implemented and planned for the library:

- [x] Implement the Read trait
- [x] Implement the Write trait
- [x] Implement SMBus Methods
- [x] Add Tests/Example for SMBus Methods
- [x] Add sensor library for handy sensors (and examples)
- [ ] Add higher-level APIs/Macros for simplifying access to devices
      with large register sets
- [ ] Add Support for Non-SMBus ioctl methods
- [ ] Add examples for non-smbus ioctl methods
- [ ] Unit Testing

Cross Compiling
---------------

Most likely, the machine you are running on is not your development
machine (although it could be).  In those cases, you will need to
cross-compile.  See https://github.com/japaric/rust-cross for pointers.

License
-------

```
Copyright (c) 2015, Paul Osborne <ospbau@gmail.com>

Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
http://www.apache.org/license/LICENSE-2.0> or the MIT license
<LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
option.  This file may not be copied, modified, or distributed
except according to those terms.
```
