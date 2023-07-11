//! This module represents a hardware system, i.e. a main board,
//! a set of peripheral boards, and the connections between them.

use std::vec::Vec;
use serde::{Deserialize, Serialize};

use quote::quote;
use syn;
use prettyplease;
use proc_macro2::TokenStream;
use std::fs;

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
    /// Generate a module based on the system
    pub fn generate_system_module(&self) -> std::io::Result<()> {
        let num_boards = self.boards.len();
        let output_tokens = quote! {
            #![no_std]
            pub struct System {
                num_boards: u8,
            }

            impl System {
                pub fn new() -> Self {
                    Self {
                        num_boards: #num_boards,
                    }
                }
            }
        };
        let syn_code: syn::File = syn::parse2(output_tokens).unwrap();
        let code = prettyplease::unparse(&syn_code);
        fs::write("tmp/output.rs", code.as_str())?;
        Ok(())
    }
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
