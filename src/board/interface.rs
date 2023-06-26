/// This module defines interfaces that a development board has

use serde::{Serialize, Deserialize};
use std::fmt;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum InterfaceDirection {
    Controller,
    Peripheral,
}

impl fmt::Display for InterfaceDirection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Interface {
    GPIO,
    ADC,
    PWM,
    UART,
    I2C(InterfaceDirection),
    SPI(InterfaceDirection),
    PIO,
    I2S(InterfaceDirection),
    USB(InterfaceDirection),
}

impl fmt::Display for Interface {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}