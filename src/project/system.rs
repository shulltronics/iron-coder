//! This module represents a hardware system, i.e. a main board,
//! a set of peripheral boards, and the connections between them.

use std::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::board::Board;
use crate::board::pinout::InterfaceType;

/// A Connection is a physical bus connecting two Boards (e.g. I2C, GPIO, SPI, etc).
/// TODO - determine best way of representing a connection. Can it represent a bus connecting
/// more than one board? i.e. one I2C main device talking to multiple peripherals, or a CAN
/// network. 
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Connection {
    pub start_board: Board,
    pub end_board: Board,
    pub interface: InterfaceType,
}

impl Connection {

    pub fn new(start_board: Board, end_board: Board, iface_type: InterfaceType) -> Self {
        Self {
            start_board: start_board,
            end_board: end_board,
            interface: iface_type,
        }
    }

    pub fn display(&self, _ctx: &egui::Context, ui: &mut egui::Ui) -> egui::Response {
        let label = egui::Label::new(format!("{:?}", self)).sense(egui::Sense::click());
        ui.add(label)
    }
}

/// A system represents the development boards and their interconnections
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct System {
    /// The list of boards in this system. The first board in this list should always be a "main board"
    pub boards: Vec<Board>,
    /// The list of connections between boards. This is what the template generator will use to create
    /// the system module.
    pub connections: Vec<Connection>,
}

impl Default for System {
    fn default() -> Self {
        Self {
            boards: Vec::new(),
            connections: Vec::new(),
        }
    }
}

impl System {
    
}
