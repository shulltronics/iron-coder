//! This module represents a hardware system, i.e. a main board,
//! a set of peripheral boards, and the connections between them.

use std::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::board::{Board, BoardMiniWidget};
use crate::board::pinout::Interface;

/// This struct represents the development boards and their connecting interfaces
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct System {
    /// The list of boards in this system. The first board in this list should always be a "main board"
    pub boards: Vec<Board>,
    /// The list of connection between boards. This is what the template generator will use to create
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

/// A Connection is a physical bus connecting two Boards (e.g. I2C, GPIO, SPI, etc).
/// TODO - determine best way of representing a connection. Can it represent a bus connecting
/// more than one board? i.e. one I2C main device talking to multiple peripherals, or a CAN
/// network. 
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Connection {
    // pub main_board_idx: usize,
    pub secondary_board_idx: usize,
    pub interface: Interface,
}

impl Connection {

    pub fn new(secondary_idx: usize, interface: Interface) -> Self {
        Self {
            // main_board_idx: main_idx,
            secondary_board_idx: secondary_idx,
            interface: interface,
        }
    }

    pub fn display(&self, ctx: &egui::Context, ui: &mut egui::Ui) -> egui::Response {
        let label = egui::Label::new(format!("{:?}", self)).sense(egui::Sense::click());
        ui.add(label)
    }
}

//   maybe there is no such thing as a "default" connection?
// impl Default for Connection {
//     fn default() -> Self {
//         Self {

//         }
//     }
// }