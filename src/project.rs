use log::{info, warn};

use std::io;
use std::fs;
use std::path::PathBuf;
use fs_extra;

use std::collections::HashMap;
use std::vec::Vec;

// for invoking external programs
use std::process::Command;

use rfd::FileDialog;
use toml;

use serde::{Serialize, Deserialize};

use crate::board::Board;
use crate::editor::CodeEditor;

pub mod display;

/// A Project represents the highest level of Iron Coder, which contains
/// a set of development boards and the project/source code directory

const PROJECT_FILE_NAME: &'static str = ".ironcoder.toml";

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct Project {
    name: String,
    location: Option<PathBuf>,
    #[serde(skip)]
    file_tree: HashMap<PathBuf, bool>,
    // file_tree: Option<Vec<(bool, PathBuf)>>,
    boards: Vec<Board>,
    #[serde(skip)]
    pub code_editor: CodeEditor,
    #[serde(skip)]
    terminal_buffer: String,
}

impl Default for Project {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            location: None,
            file_tree: HashMap::new(),
            boards: Vec::new(),
            code_editor: CodeEditor::default(),
            terminal_buffer: String::new(),
        }
    }
}

impl Clone for Project {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            location: self.location.clone(),
            file_tree: self.file_tree.clone(),
            boards: self.boards.clone(),
            code_editor: CodeEditor::default(),
            terminal_buffer: self.terminal_buffer.clone(),
        }
    }
}

// backend functionality for Project struct
impl Project {
    
    // Helper function for printing both to logs and to built-in terminal
    fn info_logger(&mut self, msg: &str) {
        info!("{}", msg);
        let msg = msg.to_owned() + "\n";
        self.terminal_buffer += &msg;
    }

    pub fn get_name(&self) -> String {
        return self.name.clone();
    }

    pub fn get_location(&self) -> String {
        if let Some(project_folder) = &self.location {
            // let s = project_folder.display().to_string();
            return project_folder.display().to_string().clone();
        } else {
            return String::from("");
        }
    }

    pub fn add_board(&mut self, board: Board) {
        // don't duplicate a board
        if self.boards.contains(&board) {
            info!("project <{}> already contains board <{:?}>", self.name, board);
            self.terminal_buffer += "project already contains that board\n";
            return;
        }
        self.boards.push(board);
    }

    pub fn get_boards(&self) -> Vec<Board> {
        return self.boards.clone();
    }

    // this method will populate the project board list via the app-wide
    // 'known boards' list
    pub fn load_board_resources(&mut self, known_boards: Vec<Board>) {
        for b in self.boards.iter_mut() {
            // returns true if the current, project board is equal to the current known_board
            let predicate = |known_board: &&Board| {
                return known_board == &b;
            };
            if let Some(known_board) = known_boards.iter().find(predicate) {
                *b = known_board.clone();
            } else {
                warn!("Could not find the project board in the known boards list. Was the project manifest \
                       generated with an older version of Iron Coder?")
            }
        }
    }

    // TODO -- when populating Boards list, get fields from known app wide boards list
    pub fn open(&mut self) -> io::Result<()> {
        if let Some(project_folder) = FileDialog::new().pick_folder() {
            let project_file = project_folder.join(PROJECT_FILE_NAME);
            let toml_str = fs::read_to_string(project_file)?;
            let p: Project = match toml::from_str(&toml_str) {
                Ok(p) => {
                    p
                },
                Err(e) => {
                    warn!("error opening project. perhaps the file is misformatted? Err: {:?}", e);
                    self.info_logger("error opening project");
                    return Ok(());
                }
            };
            *self = p;
            self.location = Some(project_folder);
        } else {
            info!("project open aborted");
        }
        Ok(())
    }

    pub fn save_as(&mut self) -> io::Result<()> {
        if let Some(project_folder) = FileDialog::new().pick_folder() {
            // check if there is an existing .ironcoder.toml file that we might overwrite
            for entry in std::fs::read_dir(&project_folder).unwrap() {
                if entry.unwrap().file_name().to_str().unwrap() == PROJECT_FILE_NAME {
                    warn!("you might be overwriting an existing Iron Coder project! \
                           Are you sure you wish to continue?");
                    self.terminal_buffer += "beware of overwriting and existing project file!\n";
                    return Ok(());
                }
            }
            self.location = Some(project_folder);
            // TODO copy over template files to the new project directory
            // 1. find template directory based on "programmable board" (for now just use board 0)
            // 2. recursively copy those files into project_folder
            if let Some(template_dir) = self.boards[0].get_template_dir() {
                // copy_recursive(template_dir, project_dir)
                let options = fs_extra::dir::CopyOptions::new();
                for entry in std::fs::read_dir(template_dir).unwrap() {
                    let entry = entry.unwrap().path();
                    if let Err(e) = fs_extra::dir::copy(entry.clone(), self.location.clone().unwrap(), &options) {
                        warn!("couldn't copy template item {:?} to new project folder; {:?}", entry, e);
                    }
                }
            }
        } else {
            info!("project save aborted");
            return Ok(());
        }
        self.save()
    }

    // TODO - have this save all project files, maybe, except the target directory
    pub fn save(&mut self) -> io::Result<()> {
        if self.location == None {
            info!("no project location, calling save_as...");
            self.save_as()
        } else {
            let project_folder = self.location.clone().unwrap();
            let args = [
                "-Z",
                "unstable-options",
                "-C",
                project_folder.as_path().to_str().unwrap(),
                "init",
                "--name",
                self.name.as_str(),
                "--vcs",
                "none",
            ];
            let output = Command::new("cargo").args(args).output().unwrap();
            if output.status.success() {
                self.info_logger("successfully created cargo package");
            } else {
                let utf8 = std::str::from_utf8(output.stderr.as_slice()).unwrap();
                warn!("couldn't create cargo package, maybe because \
                            it's already been created? {}", utf8);
            }
            let project_file = project_folder.join(PROJECT_FILE_NAME);
            info!("saving project to {}", project_file.display().to_string());
            let contents: String = toml::to_string(self).unwrap();
            fs::write(project_file, contents)?;
            Ok(())
        }
    }

    // builds the code
    fn build(&mut self) {
        // Make sure we have a valid path
        if let Some(path) = &self.location {
            info!("building project at {}", path.display().to_string());
            self.code_editor.save_all().unwrap_or_else(|_| warn!("error saving tabs!"));
            let args = [
                "-Z",
                "unstable-options",
                "-C",
                path.as_path().to_str().unwrap(),
                "build"
            ];
            // let args = ["version"];
            let mut build_command = Command::new("cargo");
            build_command.args(args);
            match build_command.output() {
                Ok(output) => {
                    let mut utf8 = std::str::from_utf8(output.stderr.as_slice()).unwrap();
                    self.terminal_buffer += utf8;
                    utf8 = std::str::from_utf8(output.stdout.as_slice()).unwrap();
                    self.terminal_buffer += utf8;
                },
                Err(e) => {
                    warn!("error executing cargo build. Err: {:?}", e);
                },
            }
        } else {
            self.info_logger("project needs a valid working directory before building");
        }
    }

    // loads the code (for now using 'cargo run')
    fn load_to_board(&mut self) {
        if let Some(path) = &self.location {
            let args = [
                "-Z",
                "unstable-options",
                "-C",
                path.as_path().to_str().unwrap(),
                "run"
            ];
            // let args = ["version"];
            let mut build_command = Command::new("cargo");
            build_command.args(args);
            match build_command.output() {
                Ok(output) => {
                    // TODO -- should I send stdout too?
                    let mut utf8 = std::str::from_utf8(output.stderr.as_slice()).unwrap();
                    self.terminal_buffer += utf8;
                    utf8 = std::str::from_utf8(output.stdout.as_slice()).unwrap();
                    self.terminal_buffer += utf8;
                },
                Err(e) => {
                    warn!("error executing cargo run. Err: {:?}", e);
                },
            }
        } else {
            self.info_logger("project needs a valid working directory before building");
        }
    }

    pub fn new_file(&mut self) -> io::Result<()> {
        if self.location == None {
            self.info_logger("must save project before adding files/directories");
            return Ok(());
        }
        if let Some(pathbuf) = FileDialog::new().set_directory(self.location.clone().unwrap()).save_file() {
            fs::File::create_new(pathbuf)?;
        } else {
            warn!("error getting file path");
        }
        Ok(())
    }

}