//! Title: Iron Coder Project Module - Module
//! Description: This module contains the Project struct and its associated functionality.

use clap::Error;
use egui::InputState;
use log::{info, warn, debug};

// use std::error::Error;
use std::io::BufRead;
use std::io;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use std::process::Child;
use std::process::ChildStdin;
use std::process::ChildStdout;
use std::sync::{Arc, Mutex};

use rfd::FileDialog;

use serde::{Serialize, Deserialize};

use crate::board::Board;
use crate::app::code_editor::CodeEditor;

pub mod display;
use display::ProjectViewType;
use display::BottomPaneViewType;


pub mod egui_helpers;

mod system;
mod test;

use system::System;

use git2::Repository;

const PROJECT_FILE_NAME: &'static str = ".ironcoder.toml";

pub type Result = core::result::Result<(), ProjectIOError>;

// Simulator Imports
pub mod simulator_helpers;
use toml::de::from_str;
use toml::Value;

#[derive(Debug, Deserialize, Clone)]
struct Package {
    name: String,
    version: Option<String>,
    edition: Option<String>,
}



#[non_exhaustive]
#[derive(Debug)]
pub enum ProjectIOError {
    FilePickerAborted,
    NoMainBoard,
    NoProjectTemplate,
    NoProjectDirectory,
    FilesystemError,
    LoadToTomlError,
}

/// A Project represents the highest level of Iron Coder, which contains
/// a main, programmable development board, a set of peripheral development boards,
/// and the project/source code directory
#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Project {
    name: String,
    location: Option<PathBuf>,
    pub system: System,
    #[serde(skip)]
    pub code_editor: CodeEditor,
    #[serde(skip)]
    terminal_buffer: String,
    persistant_buffer: String,
    output_buffer: String,
    simulator_command_buffer: String,
    pub update_directory: bool,
    directory: String,
    #[serde(skip)]
    receiver: Option<std::sync::mpsc::Receiver<String>>,
    current_view: ProjectViewType,
    #[serde(skip)]
    bottom_view: Option<BottomPaneViewType>,
    #[serde(skip)]
    pub known_boards: Vec<Board>,
    #[serde(skip)]
    repo: Option<Repository>,
    #[serde(skip)]
    pub terminal_app: Option<Child>,
    pub spawn_child: bool,
    #[serde(skip)]
    terminal_stdin: Option<ChildStdin>,
    #[serde(skip)]
    terminal_stdout: Option<ChildStdout>,
    #[serde(skip)]
    renode_process: Option<Child>,
    renode_output: Arc<Mutex<String>>,
    #[serde(skip)]
    stdin: Option<Arc<Mutex<std::process::ChildStdin>>>,
}

// backend functionality for Project struct
impl Project {
    // Helper function for printing both to logs and to built-in terminal
    fn info_logger(&mut self, msg: &str) {
        info!("{}", msg);
        let msg = msg.to_owned() + "\n";
        self.output_buffer += &msg;
    }

    pub fn borrow_name(&mut self) -> &mut String {
        return &mut self.name;
    }

    pub fn has_main_board(&self) -> bool {
        if let Some(_) = self.system.main_board {
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
                } else {
                    self.system.main_board = Some(board.clone());
                }
            },
            false => {
                match board.is_discrete(){
                    true => {
                        // TODO Can't display multiple discrete components right now -> logic at project/display.rs "display_system_editor_boards", also removing board finds first occurrence in list
                        if self.system.discrete_components.contains(&board) {
                            info!("project <{}> already contains board <{:?}>", self.name, board);
                            self.terminal_buffer += "project already contains that component\n";
                            return;
                        } else {
                            self.system.discrete_components.push(board.clone());
                        }
                    },
                    false => {
                        // don't duplicate a peripheral board
                        if self.system.peripheral_boards.contains(&board) {
                            info!("project <{}> already contains board <{:?}>", self.name, board);
                            self.output_buffer += "project already contains that board\n";
                            return;
                        } else {
                            self.system.peripheral_boards.push(board.clone());
                        }
                    },

                }
            }
        }
    }

    /// Populate the project board list via the app-wide 'known boards' list
    fn load_board_resources(&mut self) {
        info!("updating project boards from known boards list.");
        for b in self.system.get_all_boards_mut().iter_mut() {
            // returns true if the current, project board is equal to the current known_board
            let predicate = |known_board: &&Board| {
                return known_board == b;
            };
            if let Some(known_board) = self.known_boards.iter().find(predicate) {
                **b = known_board.clone();
            } else {
                warn!("Could not find the project board in the known boards list. Was the project manifest \
                       generated with an older version of Iron Coder?")
            }
        }
    }

    /// This method will reload the project based on the current project location
    pub fn reload(&mut self) -> Result {
        if let Some(location) = self.location.clone() {
            self.load_from(&location)
        } else {
            Err(ProjectIOError::NoProjectDirectory)
        }
    }

    /// Load a project from a specified directory, and sync the board assets.
    pub fn load_from(&mut self, project_directory: &Path) -> Result {
        let project_file = project_directory.join(PROJECT_FILE_NAME);
        let toml_str = match fs::read_to_string(project_file) {
            Ok(s) => s,
            Err(e) => {
                warn!("error reading project file: {:?}", e);
                return Err(ProjectIOError::FilesystemError);
            },
        };
        let p: Project = match toml::from_str(&toml_str) {
            Ok(p) => {
                p
            },
            Err(_e) => {
                return Err(ProjectIOError::LoadToTomlError)
            }
        };
        // Now load in certain fields without overwriting others:
        self.code_editor.close_all_tabs();
        self.name = p.name;
        self.location = Some(project_directory.to_path_buf());
        self.system = p.system;
        self.current_view = p.current_view;
        // sync the assets with the global ones
        self.load_board_resources();
        self.terminal_buffer.clear();
        // Open the repo in the project directory
        self.repo = match Repository::open(self.get_location()) {
            Ok(repo) => Some(repo),
            Err(e) => {
                info!("Could not open repo: {}", e);
                None
            },
        };

        Ok(())
    }

    /// Prompt the user to select project directory to open
    pub fn open(&mut self) -> Result {
        if let Some(project_directory) = FileDialog::new().pick_folder() {
            self.load_from(&project_directory)
        } else {
            info!("project open aborted");
            Err(ProjectIOError::FilePickerAborted)
        }
    }

    /// Open a file dialog to select a project folder, and then call the save method
    /// TODO - make file dialog have default directory
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
            // TODo: find template directory based on "programmable board" (for now just use board 0) -- No longer relevant?
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

    // TODO - have this save all project files, maybe, except the target directory -- FIXED (note: currently only saves all open tabs)
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

            self.code_editor.save_all().unwrap_or_else(|_| warn!("error saving tabs!"));
            Ok(())
        }
    }

    /// Build the code with Cargo
    pub fn build(&mut self, ctx: &egui::Context) {
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
            self.info_logger("Successfully flashed board.");
        } else {
            self.info_logger("project needs a valid working directory before building");
        }

    }

    // // this is used when the simulator is asked to load the program
    // pub fn build_and_create_script(&mut self, ctx: &egui::Context) {
    //     // Build the project
    //     self.build(ctx);
    
    //     // Get the ELF file path
    //     if let Some(elf_path) = self.get_elf_file_path(self.location.as_ref().expect("No project location found.")) {
    //         let script_path = Path::new(".\\src\\app\\simulator\\renode\\scripts\\generated/currentScript.resc");
    
    //         // Create Renode script
    //         if let Err(e) = simulator_helpers::create_renode_script(&elf_path, script_path) {
    //             self.info_logger(&format!("Error creating Renode script: {}", e));
    //         } else {
    //             self.info_logger("Renode script created successfully.");
    //         }
    //     } else {
    //         self.info_logger("No ELF file found.");
    //     }
    // }
    
    // // since different projects might have different names, we dynamically obtain them so we have the right file
    // fn get_package_name_from_toml(&self, project_path: &Path) -> Option<String> {
    //     // Load the Cargo.toml content
    //     let toml_path = project_path.join("Cargo.toml");
    //     let toml_content = fs::read_to_string(toml_path).ok()?;
    
    //     // Parse the TOML content
    //     let parsed_toml: toml::Value = toml_content.parse().ok()?;
    
    //     // Extract the package name
    //     parsed_toml
    //         .get("package")
    //         .and_then(|pkg| pkg.get("name"))
    //         .and_then(|name| name.as_str())
    //         .map(|s| s.to_string())
    // }
    
    // // Get the path to the ELF file after building the project
    // fn get_elf_file_path(&self, project_path: &Path) -> Option<PathBuf> {
    //     let target = self.get_project_build_target(project_path)
    //     .unwrap_or_else(|| "thumbv6m-none-eabi".to_string());

    //     if let Some(package_name) = self.get_package_name_from_toml(project_path) {
    //         let target_dir = project_path.join(format!("target/{}/debug", target));
    //         let elf_file_path = target_dir.join(&package_name);
    
    //         // Try with and without an extension
    //         if elf_file_path.exists() {
    //             Some(elf_file_path)
    //         } else {
    //             // Try adding the .elf extension
    //             let elf_with_extension = elf_file_path.with_extension("elf");
    //             if elf_with_extension.exists() {
    //                 Some(elf_with_extension)
    //             } else {
    //                 None
    //             }
    //         }
    //     } else {
    //         None
    //     }
    // }

    // fn get_project_build_target(&self, project_path: &Path) -> Option<String> {
    //     let cargo_config_path = project_path.join(".cargo/config.toml");
        
    //     if cargo_config_path.exists() {
    //         let config_content = fs::read_to_string(&cargo_config_path).ok()?;
    //         let parsed_toml: Value = config_content.parse().ok()?;

    //         return parsed_toml.get("build")
    //             .and_then(|build| build.get("target"))
    //             .and_then(|target| target.as_str())
    //             .map(|s| s.to_string());
    //     }

    //     // Fallback: If `.cargo/config.toml` doesn’t exist, check `Cargo.toml`
    //     let cargo_toml_path = project_path.join("Cargo.toml");

    //     if cargo_toml_path.exists() {
    //         let cargo_content = fs::read_to_string(&cargo_toml_path).ok()?;
    //         let parsed_toml: Value = cargo_content.parse().ok()?;

    //         return parsed_toml.get("package")
    //             .and_then(|pkg| pkg.get("metadata"))
    //             .and_then(|metadata| metadata.get("build-target"))
    //             .and_then(|target| target.as_str())
    //             .map(|s| s.to_string());
    //     }

    //     None
    // }


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
    pub fn generate_cargo_template(&mut self, ctx: &egui::Context) -> Result {
        info!("generating project template");
        let mut cmds: Vec<duct::Expression> = vec![];
        if let Some(mb) = &self.system.main_board {
            if let Some(template_dir) = mb.get_template_dir() {
                let cmd = duct::cmd!(
                    "cargo",
                    "generate",
                    "--path",
                    template_dir.as_path().to_str().unwrap(),
                    "--name",
                    self.name.clone(),
                    "--destination",
                    self.get_location(),
                    "--init",
                );
                cmds.push(cmd);
            } else {
                return Err(ProjectIOError::NoProjectTemplate);
            }
            // iterate through BSP paths and add the crates to the project
            // TODO: This needs to be changed, likely an issue with
            // updating the crates in the main toml file. Figure out why!
            /*
            for b in self.system.get_all_boards() {
                if let Some(local_bsp) = b.bsp_path {
                    let cmd = duct::cmd!(
                        "cargo",
                        "-Z",
                        "unstable-options",
                        "-C",
                        self.location.clone().unwrap(),
                        "add",
                        "--path",
                        local_bsp,
                    );
                    cmds.push(cmd);
                }
            }
            */
            self.run_background_commands(&cmds, ctx);
        } else {
            return Err(ProjectIOError::NoMainBoard);
        }
        // Create a repo to store code
        self.repo = match Repository::init(self.get_location()) {
            Ok(repo) => Some(repo),
            Err(_e) => return Err(ProjectIOError::NoProjectDirectory),
        };

        Ok(())
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
