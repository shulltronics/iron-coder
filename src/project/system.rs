//! Title: Iron Coder Poject Module - System
//! Description: This  module represents a hardware system, i.e. a main board,
//! a set of peripheral boards, and the connections between them.

use log::{info, warn};

use std::collections::HashSet;
use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};

use syn::Ident;
use quote::quote;

use crate::board::Board;
use crate::board::pinout::InterfaceMapping;

pub type Result = core::result::Result<(), SystemError>;

#[non_exhaustive]
#[derive(Debug)]
pub enum SystemError {
    BoardNotInSystemError,
}

/// A Connection is a physical bus connecting two Boards (e.g. I2C, GPIO, SPI, etc).
/// TODO - determine best way of representing a connection. Can it represent a bus connecting
/// more than one board? i.e. one I2C main device talking to multiple peripherals, or a CAN
/// network. 
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Connection {
    /// The name of the connection. Iron Coder will search for this name
    /// in your source code to determine which variable is associated with this connection.
    pub name: String,
    pub start_board: Board,
    pub start_pin: String,
    pub end_board: Board,
    pub end_pin: String,
    pub interface_mapping: InterfaceMapping,
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
    /// An optional board + pin for the current in-progress connecion.
    pub in_progress_connection_start: Option<(Board, String)>,
    pub in_progress_connection_end: Option<(Board, String)>,
}

/// A datastructure that will hold all of the information we need to populate the System module.
#[derive(Default)]
struct TokenStreamAccumulator {
    /// A set of crate Idents that need to be included in the system module.
    required_bsp_crates: HashSet<Ident>,
    // /// A vector of <field>: <type> to include in the System struct declaration.
    // struct_field_and_type_list: Vec<TokenStream>,
    // /// A vector of <field>: <constructor> to include in the System struct constructor.
    // struct_field_and_constructor_list: Vec<TokenStream>,
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

    /// Return a vector of mutable references to each board. This is used for reloading board
    /// resources on startup, or when opening a project from the filesystem.
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

    /// Try to remove the provided Board from the system, along with all of it's connections.
    /// If everything is good, return Ok(()), otherwise return an error indicating what went wrong.
    pub fn remove_board(&mut self, board: Board) -> Result {
        if let Some(ref mb) = self.main_board {
            if *mb == board {
                self.main_board = None;
                self.remove_connections_involving_board(board);
                return Ok(())
            }
        }
        if let Some(idx) = self.peripheral_boards.iter().position(|elem| *elem == board) {
            self.peripheral_boards.remove(idx);
            self.remove_connections_involving_board(board);
            return Ok(());
        }
        return Err(SystemError::BoardNotInSystemError);
    }

    /// Iterate through the connection list, and remove connections that involve the provided board
    fn remove_connections_involving_board(&mut self, board: Board) {
        self.connections = self.connections.iter().filter(|elem| {
            !((**elem).start_board == board || (**elem).end_board == board)
        }).cloned().collect();
    }

    /// Generate a module based on the system. Lots to improve here. For now, this just saves
    /// the module to the project root (i.e. doesn't account for the existance of a Cargo project).
    pub fn generate_system_module(&mut self, save_to: &Path) -> Result {

        // Fold through the list of connections, and capture the required information
        let TokenStreamAccumulator {
            required_bsp_crates,
            ..
        } = self.connections.iter().fold(TokenStreamAccumulator::default(), |mut acc, elem| {

            let Connection {
                start_board,
                end_board,
                ..
            } = elem;

            // get starting board info
            if let Some(start_board_bsp_info) = &start_board.bsp_parse_info {
                info!("  found some bsp info");
                if let Some(start_board_crate_ident) = &start_board_bsp_info.bsp_crate_identifier {
                    info!("    found a crate ident");
                    acc.required_bsp_crates.insert(start_board_crate_ident.clone());
                }
            }

            // get ending board info
            if let Some(end_board_bsp_info) = &end_board.bsp_parse_info {
                if let Some(end_board_crate_ident) = &end_board_bsp_info.bsp_crate_identifier {
                    acc.required_bsp_crates.insert(end_board_crate_ident.clone());
                }
            }

            return acc;
        });

        info!("after folding, num required crates is {}", required_bsp_crates.len());

        let r = required_bsp_crates.iter();

        /************* MODULE CODE HERE *************/
        let output_tokens = quote!
        {
            #(use #r;)*

            // todo - include needed imports

            pub struct System {
                // #(#struct_field_and_type_list),*
            }

            impl System {
                pub fn new() -> Self {
                    Self {
                        // #(#struct_field_and_constructor_list),*
                    }
                }
            }

            // #(#connection_impls)*

        };
        /************* End Module Code *************/

        // now output the module code to a file, passing through the prettyplease formatter
        let syn_code: syn::File = match syn::parse2(output_tokens) {
            Ok(syn_code) => syn_code,
            Err(e) => {
                warn!("couldn't parse output_tokens! {:?}", e);
                syn::parse_str("// error generating module").unwrap()
            }
        };
        let code = prettyplease::unparse(&syn_code);
        match fs::write(save_to, code.as_str()) {
            Ok(_) => (),
            Err(e) => {
                warn!("error writing code to {:?}: {:?}", save_to.display(), e);
            }
        }
        
        Ok(())

    }


}
