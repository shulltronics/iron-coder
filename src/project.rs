use std::path::{Path, PathBuf};
use std::io;

use std::vec::Vec;

use crate::board::Board;
use crate::editor::CodeEditor;

/// A Project represents the highest level of Iron Coder, which contains
/// a set of development boards and the firmware source code

// serialize and deserialize a project
pub struct Project {
    location: Option<PathBuf>,
    boards: Vec<Board>,
    repo: Option<CodeEditor>,
}

impl Default for Project {
    fn default() -> Self {
        Self {
            location: None,
            boards: Vec::new(),
            repo: None,
        }
    }
}

impl Project {
    // pub fn open(&self) -> io::Result<()> {
    //     Ok(())
    // }

    // pub fn save(&self) -> io::Result<()> {
    //     Ok(())
    // }
}