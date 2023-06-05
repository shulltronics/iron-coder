// This module contains functionality for the code editor.
// The namesake struct CodeEditor contains the state of the editor,
// which includes:
// * multiple tabs of source files

use std::string::String;
use serde;
use egui::{Ui, Response};
use egui::widgets::text_edit::TextEdit;
use egui::widgets::Widget;

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct CodeEditor {
    code: String,
    // editor: TextEdit,
}

impl Default for CodeEditor {
    fn default() -> Self {
        Self {
            code: "// welcome to Iron Coder!".to_string()
            // editor: egui::TextEdit::multiline(&mut String::from(""))
            //             .font(egui::TextStyle::Monospace) // for cursor height
            //             .code_editor()
            //             .desired_rows(10)
            //             .lock_focus(true)
            //             .desired_width(f32::INFINITY)
            //             .frame(false),
                        // .layouter(&mut layouter),
        }
        
    }
}

impl CodeEditor {
    // todo
}

impl Widget for CodeEditor {
    fn ui(&mut self, ui: &mut Ui) -> Response {
        let CodeEditor { code } = self;
        ui.add(
            egui::TextEdit::multiline(code)
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