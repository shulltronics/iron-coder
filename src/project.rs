use std::path::{Path, PathBuf};
use std::io;
use std::fs;

use std::vec::Vec;

use rfd::FileDialog;
use toml;

use serde::{Serialize, Deserialize};

use egui::widget_text::RichText;
use egui::Sense;
use egui::widgets::Label;

use std::collections::HashMap;

use crate::board::Board;
use crate::editor::CodeEditor;
use crate::icons::Icon;

/// A Project represents the highest level of Iron Coder, which contains
/// a set of development boards and the project/source code directory

const PROJECT_FILE_NAME: &'static str = ".ironcoder.toml";

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct Project {
    name: String,
    // #[serde(skip)]
    location: Option<PathBuf>,
    #[serde(skip)]
    file_tree: HashMap<PathBuf, bool>,
    // file_tree: Option<Vec<(bool, PathBuf)>>,
    boards: Vec<Board>,
    #[serde(skip)]
    pub code_editor: CodeEditor,
}

impl Default for Project {
    fn default() -> Self {
        let mut editor = CodeEditor::default();
        let code_path = Path::new("./boards/Adafruit/Feather_RP2040/examples/blinky/src/main.rs");
        editor.load_from_file(code_path).unwrap();
        let code_path = Path::new("./boards/Adafruit/Feather_RP2040/examples/blinky/src/test.rs");
        editor.load_from_file(code_path).unwrap();
        Self {
            name: "".to_string(),
            location: None,
            file_tree: HashMap::new(),
            boards: Vec::new(),
            code_editor: editor,
        }
    }
}

impl Project {
    // pub fn new() -> Self {
    //     Ok(())
    // }

    pub fn get_name(&self) -> String {
        return self.name.clone();
    }

    pub fn add_board(&mut self, board: Board) {
        // don't duplicate a board
        if self.boards.contains(&board) {
            println!("project <{}> already contains board {:?}", self.name, board);
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
                    println!("error opening project.. perhaps the file is misformatted?\n{:?}", e);
                    return Ok(());
                }
            };
            *self = p;
            self.location = Some(project_folder);
        } else {
            println!("project open aborted");
        }
        Ok(())
    }

    pub fn save(&mut self) -> io::Result<()> {
        if self.location == None {
            // TODO -- move this to "save_as" and call that method here
            if let Some(p) = FileDialog::new().pick_folder() {
                self.location = Some(p);
            } else {
                println!("project save aborted");
                return Ok(());
            }
        }
        let project_folder = self.location.clone().unwrap();
        let project_file = project_folder.join(PROJECT_FILE_NAME);
        println!("saving project to {}", project_file.display().to_string());
        let contents: String = toml::to_string(self).unwrap();
        fs::write(project_file, contents)?;
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
            if child.file_type().unwrap().is_file() {
                let button = egui::widgets::Button::image_and_text(
                    self.code_editor.icons.get("file_icon").unwrap().texture_id(ctx),
                    egui::Vec2::new(7.0, 7.0),
                    text,
                ).frame(false);
                if ui.add(button).clicked() {
                    match self.code_editor.load_from_file(child.path().as_path()) {
                        Ok(_) => (),
                        Err(_) => println!("error opening file"),
                    }
                }
            } else {
                // here it's a directory
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

    // Show the project tree in a Ui
    pub fn display(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        // ui.style_mut().spacing.indent = 10.0;
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

    // pub fn save_as()
}