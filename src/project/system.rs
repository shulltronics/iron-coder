use std::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::board::{Board, BoardMiniWidget};
use crate::board::interface::Interface;

// This struct represents the development boards and their connecting interfaces
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct System {
    pub boards: Vec<Board>,
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
    pub main_board_idx: usize,
    pub secondary_board_idx: usize,
    pub interface: Interface,
}

impl Connection {

    pub fn new(main_idx: usize, secondary_idx: usize, interface: Interface) -> Self {
        Self {
            main_board_idx: main_idx,
            secondary_board_idx: secondary_idx,
            interface: interface,
        }
    }

    pub fn display(&self, ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.label(format!("{:?}", self));
    }
}

//   maybe there is no such thing as a "default" connection?
// impl Default for Connection {
//     fn default() -> Self {
//         Self {

//         }
//     }
// }