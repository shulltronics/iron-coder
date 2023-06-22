use log::{info, warn, debug};

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
        // If there is an open channel, see if we can get some data from it
        if let Some(rx) = &self.receiver {
            while let Ok(s) = rx.try_recv() {
                self.terminal_buffer += s.as_str();
            }
        }
        egui::CollapsingHeader::new("Terminal").show(ui, |ui| {
            egui::ScrollArea::both()
            .auto_shrink([false; 2])
            .stick_to_bottom(true)
            .show(ui, |ui| {
                if ui.add(
                    egui::TextEdit::multiline(&mut self.terminal_buffer)
                    .code_editor()
                    .interactive(false)
                    .desired_width(f32::INFINITY)
                    .frame(false)
                ).clicked() {
                    self.terminal_buffer.clear();
                }
            });
        });
    }

    // show the project tree in a Ui
    fn display_project_tree(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
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
                self.build(ctx);
            }

            ui.separator();

            let button = egui::widgets::Button::image_and_text(
                self.code_editor.icons.get("load_icon").unwrap().texture_id(ctx),
                egui::Vec2::new(9.0, 9.0),
                " load onto board",
            ).frame(false);
            if ui.add(button).clicked() {
                self.load_to_board(ctx);
            }
        });
    }

    pub fn display_project_sidebar(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        egui::containers::scroll_area::ScrollArea::both().show(ui, |ui| {
            // option to add a new top-level directory
            let dir_button = egui::widgets::Button::new("+ dir/file").frame(false);
            if ui.add(dir_button).clicked() {
                self.new_file().unwrap_or_else(|_| warn!("couldn't create new file"));
            }
            // show the project tree
            self.display_project_tree(ctx, ui);
            // show the board widgets
            for b in self.boards.clone().iter() {
                ui.add(b.clone());
                // show the required crates
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    let label = egui::RichText::new("Required Crates").underline();
                    ui.label(label);
                });
                if let Some(required_crates) = b.required_crates() {
                    for rc in required_crates.iter() {
                        ui.horizontal(|ui| {
                            if ui.link(rc).clicked() {
                                if let Some(path) = &self.location {
                                    let cmd = duct::cmd!("cargo", "-Z", "unstable-options", "-C", path.as_path().to_str().unwrap(), "add", rc.as_str());
                                    self.run_background_command(cmd, ctx);
                                } else {
                                    self.terminal_buffer += "save project first!\n";
                                }
                                
                            };
                        });
                    }
                }
                ui.separator();
                // show the related crates
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    let label = egui::RichText::new("Related Crates").underline();
                    ui.label(label);
                });
                if let Some(related_crates) = b.related_crates() {
                    for rc in related_crates.iter() {
                        ui.horizontal(|ui| {
                            if ui.link(rc).clicked() {
                                info!("TODO - deal with the related crate!")
                            };
                        });
                    }
                }
            }
        });
    }

    // Show the project editor page
    pub fn display_project_editor(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
            let label = RichText::new("Project Name").underline();
            ui.label(label);
            ui.text_edit_singleline(&mut self.name);

            let label = RichText::new("Project Boards").underline();
            ui.label(label);
        });
        // compute outer margin based on how many widgets to show:
        let width_per_board = 120.0;
        let mut margin_val = 0.0;
        let num_boards = self.boards.len();
        if num_boards > 0 {
            let w = ui.available_width();
            let needed = num_boards as f32 * width_per_board;
            margin_val = (w - needed) / 2.0;
        }
        egui::Frame::default().outer_margin(egui::Margin::symmetric(margin_val, 10.0)).show(ui, |ui| {
            if num_boards > 0 {
                ui.columns(num_boards, |columns| {
                    for (i, b) in self.boards.clone().into_iter().enumerate() {
                        let this_r = columns[i].add(BoardMiniWidget(b));
                        // ui.painter().rect_stroke(this_r.rect, 0.0, (1.0, egui::Color32::WHITE));
                        if this_r.clicked() {
                            self.boards.remove(i);
                        }
                    }
                });
            }
        });
    }
}