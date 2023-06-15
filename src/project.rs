use std::path::{Path, PathBuf};
use std::io;

use std::vec::Vec;

use rfd::FileDialog;

use crate::board::Board;
use crate::editor::CodeEditor;

/// A Project represents the highest level of Iron Coder, which contains
/// a set of development boards and the firmware source code directory

// serialize and deserialize a project
pub struct Project {
    location: Option<PathBuf>,
    boards: Vec<Board>,
    // repo: Option<CodeEditor>,
}

impl Default for Project {
    fn default() -> Self {
        Self {
            location: None,
            boards: Vec::new(),
            // repo: None,
        }
    }
}

impl Project {
    // pub fn new() -> Self {
    //     Ok(())
    // }

    pub fn open(&mut self) -> io::Result<()> {
        if let Some(pathbuf) = FileDialog::new().pick_folder() {
            println!("chosen path: {}", pathbuf.display().to_string());
            self.location = Some(pathbuf);
        } else {
            println!("error with folder dialog");
        }
        Ok(())
    }

    pub fn save(&self) -> io::Result<()> {
        if self.location != None {
            println!("project location: {:?}", self.location);
        } else {
            println!("project doesn't have a location!");
        }
        Ok(())
    }
}