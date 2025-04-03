//! Title: Iron Coder Project Module - Display
//! Description: This file contains methods that help display
//! the main project window using equi. It also contains some
//! helper functions for drawing connections between pins on
//! the system editor.

use egui::{Color32, Key, PointerButton, Pos2, Rect, Response, TextBuffer, Vec2, Widget};
use egui_extras::RetainedImage;
use log::{info, warn};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use egui::text_selection::visuals::paint_cursor;
use egui::widget_text::RichText;
use egui::widgets::Button;
use git2::{Repository, StatusOptions};

use crate::board;
use crate::project::Project;
use crate::app::icons::IconSet;
use crate::app::{Mode, Warnings, Git};

use enum_iterator;
use rfd::FileDialog;
use serde::{Serialize, Deserialize};
use crate::board::{Board, BoardTomlInfo, BoardType};
use crate::board::svg_reader::{Error, SvgBoardInfo};
use super::system;

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
                let mut changes: std::vec::Vec<String> = std::vec::Vec::new();
                for entry in repo_statuses.unwrap().iter() {
                    if entry.status().contains(git2::Status::WT_NEW) || entry.status().contains(git2::Status::WT_MODIFIED)
                    || entry.status().contains(git2::Status::INDEX_MODIFIED){
                        changes.push(entry.path().unwrap().to_string());
                    }
                }

                // Print the changes
                info!("Changes to be committed:");
                for change in changes.iter() {
                    info!("{}", change);
                }

                let mut index = repo.index().unwrap();
                for change in changes.iter() {
                    info!("Removing {} from the index", change);
                    index.remove_all([change.clone()].iter(), None).unwrap();   
                }
                index.write().unwrap();

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
        let id = egui::Id::new("show_generate_boards");
        let mut should_show_generate_board_window = ctx.data_mut(|data| {
            data.get_temp_mut_or(id, false).clone()
        });
        let mut board: Option<board::Board> = None;
        let mut boards: std::vec::Vec<Board> = self.known_boards.clone();
        boards.sort();
        // create the window
        let response = egui::Window::new("Components")
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
                    for (i, b) in boards.into_iter().enumerate() {
                        let col = i % num_cols;
                        // When a board is clicked, add it to the new project
                        if columns[col].add(board::display::BoardSelectorWidget(b.clone())).clicked() {
                            board = Some(b.clone());
                        }
                    }

                    let last_col = self.known_boards.len();
                    if columns[last_col % num_cols].add(Button::new("Generate New Board").min_size(Vec2::new(60.0, 45.0))).clicked() {
                        should_show_generate_board_window = true;
                        ctx.data_mut(|data| {
                            data.insert_temp(id, should_show_generate_board_window);
                        });
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

    pub fn display_generate_new_board(&mut self, ctx: &egui::Context, should_show: &mut bool) {
        let board_toml_info_id = egui::Id::new("board_toml_info");
        let display_png_error_id= egui::Id::new("display_png_convert_error");
        let display_svg_error_id= egui::Id::new("display_svg_file_select_error");
        let error_string_id = egui::Id::new("select_image_error_string");
        let should_show_new_board_image_id = egui::Id::new("should_show_new_board_image");
        let new_board_svg_string_id = egui::Id::new("new_board_svg_string");
        let generating_svg_id = egui::Id::new("generating_svg_from_png");
        let png_file_path_id = egui::Id::new("new_board_png_file_path_id");
        let screen_rect = ctx.input(|i: &egui::InputState| i.screen_rect());
        let min_rect = screen_rect.shrink2(Vec2::new(100.0, 60.0));
        let max_rect = screen_rect.shrink(50.0);
        let response = egui::Window::new("Generate New Board")
            .open(should_show)
            .collapsible(false)
            .resizable(false)
            .movable(false)
            .vscroll(true)
            .min_size(min_rect.size())
            .max_size(max_rect.size())
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label("Fill out all fields. Press X to cancel");

                let mut board_toml_info = ctx.data_mut(|data| {
                    data.get_temp_mut_or(board_toml_info_id, BoardTomlInfo::default()).clone()
                });
                let mut png_svg_convert_error = ctx.data_mut(|data| {
                    data.get_temp_mut_or(display_png_error_id, false).clone()
                });
                let mut display_svg_error = ctx.data_mut(|data| {
                    data.get_temp_mut_or(display_svg_error_id, false).clone()
                });
                let error_string : String = ctx.data_mut(|data| {
                    data.get_temp_mut_or(error_string_id, "".to_string()).clone()
                });
                let generating_svg_from_png = ctx.data_mut(|data| {
                    data.get_temp_mut_or(generating_svg_id, false).clone()
                });

                BoardTomlInfo::update_general_form_UI(&mut board_toml_info, ctx, ui);

                ctx.data_mut(|data| {
                    data.insert_temp(board_toml_info_id, board_toml_info);
                });

                board_toml_info = ctx.data_mut(|data| {
                    data.get_temp_mut_or(board_toml_info_id, BoardTomlInfo::default()).clone()
                });

                ui.label(RichText::new("Next: Pick Between 3 Options for Selecting Board Image").underline());

                if ui.button("Select Default Board Image").clicked() {
                    if self.general_form_input_valid(ctx) {
                        self.clear_required_flag_messages(ctx);
                        match fs::read_to_string(PathBuf::from("./iron-coder-boards/default_board.svg")) {
                            Ok(svg_string) => {
                                ctx.data_mut(|data| {
                                    data.insert_temp(new_board_svg_string_id, svg_string);
                                });

                                ctx.data_mut(|data| {
                                    data.insert_temp(should_show_new_board_image_id, true);
                                });

                                ctx.data_mut(|data| {
                                    data.insert_temp(display_png_error_id, false);
                                });
                                ctx.data_mut(|data| {
                                    data.insert_temp(display_svg_error_id, false);
                                });
                            }
                            Err(e) => {
                                ctx.data_mut(|data| {
                                    data.insert_temp(display_svg_error_id, true);
                                });
                                ctx.data_mut(|data| {
                                    data.insert_temp(error_string_id, format!("{e:?}"));
                                });

                            }
                        };
                    }
                }

                ui.horizontal(|ui| {
                    if ui.button("Select SVG for Board Image").clicked() {
                        if self.general_form_input_valid(ctx) {
                            self.clear_required_flag_messages(ctx);
                            if let Some(svg_file_path) = FileDialog::new()
                                .set_title("Select Image File for Board (File Type Must be SVG)")
                                .add_filter("SVG Filter", &["svg"])
                                .pick_file()
                            {
                                match fs::read_to_string(svg_file_path.clone()) {
                                    Ok(svg_string) => {
                                        ctx.data_mut(|data| {
                                            data.insert_temp(new_board_svg_string_id, svg_string);
                                        });

                                        self.change_svg_size(ctx);

                                        ctx.data_mut(|data| {
                                            data.insert_temp(should_show_new_board_image_id, true);
                                        });

                                        ctx.data_mut(|data| {
                                            data.insert_temp(display_png_error_id, false);
                                        });
                                        ctx.data_mut(|data| {
                                            data.insert_temp(display_svg_error_id, false);
                                        });
                                    }
                                    Err(e) => {
                                        ctx.data_mut(|data| {
                                            data.insert_temp(display_svg_error_id, true);
                                        });
                                        ctx.data_mut(|data| {
                                            data.insert_temp(error_string_id, format!("{e:?}"));
                                        });

                                    }
                                };
                            }
                        }
                    }
                    if display_svg_error {
                        ui.label(format!("Error accessing SVG: {}", error_string));
                    }
                });

                ui.vertical(|ui| {
                    if ui.button("Select PNG for Board Image").clicked() {
                        if self.general_form_input_valid(ctx) {
                            self.clear_required_flag_messages(ctx);
                            if let Some(png_file_path) = FileDialog::new()
                                .set_title("Select Image File for Board (File Type Must be PNG)")
                                .add_filter("PNG Filter", &["png"])
                                .pick_file()
                            {
                                ui.label("Generating SVG from PNG File...");
                                ui.label("Files larger than 680 kB will take multiple seconds to load.");
                                ctx.data_mut(|data| {
                                    data.insert_temp(generating_svg_id, true);
                                });
                                ctx.data_mut(|data| {
                                    data.insert_temp(png_file_path_id, png_file_path);
                                });
                            }
                        }
                    }

                    if png_svg_convert_error {
                        ui.label(format!("Error converting PNG to SVG: {}", error_string));
                    }
                });

                if generating_svg_from_png {
                    ctx.data_mut(|data| {
                        data.insert_temp(generating_svg_id, false);
                    });
                    let png_file_path = ctx.data_mut(|data| {
                        data.get_temp_mut_or(png_file_path_id, PathBuf::default()).clone()
                    });
                    match SvgBoardInfo::from_png(png_file_path.as_ref()) {
                        Ok(svg_string) => {
                            ctx.data_mut(|data| {
                                data.insert_temp(new_board_svg_string_id, svg_string);
                            });

                            ctx.data_mut(|data| {
                                data.insert_temp(should_show_new_board_image_id, true);
                            });

                            ctx.data_mut(|data| {
                                data.insert_temp(display_png_error_id, false);
                            });
                            ctx.data_mut(|data| {
                                data.insert_temp(display_svg_error_id, false);
                            });
                        }
                        Err(e) => {
                            ctx.data_mut(|data| {
                                data.insert_temp(display_png_error_id, true);
                            });
                            ctx.data_mut(|data| {
                                data.insert_temp(error_string_id, format!("{e:?}"));
                            });

                        }
                    };
                }
        });

        if response.is_some() {
            // unwrap ok here because we check that response is Some.
            ctx.move_to_top(response.unwrap().response.layer_id);
        }

        if !*should_show {
            self.clear_required_flag_messages(ctx);
            ctx.data_mut(|data| {
                data.insert_temp(board_toml_info_id, BoardTomlInfo::default().clone());
            });
            ctx.data_mut(|data| {
                data.insert_temp(generating_svg_id, false);
            });
            ctx.data_mut(|data| {
                data.insert_temp(png_file_path_id, PathBuf::default());
            });
        }
    }

    pub fn general_form_input_valid(&mut self, ctx: &egui::Context) -> bool{
        // Input validation before move to next screen
        let mut invalid_field_flag : bool = false;
        let mut duplicate_name_flag : bool = false;
        let board_toml_info_id = egui::Id::new("board_toml_info");
        let name_required_id = egui::Id::new("name_required");
        let name_duplicated_id = egui::Id::new("name_duplicated");
        let manufacture_required_id = egui::Id::new("manufacturer_required");
        let standard_required_id = egui::Id::new("standard_required");
        let cpu_required_id = egui::Id::new("cpu_required");
        let flash_required_id = egui::Id::new("flash_required");
        let ram_required_id = egui::Id::new("ram_required");
        let req_crates_required_id = egui::Id::new("req_crates_required");
        let rel_crates_required_id = egui::Id::new("rel_crates_required");

        let mut board_toml_info = ctx.data_mut(|data| {
            data.get_temp_mut_or(board_toml_info_id, BoardTomlInfo::default()).clone()
        });

        for board in self.known_boards.iter() {
            if board.get_name().to_lowercase().replace(" ", "").trim().eq(board_toml_info.name.to_lowercase().replace(" ", "").trim()){
                invalid_field_flag = true;
                duplicate_name_flag = true;
                ctx.data_mut(|data| {
                    data.insert_temp(name_duplicated_id, true);
                });
            }
        }

        if !duplicate_name_flag {
            ctx.data_mut(|data| {
                data.insert_temp(name_duplicated_id, false);
            });
        }

        if board_toml_info.name.is_empty() {
            invalid_field_flag = true;
            ctx.data_mut(|data| {
                data.insert_temp(name_required_id, true);
            });
        } else {
            ctx.data_mut(|data| {
                data.insert_temp(name_required_id, false);
            });
        }
        if board_toml_info.manufacturer.is_empty() {
            invalid_field_flag = true;
            ctx.data_mut(|data| {
                data.insert_temp(manufacture_required_id, true);
            });
        } else {
            ctx.data_mut(|data| {
                data.insert_temp(manufacture_required_id, false);
            });
        }
        if board_toml_info.standard.is_empty() {
            invalid_field_flag = true;
            ctx.data_mut(|data| {
                data.insert_temp(standard_required_id, true);
            });
        } else {
            ctx.data_mut(|data| {
                data.insert_temp(standard_required_id, false);
            });
        }
        if board_toml_info.cpu.is_empty() && board_toml_info.board_type == BoardType::Main {
            invalid_field_flag = true;
            ctx.data_mut(|data| {
                data.insert_temp(cpu_required_id, true);
            });
        } else {
            ctx.data_mut(|data| {
                data.insert_temp(cpu_required_id, false);
            });
        }
        if board_toml_info.flash == 0 && board_toml_info.board_type == BoardType::Main {
            invalid_field_flag = true;
            ctx.data_mut(|data| {
                data.insert_temp(flash_required_id, true);
            });
        } else {
            ctx.data_mut(|data| {
                data.insert_temp(flash_required_id, false);
            });
        }
        if board_toml_info.ram == 0 && board_toml_info.board_type == BoardType::Main {
            invalid_field_flag = true;
            ctx.data_mut(|data| {
                data.insert_temp(ram_required_id, true);
            });
        } else {
            ctx.data_mut(|data| {
                data.insert_temp(ram_required_id, false);
            });
        }
        if board_toml_info.required_crates.contains(&String::new()) {
            invalid_field_flag = true;
            ctx.data_mut(|data| {
                data.insert_temp(req_crates_required_id, true);
            });
        } else {
            ctx.data_mut(|data| {
                data.insert_temp(req_crates_required_id, false);
            });
        }
        if board_toml_info.related_crates.contains(&String::new()) {
            invalid_field_flag = true;
            ctx.data_mut(|data| {
                data.insert_temp(rel_crates_required_id, true);
            });
        } else {
            ctx.data_mut(|data| {
                data.insert_temp(rel_crates_required_id, false);
            });
        }

        !invalid_field_flag
    }

    pub fn clear_required_flag_messages(&mut self, ctx: &egui::Context){
        let name_required_id = egui::Id::new("name_required");
        let name_duplicated_id = egui::Id::new("name_duplicated");
        let manufacture_required_id = egui::Id::new("manufacturer_required");
        let standard_required_id = egui::Id::new("standard_required");
        let cpu_required_id = egui::Id::new("cpu_required");
        let flash_required_id = egui::Id::new("flash_required");
        let ram_required_id = egui::Id::new("ram_required");
        let req_crates_required_id = egui::Id::new("req_crates_required");
        let rel_crates_required_id = egui::Id::new("rel_crates_required");
        let pins_required_id = egui::Id::new("pins_required");
        let display_png_error_id= egui::Id::new("display_png_convert_error");
        let display_svg_error_id= egui::Id::new("display_svg_file_select_error");
        let error_string_id = egui::Id::new("select_image_error_string");
        ctx.data_mut(|data| {
            data.insert_temp(name_required_id, false);
        });
        ctx.data_mut(|data| {
            data.insert_temp(name_duplicated_id, false);
        });
        ctx.data_mut(|data| {
            data.insert_temp(manufacture_required_id, false);
        });
        ctx.data_mut(|data| {
            data.insert_temp(standard_required_id, false);
        });
        ctx.data_mut(|data| {
            data.insert_temp(cpu_required_id, false);
        });
        ctx.data_mut(|data| {
            data.insert_temp(flash_required_id, false);
        });
        ctx.data_mut(|data| {
            data.insert_temp(ram_required_id, false);
        });
        ctx.data_mut(|data| {
            data.insert_temp(req_crates_required_id, false);
        });
        ctx.data_mut(|data| {
            data.insert_temp(rel_crates_required_id, false);
        });
        ctx.data_mut(|data| {
            data.insert_temp(pins_required_id, false);
        });
        ctx.data_mut(|data| {
            data.insert_temp(display_png_error_id, false);
        });
        ctx.data_mut(|data| {
            data.insert_temp(display_svg_error_id, false);
        });
        ctx.data_mut(|data| {
            data.insert_temp(error_string_id, "".to_string());
        });
    }

    pub fn change_svg_size(&mut self, ctx: &egui::Context){
        // CHECK IF WE NEED TO CHANGE SVG IMAGE SIZE
        let new_board_svg_string_id = egui::Id::new("new_board_svg_string");
        let mut svg_string = ctx.data_mut(|data| {
            data.get_temp_mut_or(new_board_svg_string_id, "".to_string()).clone()
        });

        let mut width = 0.0;
        let mut height = 0.0;
        let mut index = 0;
        while let Some(width_start) = svg_string[index..].find("width=\"") {
            let width_start = index + width_start;

            // Ignore if "inkscape:window-width="
            if width_start > 0 && svg_string[width_start - 1..].starts_with('-') {
                index = width_start + 7;
            } else if let Some(width_end) = svg_string[width_start + 7..].find(|c: char| !c.is_ascii_digit() && c != '.') {
                let width_value = &svg_string[width_start + 7..width_start + 7 + width_end];
                width = width_value.parse().unwrap_or_else(|parse_float_error| 100.0);
                break;
            }
        }

        index = 0; // Reset index for height extraction
        while let Some(height_start) = svg_string[index..].find("height=\"") {
            let height_start = index + height_start;

            // Ignore if "inkscape:window-height="
            if height_start > 0 && svg_string[height_start - 1..].starts_with('-') {
                index = height_start + 8;
            } else if let Some(height_end) = svg_string[height_start + 8..].find(|c: char| !c.is_ascii_digit() && c != '.') {
                let height_value = &svg_string[height_start + 8..height_start + 8 + height_end];
                height = height_value.parse().unwrap_or_else(|parse_float_error| 100.0);
                break;
            }
        }

        if width > 64.0 || height > 50.0 {
            // MUST RESIZE
            while width > 64.0 || height > 50.0 {
                width = width / 2.0;
                height = height / 2.0;
            }

            index = 0;
            while let Some(width_start) = svg_string[index..].find("width=\"") {
                let width_start = index + width_start;

                // Ignore if "inkscape:window-width="
                if width_start > 0 && svg_string[width_start - 1..].starts_with('-') {
                    index = width_start + 7;
                } else if let Some(width_end) = svg_string[width_start + 7..].find(|c: char| !c.is_ascii_digit() && c != '.') {
                    svg_string.replace_range(width_start + 7..width_start + 7 + width_end, width.to_string().as_str());
                    index = width_start + 7;
                }
            }

            index = 0;
            while let Some(height_start) = svg_string[index..].find("height=\"") {
                let height_start = index + height_start;

                // Ignore if "inkscape:window-width="
                if height_start > 0 && svg_string[height_start - 1..].starts_with('-') {
                    index = height_start + 8;
                } else if let Some(height_end) = svg_string[height_start + 8..].find(|c: char| !c.is_ascii_digit() && c != '.') {
                    svg_string.replace_range(height_start + 8..height_start + 8 + height_end, height.to_string().as_str());
                    index = height_start + 8;
                }
            }

            index = 0;
            let viewbox_string = width.to_string() + " " + height.to_string().as_str();
            if let Some(viewbox_start) = svg_string.find("viewBox=\"0 0 ") {
                let viewbox_start = index + viewbox_start;

                if let Some(viewbox_end) = svg_string[viewbox_start + 13..].find("\"") {
                    svg_string.replace_range(viewbox_start + 13..viewbox_start + 13 + viewbox_end, viewbox_string.as_str());
                }
            }

            ctx.data_mut(|data| {
                data.insert_temp(new_board_svg_string_id, svg_string);
            });
        }
    }

    pub fn save_new_board_info(&mut self, ctx: &egui::Context) {
        let board_toml_info_id = egui::Id::new("board_toml_info");
        let pin_rects_id = egui::Id::new("new_board_pin_rects");
        let image_pos_id = egui::Id::new("image_rect_pos");
        let pin_names_id = egui::Id::new("pin_names_id");
        let new_board_svg_string_id = egui::Id::new("new_board_svg_string");
        let save_error_string_id = egui::Id::new("save_board_error_string");
        let save_failure_id = egui::Id::new("save_board_FAILED");
        let mut board_toml_info = ctx.data_mut(|data| {
            data.get_temp_mut_or(board_toml_info_id, BoardTomlInfo::default()).clone()
        });
        let mut svg_string = ctx.data_mut(|data| {
            data.get_temp_mut_or(new_board_svg_string_id, "".to_string()).clone()
        });
        let mut pin_rects : Vec<Rect>  = ctx.data_mut(|data| {
            data.get_temp_mut_or(pin_rects_id, std::vec::Vec::new()).clone()
        });
        let image_pos = ctx.data_mut(|data| {
            data.get_temp_mut_or(image_pos_id, Pos2::new(0.0,0.0)).clone()
        });
        let pin_names : Vec<String>  = ctx.data_mut(|data| {
            data.get_temp_mut_or(pin_names_id, std::vec::Vec::new()).clone()
        });


        let mut new_board_file_path = Path::new("./iron-coder-boards");
        let board_name_folder = String::from(board_toml_info.name.trim().replace(" ", "_"));
        let board_name_toml = String::from(board_name_folder.clone().to_lowercase() + ".toml");

        let board_directory = new_board_file_path.join(board_toml_info.manufacturer.trim().clone()).join(board_name_folder.clone());

        let create_dir_res = fs::create_dir_all(board_directory.clone());
        match create_dir_res {
            Ok(r) => {
                let binding = board_directory.join(board_name_toml);
                new_board_file_path = binding.as_ref();

                let toml_res = fs::write(new_board_file_path, board_toml_info.generate_toml_string());

                match toml_res {
                    Ok(r) => {}
                    Err(e) => {
                        ctx.data_mut(|data| {
                            data.insert_temp(save_error_string_id, format!("Create TOML File Failed. {e:?}"));
                        });
                        ctx.data_mut(|data| {
                            data.insert_temp(save_failure_id, true);
                        });
                    }
                }

                let board_name_svg = String::from(board_name_folder.clone().to_lowercase() + ".svg");

                let mut pin_rects_string = String::new();

                let mut index = 0;
                for pin in pin_rects{
                    let x = (pin.center().x - image_pos.clone().x)/10.0;
                    let y = (pin.center().y - image_pos.clone().y)/10.0;
                    let radius = pin.height()/2.0/10.0;
                    let pin_string = format!("    <circle\n       \
                           style=\"fill:#ff00ff;fill-opacity:0.561475;stroke-width:1.19048\"\n       \
                           id=\"{}\"\n       \
                           cx=\"{}\"\n       \
                           cy=\"{}\"\n       \
                           r=\"{}\" />\n", pin_names[index], x, y, radius);
                    pin_rects_string.push_str(pin_string.as_str());
                    index += 1;
                }

                if svg_string.contains("  </g>\n</svg>") {
                    pin_rects_string.push_str("  </g>\n</svg>");
                    svg_string = svg_string.replace("  </g>\n</svg>", pin_rects_string.as_str());

                } else if svg_string.contains("</svg>") {
                    pin_rects_string.push_str("</svg>");
                    svg_string = svg_string.replace("</svg>", pin_rects_string.as_str());

                } else if svg_string.is_empty() {
                    ctx.data_mut(|data| {
                        data.insert_temp(save_error_string_id, "Read SVG from String failed.");
                    });
                    ctx.data_mut(|data| {
                        data.insert_temp(save_failure_id, true);
                    });
                } else {
                    ctx.data_mut(|data| {
                        data.insert_temp(save_error_string_id, "Copy Pin Rects failed.");
                    });
                    ctx.data_mut(|data| {
                        data.insert_temp(save_failure_id, true);
                    });
                }

                let svg_res = fs::write(board_directory.join(board_name_svg), svg_string);

                match svg_res {
                    Ok(r) => {}
                    Err(e) => {
                        ctx.data_mut(|data| {
                            data.insert_temp(save_error_string_id, format!("Create SVG file failed. {e:?}"));
                        });
                        ctx.data_mut(|data| {
                            data.insert_temp(save_failure_id, true);
                        });
                    }
                }

            }
            Err(e) => {
                ctx.data_mut(|data| {
                    data.insert_temp(save_error_string_id, format!("Create new board directory failed. {e:?}"));
                });
                ctx.data_mut(|data| {
                    data.insert_temp(save_failure_id, true);
                });
            }
        }


    }

    pub fn display_new_board_png(&mut self, ctx: &egui::Context, should_show: &mut bool) {
        let new_board_svg_string_id = egui::Id::new("new_board_svg_string");
        let pin_rects_id = egui::Id::new("new_board_pin_rects");
        let pin_names_id = egui::Id::new("pin_names_id");
        let image_pos_id = egui::Id::new("image_rect_pos");
        let pin_radius_id = egui::Id::new("pin_radius_id");
        let pin_name_box_id = egui::Id::new("pin_name_box_id");
        let board_toml_info_id = egui::Id::new("board_toml_info");
        let pins_required_id = egui::Id::new("pins_required");
        let file_select_error_id = egui::Id::new("file_select_error_again");
        let error_string_id = egui::Id::new("select_image_error_string");
        let board_object_id = egui::Id::new("board_image_placeholder");
        let board_loaded_id = egui::Id::new("board_loaded_id");
        let screen_rect = ctx.input(|i: &egui::InputState| i.screen_rect());
        let max_rect = screen_rect.shrink(50.0);
        let mut done = false;
        let response = egui::Window::new("Designate Pinouts (Press X to cancel)")
            .open(should_show)
            .collapsible(false)
            .resizable(false)
            .movable(false)
            .fixed_size(max_rect.size())
            .max_height(max_rect.height())
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {

                let svg_string  = ctx.data_mut(|data| {
                    data.get_temp_mut_or(new_board_svg_string_id, "".to_string()).clone()
                });
                let mut pin_rects  = ctx.data_mut(|data| {
                    data.get_temp_mut_or(pin_rects_id, std::vec::Vec::new()).clone()
                });
                let mut pin_names  = ctx.data_mut(|data| {
                    data.get_temp_mut_or(pin_names_id, std::vec::Vec::new()).clone()
                });
                let mut pin_name_box = ctx.data_mut(|data| {
                    data.get_temp_mut_or(pin_name_box_id, String::new()).clone()
                });
                let mut pin_radius = ctx.data_mut(|data| {
                    data.get_temp_mut_or(pin_radius_id, 8.0).clone()
                });
                let mut pins_required = ctx.data_mut(|data| {
                    data.get_temp_mut_or(pins_required_id, false).clone()
                });
                let board_loaded = ctx.data_mut(|data| {
                    data.get_temp_mut_or(board_loaded_id, false).clone()
                });
                let board_info = ctx.data_mut(|data| {
                    data.get_temp_mut_or(board_object_id, SvgBoardInfo::default()).clone()
                });
                let mut instruction_in_red = false;

                if !board_loaded {
                    match SvgBoardInfo::from_string(svg_string) {

                        Ok(svg_board_info) => {
                            ctx.data_mut(|data| {
                                data.insert_temp(board_loaded_id,true);
                            });
                            ctx.data_mut(|data| {
                                data.insert_temp(board_object_id, svg_board_info);
                            });
                        },
                        Err(e) => {
                            let file_select_error = ctx.data_mut(|data| {
                                data.get_temp_mut_or(file_select_error_id, false).clone()
                            });
                            let error_string : String = ctx.data_mut(|data| {
                                data.get_temp_mut_or(error_string_id, "".to_string()).clone()
                            });

                            ui.label(format!("Error with SVG parsing. {e:?} error thrown."));
                            if format!("{e:?}").eq("ImageNotPNG"){
                                ui.label("SVG must be derived from PNG Image");
                            }

                            if ui.button("Select Default Board Image").clicked() {
                                match fs::read_to_string(PathBuf::from("./iron-coder-boards/default_board.svg")) {
                                    Ok(svg_string) => {
                                        ctx.data_mut(|data| {
                                            data.insert_temp(new_board_svg_string_id, svg_string);
                                        });
                                    }
                                    Err(e) => {
                                        ctx.data_mut(|data| {
                                            data.insert_temp(file_select_error_id, true);
                                        });
                                        ctx.data_mut(|data| {
                                            data.insert_temp(error_string_id, format!("{e:?}"));
                                        });

                                    }
                                };
                            }

                            if ui.button("Pick a different SVG file").clicked() {
                                if let Some(svg_file_path) = FileDialog::new()
                                    .set_title("Select Image File for Board (Must be a SVG File)")
                                    .add_filter("SVG Filter", &["svg"])
                                    .pick_file()
                                {
                                    match fs::read_to_string(svg_file_path.clone()) {
                                        Ok(svg_string) => {
                                            ctx.data_mut(|data| {
                                                data.insert_temp(new_board_svg_string_id, svg_string);
                                            });
                                        }
                                        Err(e) => {
                                            ctx.data_mut(|data| {
                                                data.insert_temp(error_string_id, format!("{e:?}"));
                                            });
                                            ctx.data_mut(|data| {
                                                data.insert_temp(file_select_error_id, true);
                                            });
                                        }
                                    };
                                }
                            }
                            if ui.button("Pick a different PNG file").clicked() {
                                if let Some(png_file_path) = FileDialog::new()
                                    .set_title("Select Image File for Board (Must be a PNG file)")
                                    .add_filter("PNG Filter", &["png"])
                                    .pick_file()
                                {
                                    match SvgBoardInfo::from_png(png_file_path.as_ref()) {
                                        Ok(svg_string) => {
                                            ctx.data_mut(|data| {
                                                data.insert_temp(new_board_svg_string_id, svg_string);
                                            });
                                        }
                                        Err(e) => {
                                            ctx.data_mut(|data| {
                                                data.insert_temp(error_string_id, format!("{e:?}"));
                                            });
                                            ctx.data_mut(|data| {
                                                data.insert_temp(file_select_error_id, true);
                                            });
                                        }
                                    };
                                }
                            }
                            if file_select_error {
                                ui.label(format!("Error reading new file: {}", error_string));
                            }
                        },
                    };

                } else {
                    let available_width = max_rect.width();
                    let mut num_cols = 2;
                    if available_width < 640.0 * 2.0 {
                        num_cols = 1;
                    }
                    ui.columns(num_cols, |cols_ui| {
                        // Display designate pins
                        cols_ui[0].horizontal(|ui_h| {
                            ui_h.label(RichText::new("Add Pin:").underline());
                            ui_h.label(" Left-Click anywhere on image. You must select a name first.");
                        });
                        cols_ui[0].horizontal(|ui_h| {
                            ui_h.label(RichText::new("Delete Pin:").underline());
                            ui_h.label(" Right-Click existing pin");
                        });
                        cols_ui[0].horizontal(|ui_h| {
                            ui_h.label(RichText::new("Change a Pin's Name:").underline());
                            ui_h.label(" Select a Name and Left-Click existing pin");
                        });

                        // Display image
                        let retained_image = RetainedImage::from_color_image(
                            "pic",
                            board_info.image,
                        );

                        let display_size = board_info.physical_size * 10.0;

                        let image_rect = retained_image.show_size(&mut cols_ui[0], display_size).rect;

                        ctx.data_mut(|data| {
                            data.insert_temp(image_pos_id, image_rect.left_top());
                        });

                        cols_ui[0].allocate_rect(image_rect, egui::Sense::hover());

                        // Designate pins
                        if cols_ui[0].ui_contains_pointer() {
                            if let Some(cursor_origin) = ctx.pointer_latest_pos(){
                                let mut hovering_over_pin = false;
                                // Check all existing pins to see if cursor is hovering over (prevent overlapping pins)
                                // Also delete pins that are right-clicked
                                for i in 0..pin_rects.len() {
                                    if cols_ui[0].rect_contains_pointer(pin_rects[i]){
                                        hovering_over_pin = true;
                                        if let response = cols_ui[0].interact(cols_ui[0].clip_rect(), cols_ui[0].id(), egui::Sense::click()) {
                                            if response.secondary_clicked(){
                                                pin_rects.remove(i);
                                                pin_names.remove(i);
                                            }
                                            if response.clicked(){
                                                pin_names[i] = pin_name_box.clone();
                                            }
                                        }
                                        break;
                                    }
                                }

                                // Display visual pin icon helper
                                if !hovering_over_pin {
                                    cols_ui[0].painter().circle_filled(cursor_origin, pin_radius, Color32::DARK_RED);
                                }

                                // Highlight instruction
                                if pin_name_box.is_empty() {
                                    instruction_in_red = true;
                                    cols_ui[0].label(RichText::new("You must add pin names in the pinout form and select a name from the dropdown before adding pins to your component.").color(Color32::RED));
                                }

                                // Add pin if left click
                                if let response = cols_ui[0].interact(cols_ui[0].clip_rect(), cols_ui[0].id(), egui::Sense::click()) {
                                    if !hovering_over_pin && response.clicked() && !pin_name_box.is_empty() {
                                        let pin_rect = Rect::from_center_size(cursor_origin, Vec2::new(pin_radius * 2.0, pin_radius * 2.0));
                                        pin_rects.push(pin_rect);
                                        pin_names.push(pin_name_box.clone())
                                    }
                                }
                            }
                        }

                        if !instruction_in_red {
                            cols_ui[0].label(RichText::new("You must add pin names in the pinout form and select a name from the dropdown before adding pins to your component."));
                        }

                        ctx.data_mut(|data| {
                            data.insert_temp(pin_rects_id, pin_rects.clone());
                        });

                        // Display drawn pins and names
                        let mut index = 0;
                        for pin in pin_rects {
                            cols_ui[0].painter().circle_filled(pin.center(), pin_radius, Color32::BLUE);
                            let name = match pin_names.get(index) {
                                None => {"pinx"}
                                Some(name) => {name}
                            };
                            cols_ui[0].painter().text(pin.center(), egui::Align2::CENTER_CENTER, name, egui::FontId::monospace(pin_radius * 1.25), Color32::WHITE);

                            index += 1;
                        }

                        let mut board_toml_info = ctx.data_mut(|data| {
                            data.get_temp_mut_or(board_toml_info_id, BoardTomlInfo::default()).clone()
                        });
                        let pin_names_dropdown = board_toml_info.get_all_pin_names();

                        cols_ui[0].horizontal(|ui| {
                            ui.label("Pin Name List:");
                            egui::ComboBox::from_label("Select pin name for new or existing pins!")
                                .selected_text(format!("{:?}", pin_name_box))
                                .show_ui(ui, |ui| {
                                    let mut pin_name_found = false;
                                    for pin in pin_names_dropdown.clone(){
                                        if !pin.is_empty(){
                                            pin_name_found = true;
                                            ui.selectable_value(&mut pin_name_box, pin.clone(), pin.clone());
                                        }
                                    }
                                    if pin_names_dropdown.len() == 0 || !pin_name_found {
                                        pin_name_box = "".to_string();
                                        ui.selectable_value(&mut pin_name_box, "".to_string(), "Cannot select name. Add pin names in the pin information form!");
                                    }
                                }
                                );
                        });

                        cols_ui[0].horizontal(|ui| {
                            ui.label("Pin Radius: ");
                            ui.add(egui::Slider::new(&mut pin_radius, 4.0..=15.0));
                        });

                        ctx.data_mut(|data| {
                            data.insert_temp(pin_radius_id, pin_radius.clone());
                        });

                        ctx.data_mut(|data| {
                            data.insert_temp(pin_name_box_id, pin_name_box.clone());
                        });

                        ctx.data_mut(|data| {
                            data.insert_temp(pin_names_id, pin_names.clone());
                        });

                        cols_ui[0].horizontal(|ui| {
                            let pin_rects : Vec<Rect>  = ctx.data_mut(|data| {
                                data.get_temp_mut_or(pin_rects_id, std::vec::Vec::new()).clone()
                            });
                            let pin_names_form = board_toml_info.get_all_pin_names();
                            if ui.button("Done - Generate Board").clicked() {
                                if pin_names_form.len() == 0 || ( pin_names_form.len() == 1 && pin_names_form[0].is_empty() ) || pin_rects.is_empty() || pin_names_form.contains(&String::new()) {
                                    ctx.data_mut(|data| {
                                        data.insert_temp(pins_required_id, true);
                                    });
                                } else {
                                    self.save_new_board_info(ctx);
                                    done = true;

                                    let new_board_confirmation_screen_id = egui::Id::new("show_new_board_confirmation_screen");
                                    let reload_boards_id = egui::Id::new("reload_boards_from_filesystem");
                                    ctx.data_mut(|data| {
                                        data.insert_temp(new_board_confirmation_screen_id, true);
                                    });
                                    ctx.data_mut(|data| {
                                        data.insert_temp(reload_boards_id, true);
                                    });
                                }
                            }
                            if pins_required {
                                ui.label(RichText::new("Resolve all errors.").color(Color32::RED));
                            }
                        });

                        // Display pinout form
                        egui::containers::scroll_area::ScrollArea::vertical()
                            .max_height(max_rect.shrink(80.0).height())
                            .show(&mut cols_ui[1 % num_cols], |ui| {
                                if pins_required {
                                    ui.label(RichText::new("Resolve all errors.\nNote: Must designate at least one pin. Add pin name to form and click image to designate pin location.").color(Color32::RED));
                                }
                                ui.add(egui::Label::new(RichText::new("Add Pinout Information").underline()));
                                BoardTomlInfo::update_pinout_form_UI(&mut board_toml_info, ctx, ui);
                            });

                        ctx.data_mut(|data| {
                            data.insert_temp(board_toml_info_id, board_toml_info);
                        });

                    });
                }

            });

        if response.is_some() {
            // unwrap ok here because we check that response is Some.
            ctx.move_to_top(response.unwrap().response.layer_id);
        }

        if done {
            *should_show = false;
        }

        if !*should_show {
            let board_toml_info_id = egui::Id::new("board_toml_info");
            ctx.data_mut(|data| {
                data.insert_temp(board_toml_info_id, BoardTomlInfo::default().clone());
            });

            ctx.data_mut(|data| {
                data.insert_temp(new_board_svg_string_id, "".to_string());
            });

            ctx.data_mut(|data| {
                data.insert_temp(pin_rects_id, Vec::<egui::Rect>::new().clone());
            });

            ctx.data_mut(|data| {
                data.insert_temp(pin_name_box_id, String::new().clone());
            });

            ctx.data_mut(|data| {
                data.insert_temp(pin_names_id, Vec::<String>::new().clone());
            });
            ctx.data_mut(|data| {
                data.insert_temp(board_loaded_id,false);
            });
        }
    }

    pub fn display_new_board_confirmation(&mut self, ctx: &egui::Context, should_show: &mut bool) {

        let response = egui::Window::new("Component Creation Successful!")
            .open(should_show)
            .collapsible(false)
            .resizable(false)
            .movable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label("Close this window to return to the project editor and board selection window.");
            });

        if response.is_some() {
            // unwrap ok here because we check that response is Some.
            ctx.move_to_top(response.unwrap().response.layer_id);
        }

    }

    pub fn display_new_board_failure(&mut self, ctx: &egui::Context, should_show: &mut bool) {
        let save_error_string_id = egui::Id::new("save_board_error_string");
        let error_message = ctx.data_mut(|data| {
            data.get_temp_mut_or(save_error_string_id, "".to_string()).clone()
        });
        let response = egui::Window::new("Component Creation Failed.")
            .open(should_show)
            .collapsible(false)
            .resizable(false)
            .movable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label(format!("{error_message}"));
            });

        if response.is_some() {
            // unwrap ok here because we check that response is Some.
            ctx.move_to_top(response.unwrap().response.layer_id);
        }

    }


    /// Show the boards in egui "Area"s so we can move them around!
    pub fn display_system_editor_boards(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {

        let mut pin_locations: HashMap<(board::Board, String), egui::Pos2> = HashMap::new();

        // iterate through the system boards and draw them on the screen
        for board in self.system.get_all_boards().iter_mut() {

            let scale_id = egui::Id::new("system_editor_scale_factor");
            // set the editor scale factor in memory:
            let scale = ctx.data_mut(|data| {
                data.get_temp_mut_or(scale_id, 5.0).clone()
            });

            // Get the response of the board/pin Ui
            let board_id = egui::Id::new(board.get_name());
            let response = egui::Area::new(board_id).show(ctx, |ui| {

                let mut pin_clicked: Option<String> = None;

                if let Some(svg_board_info) = board.clone().svg_board_info {
                    let retained_image = RetainedImage::from_color_image(
                        "pic",
                        svg_board_info.image,
                    );

                    let display_size = svg_board_info.physical_size * scale;

                    let image_rect = retained_image.show_max_size(ui, display_size).rect;

                    // Actions for board-level stuff -- Gets rid of red box
                    ui.allocate_rect(image_rect, egui::Sense::hover())
                        .context_menu(|ui| {
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

                        // Check if a connection is in progress by checking the "connection_in_progress" Id from the ctx memory.
                        // This is set to true if the user selects "add connection" from the parent container's context menu.
                        let id = egui::Id::new("connection_in_progress");
                        let mut connection_in_progress = ctx.data_mut(|data| {
                            data.get_temp_mut_or(id, false).clone()
                        });

                        if connection_in_progress {
                            ctx.output_mut(|o| {
                                o.cursor_icon = egui::CursorIcon::PointingHand;
                            });
                        }
                        
                        if connection_in_progress && r.clicked() {
                            // check conditions for starting/ending a connection
                            match self.system.in_progress_connection_start {
                                None => {
                                    info!("inserting connection position data");
                                    ctx.data_mut(|data| {
                                        data.insert_temp(egui::Id::new("connection_start_pos"), r.rect.center());
                                    });
                                    self.system.in_progress_connection_start = Some((board.clone(), pin_name.clone()));
                                },
                                Some((ref start_board, ref start_pin)) => {
                                    // add the connection to the system struct
                                    let c = super::system::Connection {
                                        name: format!("connection_{}", self.system.connections.len()),
                                        start_board: start_board.clone(),
                                        start_pin: start_pin.clone(),
                                        end_board: board.clone(),
                                        end_pin: pin_name.clone(),
                                        interface_mapping: board::pinout::InterfaceMapping::default(),
                                    };
                                    self.system.connections.push(c);
                                    // clear the in_progress_connection fields
                                    self.system.in_progress_connection_start = None;
                                    self.system.in_progress_connection_end = None;
                                    // and end the connection.
                                    connection_in_progress = false;
                                    ctx.data_mut(|data| {
                                        data.insert_temp(id, connection_in_progress);
                                        data.remove::<egui::Pos2>(egui::Id::new("connection_start_pos"));
                                    });
                                },
                            }
                        }
                    }
                }
                // return value from this scope
                pin_clicked
            });

            // extract response from board (i.e. the egui Area), and from pin
            let board_response = response.response;
            let pin_response = response.inner;

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
                data.insert_temp(id, false);
                data.remove::<egui::Pos2>(egui::Id::new("connection_start_pos"));
                self.system.in_progress_connection_start = None;
            });
        }

        // check if a connection is in progress. Be sure to use the painter outside of the data
        // closure to avoid deadlock situation.
        if let Some(true) = ctx.data(|data| {
            let id = egui::Id::new("connection_in_progress");
            data.get_temp::<bool>(id)
        }) {
            if let Some(sp) = ctx.data(|data| {
                let id = egui::Id::new("connection_start_pos");
                data.get_temp::<egui::Pos2>(id)
            }) {
                if let Some(ep) = ctx.pointer_latest_pos() {
                    draw_connection(ctx, ui, sp, ep, egui::Color32::GREEN);
                }
            }
        }

        // go through the system connections and see if this pin is a part of any of them
        let mut connection_to_remove: Option<system::Connection> = None;
        for connection in self.system.connections.iter_mut() {
            // get the start and end pin locations. If they're not in the map (which they should be...), just skip
            let start_loc: egui::Pos2 = match pin_locations.get(&(connection.start_board.clone(), connection.start_pin.clone())) {
                Some(sl) => *sl,
                None => continue,
            };
            let end_loc: egui::Pos2 = match pin_locations.get(&(connection.end_board.clone(), connection.end_pin.clone())) {
                Some(el) => *el,
                None => continue,
            };
            // draw the connection and perform interactions.
            let c = match connection.interface_mapping.interface.iface_type {
                board::pinout::InterfaceType::I2C => egui::Color32::RED,
                board::pinout::InterfaceType::UART => egui::Color32::BLUE,
                board::pinout::InterfaceType::SPI => egui::Color32::YELLOW,
                board::pinout::InterfaceType::NONE => egui::Color32::GREEN,
                _ => egui::Color32::WHITE,
            };
            let resp = draw_connection(ctx, ui, start_loc, end_loc, c);
            // Connection-level right click menu
            resp.context_menu(|ui| {
                ui.label("connection name:");
                ui.text_edit_singleline(&mut connection.name);
                ui.separator();
                ui.label("connection type:");
                for iface_type in enum_iterator::all::<board::pinout::InterfaceType>() {
                    ui.selectable_value(&mut connection.interface_mapping.interface.iface_type, iface_type, format!("{:?}", iface_type));
                }
                ui.separator();
                if ui.button("delete connection").clicked() {
                    connection_to_remove = Some(connection.clone());
                }
            });
        }

        // remove the connection if it was selected for deletion
        if let Some(conn) = connection_to_remove {
            self.system.connections.retain(|elem| {
                elem.name != conn.name
            });
        }

    }

    /// Show the project HUD with information about the current system. Return a "Mode" so that
    /// the calling module (app) can update the GUI accordingly.
    pub fn display_system_editor_top_bar(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, warning_flags: &mut Warnings) -> Option<Mode> {

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
                // .desired_width(f32::INFINITY)
                .clip_text(false)
                .frame(false)
                .hint_text("enter project name here")
                .font(font);
            ui.add(te);
        }).response.rect;

        // Show the know boards list, if needed
        let known_board_id = egui::Id::new("show_known_boards");
        let mut should_show_boards_window = ctx.data_mut(|data| {
            data.get_temp_mut_or(known_board_id, false).clone()

        });
        // generate the button
        let tid = icons.get("plus_icon").expect("error fetching plus_icon!").clone();
        let add_board_button = egui::Button::image_and_text(tid, "add component")
            .frame(false);
        let mut cui = ui.child_ui(top_hud_rect, egui::Layout::left_to_right(egui::Align::Center));
        if cui.add(add_board_button).clicked() {
            should_show_boards_window = true;
        }
        if let Some(b) = self.display_known_boards(ctx, &mut should_show_boards_window) {
            self.add_board(b);
        }
        ctx.data_mut(|data| {
            data.insert_temp(known_board_id, should_show_boards_window);
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
                if  self.name == "" {
                    warning_flags.display_unnamed_project_warning = true;
                }
                else if self.name.contains(char::is_whitespace) {
                    warning_flags.display_invalid_name_warning = true;
                    println!("Invalid name, remove whitespace!");
                }
                else {
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
            }
            else {
                if !self.has_main_board() {
                    warning_flags.display_mainboard_warning = true;
                }
            }
        }

        // Below code should go into a "bottom_bar" display function
        // Show some system stats
        // ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
        //     ui.label(format!("number of connections: {}", self.system.connections.len()));
        //     ui.label(format!("number of boards: {}", self.system.get_all_boards().len()));
        // });

        // let painter = ui.painter();
        // let rect = ui.min_rect();
        // painter.rect(rect, egui::Rounding::none(), egui::Color32::TRANSPARENT, egui::Stroke::new(2.0, egui::Color32::GOLD));
        return ret;
    }

}



/// Given a start and end position, draw a line representing the connection.
/// Return a response that indicates if the pointer is nearby, i.e. hovering, over the line.
/// Also handles click events.
fn draw_connection(ctx: &egui::Context, ui: &mut egui::Ui, src_pos: egui::Pos2, dst_pos: egui::Pos2, color: egui::Color32) -> Response {

    let mut response = ui.allocate_rect(egui::Rect::from_points(&[src_pos, dst_pos]), egui::Sense::click());
    // these are public fields, but not exposed in egui documentation!
    response.hovered = false;
    response.clicked = false;

    let mut connection_stroke = egui::Stroke { width: 2.0, color };

    let mid_x = src_pos.x + (dst_pos.x - src_pos.x) / 2.0;
    // let mid_y = src_pos.y + (dst_pos.y - src_pos.y) / 2.0;
    // let mid_pos1 = egui::Pos2::new(mid_x, src_pos.y);
    // let mid_pos2 = egui::Pos2::new(mid_x, dst_pos.y);

    let control_scale = ((dst_pos.x - src_pos.x) / 2.0).max(30.0);
    let src_control = src_pos + egui::Vec2::X * control_scale;
    let dst_control = dst_pos - egui::Vec2::X * control_scale;

    let mut line = egui::epaint::CubicBezierShape::from_points_stroke(
        [src_pos, src_control, dst_control, dst_pos],
        false,
        egui::Color32::TRANSPARENT,
        connection_stroke,
    );
    // let mut line = egui::epaint::PathShape::line(
    //     Vec::from([src_pos, mid_pos1, mid_pos2, dst_pos]),
    //     connection_stroke,
    // );

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
        const THRESH: f32 = 12.0;
        line.for_each_flattened_with_t(TOL, &mut |pos, _| {
            if pos.distance(cursor_pos) < THRESH {
                response.hovered = true;
                // using any_click allows clicks, context menu, etc to be handled.
                if ctx.input(|i| i.pointer.any_click()) == true {
                    response.clicked = true;
                }
                response.rect = egui::Rect::from_center_size(cursor_pos, egui::Vec2::new(THRESH, THRESH));
            }
        });
    }

    if response.hovered() {
        connection_stroke.color = connection_stroke.color.gamma_multiply(0.5);
        line = egui::epaint::CubicBezierShape::from_points_stroke(
            [src_pos, src_control, dst_control, dst_pos],
            false,
            egui::Color32::TRANSPARENT,
            connection_stroke,
        );
        // line = egui::epaint::PathShape::line(
        //     Vec::from([src_pos, mid_pos1, mid_pos2, dst_pos]),
        //     connection_stroke,
        // );
    }

    // painter.add(bezier);
    painter.add(line);

    response

}
