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

use crate::board::Board;
use crate::editor::CodeEditor;

/// A Project represents the highest level of Iron Coder, which contains
/// a set of development boards and the project/source code directory

const PROJECT_FILE_NAME: &'static str = ".ironcoder.toml";

#[derive(Serialize, Deserialize)]
pub struct Project {
    name: String,
    #[serde(skip)]
    location: Option<PathBuf>,
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

    // Show the project tree in a Ui
    pub fn display(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        let project_folder = match &self.location {
            None => {
                ui.label("To show the project tree, save this project somewhere");
                return;
            },
            Some(l) => l,
        };
        let children = project_folder.as_path().read_dir().unwrap();
        for _child in children {
            let child = _child.unwrap();
            let file_name = child.file_name().into_string().unwrap();
            let text = RichText::new(file_name);
            let label = Label::new(text).sense(Sense::click());
            if child.file_type().unwrap().is_file() {
                if ui.add(label).clicked() {
                    self.code_editor.load_from_file(child.path().as_path());
                }
            } else {
                ui.add(label);
            }
            // ui.label(child.unwrap().file_name().into_string().unwrap());
        }
    }

    // pub fn save_as()
}