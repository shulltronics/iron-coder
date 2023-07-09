/// This module defines interfaces that a development board has
use enum_iterator::Sequence;

use serde::{Serialize, Deserialize};
use std::fmt;

#[derive(Serialize, Deserialize, Clone, Debug, Sequence)]
pub enum InterfaceDirection {
    Controller,
    Peripheral,
}

impl fmt::Display for InterfaceDirection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Sequence)]
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

/// And InterfaceMapping is a map of an Interface to a set of pins on the Board.
pub struct InterfaceMapping {
    interface: Interface,
    pins: Vec<usize>,
}

/// A Pinout is a description of the available interfaces on a Board
pub struct Pinout {
    pinout: Vec<InterfaceMapping>,
}