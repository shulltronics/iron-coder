// This module contains functionality for the code editor.
// The namesake struct CodeEditor contains the state of the editor,
// which includes:
// * multiple tabs of source files

use std::string::String;
use serde;
use egui::Ui;
use egui::containers::scroll_area::ScrollArea;

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
    // I would honestly prefer to implement the CodeEditor display
    // via the Widget trait (see below -- commented out). But I was
    // fighting the borrow checker too much. This seems to work.
    pub fn display(&mut self, ctx: &egui::Context, ui: &mut Ui) {
        let CodeEditor { code } = self;
        // control pane for editor actions
        egui::TopBottomPanel::bottom("editor_control_panel").show(ctx, |ui| {
            ui.label("TODO -- editor control pane");
        });

        ui.add(
            egui::TextEdit::multiline(code)
                .font(egui::TextStyle::Monospace) // for cursor height
                .code_editor()
                .desired_rows(10)
                .lock_focus(true)
                .desired_width(f32::INFINITY)
                .frame(false),
                // .layouter(&mut layouter),
        );
    }

}

// impl Widget for CodeEditor {
//     fn ui(&mut self, ui: &mut Ui) -> Response {
//         let CodeEditor { code } = self;
//         ui.add(
//             egui::TextEdit::multiline(code)
//                 .font(egui::TextStyle::Monospace) // for cursor height
//                 .code_editor()
//                 .desired_rows(10)
//                 .lock_focus(true)
//                 .desired_width(f32::INFINITY)
//                 .frame(false),
//                 // .layouter(&mut layouter),
//         )
//     }
// }