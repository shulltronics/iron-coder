use std::string::String;
use serde;
use egui::{Ui, Response};
use egui::widgets::Widget;

// This struct represents the code editor
#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct CodeEditor {
    code: String,
}

impl Default for CodeEditor {
    fn default() -> Self {
        Self {
            code: "// welcome to Iron Coder!".to_string()
        }
    }
}

impl CodeEditor {
    // todo
}

impl Widget for CodeEditor {
    fn ui(mut self, ui: &mut Ui) -> Response {
        ui.add(
            egui::TextEdit::multiline(&mut self.code)
                .font(egui::TextStyle::Monospace) // for cursor height
                .code_editor()
                .desired_rows(10)
                .lock_focus(true)
                .desired_width(f32::INFINITY)
                .frame(false),
                // .layouter(&mut layouter),
        )
    }
}