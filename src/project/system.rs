use std::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::board::{Board, BoardMiniWidget};

// This struct represents the development boards and their connecting interfaces
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct System {
    pub boards: Vec<Board>,
}

impl Default for System {
    fn default() -> Self {
        Self {
            boards: Vec::new(),
        }
    }
}

impl System {

}