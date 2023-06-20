use log::warn;

use std::path::Path;

use egui::widget_text::RichText;

use crate::project::Project;
use crate::board::BoardMiniWidget;

// this block contains the display related
// methods for showing the Project in egui.
impl Project {
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

    // Show the project editor page
    // new_project will have either a blank project or a copy of the current project,
    // depending on how we got here (i.e. via edit current or create new).
    pub fn display_project_editor(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
            ui.horizontal(|ui| {
                ui.label("Project Name: ");
                // with this we can edit an existing project
                ui.text_edit_singleline(&mut self.name);
            });
            ui.label("Current project boards:");
            ui.label(self.boards.len().to_string());
            ui.horizontal(|ui| {
                for (i, b) in self.boards.clone().into_iter().enumerate() {
                    // let _b = b.clone();
                    // println!("{:?}", b);
                    ui.add_sized(egui::vec2(100.0, 100.0), BoardMiniWidget(b));
                    // ui.add_sized(egui::vec2(100.0, 100.0), b);
                }
            });
        });
    }
}