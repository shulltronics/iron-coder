//! This module describes an Iron Coder project.

use log::{info, warn, debug};

// use std::error::Error;
use std::io::BufRead;
use std::io;
use std::fs;
use std::path::{Path, PathBuf};

use std::vec::Vec;

use rfd::FileDialog;
use toml;

use serde::{Serialize, Deserialize};

use crate::board::Board;
use crate::app::code_editor::CodeEditor;

pub mod display;
use display::ProjectViewType;

pub mod egui_helpers;
pub mod node_graph;
use node_graph::SystemEditorState;

mod system;
use system::System;

pub mod code_generation;

const PROJECT_FILE_NAME: &'static str = ".ironcoder.toml";

/// A Project represents the highest level of Iron Coder, which contains
/// a main, programmable development board, a set of peripheral development boards,
/// and the project/source code directory
#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct Project {
    name: String,
    location: Option<PathBuf>,
    pub system: System,
    #[serde(skip)]
    pub code_editor: CodeEditor,
    #[serde(skip)]
    terminal_buffer: String,
    graph_editor: SystemEditorState,
    #[serde(skip)]
    receiver: Option<std::sync::mpsc::Receiver<String>>,
    current_view: ProjectViewType,
}

impl Default for Project {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            location: None,
            system: System::default(),
            code_editor: CodeEditor::default(),
            terminal_buffer: String::new(),
            graph_editor: SystemEditorState::default(),
            receiver: None,
            current_view: ProjectViewType::BoardsView,
        }
    }
}

impl Clone for Project {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            location: self.location.clone(),
            system: self.system.clone(),
            code_editor: CodeEditor::default(),
            terminal_buffer: self.terminal_buffer.clone(),
            graph_editor: self.graph_editor.clone(),
            receiver: None,
            current_view: self.current_view.clone(),
        }
    }
}

use egui_node_graph::NodeTemplateTrait;
// backend functionality for Project struct
impl Project {
    
    // Helper function for printing both to logs and to built-in terminal
    fn info_logger(&mut self, msg: &str) {
        info!("{}", msg);
        let msg = msg.to_owned() + "\n";
        self.terminal_buffer += &msg;
    }

    pub fn borrow_name(&mut self) -> &mut String {
        return &mut self.name;
    }

    pub fn borrow_boards(&mut self) -> &mut Vec<Board> {
        return &mut self.system.boards;
    }

    pub fn has_main_board(&self) -> bool {
        if self.system.boards.len() > 0 && self.system.boards[0].is_main_board() {
            return true;
        } else {
            return false;
        }
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
        match board.is_main_board() {
            true => {
                if self.has_main_board() {
                    info!("project already contains a main board! aborting.");
                    return;
                }
            },
            false => {
                // don't duplicate a board
                if self.system.boards.contains(&board) {
                    info!("project <{}> already contains board <{:?}>", self.name, board);
                    self.terminal_buffer += "project already contains that board\n";
                    return;
                }
            }
        }
        self.system.boards.push(board.clone());
        let node_kind = board.clone();
        let user_state = &mut self.system;
        let new_node = self.graph_editor.graph.add_node(
            node_kind.node_graph_label(user_state),
            node_kind.user_data(user_state),
            |graph, node_id| node_kind.build_node(graph, user_state, node_id),
        );
        self.graph_editor.node_positions.insert(
            new_node,
            egui::Pos2::ZERO,
        );
        self.graph_editor.node_order.push(new_node);
    }

    /// Populate the project board list via the app-wide 'known boards' list
    pub fn load_board_resources(&mut self, known_boards: Vec<Board>) {
        for b in self.system.boards.iter_mut() {
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

    /// Open a file dialog to select a project folder, and then call the save method
    pub fn save_as(&mut self, create_containing_folder: bool) -> io::Result<()> {
        if let Some(mut project_folder) = FileDialog::new().pick_folder() {
            // if indicated, create a new folder for the project (with same name as project)
            if create_containing_folder {
                project_folder = project_folder.join(self.name.clone());
                fs::create_dir(project_folder.as_path())?;
            }
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
            // TODo: find template directory based on "programmable board" (for now just use board 0)
            // if let Some(template_dir) = self.system.boards[0].get_template_dir() {
            //     // copy_recursive(template_dir, project_dir)
            //     let options = fs_extra::dir::CopyOptions::new();
            //     for entry in std::fs::read_dir(template_dir).unwrap() {
            //         let entry = entry.unwrap().path();
            //         if let Err(e) = fs_extra::copy_items(&[entry.clone()], self.location.clone().unwrap(), &options) {
            //             warn!("couldn't copy template item {:?} to new project folder; {:?}", entry, e);
            //         }
            //     }
            // }
        } else {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "project save aborted!"));
        }
        self.save()
    }

    // TODO - have this save all project files, maybe, except the target directory
    pub fn save(&mut self) -> io::Result<()> {
        if self.location == None {
            info!("no project location, calling save_as...");
            self.save_as(true)
        } else {
            let project_folder = self.location.clone().unwrap();
            let project_file = project_folder.join(PROJECT_FILE_NAME);
            info!("saving project file to {}", project_file.display().to_string());
            
            match toml::to_string(self) {
                Ok(contents) => {
                    fs::write(project_file, contents)?;
                },
                Err(e) => {
                    warn!("couldn't save project to toml file!! {:?}", e);
                }
            }
            Ok(())
        }
    }

    /// Build the code with Cargo
    fn build(&mut self, ctx: &egui::Context) {
        // Make sure we have a valid path
        if let Some(path) = &self.location {
            info!("building project at {}", path.display().to_string());
            self.code_editor.save_all().unwrap_or_else(|_| warn!("error saving tabs!"));
            let cmd = duct::cmd!("cargo", "-Z", "unstable-options", "-C", path.as_path().to_str().unwrap(), "build");
            self.run_background_commands(&[cmd], ctx);
        } else {
            self.info_logger("project needs a valid working directory before building");
        }
    }

    /// Load the code (for now using 'cargo run')
    fn load_to_board(&mut self, ctx: &egui::Context) {
        if let Some(path) = &self.location {
            let cmd = duct::cmd!("cargo", "-Z", "unstable-options", "-C", path.as_path().to_str().unwrap(), "run");
            self.run_background_commands(&[cmd], ctx);
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

    /// This method will run a series of command sequentially on a separate
    /// thread, sending their output through the channel to the project's terminal buffer
    /// TODO - fix bug that calling this command again before a former call's thread is 
    ///   complete will overwrite the rx channel in the Project object. Possible solution
    ///   might be to add a command to a queue to be evaluated.
    fn run_background_commands(&mut self, cmds: &[duct::Expression], ctx: &egui::Context) {
        // create comms channel
        let context = ctx.clone();
        let commands = cmds.to_owned();
        let (tx, rx) = std::sync::mpsc::channel();
        self.receiver = Some(rx);
        let _ = std::thread::spawn(move || {
            for cmd in commands.iter() {
                let reader = cmd.stderr_to_stdout().unchecked().reader().unwrap();
                let mut lines = std::io::BufReader::new(reader).lines();
                while let Some(line) = lines.next() {
                    let line = line.unwrap() + "\n";
                    debug!("sending line through channel");
                    tx.send(line).unwrap();
                    context.request_repaint();
                }
            }
            info!("leaving thread");
        });
    }

    /// Generate the Cargo project template based on the main board template (if it has one).
    /// The template will be written to the project directory.
    /// TODO - generally more useful error returns, i.e. if the cargo generate command returns a 
    /// non-zero exit status, or if the project directory already contains a Cargo project.
    pub fn generate_cargo_template(&mut self, ctx: &egui::Context) -> Result<(), String> {
        info!("generating project template");
        if self.system.boards.len() == 0 {
            return Err(String::from("The system needs a main board before generating the template!"));
        }
        // TODO -- put this somewhere configurable, like in a CliOpts struct :)
        let template_dir = Path::new("templates/basic/");
        let cmd = duct::cmd!(
            "cargo",
            "generate",
            "--path",
            template_dir.to_str().unwrap(),
            "--name",
            self.name.clone(),
            "--destination",
            self.get_location(),
            "--init",
        );
        self.run_background_commands(&[cmd], ctx);
        Ok(())
    }

    pub fn add_crates_to_project(&mut self, ctx: &egui::Context) {
        // TESTING
        for _b in self.system.boards.clone().iter() {
            // nothing for now
        }

        if let Some(project_folder) = self.location.clone() {
            for b in self.system.boards.clone().iter() {
                if let Some(rc) = b.required_crates() {
                    info!("installing required crates for board {:?}", b);
                    let mut cmds: Vec<duct::Expression> = rc.iter().map(|c| {
                        duct::cmd!("cargo", "-Z", "unstable-options", "-C",
                            project_folder.as_path().to_str().unwrap(), "add",
                            c)
                    }).collect();
                    let init_cmd = duct::cmd!("cargo", "-Z", "unstable-options", "-C",
                        project_folder.as_path().to_str().unwrap(), "init",
                        "--name", self.name.as_str(), "--vcs", "none");
                    cmds.insert(0, init_cmd);
                    self.run_background_commands(cmds.as_slice(), ctx);
                }
            }
        }
    }

    /// Attempt to load code snippets for the provided crate
    fn load_snippets(&self, base_dir: &Path, crate_name: String) -> io::Result<String> {
        let snippets_dir = base_dir.join(crate_name.clone());
        if let Ok(true) = snippets_dir.try_exists() {
            for entry in snippets_dir.read_dir().unwrap() {
                let entry = entry.unwrap();
                let contents = std::fs::read_to_string(entry.path())?;
                return Ok(contents);
            }
        } else {
            warn!("couldn't load code snippets for crate {}", crate_name);
        }
        Ok("".to_string())
    }

}