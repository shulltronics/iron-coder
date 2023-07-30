//! Iron Coder BSP for PiTFT display

#![no_std]

use ili9341;
use embedded_hal::{
    digital::v2::OutputPin,
    blocking::spi::Write,
    blocking::delay::DelayMs,
    blocking::i2c::WriteRead,
};
use embedded_hal_one::i2c::I2c;
use display_interface_spi::SPIInterface;

pub struct Display<SPI, DC, CS, RST, I2C>
    where SPI: Write<u8>,
          CS: OutputPin,
          DC: OutputPin,
          RST: OutputPin,
          I2C: I2c,
{
    pub display: Option<ili9341::Ili9341<SPIInterface<SPI, DC, CS>, RST>>,
    pub touchscreen: Option<ft6236::FT6236<I2C>>,
}

impl<SPI, DC, CS, RST, I2C> Display<SPI, DC, CS, RST, I2C>
where SPI: Write<u8>,
      CS: OutputPin,
      DC: OutputPin,
      RST: OutputPin,
      I2C: I2c,
{

    pub fn new() -> Self {
        Self {
            display: None,
            touchscreen: None,
        }
    }

    pub fn init_display<DELAYMS>(&mut self, spi: SPI, dc: DC, cs: CS, rst: RST, delay: &mut DELAYMS)
        where DELAYMS: DelayMs<u16>
    {
        let iface = SPIInterface::new(spi, dc, cs);
        let size = ili9341::DisplaySize240x320;
        let orient = ili9341::Orientation::LandscapeFlipped;
        let ili = ili9341::Ili9341::new(
            iface,
            rst,
            delay,
            orient,
            size).unwrap();
        self.display = Some(ili);
    }

    pub fn init_touchscreen(&mut self, i2c: I2C)
    {
        let ts = ft6236::FT6236::new(i2c);
        self.touchscreen = Some(ts);
    }
}
