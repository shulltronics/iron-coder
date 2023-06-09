use log::{info, warn};

use std::path::Path;
use std::sync::Arc;

use egui::widget_text::RichText;
use egui::widgets::Button;

use crate::project::Project;
use crate::board::BoardMiniWidget;
use crate::app::icons::IconSet;

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ProjectViewType {
    FileTree,
    BoardsView,
    CrateView(String),
}

// this block contains the display related
// methods for showing the Project in egui.
impl Project {
    
    /// Recursively display the project directory.
    /// <dir> is the starting location, <level> is the recursion depth
    fn display_directory(&mut self, dir: &Path, level: usize, ctx: &egui::Context, ui: &mut egui::Ui) {
        let iconref: Arc<IconSet> = ctx.data_mut(|data| {
            data.get_temp("icons".into()).unwrap()
        });
        let icons = iconref.clone();
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
                    icons.get("file_icon").unwrap().texture_id(ctx),
                    egui::Vec2::new(7.0, 7.0),
                    text,
                ).frame(false);
                let resp = ui.add(button);
                if resp.clicked() {
                    self.code_editor.load_from_file(child.path().as_path()).unwrap_or_else(|_| warn!("error loading file contents"));
                }
            } else {
                // DIRECTORY case
                egui::CollapsingHeader::new(child.path().file_name().unwrap().to_str().unwrap()).show(ui, |ui| {
                    self.display_directory(child.path().as_path(), level+1, ctx, ui);
                });
            }
        }
    }

    /// show the terminal pane
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

    /// show the project tree in a Ui
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

    /// Show the project toolbar, with buttons to perform various actions
    pub fn display_project_toolbar(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        let iconref: Arc<IconSet> = ctx.data_mut(|data| {
            data.get_temp("icons".into()).expect("error loading shared icons!")
        });
        let icons = iconref.clone();
        ui.horizontal(|ui| {
            // COMPILE CODE
            let button = egui::widgets::Button::image_and_text(
                icons.get("build_icon").unwrap().texture_id(ctx),
                egui::Vec2::new(9.0, 9.0),
                " build project",
            ).frame(false);
            if ui.add(button).clicked() {
                self.build(ctx);
            }

            ui.separator();
            // LOAD CODE ONTO BOARD
            let button = egui::widgets::Button::image_and_text(
                icons.get("load_icon").unwrap().texture_id(ctx),
                egui::Vec2::new(9.0, 9.0),
                " load onto board",
            ).frame(false);
            if ui.add(button).clicked() {
                self.load_to_board(ctx);
            }

            ui.separator();
            // GENERATE PROJECT TEMPLATE
            if ui.button("Gen Template").clicked() {
                info!("generating project template");
                match self.generate_cargo_template(ctx) {
                    Ok(()) => {
                        info!("generate_cargo_template returned Ok(()).");
                    },
                    Err(e) => {
                        warn!("generate_cargo_template returned error: {:?}", e);
                    },
                }
            }

            ui.separator();
            // GENERATE SYSTEM MODULE
            if ui.button("Gen Sys Mod").clicked() {
                info!("generating system module...");
                match self.generate_system_module() {
                    Ok(()) => {
                        info!("generate_system_module returned Ok(()).");
                    },
                    Err(e) => {
                        warn!("generate_system_module returned error: {:?}", e);
                    },
                }
            }

        });
    }

    /// In the provided Ui, create a multi-column layout (tabs) that switches the current view state.
    fn display_sidebar_tabs(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui) {
        // show the tabs to switch between view modes
        ui.columns(2, |columns| {
            let mut new_view: ProjectViewType;
            let button = Button::new("File Explorer").frame(false);
            if columns[0].add(button).clicked() {
                new_view = ProjectViewType::FileTree;
                self.current_view = new_view;
            };
            // ui.separator();
            let button = Button::new("Project Boards").frame(false);
            if columns[1].add(button).clicked() {
                new_view = ProjectViewType::BoardsView;
                self.current_view = new_view;
            };
        });
    }

    pub fn show_crate_info(&mut self, crate_name: String) {
        self.current_view = ProjectViewType::CrateView(crate_name);
    }

    /// Show the project view
    pub fn display_project_sidebar(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {

        self.display_sidebar_tabs(ctx, ui);
        ui.separator();

        egui::containers::scroll_area::ScrollArea::both().show(ui, |ui| {
            // The sidebar will display information according to the current view
            match &self.current_view {
                ProjectViewType::BoardsView => {
                     // show the board widgets
                    for b in self.system.boards.clone().iter() {
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
                                            self.run_background_commands(&[cmd], ctx);
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
                                        self.show_crate_info(rc.clone());
                                    };
                                });
                            }
                        }
                    }
                },
                ProjectViewType::CrateView(s) => {
                    ui.label(s);
                    let code_snippets: &Path = Path::new("./assets/code-snippets/");
                    let mut snippet = self.load_snippets(code_snippets, s.clone()).unwrap();
                    let te = egui::TextEdit::multiline(&mut snippet)
                        .code_editor()
                        .interactive(false)
                        .desired_width(f32::INFINITY)
                        .frame(false);
                    let resp = ui.add(te);
                    let resp = resp.interact(egui::Sense::drag());
                    // check if the drag was released. if so, store the snippet in memory 
                    // so we can retrieve it in the CodeEditor
                    if resp.drag_released() {
                        info!("drag released! storing snippet in memory.");
                        let id = egui::Id::new("released_code_snippet");
                        ctx.memory_mut(|mem| {
                            mem.data.insert_temp(id, snippet.clone());
                        })
                    }
                },
                ProjectViewType::FileTree => {
                    // option to add a new top-level directory
                    let dir_button = egui::widgets::Button::new("+ dir/file").frame(false);
                    if ui.add(dir_button).clicked() {
                        self.new_file().unwrap_or_else(|_| warn!("couldn't create new file"));
                    }
                    // show the project tree
                    self.display_project_tree(ctx, ui);
                },
            }
        });
    }

    // Show the boards in egui "Area"s so we can move them around!
    pub fn display_system_editor(&mut self, ctx: &egui::Context, _ui: &mut egui::Ui) {
        
        let mut recs: Vec<(egui::Rect, egui::Rect)> = vec![(egui::Rect::NOTHING, egui::Rect::NOTHING); self.system.connections.len()];
        let mut board_to_remove = None;

        for (board_idx, board) in self.system.boards.iter_mut().enumerate() {
            // show the board in a Window
            let window = egui::Window::new(board.get_name())
                .open(&mut true)
                .title_bar(false)
                .resizable(false)
                .movable(true)
                .show(ctx, |ui| {
                    ui.add(BoardMiniWidget(board.clone()));
                    let mut connection_to_remove = None;
                    self.system.connections.iter().enumerate().for_each(|(connection_idx, connection)| {
                        // add the connection info to the Board Ui
                        let resp = connection.display(ctx, ui);
                        // remove it if it is clicked (TODO -- improve this)
                        if resp.clicked() { connection_to_remove = Some(connection_idx) }
                        // save the rect for future drawing
                        if *board == connection.end_board {
                            recs[connection_idx].1 = resp.rect;
                        }
                    });
                    if let Some(connection_to_remove) = connection_to_remove {
                        self.system.connections.remove(connection_to_remove);
                        recs.remove(connection_to_remove);
                    }
                }).unwrap().response;

                // create a right-clickable menu to add a connection from the selected board
            window.context_menu(|ui| {
                ui.menu_button("add connection", |ui| {
                    // for interface in all::<Interface>().collect::<Vec<_>>().iter() {
                    for po in board.get_pinout().iter() {

                        let label = format!("{:?}", po);
                        if ui.button(label).clicked() {
                            // self.system.connections.push(Connection { secondary_board_idx: 0, interface: po.clone().interface })
                        }
                    }
                });
                ui.menu_button("rust-analyser stuff", |ui| {
                    for s in board.ra_values.iter() {
                        if ui.label(format!("{:?}", s.label)).clicked() {
                            info!("{:?}", s);
                        }
                    }
                });
                
                if ui.button("print syntax tree for BSP").clicked() {
                    println!("{:#?}", board.log_syn_file_to_string());
                }

                if ui.button("update pinout with BSP info").clicked() {
                    if let Err(e) = board.update_pinout_from_bsp() {
                        warn!("{:?}", e);
                    }
                }

                if ui.button("remove board from system").clicked() {
                    // TODO -- also remove all connections that involved this board, to prevent a crash
                    board_to_remove = Some(board_idx);
                }
            });
        } // for each Board

        if let Some(board_idx) = board_to_remove {
            self.system.boards.remove(board_idx);
        }

        // iterate through connections and draw a line to represent each one
        // self.system.connections.iter().enumerate().for_each(|(idx, connection)| {
        //     // the length of our recs vector needs to be equal to the number of connections
        //     assert!(recs.len() == self.system.connections.len());
        //     let start = recs[idx].0.max;
        //     let end   = recs[idx].1.min;
        //     ui.painter().line_segment([start, end], egui::Stroke::new(2.0, egui::Color32::GREEN));
        // });

    }

}
