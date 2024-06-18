/// This module defines interfaces that a development board has
use enum_iterator::Sequence;

use syn;

use serde::{Serialize, Deserialize};
use std::fmt;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Sequence)]
#[non_exhaustive]
pub enum InterfaceDirection {
    Unknown,
    Input,
    Output,
    Bidirectional,
}

impl fmt::Display for InterfaceDirection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq, Sequence)]
#[non_exhaustive]
/// The various types of electrical interfaces we use with dev boards
pub enum InterfaceType {
    NONE,
    GPIO,
    ADC,
    PWM,
    UART,
    I2C,
    SPI,
}

impl fmt::Display for InterfaceType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Sequence)]
pub struct Interface {
    pub iface_type: InterfaceType,
    pub direction: InterfaceDirection,
}

impl fmt::Display for Interface {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Default for Interface {
    fn default() -> Self {
        Self {
            iface_type: InterfaceType::NONE,
            direction: InterfaceDirection::Unknown,
        }
    }
}

/// And InterfaceMapping is a map of an Interface to a set of pins on the Board.
/// TODO: I think a "pin" should be able to be referenced by multiple different criteria,
/// such as the "silkscreen labal", the physical pin number (i.e. counting around the board),
/// the logical pin number, or possibly some other criteria.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(default)]
pub struct InterfaceMapping {
    pub interface: Interface,
    pub pins: Vec<String>,
    #[serde(skip)]
    pub bsp_field: Option<syn::Field>,
}

impl Default for InterfaceMapping {
    fn default() -> Self {
        Self {
            interface: Interface::default(),
            pins: Vec::new(),
            bsp_field: None,
        }
    }
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