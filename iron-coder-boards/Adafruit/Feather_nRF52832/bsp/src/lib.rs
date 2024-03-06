//! Iron Coder BSP for nRF52832 microcontroller

#![no_std]

use embedded_hal::blocking::i2c::Write;
use sh1107::{self, prelude::*};

// use embedded_graphics::{
//     mono_font::{ascii::FONT_6X10, MonoTextStyle},
//     pixelcolor::Rgb565,
//     prelude::*,
//     text::Text,
// };

pub struct Board<I2C>
    where I2C: Write
{
    pub display: Option<GraphicsMode<I2cInterface<I2C>>>
}

impl<I2C, E> Board<I2C>
    where I2C: Write<Error = E>, E: core::fmt::Debug
{
    pub fn new() -> Self {
        Self {
            display: None,
        }
    }

    pub fn init_display(&mut self, i2c: I2C) {

        let disp_size = DisplaySize::Display64x128;
        let mut rotation = DisplayRotation::Rotate270;

        let mut d: GraphicsMode<_> = sh1107::Builder::new()
            .with_size(disp_size)
            .with_rotation(rotation)
            .connect_i2c(i2c)
            .into();

        d.init().unwrap();
        d.flush().unwrap();

        // // Create a new character style
	    // let style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);
	    // // Create a text at position (20, 30) and draw it using the previously defined style
	    // Text::new("Hello Rust!", Point::new(20, 30), style).draw(d).unwrap();
	    // d.flush().unwrap();

        self.display = Some(d);
    }
}