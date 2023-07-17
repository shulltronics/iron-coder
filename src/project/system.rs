//! This module represents a hardware system, i.e. a main board,
//! a set of peripheral boards, and the connections between them.

use std::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::board::Board;
use crate::board::pinout::InterfaceType;

pub type Result = core::result::Result<(), SystemError>;

#[non_exhaustive]
#[derive(Debug)]
pub enum SystemError {
    BoardNotInSystem,
    UnknownError,
}

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
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct System {
    /// The main, programmable board in the system.
    pub main_board: Option<Board>,
    /// The list of peripheral boards in the system.
    pub peripheral_boards: Vec<Board>,
    /// The list of connections between boards. This is what the template generator will use to create
    /// the system module.
    pub connections: Vec<Connection>,
}

impl System {

    /// Return a vector of all the system boards. If there is no main board, this returns an
    /// empty vector.
    pub fn get_all_boards(&self) -> Vec<Board> {
        let mut boards = Vec::new();
        if let Some(mb) = self.main_board.clone() {
            boards.push(mb);
        }
        boards.append(&mut self.peripheral_boards.clone());
        return boards;
    }

    pub fn get_all_boards_mut(&mut self) -> Vec<&mut Board> {
        let mut boards = Vec::new();
        if let Some(ref mut mb) = self.main_board {
            boards.push(mb);
        }
        self.peripheral_boards.iter_mut().for_each(|board| {
            boards.push(board);
        });
        return boards;
    }

    /// Try to remove the provided Board from the system. If everything is good, return Ok(()),
    /// otherwise return an error indicating what went wrong.
    pub fn remove_board(&mut self, board: Board) -> Result {
        if let Some(ref mb) = self.main_board {
            if *mb == board {
                self.main_board = None;
                return Ok(())
            }
        }
        if let Some(idx) = self.peripheral_boards.iter().position(|elem| *elem == board) {
            self.peripheral_boards.remove(idx);
            return Ok(());
        }
        return Err(SystemError::BoardNotInSystem);
    }

    // Generate a module based on the system. Lots to improve here. For now, this just saves
    // the module to the project root (i.e. doesn't account for the existance of a Cargo project).
    // pub fn generate_system_module(&mut self) -> Result<(), String> {

    //     let SystemBoardTokens {
    //         use_statements,
    //         field_type_token_streams,
    //         field_constructor_token_streams,
    //         ..
    //     } = match self.gather_board_fields() {
    //         Ok(sbt) => sbt,
    //         Err(e) => {
    //             warn!("error in gather_board_fields method: {:?}", e);
    //             return Err("error gathering board fields".to_string());
    //         }
    //     }
        
    //     ;

    //     /************* MODULE CODE HERE *************/
    //     let output_tokens = quote!
    //     {
    //         #(#use_statements)*
    //         // #(use #crate_identifiers);*

    //         // todo - include needed imports

    //         pub struct System {
    //             #(#field_type_token_streams),*
    //         }

    //         impl System {
    //             pub fn new() -> Self {
    //                 Self {
    //                     #(#field_constructor_token_streams),*
    //                 }
    //             }
    //         }

    //         // #(#connection_impls)*

    //     };
    //     /************* End Module Code *************/

    //     // now output the module code to a file, passing through the prettyplease formatter
    //     let syn_code: syn::File = match syn::parse2(output_tokens) {
    //         Ok(syn_code) => syn_code,
    //         Err(e) => {
    //             warn!("couldn't parse output_tokens! {:?}", e);
    //             syn::parse_str("// error generating module").unwrap()
    //         }
    //     };
    //     let code = prettyplease::unparse(&syn_code);
    //     fs::write(self.get_location() + "/src/sys_mod_output_testing.rs", code.as_str()).unwrap();
        
    //     Ok(())

    // }

}
