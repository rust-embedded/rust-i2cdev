Rust Spidev
===========

[![Build Status](https://travis-ci.org/rust-embedded/rust-i2cdev.svg?branch=master)](https://travis-ci.org/rust-embedded/rust-i2cdev)
[![Version](https://img.shields.io/crates/v/i2cdev.svg)](https://crates.io/crates/i2cdev)
[![License](https://img.shields.io/crates/l/i2cdev.svg)](https://github.com/rust-embedded/rust-i2cdev/blob/master/README.md#license)

[Documentation](https://rust-embedded.github.io/rust-i2cdev)

The Rust `i2cdev` crate seeks to provide full access to the Linux i2cdev
driver interface in Rust without the need to wrap any C code or directly make
low-level system calls.  The documentation for the i2cdev interace can
be found at https://www.kernel.org/doc/Documentation/i2c/dev-interface and
in the [lm-sensors projects](http://www.lm-sensors.org/).

Example/API
-----------

The source includes an example of using the library to talk to a Wii
Nunchuck (which has an i2c interface).
[Go View the Example Example](https://github.com/rust-embedded/rust-i2cdev/blob/master/examples/nunchuck.rs).

Here's a real quick example showing the guts of how you create a
device and start talking to it...  This device only requires basic
functions (read/write) which are done via the Read/Write traits.

```rust
extern crate i2cdev;
use i2cdev::*;
use std::thread;

const NUNCHUCK_SLAVE_ADDR: u16 = 0x52;

// real code should not use unwrap() so liberally
fn i2cfun() -> Result<(), I2CError> {
    let mut dev = try!(I2CDevice::new("/dev/i2c-1", NUNCHUCK_SLAVE_ADDR));
    try!(self.i2cdev.smbus_write_byte_data(0xF0, 0x55));
    try!(self.i2cdev-smbus_write_byte_data(0xFB, 0x00))
    thread::sleep_ms(100);
    loop {
        let mut buf: [u8: 6] = [0: 6];
        self.i2cdev.write_all(&[0x00]).unwrap();
        thread::sleep_ms(10);
        self.i2cdev.read(&mut buf).unwrap();
        println!("Reading: {:?}", buf); 
    }
}
```

In addition to the Read/Write traits, the following methods are
available via the I2CSMBus trait:

```rust
pub trait I2CSMBus {
    fn smbus_write_quick(&self, bit: bool) -> Result<(), Error>;
    fn smbus_read_byte(&self) -> Result<u8, Error>;
    fn smbus_write_byte(&self, value: u8) -> Result<(), Error>;
    fn smbus_read_byte_data(&self, register: u8) -> Result<u8, Error>;
    fn smbus_write_byte_data(&self, register: u8, value: u8) -> Result<(), Error>;
    fn smbus_read_word_data(&self, register: u8) -> Result<u16, Error>;
    fn smbus_write_word_data(&self, register: u8, value: u16) -> Result<(), Error>;
    fn smbus_process_word(&self, register: u8, value: u16) -> Result<u16, Error>;
    fn smbus_read_block_data(&self, register: u8) -> Result<Vec<u8>, Error>;
    fn smbus_write_block_data(&self, register: u8, values: &[u8]) -> Result<(), Error>;
    fn smbus_process_block(&self, register: u8, values: &[u8]) -> Result<(), Error>;
}
```

Features
--------

The following features are implemented and planned for the library:

- [x] Implement the Read trait
- [x] Implement the Write trait
- [x] Implement SMBus Methods
- [x] Add Tests/Example for SMBus Methods
- [/] Add sensor library for handy sensors (and examples)
- [ ] Add higher-level APIs/Macros for simplifying access to devices
      with large register sets
- [ ] Add Support for Non-SMBus ioctl methods
- [ ] Add examples for non-smbus ioctl methods
- [ ] Unit Testing

Cross Compiling
---------------

Most likely, the machine you are running on is not your development
machine (although it could be).  In those cases, you will need to
cross-compile.  The following basic instructions should work for the
raspberry pi or beaglebone black:

1. Install rust and cargo
2. Install an appropriate cross compiler.  On an Ubuntu system, this
   can be done by doing `sudo apt-get install g++-arm-linux-gnueabihf`.
3. Build or install rust for your target.  This is necessary in order
   to have libstd available for your target.  For arm-linux-gnueabihf,
   you can find binaries at https://github.com/japaric/ruststrap.
   With this approach or building it yourself, you will need to copy
   the ${rust}/lib/rustlib/arm-unknown-linux-gnueabihf to your system
   rust library folder (it is namespaced by triple, so it shouldn't
   break anything).
4. Tell cargo how to link by adding the lines below to your
   ~/.cargo/config file.
5. Run your build `cargo build --target=arm-unknown-linux-gnueabi`.

The following snippet added to my ~/.cargo/config worked for me:

```
[target.arm-unknown-linux-gnueabihf]
linker = "arm-linux-gnueabihf-gcc"
```

License
-------

Copyright (c) 2015, Paul Osborne <ospbau@gmail.com>

Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
http://www.apache.org/license/LICENSE-2.0> or the MIT license
<LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
option.  This file may not be copied, modified, or distributed
except according to those terms.
