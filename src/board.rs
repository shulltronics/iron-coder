use std::path::Path;
use std::fs;
use std::vec::Vec;
// this function reads the boards directory and returns a Vec in RAM
// the boards directory is structured as:
// boards/
// -- manufacturer/
// -- -- board/
// -- -- -- <name>.toml
// -- -- -- <name>.png
pub fn get_boards_test(boards_dir: &Path) -> Vec<Board> {
    let mut r = Vec::new();
    if let Ok(entries) = fs::read_dir(boards_dir) {
        for entry in entries {
            let entry = entry.expect("error with entry");
            // if the entry is a directory, recursively go get the files
            if entry.file_type().expect("error parsing file type").is_dir() {
                println!("recursing.. {:?}", entry.path());
                r.append(&mut get_boards_test(&entry.path()));
            } else if entry.path().extension().unwrap() == "toml" {
            // the entry is a board file in toml format
                println!("got file {:?}", entry.file_name());
                // TODO gracefully handle improperly formatted board files
                let toml_str = fs::read_to_string(entry.path()).unwrap();
                let b: Board = toml::from_str(&toml_str).unwrap();
                r.push(b);
            }
        }
    }
    return r;
}

// These are the various standard development board specifications
pub enum BoardStandards {
    Feather,
    Arduino,
    RaspberryPi,
    ThingPlus,
    MicroMod,
}

// The board struct defines a board type
use serde::Deserialize;
#[derive(Deserialize, Debug, Clone)]
pub struct Board {
    name: String,
    // standard: Option<BoardStandards>,
    manufacturer: String,
}

impl Board {
    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }
}

pub struct BoardSelectorWidget;

impl BoardSelectorWidget {
    pub fn new() -> BoardSelectorWidget {
        return Self;
    }
}

use egui::{Ui, Response};
use egui::widgets::Widget;
// Trait implementation
impl Widget for BoardSelectorWidget {
    
    fn ui(self, ui: &mut Ui) -> Response {
        let response = ui.allocate_response(egui::vec2(100.0, 200.0), egui::Sense::click());
        // let (pos, text_galley, response) = self.layout_in_ui(ui);
        // let rect = ui.painter().clip_rect();
        // ui.painter().rect_filled(
        //     egui::Rect::from_two_pos(egui::Pos2::ZERO, egui::Pos2::new(20.0, 20.0)),
        //     egui::Rounding::none(),
        //     egui::Color32::YELLOW,
        // );
        ui.painter().rect_stroke(response.rect, 0.0, (1.0, egui::Color32::WHITE));
        return response;
    }
}

impl Widget for Board {

    fn ui(self, ui: &mut Ui) -> Response {
        // let Self {
        //     name,
        //     manufacturer,
        // } = self;
        let response = ui.allocate_response(egui::vec2(100.0, 200.0), egui::Sense::click());
        // let (pos, text_galley, response) = self.layout_in_ui(ui);
        // let rect = ui.painter().clip_rect();
        // ui.painter().rect_filled(
        //     egui::Rect::from_two_pos(egui::Pos2::ZERO, egui::Pos2::new(20.0, 20.0)),
        //     egui::Rounding::none(),
        //     egui::Color32::YELLOW,
        // );
        ui.painter().rect_stroke(response.rect, 0.0, (1.0, egui::Color32::WHITE));
        ui.label(self.get_name());
        return response;
    }

}

