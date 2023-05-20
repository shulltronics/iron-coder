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
pub fn get_boards(boards_dir: &Path) -> Vec<Board> {
    println!("Installing boards...");
    // iterate through and construct the boards
    for manufacturer in fs::read_dir(boards_dir).expect("Error opening boards directory!") {
        let man = manufacturer.expect("Error with entry!");
        println!("{:?}", man.path());
        if man.path().is_dir() {
            for b in fs::read_dir(man.path()).expect("Error opening manufacturer directory!") {
                let b = b.expect("Error with b!");
                println!("{:?}", b.path());
                if b.path().is_dir() {
                    for file in fs::read_dir(b.path()).expect("error reading files") {
                        let file = file.expect("error opening file");
                        println!("{:?}", file.path());
                    }
                }
            }
        }
    }
    return Vec::new();
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
pub struct Board {
    name: &'static str,
    standard: BoardStandards,
    manufacturer: &'static str,
}

impl Board {
    pub fn get_name(&self) -> &'static str {
        self.name
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

