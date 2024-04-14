//! Title: Iron Coder Project Module - Display
//! Description: This file contains methods that help display
//! the main project window using equi. It also contains some
//! helper functions for drawing connections between pins on
//! the system editor.

use egui::{Key, Response};
use egui_extras::RetainedImage;
use log::{info, warn};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use egui::widget_text::RichText;
use egui::widgets::Button;

use git2::{Repository, StatusOptions};

use crate::board::Board;
use crate::{project::Project, board};
use crate::app::icons::IconSet;
use crate::app::{Mode, Warnings, Git};


use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub enum ProjectViewType {
    #[default]
    BoardsView,
    FileTree,
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
                    icons.get("file_icon").unwrap().clone(),
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
        let send_string = "";

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
                ui.add(
                    egui::TextEdit::multiline(&mut self.terminal_buffer)
                    .code_editor()
                    .interactive(false)
                    .desired_width(f32::INFINITY)
                    .frame(false)
                )
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
    pub fn display_project_toolbar(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, git_things: &mut Git) {
        let iconref: Arc<IconSet> = ctx.data_mut(|data| {
            data.get_temp("icons".into()).expect("error loading shared icons!")
        });
        let icons = iconref.clone();
        ui.horizontal(|ui| {
            // COMPILE CODE
            let button = egui::widgets::Button::image_and_text(
                icons.get("build_icon").unwrap().clone(),
                " build project",
            ).frame(false);
            if ui.add(button).clicked() {
                self.build(ctx);
            }

            ui.separator();
            // LOAD CODE ONTO BOARD
            let button = egui::widgets::Button::image_and_text(
                icons.get("load_icon").unwrap().clone(),
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
                info!("attempting to generate system module...");
                let src_location = match &self.location {
                    Some(l) => l.join("src/system.rs"),
                    None => {
                        info!("can't generate module without a project location.");
                        return;
                    },
                };
                match self.system.generate_system_module(&src_location) {
                    Ok(()) => {
                        info!("generate_system_module returned Ok(()).");
                    },
                    Err(e) => {
                        warn!("generate_system_module returned error: {:?}", e);
                    },
                }
            }

            ui.separator();
            let button = Button::image_and_text(
                icons.get("trash_icon").unwrap().clone(),
                " clear terminal",
            ).frame(false);
            if ui.add(button).clicked() {
                self.terminal_buffer.clear();
            }
            // Open a window to add changes
            // Commit the changes to the git repo with a user message
            ui.separator();

            if ui.button("Commit").clicked() {
                // Open the repo
                let repo = match Repository::open(self.get_location()) {
                    Ok(repo) => repo,
                    Err(e) => {
                        panic!("Error opening repository: {:?}", e);
                    }
                };

                let mut status_options = StatusOptions::new();
                status_options.include_untracked(true);

                // Get the status of the repo
                let repo_statuses = repo.statuses(Some(&mut status_options));

                // Check if there are any changes or new files and save them in a vector
                let mut changes: Vec<String> = Vec::new();
                for entry in repo_statuses.unwrap().iter() {
                    if entry.status().contains(git2::Status::WT_NEW) || entry.status().contains(git2::Status::WT_MODIFIED){
                        changes.push(entry.path().unwrap().to_string());
                    }
                }

                // Print the changes
                info!("Changes to be committed:");
                for change in changes.iter() {
                    info!("{}", change);
                }

                for change in changes.iter() {
                    let mut index = repo.index().unwrap();
                    index.remove_all([change.clone()].iter(), None).unwrap();
                    index.write().unwrap();
                }

                // Open a window to choose the changes to commit
                git_things.display = true;
                git_things.changes = changes;
                git_things.repo = Some(repo);
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

    /// Show the crate info
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
                    let boards = self.system.get_all_boards();
                    // Now, show the board widgets
                    for b in boards.iter() {
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

    /// Display the list of available boards in a window, and return one if it was clicked
    pub fn display_known_boards(&mut self, ctx: &egui::Context, should_show: &mut bool) -> Option<board::Board> {

        let mut board: Option<board::Board> = None;
        // create the window
        let response = egui::Window::new("Boards")
        .open(should_show)
        .collapsible(false)
        .resizable(false)
        .movable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            // Create a grid-based layout to show all the board widgets
            let available_width = ui.available_width();
            let mut num_cols = (available_width / 260.0) as usize;
            if num_cols == 0 {
                num_cols = 1;
            }
            egui::containers::scroll_area::ScrollArea::vertical().show(ui, |ui| {
                ui.columns(num_cols, |columns| {
                    for (i, b) in self.known_boards.clone().into_iter().enumerate() {
                        let col = i % num_cols;
                        // When a board is clicked, add it to the new project
                        if columns[col].add(board::display::BoardSelectorWidget(b)).clicked() {
                            board = Some(self.known_boards[i].clone());
                        }
                    }
                });
            });
        });

        if response.is_some() {
            // unwrap ok here because we check that response is Some.
            ctx.move_to_top(response.unwrap().response.layer_id);
        }

        return board;

    }

    // Show the boards in egui "Area"s so we can move them around!
    pub fn display_system_editor_boards(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {

        let mut pin_locations: HashMap<(Board, String), egui::Pos2> = HashMap::new();

        // iterate through the system boards and draw them on the screen
        for (board_idx, board) in self.system.get_all_boards().iter_mut().enumerate() {

            let scale_id = egui::Id::new("system_editor_scale_factor");
            // set the editor scale factor in memory:
            let mut scale = ctx.data_mut(|data| {
                data.get_temp_mut_or(scale_id, 5.0).clone()
            });
            if ctx.input(|io| io.key_pressed(egui::Key::Q)) {
                scale += 0.5;
            } else if ctx.input(|io| io.key_pressed(egui::Key::W)) {
                scale -= 0.5;
            }
            ctx.data_mut(|data| {
                data.insert_temp(scale_id, scale);
            });

            // Get the response of the board/pin Ui
            let response = egui::Area::new(board_idx.to_string()).show(ctx, |ui| {

                let mut pin_clicked: Option<String> = None;

                if let Some(svg_board_info) = board.clone().svg_board_info {
                    let retained_image = RetainedImage::from_color_image(
                        "pic",
                        svg_board_info.image,
                    );

                    let display_size = svg_board_info.physical_size * scale;

                    let image_rect = retained_image.show_max_size(ui, display_size).rect;

                    // iterate through the pin_nodes of the board, and check if their rects (properly scaled and translated)
                    // contain the pointer. If so, actually draw the stuff there.
                    for (pin_name, mut pin_rect) in board.clone().svg_board_info.unwrap().pin_rects {
                        // scale the rects the same amount that the board image was scaled
                        pin_rect.min.x *= scale;
                        pin_rect.min.y *= scale;
                        pin_rect.max.x *= scale;
                        pin_rect.max.y *= scale;
                        // translate the rects so they are in absolute coordinates
                        pin_rect = pin_rect.translate(image_rect.left_top().to_vec2());
                        pin_locations.insert((board.clone(), pin_name.clone()), pin_rect.center());

                        // render the pin overlay, and check for clicks/hovers
                        let r = ui.allocate_rect(pin_rect, egui::Sense::click());
                        if r.clicked() {
                            pin_clicked = Some(pin_name.clone());
                        }
                        if r.hovered() {
                            ui.painter().circle_filled(r.rect.center(), r.rect.height()/2.0, egui::Color32::GREEN);
                        }
                        r.clone().on_hover_text(String::from(board.get_name()) + ":" + &pin_name);
                        r.clone().context_menu(|ui| {
                            ui.label("a pin-level menu option");
                        });

                        // see if a current connection is happening from this pin. If so,
                        // of if the button was just clicked, update the start location.
                        ctx.data_mut(|data| {
                            // let this_project = &mut self;
                            let id = egui::Id::new("connection_in_progress");
                            if let Some((ref conn_board, ref pin)) = self.system.in_progress_connection_start {
                                // if a connection is in progress, update the start location if this pin is the start pin
                                if conn_board == board && pin == &pin_name {
                                    data.insert_temp(id, r.rect.center());
                                // otherwise, if the current pin was clicked, it must be the end connection!
                                } else if r.clicked() {
                                    data.remove::<egui::Pos2>(id);
                                    let conn_start = self.system.in_progress_connection_start.clone();
                                    // let start_board = conn_start.unwrap().0;
                                    self.system.in_progress_connection_end = Some((board.clone(), pin_name.clone()));
                                    let c = super::system::Connection {
                                        start_board: conn_start.clone().unwrap().0,
                                        start_pin: conn_start.clone().unwrap().1.clone(),
                                        end_board: board.clone(),
                                        end_pin: pin_name.clone(),
                                        interface_mapping: board::pinout::InterfaceMapping::default(),
                                    };
                                    self.system.connections.push(c);
                                }
                            } else if r.clicked() {
                                info!("inserting connection position data");
                                data.insert_temp(id, r.rect.center());
                                self.system.in_progress_connection_start = Some((board.clone(), pin_name.clone()));
                            }
                        });

                    }

                }

                return pin_clicked;

            });

            // extract response from board (i.e. the egui Area), and from pin
            let board_response = response.response;
            let pin_response = response.inner;

            // Actions for board-level stuff
            board_response.context_menu(|ui| {
                ui.menu_button("pinout info", |ui| {
                    for po in board.get_pinout().iter() {
                        let label = format!("{:?}", po);
                        if ui.button(label).clicked() {
                            info!("No action coded for this yet.");
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
                if ui.button("remove board from system").clicked() {
                    self.system.remove_board(board.clone()).unwrap_or_else(|_| {
                        warn!("error removing board from system.");
                    });
                }
            });

            // Actions for pin-level stuff
            if let Some(pin) = pin_response {
                info!("pin {} clicked!", pin);
            }

        } // for each Board

        // check for any key presses that might end the current in-progress connection.
        // be careful to avoid deadlocks in the ctx access closure!
        if let Some(_) = ctx.input(|io| {
            if io.key_pressed(egui::Key::Escape) {
                return Some(());
            } else {
                return None;
            }
        }) {
            ctx.data_mut(|data| {
                let id = egui::Id::new("connection_in_progress");
                data.remove::<egui::Pos2>(id);
                self.system.in_progress_connection_start = None;
            });
        }

        // check if a connection is in progress. Be sure to use the painter outside of the data
        // closure to avoid deadlock situation.
        if let Some(sp) = ctx.data(|data| {
            let id = egui::Id::new("connection_in_progress");
            data.get_temp::<egui::Pos2>(id)
        }) {
            if let Some(ep) = ctx.pointer_latest_pos() {
                draw_connection(ctx, ui, sp, ep, egui::Color32::GREEN);
            }
        }

        // go through the system connections and see if this pin is a part of any of them
        for connection in self.system.connections.iter() {
            // get the start and end pin locations. If they're not in the map (which they should be...), just skip
            let start_loc: egui::Pos2 = match pin_locations.get(&(connection.start_board.clone(), connection.start_pin.clone())) {
                Some(sl) => *sl,
                None => continue,
            };
            let end_loc: egui::Pos2 = match pin_locations.get(&(connection.end_board.clone(), connection.end_pin.clone())) {
                Some(el) => *el,
                None => continue,
            };

            let resp = draw_connection(ctx, ui, start_loc, end_loc, egui::Color32::RED);
            resp.on_hover_ui(|ui| {
                ui.label("connection");
                ui.label(connection.start_board.get_name().to_string() + ":" + connection.start_pin.as_str());
                ui.label(connection.end_board.get_name().to_string() + ":" + connection.end_pin.as_str());
            });

        }

    }

    /// Show the project HUD with information about the current system. Return a "Mode" so that
    /// the calling module (app) can update the GUI accordingly.
    pub fn display_system_editor_hud(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, warning_flags: &mut Warnings) -> Option<Mode> {

        // prepare the return value
        let mut ret: Option<Mode> = None;

        // get the app-wide icons
        let icons_ref: Arc<IconSet> = ctx.data_mut(|data| {
            data.get_temp("icons".into()).expect("couldn't load icons!")
        });
        let icons = icons_ref.clone();

        // push the top of the HUD down just a bit.
        ui.add_space(6.0);

        // display the project name
        let font = egui::FontId::monospace(20.0);
        let top_hud_rect = ui.vertical_centered(|ui| {
            let te = egui::TextEdit::singleline(self.borrow_name())
                .horizontal_align(egui::Align::Center)
                .frame(false)
                .hint_text("enter project name here")
                .font(font);
            ui.add(te);
        }).response.rect;

        // Show the know boards list, if needed
        let id = egui::Id::new("show_known_boards");
        let mut should_show_boards_window = ctx.data_mut(|data| {
            data.get_temp_mut_or(id, false).clone()

        });
        // generate the button
        let tid = icons.get("plus_icon").expect("error fetching plus_icon!").clone();
        let add_board_button = egui::Button::image_and_text(tid, "add board")
            .frame(false);
        let mut cui = ui.child_ui(top_hud_rect, egui::Layout::left_to_right(egui::Align::Center));
        if cui.add(add_board_button).clicked() {
            should_show_boards_window = true;
        }
        if let Some(b) = self.display_known_boards(ctx, &mut should_show_boards_window) {
            self.add_board(b);
        }
        ctx.data_mut(|data| {
            data.insert_temp(id, should_show_boards_window);
        });

        // let location_text = self.get_location();
        // let label = RichText::new(format!("Project Folder: {}", location_text)).underline();
        // ui.label(label);

        // generate the button
        let tid = icons.get("right_arrow_icon").expect("error fetching right_arrow_icon!").clone();
        let start_dev_button = egui::Button::image_and_text(tid, "start development")
            .frame(false);
        let mut cui = ui.child_ui(top_hud_rect, egui::Layout::right_to_left(egui::Align::Center));
        if cui.add(start_dev_button).clicked() {
            if self.has_main_board() {
                match self.save() {
                    Ok(()) => {
                        ret = Some(Mode::DevelopProject);
                    },
                    Err(e) => {
                        warn!("couldn't save project: {:?}", e);
                    },
                }
                // generate template code on initialization of project
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
            else {
                if !self.has_main_board() {
                    warning_flags.display_mainboard_warning = true;
                }
                if  self.name == "" {
                    warning_flags.display_unnamed_project_warning = true;
                }
            }
        }

        // Show some system stats
        ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
            ui.label(format!("number of connections: {}", self.system.connections.len()));
            ui.label(format!("number of boards: {}", self.system.get_all_boards().len()));
        });

        // let painter = ui.painter();
        // let rect = ui.min_rect();
        // painter.rect(rect, egui::Rounding::none(), egui::Color32::TRANSPARENT, egui::Stroke::new(2.0, egui::Color32::GOLD));
        return ret;
    }

}



/// Modified from egui_node_graph. Given a start and end position, draw a line representing the connection.
/// Return a response (a bit hacky through egui api), that indicates if the pointer is nearby, i.e. hovering, over the line.
fn draw_connection(ctx: &egui::Context, ui: &mut egui::Ui, src_pos: egui::Pos2, dst_pos: egui::Pos2, color: egui::Color32) -> Response {

    let mut response = ui.allocate_rect(egui::Rect::from_points(&[src_pos, dst_pos]), egui::Sense::hover());
    // these are public fields, but not exposed in egui documentation!
    response.hovered = false;
    response.clicked = [false; 5];

    let mut connection_stroke = egui::Stroke { width: 3.0, color };

    let control_scale = ((dst_pos.x - src_pos.x) / 2.0).max(30.0);
    let src_control = src_pos + egui::Vec2::X * control_scale;
    let dst_control = dst_pos - egui::Vec2::X * control_scale;

    let mut bezier = egui::epaint::CubicBezierShape::from_points_stroke(
        [src_pos, src_control, dst_control, dst_pos],
        false,
        egui::Color32::TRANSPARENT,
        connection_stroke,
    );

    // construct the painter *before* changing the response rectangle. In fact, expand the rect a bit
    // to avoid clipping the curve. This is done so that the layer order can be changed.
    let mut painter = ui.painter_at(response.rect.expand(10.0));
    let mut layer_id = painter.layer_id();
    layer_id.order = egui::Order::Middle;
    painter.set_layer_id(layer_id);

    if let Some(cursor_pos) = ctx.pointer_interact_pos() {
        // the TOL here determines the spacing of the segments that this line is broken into
        // it was determined experimentally, and used in conjunction with THRESH helps to detect
        // if we are hovering over the line.
        const TOL: f32 = 0.01;
        const THRESH: f32 = 15.0;
        bezier.for_each_flattened_with_t(TOL, &mut |pos, _| {
            if pos.distance(cursor_pos) < THRESH {
                response.hovered = true;
                response.rect = egui::Rect::from_center_size(cursor_pos, egui::Vec2::new(THRESH, THRESH));

            }
        });
    }

    if response.hovered() {
        connection_stroke.color = connection_stroke.color.gamma_multiply(0.5);
        bezier = egui::epaint::CubicBezierShape::from_points_stroke(
            [src_pos, src_control, dst_control, dst_pos],
            false,
            egui::Color32::TRANSPARENT,
            connection_stroke,
        );
    }

    painter.add(bezier);

    response

}
