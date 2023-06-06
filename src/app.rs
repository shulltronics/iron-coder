//! Iron Coder is an app for developing embedded firmware in Rust
use std::path::Path;

use crate::board;
use crate::editor;

/// The current GUI mode
#[derive(serde::Deserialize, serde::Serialize)]
enum Mode {
    BoardSelector,
    Editor,
}

// derive Deserialize/Serialize so we can persist app state on powercycle.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct IronCoderApp {
    #[serde(skip)]
    board: board::Board,
    display_info: bool,
    #[serde(skip)]                      // serde isn't crutial b.e. in the future
    code_editor: editor::CodeEditor,    // we will load to and from disk on power cycle
    mode: Mode,
    #[serde(skip)]
    boards: Vec<board::Board>,
}

impl Default for IronCoderApp {
    fn default() -> Self {
        // Populate the boards
        let boards_dir: &Path = Path::new("./boards");
        let boards: Vec<board::Board> = board::get_boards(boards_dir);

        let mut editor = editor::CodeEditor::default();
        let code_path: &Path = Path::new("./boards/Adafruit/Feather_RP2040/examples/blinky/main.rs");
        editor.load_from_file(code_path).unwrap();

        Self {
            // Example stuff:
            board: boards[0].clone(),
            display_info: false,
            code_editor: editor,
            mode: Mode::Editor,
            boards: boards,
        }
    }
}

impl IronCoderApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {

        // we mutate cc.egui_ctx (the context) to set the overall app style
        setup_fonts_and_style(&cc.egui_ctx);

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }
        
        // Now return a default IronCoderApp
        Default::default()
    }
}

impl eframe::App for IronCoderApp {

    // Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    // Called each time the UI needs repainting, which may be many times per second.
    // Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let Self {
            board,
            display_info,
            code_editor,
            mode,
            boards
        } = self;

        // The top panel containing menu items and logo will alway be present
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                // ui.spacing_mut().window_margin.left  = 24.0;
                // ui.spacing_mut().window_margin.right = 24.0;
                // ui.spacing_mut().menu_margin.left  = 24.0;
                // ui.spacing_mut().menu_margin.right = 24.0;
                //   example of how to create and display an image
                //   TODO - figure out how to load from a file
                // let texture: &egui::TextureHandle = &ui.ctx().load_texture(
                //     "my-image",
                //     egui::ColorImage::new([64, 16], egui::Color32::WHITE),
                //     Default::default()
                // );
                // // Show the image:
                // ui.image(texture, texture.size_vec2());

                ui.menu_button("MENU", |ui| {
                    if ui.button("SAVE").clicked() {
                        println!("todo!");
                    }
                    if ui.button("OPEN").clicked() {
                        println!("todo!");
                    }
                    if ui.button("BOARDS").clicked() {
                        match mode {
                            Mode::BoardSelector => *mode = Mode::Editor,
                            Mode::Editor        => *mode = Mode::BoardSelector,
                        }
                    }
                    if ui.button("SETTINGS").clicked() {
                        println!("button clicked!");
                    }
                    if ui.button("ABOUT").clicked() {
                        *display_info = !*display_info;
                    }
                    if ui.button("QUIT").clicked() {
                        _frame.close();
                    }
                });

                if *display_info {
                    about_iron_coder(ctx, ui, display_info);
                }

                // add the logo, centered
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    pretty_header(ui, "IRON CODER");
                });
                // TODO - Understand the layout better and get the alignments right
                // ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // ui.menu_button("MENU", |ui| {
                    //     if ui.button("SETTINGS").clicked() {
                    //         println!("button clicked!");
                    //     }
                    //     if ui.button("QUIT").clicked() {
                    //         _frame.close();
                    //     }
                    // });
                // });

            });
        });

        // The GUI will look different depending on the Mode we're in
        match mode {

            // BoardSelector mode is the mode when selecting a new project
            Mode::BoardSelector => {

                let central_frame = egui::Frame::default();
                egui::CentralPanel::default().frame(central_frame).show(ctx, |ui| {
                    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                        ui.label("\nBoard Selector\nCreate a new project with the given board\n");
                    });
                    // show all the boards
                    for (i, b) in boards.clone().into_iter().enumerate() {
                        if ui.add(b).on_hover_text(boards[i].get_name()).clicked() {
                            println!("board {} was clicked!", i);
                            // TODO create a new project here
                            *mode = Mode::Editor;
                            *board = boards[i].clone();
                        }
                    }
                });
            },

            // Editor mode is the main mode for editing and building code
            Mode::Editor => {
                // Spec Viewer panel
                egui::SidePanel::right("side_panel").show(ctx, |ui| {
                    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                        // pretty_header(ui, "SPEC VIEWER");
                        ui.heading("SPEC VIEWER");
                    });

                    ui.add(board.clone());

                    // println!("available size is {:?}", ui.available_size());
                    // println!("image size is {:?}", image.size_vec2());
                    // image.show_max_size(ui, ui.available_size());
                    //ui.add(egui::widgets::Image::new(image.texture_id(), image.size_vec2()));


                    ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
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
                });

                // Editor panel
                egui::CentralPanel::default().frame(egui::Frame::default()).show(ctx, |ui| {
                    code_editor.display(ctx, ui);
                });

            },
        }

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

    // example of how to install font to an existing style 
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .insert(0, "vcr_osd_mono".to_owned());

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
        (Body, FontId::new(14.0, FontFamily::Monospace)),
        (Monospace, FontId::new(14.0, FontFamily::Monospace)),
        (Button, FontId::new(12.0, FontFamily::Monospace)),
        (Heading, FontId::new(14.0, FontFamily::Monospace)),
        (Name("HeadingBg".into()), FontId::new(18.0, FontFamily::Name("HeadingBackground".into()))),
        (Name("HeadingFg".into()), FontId::new(18.0, FontFamily::Name("HeadingForeground".into()))),
    ].into();

    // Make things look more square
    style.visuals.menu_rounding   = egui::Rounding::none();
    style.visuals.window_rounding = egui::Rounding::none();

    ctx.set_style(style);
}

fn pretty_header(ui: &mut egui::Ui, text: &str) {
    /* Displays a cool looking header in the Ui element,
     *  utilizing our custom fonts */
    use egui::{RichText, Label, Color32};
    // draw the background and get the rectangle we drew to
    let text_bg = RichText::new(text.to_uppercase())
        .text_style(egui::TextStyle::Name("HeadingBg".into()));
    let heading_bg = Label::new(text_bg);
    let rect = ui.add(heading_bg).rect;
    // put the overlay text
    let text_fg = RichText::new(text)
        .color(Color32::WHITE)
        .text_style(egui::TextStyle::Name("HeadingFg".into()));
    let heading_fg = Label::new(text_fg);
    // let location = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::ZERO);
    ui.put(rect, heading_fg);
}

fn about_iron_coder(ctx: &egui::Context, _ui: &mut egui::Ui, is_shown: &mut bool) {
    egui::Window::new("Iron Coder")
        .open(is_shown)
        .collapsible(true)
        .default_size(egui::vec2(512.0, 512.0))
        .resizable(true)
        .movable(false)
        .show(ctx, |ui| {
            ui.label(
                "Iron Coder is an app for practicing embedded Rust development.\n\
                With inspirations from Arduino and CircuitPython, Iron Coder aims\n\
                to provide a fun environment for embedded development."
            );
            ui.label("Developed by Shulltronics");
            ui.hyperlink_to("Iron Coder on Github", "https://github.com/shulltronics/iron-coder");
    });
}