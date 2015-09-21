// Copyright 2015, Paul Osborne <osbpau@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/license/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option.  This file may not be copied, modified, or distributed
// except according to those terms.

#[macro_export]
macro_rules! i2cmap {
    (
        $I2CMap:ident = {
            $( $value:expr => $regname:ident : $T:ty,)+
        }
    ) => (
        pub mod $I2CMap {$(
            pub const $regname: u8 = $value;
        )+}
    )
}


i2cmap! {
    ADXL345_REGS = {
        0x00 => devid: u8,
        0x1D => thresh_tap: u8,
    }
}


        // 0x1E => OFSX: u8,
        // 0x1F => OFSY: u8,
        // 0x20 => OFSZ: u8,
        // 0x21 => DUR: u8,
        // 0x22 => LATENT: u8,
