use log::{info, warn, debug};

use std::io;
use std::fs;
use std::path::{Path, PathBuf};

use std::collections::HashMap;
use std::vec::Vec;

// for invoking external programs
use std::process::Command;

use rfd::FileDialog;
use toml;

use serde::{Serialize, Deserialize};

use egui::widget_text::RichText;

use crate::board::Board;
use crate::editor::CodeEditor;

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

impl Project {
    
    // Helper function for printing both to logs and to built-in terminal
    fn info_logger(&mut self, msg: &str) {
        info!("{}", msg);
        let msg = msg.to_owned() + "\n";
        self.terminal_buffer += &msg;
    }

    pub fn borrow_name(&mut self) -> &mut String {
        &mut self.name
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
        } else {
            info!("project save aborted");
            return Ok(());
        }
        self.save()
    }

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
            ];
            let output = Command::new("cargo").args(args).output().unwrap();
            if output.status.success() {
                self.info_logger("successfully created cargo package");
            } else {
                warn!("couldn't create cargo package, maybe because \
                            it's already been created? {:?}", output.status);
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

    // A recursive directory display.
    // <dir> is the starting location, <level> is the recursion depth
    fn display_directory(&mut self, dir: &Path, level: usize, ctx: &egui::Context, ui: &mut egui::Ui) {
        // if the entry is a file, just show it
        // if the entry is a directory, show it, and if it's "open"
        //   also recursively display it's contents
        let children = dir.read_dir().unwrap();
        for _child in children {
            let child = _child.unwrap();
            let file_name = child.file_name().into_string().unwrap();
            let text = RichText::new(file_name);
            // FILE case
            if child.file_type().unwrap().is_file() {
                let button = egui::widgets::Button::image_and_text(
                    self.code_editor.icons.get("file_icon").unwrap().texture_id(ctx),
                    egui::Vec2::new(7.0, 7.0),
                    text,
                ).frame(false);
                let resp = ui.add(button);
                if resp.clicked() {
                    self.code_editor.load_from_file(child.path().as_path()).unwrap_or_else(|_| warn!("error loading file contents"));
                }
            } else {
                // DIRECTORY case
                // check if it's expanded or collapsed via our HashMap
                let mut is_visible: bool = match self.file_tree.get(&child.path()) {
                    None => {
                        // if the PathBuf isn't in the mapping yet, add it as not visible
                        let _ = self.file_tree.insert(child.path(), false);
                        false
                    },
                    Some(visible) => {
                        *visible
                    }
                };
                // select the proper folder icon
                let folder_icon = match is_visible {
                    true => self.code_editor.icons.get("folder_open_icon").unwrap(),
                    false => self.code_editor.icons.get("folder_closed_icon").unwrap(),
                };
                // construct and display the entry
                let button = egui::widgets::Button::image_and_text(
                    folder_icon.texture_id(ctx),
                    egui::Vec2::new(7.0, 7.0),
                    text,
                ).frame(false);
                let resp = ui.add(button);
                if resp.clicked() {
                    is_visible = !is_visible;
                    self.file_tree.insert(child.path(), is_visible);
                }
                // now recurse if the directory is marked visible.
                if is_visible {
                    ui.indent(level, |ui| {
                        self.display_directory(child.path().as_path(), level+1, ctx, ui);
                    });
                }
            }
        }
    }

    // show the terminal pane
    pub fn display_terminal(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new("Terminal").show(ui, |ui| {
            egui::ScrollArea::both()
            .auto_shrink([false; 2])
            .stick_to_bottom(true)
            .show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut self.terminal_buffer)
                    .code_editor()
                    .interactive(false)
                    .desired_width(f32::INFINITY)
                    .frame(false)
                );
            });
        });
    }

    // show the project tree in a Ui
    pub fn display_project_tree(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        let project_folder = match &self.location {
            None => {
                ui.label("There is currently no folder associated with this project. Please save it somewhere.");
                return;
            },
            Some(l) => l.clone(),   // clone here so we have no refs to self
        };
        let dir = project_folder.as_path();
        self.display_directory(dir, 0, ctx, ui);
    }

    // show the project toolbar
    pub fn display_project_toolbar(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // Buttons for various code actions, like compilation
            let button = egui::widgets::Button::image_and_text(
                self.code_editor.icons.get("build_icon").unwrap().texture_id(ctx),
                egui::Vec2::new(9.0, 9.0),
                " build project",
            ).frame(false);
            if ui.add(button).clicked() {
                self.build();
            }

            ui.separator();

            let button = egui::widgets::Button::image_and_text(
                self.code_editor.icons.get("load_icon").unwrap().texture_id(ctx),
                egui::Vec2::new(9.0, 9.0),
                " load onto board",
            ).frame(false);
            if ui.add(button).clicked() {
                self.load_to_board();
            }
        });
    }

}