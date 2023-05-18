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

pub const FEATHER_RP2040: Board = Board {
    name: "Adafruit Feather RP2040",
    standard: BoardStandards::Feather,
    manufacturer: "Adafruit Industries",
};

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