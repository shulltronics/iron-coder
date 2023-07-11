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
/// TODO: I think a "pin" should be able to be referenced by multiple different criteria,
/// such as the "silkscreen labal", the physical pin number (i.e. counting around the board),
/// the logical pin number, or possibly some other criteria.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InterfaceMapping {
    pub interface: Interface,
    pub pins: Vec<usize>,
}

/// A Pinout is a description of the available interfaces on a Board
// #[derive(Serialize, Deserialize, Clone, Debug)]
// pub struct Pinout {
//     pinout: Vec<InterfaceMapping>,
// }

pub type Pinout = Vec<InterfaceMapping>;

// impl Default for Pinout {
//     fn default() -> Self {
//         Self {
//             pinout: Vec::new(),
//         }
//     }
// }

// impl Iterator for Pinout {
//     type Item = InterfaceMapping;
//     fn next(&self) -> Option<Self::Item> {
//         self.pinout.next();
//     }
// }