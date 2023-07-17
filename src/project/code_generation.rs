//! Implementation of metaprogramming tasks (i.e. code generation)

// use log::{info, warn};
// use std::fs;
// use quote::{quote, format_ident};
// use prettyplease;
// use syn::{
//     self,
//     visit::Visit,
// };

// use crate::project::Project;
// use crate::board::{pinout, Board};

// impl Project {

//     /// For each board in our system, generate the TokenStream for it's field
//     /// in the System struct. Return these in a Vec.
//     fn gather_board_fields(&mut self) -> Result<SystemBoardTokens, BspParseError> {

//         for board in self.system.boards.iter_mut() {
//             match board.load_bsp_info() {
//                 Ok(_) => (),
//                 Err(e) => warn!("error parsing BSP for board {}: {:?}", board.get_name(), e),
//             }
//         }

//         return Ok(tokens);

//     }

//     /// Iterate through the system connections, and create a TokenStream for each one
//     /// that will be a field in the System struct related to that connection. For example,
//     /// If we have an I2C connection to a display, it might show up in the System struct.
//     fn gather_system_connection_tokens(&mut self) -> Result<(), BspParseError> {
//         Ok(())
//     }

    
// }