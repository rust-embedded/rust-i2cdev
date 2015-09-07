// Copyright 2015, Paul Osborne <osbpau@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/license/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option.  This file may not be copied, modified, or distributed
// except according to those terms.

// Reads data from Wii Nunchuck

extern crate i2cdev;
extern crate docopt;

use std::os::unix::prelude::*;
use std::fs::OpenOptions;
use std::env::args;
use docopt::Docopt;

const USAGE: &'static  str = "
Reading Wii Nunchuck data via Linux i2cdev.

Usage:
  nunchuck <device>
  nunchuck (-h | --help)
  nunchuck --version

Options:
  -h --help    Show this help text.
  --version    Show version.
";

fn read_nunchuck_data(dev: &mut AsRawFd) -> Result<(), &'static str> {
    Ok(())
}

fn main() {
    let args = Docopt::new(USAGE)
        .and_then(|d| d.argv(args().into_iter()).parse())
        .unwrap_or_else(|e| e.exit());
    let device = args.get_str("<device>");
    match OpenOptions::new().read(true).write(true).open(device) {
        Ok(mut i2cdev) => { read_nunchuck_data(&mut i2cdev).unwrap() },
        Err(err) => { println!("Unable to open {:?}, {:?}", device, err); }
    }
}
