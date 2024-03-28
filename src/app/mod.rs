//! Title: Iron Coder App Module - Module
//! Description: This module contains the IronCoderApp struct and its implementation.
//!   The IronCoderApp struct is the main application struct for the Iron Coder app.
//!   It contains all the state and methods for the app, and is the main entry point
//!   for the eframe framework to interact with the app.

use log::{error, warn, info};

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::fs::File;
use std::io::Write;
use std::io::Read;
use std::string::String;
use clap::Parser;
use egui::{Vec2, RichText, Label, Color32, Key, Modifiers, KeyboardShortcut, Ui};
use::egui_extras::install_image_loaders;
use fs_extra::dir::DirEntryAttr::Modified;
use toml::macros::insert_toml;

// use egui_modal::Modal;

// Separate modules
use crate::board;
use crate::project::Project;

pub mod icons;
use icons::{
    IconSet,
    SMALL_ICON_SIZE,
};

pub mod colorscheme;
use colorscheme::ColorScheme;

pub mod code_editor;
mod test;

/// Iron Coder CLI configuration options...
#[derive(Parser, Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[command(version)]
pub struct IronCoderOptions {
    /// The log level, one of INFO, WARN, DEBUG, TRACE. Default if INFO.
    #[arg(short, long)]
    pub verbosity: Option<String>,
    /// An alternative path to look for the Boards directory.
    #[arg(short, long)]
    pub boards_directory: Option<PathBuf>,
    /// Turn app persistence on or off. Default is off.
    #[arg(short, long)]
    pub persistence: bool,
}

// The current warning flags
#[derive(serde::Deserialize, serde::Serialize)]
pub struct Warnings {
    pub display_mainboard_warning: bool,
    pub display_unnamed_project_warning: bool,
    pub display_git_warning: bool,
}

// The current git state
#[derive(serde::Deserialize, serde::Serialize)]
pub struct Git {
    pub display : bool,
    pub changes: Vec<String>,
    pub staged_changes: Vec<String>,
    pub commit_name: String,
    pub commit_email: String,
    pub commit_message: String,
    #[serde(skip)]
    pub repo : Option<git2::Repository>,
}

/// The current GUI mode
#[non_exhaustive]
#[derive(serde::Deserialize, serde::Serialize, PartialEq)]
pub enum Mode {
    EditProject,
    DevelopProject,
}

// derive Deserialize/Serialize so we can persist app state on powercycle.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct IronCoderApp {
    project: Project,
    display_about: bool,
    display_settings: bool,
    display_boards_window: bool,
    // #[serde(skip)]
    // modal: Option<Modal>,
    mode: Mode,
    colorscheme: ColorScheme,
    #[serde(skip)]
    boards: Vec<board::Board>,
    options: IronCoderOptions,

    warning_flags: Warnings,
    git_things: Git,
}

impl Default for IronCoderApp {
    fn default() -> Self {
        // Populate the boards
        let boards_dir = Path::new("./iron-coder-boards"); // consider making this a global macro
        let boards: Vec<board::Board> = board::get_boards(boards_dir);
        Self {
            project: Project::default(),
            display_about: false,
            display_settings: false,
            display_boards_window: false,
            // modal: None,
            mode: Mode::EditProject,
            boards: boards,
            colorscheme: colorscheme::INDUSTRIAL_DARK,
            options: IronCoderOptions::default(),
            // Warning Flags
            warning_flags: Warnings {
                display_mainboard_warning: false,
                display_unnamed_project_warning: false,
                display_git_warning: false,
            },
            git_things: Git {
                display: false,
                changes: Vec::new(),
                staged_changes: Vec::new(),
                commit_name: String::new(),
                commit_email: String::new(),
                commit_message: String::new(),
                repo: None,
            },
        }
    }
}

impl IronCoderApp {
    /// Called once before the first frame.
    pub fn with_options(cc: &eframe::CreationContext<'_>, options: IronCoderOptions) -> Self {
        info!("welcome to Iron Coder! setting up initial app state...");
        // we mutate cc.egui_ctx (the context) to set the overall app style
        setup_fonts_and_style(&cc.egui_ctx);
        install_image_loaders(&cc.egui_ctx);
        // Load previous app state if it exists and is specified.
        let mut app = IronCoderApp::default();
        if options.persistence {
            if let Some(storage) = cc.storage {
                info!("loading former app state from storage...");
                app = eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
            }
        }

        // Load settings from settings.toml if it exists
        info!("reading settings and applying to app state...");
        let mut settings_file = match File::open("settings.toml") {
            Err(why) => panic!("couldn't open settings.toml: {}", why),
            Ok(file) => file,
        };

        let mut settings_string = String::new();
        match settings_file.read_to_string(&mut settings_string) {
            Err(why) => panic!("couldn't read settings.toml: {}", why),
            Ok(_) => print!("settings.toml contains:\n{}", settings_string),
        }

        if (settings_string != "") {
            // Sets the scale for the app from settings.toml
            let scale = settings_string.lines().nth(0).unwrap().split("=").nth(1).unwrap().trim().parse::<f32>().unwrap();
            info!("setting ui scale to {}", scale);
            cc.egui_ctx.set_pixels_per_point(scale);


            // Sets the color scheme for the app from settings.toml
            let mut colorscheme_name = settings_string.lines().nth(1).unwrap().split("=").nth(1).unwrap().trim().to_string();
            info!("setting colorscheme to {}", colorscheme_name);
            colorscheme_name = colorscheme_name.trim_matches('"').to_string();
            let mut colorscheme = colorscheme::INDUSTRIAL_DARK;
            for cs in colorscheme::SYSTEM_COLORSCHEMES.iter() {
                if cs.name == colorscheme_name {
                    colorscheme = cs.clone();
                }
            }
            app.colorscheme = colorscheme.clone();
            colorscheme::set_colorscheme(&cc.egui_ctx, colorscheme.clone());
        }

        app.options = options;
        info!("Reloading current project and assets...");
        app.set_colorscheme(&cc.egui_ctx);
        app.project.known_boards = app.boards.clone();
        match app.project.reload() {
            Ok(_) => (),
            Err(e) => warn!("error reloading project from disk! {:?}", e),
        }

        return app;
    }

    /// Set the colorscheme for the app
    fn set_colorscheme(&self, ctx: &egui::Context) {
        colorscheme::set_colorscheme(ctx, self.colorscheme.clone());
    }

    /// Show the menu and app title
    pub fn display_title_and_menu(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let Self {
            display_about,
            display_settings,
            mode,
            project,
            ..
        } = self;
        let icons_ref: Arc<IconSet> = ctx.data_mut(|data| {
            data.get_temp("icons".into()).expect("couldn't load icons!")
        });
        let icons = icons_ref.clone();
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                // Create a NOTHING Rect so we can track where the header is drawn
                let mut r = egui::Rect::NOTHING;
                let start_r = ui.available_size();
                ui.centered_and_justified(|ui| {
                    // This will store the Rect that the header was drawn to
                    r = pretty_header(ui, "IRON CODER");
                    // update the max value to be the original Ui size
                    r.max.x = start_r.x;
                });
                // Now use that Rect to draw the menu icon at the proper place
                ui.allocate_ui_at_rect(r, |ui| {
                    let tid = icons.get("menu_icon").unwrap().clone();
                    ui.menu_image_button(tid, |ui| {

                        let ib = egui::widgets::Button::image_and_text(
                            icons.get("save_icon").unwrap().clone(),
                            "save project"
                        ).shortcut_text("ctrl+s");
                        if ui.add(ib).clicked() {
                            if let Err(e) = project.save() {
                                error!("error saving project: {:?}", e);
                            }
                        }

                        let ib = egui::widgets::Button::image_and_text(
                            icons.get("save_icon").unwrap().clone(),
                            "save project as..."
                        );
                        if ui.add(ib).clicked() {
                            project.save_as(true).unwrap_or_else(|_| warn!("couldn't save project!"));
                        }

                        let ib = egui::widgets::Button::image_and_text(
                            icons.get("folder_icon").unwrap().clone(),
                            "open"
                        ).shortcut_text("ctrl+o");
                        if ui.add(ib).clicked() {
                            match project.open() {
                                Ok(_) => {
                                    *mode = Mode::DevelopProject;
                                },
                                Err(e) => {
                                    error!("error opening project: {:?}", e);
                                },
                            }
                        }

                        let ib = egui::widgets::Button::image_and_text(
                            icons.get("boards_icon").unwrap().clone(),
                            "new project"
                        ).shortcut_text("ctrl+n");
                        if ui.add(ib).clicked() {
                            match mode {
                                Mode::EditProject      => (),
                                Mode::DevelopProject => {
                                    // TODO -- add a popup here confirming that user
                                    // wants to leave the current project, and probably save
                                    // the project in it's current state.
                                    *project = Project::default();
                                    project.known_boards = self.boards.clone();
                                    *mode = Mode::EditProject;
                                },
                            }
                        }

                        let ib = egui::widgets::Button::image_and_text(
                            icons.get("settings_icon").unwrap().clone(),
                            "settings"
                        );
                        if ui.add(ib).clicked() {
                            *display_settings = !*display_settings;
                        }

                        let ib = egui::widgets::Button::image_and_text(
                            icons.get("about_icon").unwrap().clone(),
                            "about Iron Coder"
                        );
                        if ui.add(ib).clicked() {
                            *display_about = !*display_about;
                        }

                        let ib = egui::widgets::Button::image_and_text(
                            icons.get("quit_icon").unwrap().clone(),
                            "quit"
                        ).shortcut_text("ctrl+q");
                        //.tint(egui::Color32::WHITE);
                        // TODO: set tint to the appropriate value for the current colorscheme
                        if ui.add(ib).clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        };
                    });
                });
            });
        });
    }

    /// Returns a copy of the list of available boards.
    pub fn get_boards(&self) -> Vec<board::Board> {
        self.boards.clone()
    }

    /// Show the main view when we're developing a project
    pub fn display_project_developer(&mut self, ctx: &egui::Context) {
        let Self {
            project,
            mode,
            ..
        } = self;
        // Spec Viewer panel
        egui::SidePanel::right("project_view").show(ctx, |ui| {
            if project.label_with_action(ctx, ui).clicked() {
                *mode = Mode::EditProject;
            };
            ui.separator();
            project.display_project_sidebar(ctx, ui);
        });

        egui::Area::new("editor area").show(ctx, |_ui| {
            egui::TopBottomPanel::bottom("terminal_panel").resizable(true).max_height(_ui.available_height()*0.75).show(ctx, |ui| {
                project.display_terminal(ctx, ui);
            });
            egui::TopBottomPanel::bottom("editor_control_panel").show(ctx, |ui| {
                project.display_project_toolbar(ctx, ui, &mut self.git_things);
            });
            egui::TopBottomPanel::top("editor_tabs").show(ctx, |ui| {
                project.code_editor.display_editor_tabs(ctx, ui);
            });
            let frame = egui::Frame::canvas(&ctx.style());
            egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
                project.code_editor.display_code(ctx, ui);
            });
        });
    }

    /// Show the various parts of the project editor
    pub fn display_project_editor(&mut self, ctx: &egui::Context) {

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(mode) = self.project.display_system_editor_hud(ctx, ui, &mut self.warning_flags) {
                self.mode = mode;
            }
            self.project.display_system_editor_boards(ctx, ui);
        });

    }

    /// show/hide the settings window and update the appropriate app state.
    pub fn display_settings_window(&mut self, ctx: &egui::Context) {
        let Self {
            display_settings,
            colorscheme,
            ..
        } = self;

        if *display_settings {
            let window_response = egui::Window::new("App Settings")
            .open(display_settings)
            .collapsible(false)
            .resizable(false)
            .movable(true)
            .show(ctx, |ui| {

                // Store the text edit string representing the ui scale
                ui.heading("Font Size:");
                let id = egui::Id::new("ui_scale_string");
                let current_scale = ctx.pixels_per_point();
                let mut ui_scale_string: String = ctx.data_mut(|data| {
                    data.get_temp(id).unwrap_or(current_scale.to_string())
                });
                ui.text_edit_singleline(&mut ui_scale_string);
                ctx.data_mut(|data| data.insert_temp(id, ui_scale_string.clone()));
                // if the string is parsable into f32, update the global scale
                match ui_scale_string.parse::<f32>() {
                    Ok(scale) if scale >=0.7 => {
                        ctx.set_pixels_per_point(scale);
                    },
                    Ok(_scale) => {
                        warn!("scale can't be below 0.7!");
                    }
                    Err(_e) => {
                        warn!("scale not parsed as f32.");
                    },
                }

                // Create radio buttons for colorscheme selection
                ui.separator();
                ui.heading("Color Scheme:");
                for cs in colorscheme::SYSTEM_COLORSCHEMES.iter() {
                    // ui.radio_value(&mut colorscheme, colorscheme::SOLARIZED_DARK, cs.name);
                    let rb = egui::RadioButton::new(*colorscheme == cs.clone(), cs.name.clone());
                    if ui.add(rb).clicked() {
                        *colorscheme = cs.clone();
                    }
                }

                // create a font selector:
                ui.separator();
                ui.heading("Font Selector:");
                // Tried working on selecotr box for fonts
                // Need to figure out how fonts are configured before continuing
                // Used example here: https://github.com/emilk/egui/blob/master/examples/user_attention/src/main.rs
                //
                // eframe::egui::ComboBox::new("","")
                //     .show_ui(ui, |ui| {
                //         for font in [

                //         ] {
                //             ui.selectable_value(&mut colorscheme, font, font);
                //         }
                //     });

                for (text_style, font_id) in ctx.style().text_styles.iter() {
                    match text_style {
                        egui::TextStyle::Name(name) => {
                            match &*name.clone() {
                                "HeadingBg" => continue,  // these are special fonts
                                "HeadingFg" => continue,  //    we should ignore
                                _ => (),
                            }
                            let egui::FontId {size: _, family} = font_id;
                            // I don't really understand this dereference syntax with the Arc...
                            let font_text = egui::RichText::new(&**name)
                                            .family((family).clone()).size(12.0);
                            ui.label(font_text);
                        },
                        egui::TextStyle::Monospace => {
                            let egui::FontId {size: _, family} = font_id;
                            // I don't really understand this dereference syntax with the Arc...
                            let font_text = egui::RichText::new("Default Monospace")
                                            .family((family).clone()).size(12.0);
                            ui.label(font_text);
                        }
                        _ => (),
                    }
                }

                ui.separator();
                ui.heading("Account Settings:");
                ui.label("Add github account here.");
                // ctx.set_visuals(visuals);


                // Create a button to apply the settings
                if ui.button("Apply").clicked() {
                    // Change settings when Apply button is pressed
                    // Change the colorscheme
                    colorscheme::set_colorscheme(ctx, colorscheme.clone());

                    // Set the ui scale
                    match ui_scale_string.parse::<f32>() {
                        Ok(scale) => {
                            ctx.set_pixels_per_point(scale);
                        },
                        Err(_e) => (),
                    }


                    // Write the settings to settings.toml
                    let mut settings_file = match File::create("settings.toml") {
                        Err(why) => panic!("couldn't create settings.toml: {}", why),
                        Ok(file) => file,
                    };

                    let mut settings_string = String::new();
                    settings_string.push_str("ui_scale = ");
                    settings_string.push_str(&ui_scale_string);
                    settings_string.push_str("\n");
                    settings_string.push_str("colorscheme = \"");
                    settings_string.push_str(&colorscheme.name);
                    settings_string.push_str("\"\n");

                    match settings_file.write_all(settings_string.as_bytes()) {
                        Err(why) => panic!("couldn't write to settings.toml: {}", why),
                        Ok(_) => println!("successfully wrote to settings.toml"),
                    }
                }
            });
            // unwrap ok here because window must be open for us to get here.
            // ctx.move_to_top(window_response.unwrap().response.layer_id);
            window_response.unwrap().response.layer_id.order = egui::Order::Foreground;
        }

    }

    /// This method will show or hide the "about" window
    pub fn display_about_window(&mut self, ctx: &egui::Context) {
        let Self {
            display_about,
            ..
        } = self;

        if !*display_about { return; }
        egui::Window::new("Iron Coder")
        .open(display_about)
        .collapsible(false)
        .resizable(false)
        .movable(true)
        .show(ctx, |ui| {
            ui.label("Iron Coder Version: 0.2.0");
            ui.separator();
            ui.label(
                "Iron Coder is an app for practicing embedded Rust development.\n\
                With inspirations from Arduino and CircuitPython, Iron Coder aims\n\
                to provide a fun environment for embedded development."
            );
            ui.separator();
            ui.label("Developed by Shulltronics");
            ui.hyperlink_to("Iron Coder on Github", "https://github.com/shulltronics/iron-coder");
            ui.horizontal(|ui| {
                egui::warn_if_debug_build(ui);
            });
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.label("powered by ");
                ui.hyperlink_to("egui", "https://github.com/emilk/egui");
                ui.label(" and ");
                ui.hyperlink_to(
                    "eframe",
                    "https://github.com/emilk/egui/tree/master/crates/eframe",
                );
                ui.label(".");
            });
        });
    }

    // Displays the waring message that no main board has been selected for the project
    pub fn unselected_mainboard_warning(&mut self, ctx: &egui::Context) {
        egui::Window::new("Board Warning")
        .open(&mut self.warning_flags.display_mainboard_warning)
        .collapsible(false)
        .resizable(false)
        .movable(true)
        .show(ctx,  |ui| {
            ui.label("please select a main board to proceed.");
        });
    }
    // Displays the waring message that the project has not been named
    pub fn display_unnamed_project_warning(&mut self, ctx: &egui::Context) {
        egui::Window::new("Name Warning")
        .open(&mut self.warning_flags.display_unnamed_project_warning)
        .collapsible(false)
        .resizable(false)
        .movable(true)
        .show(ctx,  |ui| {
            ui.label("please name the project to proceed.");
        });
    }

    /// Displays the warning message that not all of the git fields have been filled out
    /// This is called when the user tries to commit changes to git
    pub fn display_git_warning(&mut self, ctx: &egui::Context) {
        egui::Window::new("Git Warning")
        .open(&mut self.warning_flags.display_git_warning)
        .collapsible(false)
        .resizable(false)
        .movable(true)
        .show(ctx,  |ui| {
            ui.label("please fill out all of the git fields to commit changes.");
        });
    }

    /// Displays the git changes window
    // Is called by the toolbar when the user clicks the commit button
    pub fn display_git_window(&mut self, ctx: &egui::Context) {
        let mut display_git = self.git_things.display;
        let mut unstaged_to_remove: Vec<String> = Vec::new();
        let mut staged_to_remove: Vec<String> = Vec::new();
        let mut staged_to_add: Vec<String> = Vec::new();
        let mut unstaged_to_add: Vec<String> = Vec::new();


        egui::Window::new("Commit")
        .open(&mut display_git)
            .collapsible(false)
            .resizable(true)
            .movable(true)
            .show(ctx, |ui| {
                let repo = self.git_things.repo.as_mut().unwrap();
                let mut index = repo.index().unwrap();

                egui::SidePanel::right("Unstaged Changes").show_inside(ui, |ui| {
                    ui.label("Staged Changes -- Currently doesn't work");
                    ui.separator();
                    ui.vertical(|ui| {
                        for (_i, change) in self.git_things.staged_changes.iter().enumerate() {
                            if ui.button(change.clone()).clicked() {
                                info!("Unstaging: {}", change.clone());
                                unstaged_to_add.push(change.clone());
                                staged_to_remove.push(change.clone());
                                index.remove_all([change.clone()].iter(), None).unwrap();
                                index.write().unwrap();
                            }
                        }
                        self.git_things.staged_changes.retain(|change| !staged_to_remove.contains(change));
                    });
                    ui.separator();
                    ui.label("Unstaged Changes");
                    // Display the files that have changed on the right side
                    ui.separator();
                    ui.vertical(|ui| {
                        // Create a button for each unstaged change in git_things.changes
                        for (_i, change) in self.git_things.changes.iter().enumerate() {
                            if ui.button(change.clone()).clicked() {
                                info!("Staging: {}", change.clone());
                                staged_to_add.push(change.clone());
                                unstaged_to_remove.push(change.clone());
                                //index.add_path(Path::new(change)).unwrap();
                                match index.add_path(Path::new(change)) {
                                    Ok(_) => {
                                        // add_path succeeded, do nothing
                                    },
                                    Err(_) => {
                                        // add_path failed, try add_all
                                        index.add_all([change.clone()].iter(), git2::IndexAddOption::DEFAULT, None).unwrap();
                                    }
                                }
                                index.write().unwrap();
                            }
                        }
                        self.git_things.changes.retain(|change| !unstaged_to_remove.contains(change));
                    });
                });
                self.git_things.staged_changes.append(&mut staged_to_add);
                self.git_things.changes.append(&mut unstaged_to_add);

                egui::CentralPanel::default().show_inside(ui, |ui|{
                    // Have a text box for the commit message
                    // Have the text box take as much space as possible
                    ui.label("Commit Message:");
                    ui.text_edit_multiline(&mut self.git_things.commit_message);
                    ui.label("Name");
                    ui.text_edit_singleline(&mut self.git_things.commit_name);
                    ui.label("Email Address");
                    ui.text_edit_singleline(&mut self.git_things.commit_email);

                    let name = self.git_things.commit_name.clone();
                    let email = self.git_things.commit_email.clone();
                    let commit_message = self.git_things.commit_message.clone();

                    // Have a button to commit the changes
                    if ui.button("Commit").clicked() {
                        if name != "" && email != "" && commit_message != "" {
                            info!("committing changes to git...");
                            info!("{}", self.git_things.commit_message.clone());

                            let signature = git2::Signature::now(&name, &email).unwrap();
                            let oid = index.write_tree().unwrap();
                            let tree = repo.find_tree(oid).unwrap();
                            let head = repo.head().unwrap();
                            let head_commit = repo.find_commit(head.target().unwrap()).unwrap();


                            match repo.commit(
                                // There is a problem with the head
                                Some("HEAD"),
                                &signature,
                                &signature,
                                &commit_message,
                                &tree,
                                &[&head_commit]
                            ) {
                                Ok(_) => {
                                    info!("commit successful!");
                                },
                                Err(e) => {
                                    error!("error committing changes to git: {:?}", e);
                                }
                            }

                            self.git_things.display = false;
                            self.git_things.commit_message.clear();
                            self.git_things.commit_name.clear();
                            self.git_things.commit_email.clear();
                        } else {
                            self.warning_flags.display_git_warning = true;
                        }
                    }
                });
            });

            // Makes sure that both commit button and x button close the window
            if self.git_things.display == false || display_git == false {
                self.git_things.display = false;
                display_git = false;
                self.git_things.commit_message.clear();
                self.git_things.commit_name.clear();
                self.git_things.commit_email.clear();
                self.git_things.changes.clear();
                self.git_things.staged_changes.clear();
            }


    }
}

impl eframe::App for IronCoderApp {

    // Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        if self.options.persistence {
            info!("saving program state.");
            eframe::set_value(storage, eframe::APP_KEY, self);
        }
    }

    // Called each time the UI needs repainting, which may be many times per second.
    // This method will call all the display methods of IronCoderApp.
    // TODO -- is this the best architecture? Is there an overhead of destructuring
    //   self in each of these method calls separately, vs once in the beginning of this
    //   method? But I can't do it the latter way while still having these as method calls.
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // render the title bar with main menu
        self.display_title_and_menu(ctx, frame);
        // depending on the Mode, render the proper main view
        match self.mode {
            Mode::EditProject => {
                self.display_project_editor(ctx);
            },
            Mode::DevelopProject => {
                self.display_project_developer(ctx);
            },
        }
        // optionally render these popup windows
        self.display_settings_window(ctx);
        self.display_about_window(ctx);
        self.unselected_mainboard_warning(ctx);
        self.display_unnamed_project_warning(ctx);

        let save_shortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::S);
        let quit_shortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::Q);
        let open_shortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::O);
        let new_shortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::N);

        if ctx.input_mut(|i| i.consume_shortcut(&save_shortcut)) {
            if let Err(e) = self.project.save() {
                error!("error saving project: {:?}", e);
            }
        }

        if ctx.input_mut(|i| i.consume_shortcut(&quit_shortcut)) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        if ctx.input_mut(|i| i.consume_shortcut(&open_shortcut)) {
            match self.project.open() {
                Ok(_) => {
                    self.mode = Mode::DevelopProject;
                },
                Err(e) => {
                    error!("error opening project: {:?}", e);
                },
            }
        }

        if ctx.input_mut(|i| i.consume_shortcut(&new_shortcut)) {
            match self.mode {
                Mode::EditProject => (),
                Mode::DevelopProject => {
                    // TODO -- add a popup here confirming that user
                    // wants to leave the current project, and probably save
                    // the project in it's current state.
                    self.project = Project::default();
                    self.project.known_boards = self.boards.clone();
                    self.mode = Mode::EditProject;
                },
            }
        }

        self.display_git_window(ctx);
        self.display_git_warning(ctx);
    }
}

/// Sets up the fonts and style for the app
fn setup_fonts_and_style(ctx: &egui::Context) {

    let mut fonts = egui::FontDefinitions::default();

    /*
     *  The below fonts are used for standard text
     */
    fonts.font_data.insert(
        "roboto_mono_regular".to_owned(),
        egui::FontData::from_static(include_bytes!(
            "../../assets/fonts/Roboto_Mono/static/RobotoMono-Regular.ttf"
        )),
    );
    fonts.font_data.insert(
        "roboto_mono_bold".to_owned(),
        egui::FontData::from_static(include_bytes!(
            "../../assets/fonts/Roboto_Mono/static/RobotoMono-Bold.ttf"
        )),
    );
    fonts.font_data.insert(
        "chintzy_cpu".to_owned(),
        egui::FontData::from_static(include_bytes!(
            "../../assets/fonts/chintzycpu/chintzy.ttf"
        )),
    );
    fonts.font_data.insert(
        "vcr_osd_mono".to_owned(),
        egui::FontData::from_static(include_bytes!(
            "../../assets/fonts/vcr_osd_mono/VCR_OSD_MONO_1.001.ttf"
        )),
    );
    fonts.font_data.insert(
        "unispace".to_owned(),
        egui::FontData::from_static(include_bytes!(
            "../../assets/fonts/unispace/unispace it.otf"
        )),
    );

    // example of how to install font to an existing style
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .insert(0, "roboto_mono_regular".to_owned());

    /*
     *  The below fonts are used for the project logo
     */
    fonts.font_data.insert(
        "platinum_sign_under".to_owned(),    // serves as the unique font identifier?
        egui::FontData::from_static(include_bytes!(
            "../../assets/fonts/platinum-sign/Platinum-Sign-Under.ttf"
        )),
    );
    fonts.font_data.insert(
        "platinum_sign_over".to_owned(),    // serves as the unique font identifier?
        egui::FontData::from_static(include_bytes!(
            "../../assets/fonts/platinum-sign/Platinum-Sign-Over.ttf"
        )),
    );

    /*
     *  Now link "font family" names to the fonts for use in widgets
     */
    // create a new font family called "heading_fonts"
    fonts.families.insert(
        egui::FontFamily::Name("HeadingBackground".into()),
        vec!(String::from("platinum_sign_under"))
    );
    fonts.families.insert(
        egui::FontFamily::Name("HeadingForeground".into()),
        vec!(String::from("platinum_sign_over"))
    );
    fonts.families.insert(
        egui::FontFamily::Name("EditorFont".into()),
        vec!(String::from("roboto_mono_regular"))
    );
    fonts.families.insert(
        egui::FontFamily::Name("MonospaceBold".into()),
        vec!(String::from("roboto_mono_bold"))
    );

    ctx.set_fonts(fonts);

    // setup our custom style
    let mut style = egui::style::Style::default();

    // Redefine text_styles
    use egui::FontId;
    use egui::FontFamily;
    use egui::TextStyle::*;
    style.text_styles = [
        (Small, FontId::new(10.0, FontFamily::Monospace)),
        (Body, FontId::new(12.0, FontFamily::Monospace)),
        (Monospace, FontId::new(12.0, FontFamily::Monospace)),
        (Button, FontId::new(12.0, FontFamily::Monospace)),
        (Heading, FontId::new(14.0, FontFamily::Monospace)),
        (Name("HeadingBg".into()), FontId::new(18.0, FontFamily::Name("HeadingBackground".into()))),
        (Name("HeadingFg".into()), FontId::new(18.0, FontFamily::Name("HeadingForeground".into()))),
        (Name("DefaultBold".into()), FontId::new(12.0, FontFamily::Name("MonospaceBold".into()))),
        (Name("DefaultRegular".into()), FontId::new(12.0, FontFamily::Monospace)),
        (Name("EditorFont".into()), FontId::new(16.0, FontFamily::Name("EditorFont".into()))),
    ].into();

    // Make things look more square
    style.visuals.menu_rounding   = egui::Rounding::ZERO;
    style.visuals.window_rounding = egui::Rounding::ZERO;
    // change width of scroll bar
    style.spacing.scroll.bar_width = 6.0;
    style.spacing.scroll.bar_inner_margin = 6.0;    // this keeps some space
    // Remove shadows
    style.visuals.window_shadow = eframe::epaint::Shadow::NONE;
    style.visuals.popup_shadow = eframe::epaint::Shadow::NONE;

    ctx.set_style(style);

    // Store icons in the egui shared Context
    ctx.data_mut(|map| {
        info!("Adding IconSet to egui Context temp data.");
        map.insert_temp("icons".into(), Arc::new(icons::load_icons(Path::new(icons::ICON_DIR))));
    });
}

/// Displays a cool looking header in the Ui element, utilizing our custom fonts
/// and returns the rect that was drawn to.
fn pretty_header(ui: &mut egui::Ui, text: &str) -> egui::Rect {
    // draw the background and get the rectangle we drew to
    let text_bg = RichText::new(text.to_uppercase())
        .text_style(egui::TextStyle::Name("HeadingBg".into()));
    let heading_bg = Label::new(text_bg);
    // expand the rectangle slightly to prevent the bug where the next
    // font draw will wrap to a new line
    let mut rect = ui.add(heading_bg).rect;
    rect.min.x -= 1.0;
    rect.max.x += 1.0;
    // put the overlay text
    let text_fg = RichText::new(text)
        .color(Color32::WHITE)
        .text_style(egui::TextStyle::Name("HeadingFg".into()));
    let heading_fg = Label::new(text_fg);
    // let location = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::ZERO);
    ui.put(rect, heading_fg);
    return rect;
}
