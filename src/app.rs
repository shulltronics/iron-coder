//! Iron Coder is an app for developing embedded firmware in Rust

use log::{info, warn};

use std::path::Path;
use std::collections::HashMap;

use egui_extras::image::RetainedImage;

// Separate modules
use crate::board;
use crate::colorscheme;
use crate::project::Project;
use crate::icons;

/// The current GUI mode
#[derive(serde::Deserialize, serde::Serialize)]
pub enum Mode {
    CreateNewProject,
    EditCurrentProject,
    DevelopCurrentProject,
}

// derive Deserialize/Serialize so we can persist app state on powercycle.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct IronCoderApp {
    project: Project,
    display_about: bool,
    display_settings: bool,
    #[serde(skip)]          // future might want this to persist, but this is nice for testing
    mode: Mode,
    #[serde(skip)]
    icons: HashMap<&'static str, RetainedImage>,
    #[serde(skip)]
    boards: Vec<board::Board>,
    #[serde(skip)]
    new_project: Project,   // this is a place holder for when we create a new project
}

impl Default for IronCoderApp {
    fn default() -> Self {
        // Populate the boards
        let boards_dir = Path::new("./boards");
        let boards: Vec<board::Board> = board::get_boards(boards_dir);
        Self {
            project: Project::default(),
            display_about: false,
            display_settings: false,
            mode: Mode::CreateNewProject,
            icons: icons::load_icons(Path::new(icons::ICON_DIR)),
            boards: boards,
            new_project: Project::default(),
        }
    }
}

impl IronCoderApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        info!("welcome to Iron Coder! setting up initial app state...");
        // we mutate cc.egui_ctx (the context) to set the overall app style
        setup_fonts_and_style(&cc.egui_ctx);
        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            info!("loading former app state from storage...");
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }
        // Now return a default IronCoderApp
        Default::default()
    }

    // Show the menu and app title
    pub fn display_title_and_menu(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let Self {
            display_about,
            display_settings,
            mode,
            icons,
            new_project,
            ..
        } = self;
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
                    let tid = icons.get("menu_icon").unwrap().texture_id(ctx);
                    ui.menu_image_button(tid, egui::Vec2::new(12.0, 12.0), |ui| {
                        
                        let ib = egui::widgets::Button::image_and_text(
                            icons.get("save_icon").unwrap().texture_id(ctx),
                            egui::Vec2::new(8.0, 8.0),
                            "save project"
                        ).shortcut_text("ctrl+s");
                        if ui.add(ib).clicked() {
                            if let Err(e) = self.project.save() {
                                println!("error saving project: {:?}", e);
                            }
                        }

                        let ib = egui::widgets::Button::image_and_text(
                            icons.get("save_icon").unwrap().texture_id(ctx),
                            egui::Vec2::new(8.0, 8.0),
                            "save project as..."
                        );
                        if ui.add(ib).clicked() {
                            self.project.save_as().unwrap_or_else(|_| warn!("couldn't save project!"));
                        }

                        let ib = egui::widgets::Button::image_and_text(
                            icons.get("folder_icon").unwrap().texture_id(ctx),
                            egui::Vec2::new(8.0, 8.0),
                            "open"
                        ).shortcut_text("ctrl+o");
                        if ui.add(ib).clicked() {
                            if let Err(e) = self.project.open() {
                                println!("error opening project: {:?}", e);
                            } else {
                                self.project.load_board_resources(self.boards.clone());
                                *mode = Mode::DevelopCurrentProject;
                            }
                        }
                        
                        let ib = egui::widgets::Button::image_and_text(
                            icons.get("boards_icon").unwrap().texture_id(ctx),
                            egui::Vec2::new(8.0, 8.0),
                            "new project"
                        ).shortcut_text("ctrl+n");
                        if ui.add(ib).clicked() {
                            match mode {
                                Mode::CreateNewProject      => (),
                                Mode::EditCurrentProject    => (),
                                Mode::DevelopCurrentProject => {
                                    *new_project = Project::default();
                                    *mode = Mode::CreateNewProject;
                                },
                            }
                        }

                        let ib = egui::widgets::Button::image_and_text(
                            icons.get("settings_icon").unwrap().texture_id(ctx),
                            egui::Vec2::new(8.0, 8.0),
                            "settings"
                        );
                        if ui.add(ib).clicked() {
                            *display_settings = !*display_settings;
                        }

                        let ib = egui::widgets::Button::image_and_text(
                            icons.get("about_icon").unwrap().texture_id(ctx),
                            egui::Vec2::new(8.0, 8.0),
                            "about Iron Coder"
                        );
                        if ui.add(ib).clicked() {
                            *display_about = !*display_about;
                        }
                   
                        let ib = egui::widgets::Button::image_and_text(
                            icons.get("quit_icon").unwrap().texture_id(ctx),
                            egui::Vec2::new(8.0, 8.0),
                            "quit"
                        ).shortcut_text("ctrl+q");
                        //.tint(egui::Color32::WHITE);
                        // TODO: set tint to the appropriate value for the current colorscheme
                        if ui.add(ib).clicked() {
                            frame.close();
                        };
                    });
                });
            });
        });
    }

    // Display the list of available boards in the Ui, and return one if it was clicked
    pub fn display_available_boards(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui) -> Option<board::Board> {
        let mut board: Option<board::Board> = None;
        // Create a grid-based layout to show all the board widgets
        let available_width = ui.available_width();
        let mut num_cols = (available_width / 260.0) as usize;
        if num_cols == 0 {
            num_cols = 1;
        }
        egui::containers::scroll_area::ScrollArea::vertical().show(ui, |ui| {
            ui.columns(num_cols, |columns| {
                for (i, b) in self.boards.clone().into_iter().enumerate() {
                    let col = i % num_cols;
                    // When a board is clicked, add it to the new project
                    if columns[col].add(board::BoardSelectorWidget(b)).clicked() {
                        board = Some(self.boards[i].clone());
                    }
                }
            });
        });
        return board;
    }

    // Show the main view when we're developing a project
    pub fn display_project_developer(&mut self, ctx: &egui::Context) {
        let Self {
            new_project,
            project,
            mode,
            boards,
            ..
        } = self;
        // Spec Viewer panel
        egui::SidePanel::right("project_view").show(ctx, |ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.heading("Project View");
                ui.label(project.get_name()).on_hover_text(project.get_location());
                if ui.button("edit project").clicked() {
                    *new_project = project.clone();
                    *mode = Mode::EditCurrentProject;
                }
                ui.separator();
            });
            project.display_project_sidebar(ctx, ui);
        });

        egui::Area::new("editor area").show(ctx, |_ui| {
            egui::TopBottomPanel::bottom("terminal_panel").resizable(true).show(ctx, |ui| {
                project.display_terminal(ctx, ui);
            });
            egui::TopBottomPanel::bottom("editor_control_panel").show(ctx, |ui| {
                project.display_project_toolbar(ctx, ui);
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

    // show/hide the settings window and update the appropriate app state.
    pub fn display_settings_window(&mut self, ctx: &egui::Context) {
        let Self {
            project,
            display_settings,
            ..
        } = self;

        if *display_settings {
            egui::Window::new("App Settings")
            .open(display_settings)
            .collapsible(false)
            .resizable(false)
            .movable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                let mut visuals = ctx.style().visuals.clone();
                ui.checkbox(&mut visuals.dark_mode, "Dark Mode");
                if visuals.dark_mode {
                    colorscheme::set_colorscheme(ctx, colorscheme::SOLARIZED_DARK);
                    project.code_editor.set_colorscheme(colorscheme::SOLARIZED_DARK);
                } else {
                    colorscheme::set_colorscheme(ctx, colorscheme::SOLARIZED_LIGHT);
                    project.code_editor.set_colorscheme(colorscheme::SOLARIZED_DARK);
                }

                // create a font selector:
                for (text_style, font_id) in ctx.style().text_styles.iter() {
                    // println!("{:?}: {:?}", text_style, font_id);
                    match text_style {
                        egui::TextStyle::Name(name) => {
                            match &*name.clone() {
                                "HeadingBg" => continue,  // these are special fonts
                                "HeadingFg" => continue,  //    we should ignore
                                _ => (),
                            }
                            let egui::FontId {size: _, family} = font_id;
                            // I don't really understand this dereference syntax with the Arc...
                            let font_text = egui::RichText::new((&**name).clone())
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
                // ctx.set_visuals(visuals);
            });
        }
        
    }

    // This method will show or hide the "about" window
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
        .movable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.label(
                "Iron Coder is an app for practicing embedded Rust development.\n\
                With inspirations from Arduino and CircuitPython, Iron Coder aims\n\
                to provide a fun environment for embedded development."
            );
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
}

impl eframe::App for IronCoderApp {

    // Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        info!("saving program state.");
        eframe::set_value(storage, eframe::APP_KEY, self);
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
            Mode::CreateNewProject => {
                // using a frame allows to add inner margins
                let frame = egui::Frame::side_top_panel(&ctx.style()).inner_margin(egui::Margin::same(25.0));
                egui::TopBottomPanel::top("board_selector_top_panel")
                .frame(frame)
                .show(ctx, |ui| {
                    self.new_project.display_project_editor(ctx, ui);
                    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                        let label = egui::widgets::Label::new("Welcome to Iron Coder! To get started on a project, select a main \
                                        board and a set of peripheral boards. Then, give your project a name. \
                                        After clicking \"Start Development\" you will be prompted to choose \
                                        a location to save you project.");
                        let egui::Vec2 {mut x, y} = ui.available_size();
                        if x > 300.0 { x = 300.0 }
                        ui.add_sized([x, 0.0], label);
                        if ui.button("Start Development").clicked() {
                            self.project = self.new_project.clone();
                            self.project.save_as().unwrap_or_else(|_| warn!("couldn't save project!"));
                            self.project.add_crates_to_project(ctx);
                            // info!("{:?}", self.project.get_boards()[0]);
                            self.mode = Mode::DevelopCurrentProject;
                        }
                    });
                });
                egui::CentralPanel::default().show(ctx, |ui| {
                    if let Some(b) = self.display_available_boards(ctx, ui) {
                        self.new_project.add_board(b);
                    }
                });
            },
            Mode::EditCurrentProject => {
                // using a frame allows to add inner margins
                let frame = egui::Frame::side_top_panel(&ctx.style()).inner_margin(egui::Margin::same(25.0));
                egui::TopBottomPanel::top("board_selector_top_panel")
                .frame(frame)
                .show(ctx, |ui| {
                    self.project.display_project_editor(ctx, ui);
                    if ui.button("Resume Development").clicked() {
                        self.mode = Mode::DevelopCurrentProject;
                    }
                });
                egui::CentralPanel::default().show(ctx, |ui| {
                    if let Some(b) = self.display_available_boards(ctx, ui) {
                        self.project.add_board(b);
                    }
                });
            },
            Mode::DevelopCurrentProject => {
                self.display_project_developer(ctx);
            },
        }
        // optionally render these popup windows
        self.display_settings_window(ctx);
        self.display_about_window(ctx);
    }
}

fn setup_fonts_and_style(ctx: &egui::Context) {

    let mut fonts = egui::FontDefinitions::default();

    /*
     *  The below fonts are used for standard text
     */
    fonts.font_data.insert(
        "roboto_mono_regular".to_owned(),
        egui::FontData::from_static(include_bytes!(
            "../assets/fonts/Roboto_Mono/static/RobotoMono-Regular.ttf"
        )),
    );
    fonts.font_data.insert(
        "roboto_mono_bold".to_owned(),
        egui::FontData::from_static(include_bytes!(
            "../assets/fonts/Roboto_Mono/static/RobotoMono-Bold.ttf"
        )),
    );
    fonts.font_data.insert(
        "chintzy_cpu".to_owned(),
        egui::FontData::from_static(include_bytes!(
            "../assets/fonts/chintzycpu/chintzy.ttf"
        )),
    );
    fonts.font_data.insert(
        "vcr_osd_mono".to_owned(),
        egui::FontData::from_static(include_bytes!(
            "../assets/fonts/vcr_osd_mono/VCR_OSD_MONO_1.001.ttf"
        )),
    );
    fonts.font_data.insert(
        "unispace".to_owned(),
        egui::FontData::from_static(include_bytes!(
            "../assets/fonts/unispace/unispace it.otf"
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
            "../assets/fonts/platinum-sign/Platinum-Sign-Under.ttf"
        )),
    );
    fonts.font_data.insert(
        "platinum_sign_over".to_owned(),    // serves as the unique font identifier?
        egui::FontData::from_static(include_bytes!(
            "../assets/fonts/platinum-sign/Platinum-Sign-Over.ttf"
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

    // we could change certain aspects of the global spacing like so:
    // style.spacing.menu_margin.left  = 64.0;
    // style.spacing.menu_margin.right = 64.0;
    // println!("{:?}", style.spacing.menu_margin.left_top());

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
    style.visuals.menu_rounding   = egui::Rounding::none();
    style.visuals.window_rounding = egui::Rounding::none();
    // change width of scroll bar
    style.spacing.scroll_bar_width = 6.0;
    style.spacing.scroll_bar_inner_margin = 6.0;    // this keeps some space
    // Remove shadows
    style.visuals.window_shadow = eframe::epaint::Shadow::NONE;
    style.visuals.popup_shadow = eframe::epaint::Shadow::NONE;

    ctx.set_style(style);

    colorscheme::set_colorscheme(ctx, colorscheme::SOLARIZED_DARK);
}

// Displays a cool looking header in the Ui element, utilizing our custom fonts
// and returns the rect that was drawn to.
fn pretty_header(ui: &mut egui::Ui, text: &str) -> egui::Rect {
    use egui::{RichText, Label, Color32};
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