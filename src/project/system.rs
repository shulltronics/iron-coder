//! This module represents a hardware system, i.e. a main board,
//! a set of peripheral boards, and the connections between them.

use log::warn;

use std::vec::Vec;
use serde::{Deserialize, Serialize};

use quote::quote;
use syn;
use prettyplease;
use proc_macro2::TokenStream;
use std::fs;

use crate::board::{Board, BoardMiniWidget};
use crate::board::pinout::Interface;

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
    /// Generate a module based on the system
    pub fn generate_system_module(&self) -> std::io::Result<()> {

        // Get relevant info about the system
        let system_fields: Vec<proc_macro2::TokenStream> = 
            self.boards.iter().map(|board| {
                let bsp_name = quote::format_ident!("{}", "todo_get_bsp_name");
                let name = board.get_name().replace(" ", "_").to_ascii_lowercase();
                let board_name = quote::format_ident!("{}", name);
                quote! {
                    #board_name: #bsp_name
                }
            }).collect();
        let num_boards = self.boards.len();
        let connection_impls: Vec<proc_macro2::TokenStream> =
            self.connections.iter().enumerate().map(|(idx, connection)| {
                let ident = quote::format_ident!("connection_{}_todo", idx);
                quote! {
                    impl System {
                        pub fn #ident() {

                        }
                    }
                }
            }).collect();

        /************* MODULE CODE HERE *************/
        let output_tokens = quote!
        {
            #![no_std]

            // todo - include needed imports

            pub struct System {
                #(#system_fields),*,
                num_boards: u8,
            }

            impl System {
                pub fn new() -> Self {
                    Self {
                        num_boards: #num_boards,
                    }
                }
            }

            #(#connection_impls)*

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
        fs::write("tmp/output.rs", code.as_str())?;
        
        Ok(())

    }
}
