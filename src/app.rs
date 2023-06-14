//! Iron Coder is an app for developing embedded firmware in Rust

use std::path::Path;
use std::collections::HashMap;

// Imports for loading and saving images
use image;
use egui::ColorImage;
use egui_extras::image::RetainedImage;

// Separate modules
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
    display_settings: bool,
    #[serde(skip)]                      // serde isn't crutial b.e. in the future
    code_editor: editor::CodeEditor,    // we will load to and from disk on power cycle
    mode: Mode,
    #[serde(skip)]
    boards: Vec<board::Board>,
    #[serde(skip)]
    icons: HashMap<&'static str, RetainedImage>,
}

impl Default for IronCoderApp {
    fn default() -> Self {
        // Populate the boards
        let boards_dir = Path::new("./boards");
        let boards: Vec<board::Board> = board::get_boards(boards_dir);

        let mut editor = editor::CodeEditor::default();
        let code_path = Path::new("./boards/Adafruit/Feather_RP2040/examples/blinky/src/main.rs");
        editor.load_from_file(code_path).unwrap();

        let icons_dir = Path::new("assets/icons/pack/white/");
        let icons = load_icons(icons_dir);

        Self {
            board: boards[0].clone(),
            display_info: false,
            display_settings: false,
            code_editor: editor,
            mode: Mode::Editor,
            boards: boards,
            icons: icons,
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
            display_settings,
            code_editor,
            mode,
            boards,
            icons,
        } = self;

        // The top panel containing menu items and logo will alway be present
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
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
                            "save"
                        ).shortcut_text("ctrl+s");
                        if ui.add(ib).clicked() {
                            println!("todo!");
                        }

                        let ib = egui::widgets::Button::image_and_text(
                            icons.get("folder_icon").unwrap().texture_id(ctx),
                            egui::Vec2::new(8.0, 8.0),
                            "open"
                        ).shortcut_text("ctrl+o");
                        if ui.add(ib).clicked() {
                            println!("todo!");
                        }
                        
                        let ib = egui::widgets::Button::image_and_text(
                            icons.get("boards_icon").unwrap().texture_id(ctx),
                            egui::Vec2::new(8.0, 8.0),
                            "boards"
                        ).shortcut_text("ctrl+n");
                        if ui.add(ib).clicked() {
                            match mode {
                                Mode::BoardSelector => *mode = Mode::Editor,
                                Mode::Editor        => *mode = Mode::BoardSelector,
                            }
                        }

                        let ib = egui::widgets::Button::image_and_text(
                            icons.get("settings_icon").unwrap().texture_id(ctx),
                            egui::Vec2::new(8.0, 8.0),
                            "setting"
                        );
                        if ui.add(ib).clicked() {
                            *display_settings = !*display_settings;
                        }

                        let ib = egui::widgets::Button::image_and_text(
                            icons.get("about_icon").unwrap().texture_id(ctx),
                            egui::Vec2::new(8.0, 8.0),
                            "about"
                        );
                        if ui.add(ib).clicked() {
                            *display_info = !*display_info;
                        }
                   
                        let ib = egui::widgets::Button::image_and_text(
                            icons.get("quit_icon").unwrap().texture_id(ctx),
                            egui::Vec2::new(8.0, 8.0),
                            "quit"
                        ).shortcut_text("ctrl+q");
                        //.tint(egui::Color32::WHITE);
                        // TODO: set tint to the appropriate value for the current colorscheme
                        if ui.add(ib).clicked() {
                            _frame.close();
                        };
                    });
                });

                // Determine if we need to display any overlay windows
                if *display_info {
                    about_iron_coder(ctx, ui, display_info);
                }
                if *display_settings {
                    settings(ctx, ui, display_settings);
                }

            });
        });

        // The GUI will look different depending on the Mode we're in
        match mode {

            // BoardSelector mode is the mode when selecting a new project
            Mode::BoardSelector => {

                // TODO - add a top panel for a search bar, etc

                let central_frame = egui::Frame::default();
                egui::CentralPanel::default().frame(central_frame).show(ctx, |ui| {
                    
                    // Create a grid-based layout to show all the board widgets
                    // println!("{:?}", ui.available_width());
                    // TODO determine # columns based on available width
                    let num_cols = 3;
                    ui.columns(num_cols, |columns| {
                        for (i, b) in boards.clone().into_iter().enumerate() {
                            let col = i % num_cols;
                            if columns[col].add(b).on_hover_text(boards[i].get_name()).clicked() {
                                println!("board {} was clicked!", i);
                                // TODO create a new project here
                                *mode = Mode::Editor;
                                *board = boards[i].clone();
                            }
                        }
                    });
                    
                    // ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    //     ui.label("\nBoard Selector\nCreate a new project with the given board\n");
                    // });
                    // // show all the boards
                    // for (i, b) in boards.clone().into_iter().enumerate() {
                    //     if ui.add(b).on_hover_text(boards[i].get_name()).clicked() {
                    //         println!("board {} was clicked!", i);
                    //         // TODO create a new project here
                    //         *mode = Mode::Editor;
                    //         *board = boards[i].clone();
                    //     }
                    // }

                
                });
            },

            // Editor mode is the main mode for editing and building code
            Mode::Editor => {
                // Spec Viewer panel
                egui::SidePanel::right("side_panel").show(ctx, |ui| {
                    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                        ui.heading("Project View");
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
                // egui::CentralPanel::default().frame(egui::Frame::default()).show(ctx, |ui| {
                //     code_editor.display(ctx, ui);
                // });
                egui::Area::new("editor area").show(ctx, |ui| {
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

    // example of how to install font to an existing style 
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .insert(0, "vcr_osd_mono".to_owned());

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

    // println!("{:#?}", style);
    ctx.set_style(style);
}

// This function returns a mapping of icon names to RetainedImages 
fn load_icons(icon_path: &Path) -> HashMap<&'static str, RetainedImage> {

    let mut icon_map = std::collections::HashMap::new();

    let icon_names_and_files: [(&str, &str); 8] = [
        ("settings_icon", "gear.png"),
        ("boards_icon", "chip.png"),
        ("about_icon", "005b_13.gif"),
        ("folder_icon", "005b_43.gif"),
        ("save_icon", "005b_23.gif"),
        ("build_icon", "005b_35.gif"),
        ("menu_icon", "005b_44.gif"),
        ("quit_icon", "005b_75.gif"),
    ];

    for (icon_name, icon_file) in icon_names_and_files.into_iter() {
        let p = icon_path.join(icon_file);
        // attempt to open the icon image file
        let im_file = match image::io::Reader::open(p) {
            Err(e) => {
                println!("error reading icon file {:?}: {:?}", icon_file, e);
                break;
            },
            Ok(im_file) => {
                im_file
            }
        };
        // attempt to decode it
        let image = match im_file.decode() {
            Err(e) => {
                println!("error decoding icon file {:?}: {:?}", icon_file, e);
                break;
            },
            Ok(image) => {
                image
            }
        };
        let size = [image.width() as _, image.height() as _];
        let image_buffer = image.to_rgba8();
        let image_samples = image_buffer.as_flat_samples();
        let color_image = ColorImage::from_rgba_unmultiplied(
            size,
            image_samples.as_slice(),
        );
        let retained_image = RetainedImage::from_color_image(
            icon_name,
            color_image,
        );
        icon_map.insert(icon_name, retained_image);
    }
    return icon_map;
}

/* Displays a cool looking header in the Ui element, utilizing our custom fonts
 * and returns the rect that was drawn to
 */
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

fn about_iron_coder(ctx: &egui::Context, _ui: &mut egui::Ui, is_shown: &mut bool) {
    egui::Window::new("Iron Coder")
        .open(is_shown)
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
    });
}

// This function opens a window to change settings of the app
fn settings(ctx: &egui::Context, _ui: &mut egui::Ui, is_shown: &mut bool) {
    egui::Window::new("App Settings")
        .open(is_shown)
        .collapsible(false)
        .resizable(false)
        .movable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            let mut visuals = ctx.style().visuals.clone();
                ui.checkbox(&mut visuals.dark_mode, "Dark Mode");
                if visuals.dark_mode {
                    visuals.extreme_bg_color = egui::Color32::BLACK;
                    visuals.panel_fill       = egui::Color32::BLACK;
                } else {
                    visuals.extreme_bg_color = egui::Color32::WHITE;
                    visuals.panel_fill       = egui::Color32::WHITE;
                }

                // create a font selector:
                for (text_style, font_id) in ctx.style().text_styles.iter() {
                    // println!("{:?}: {:?}", text_style, font_id);
                    match text_style {
                        egui::TextStyle::Name(name) => {
                            match (&**name).clone() {
                                "HeadingBg" => continue,  // these are special fonts
                                "HeadingFg" => continue,  //    we should ignore
                                _ => (),
                            }
                            let egui::FontId {size: _, family} = font_id;
                            // I don't really understand this dereference syntax with the Arc...
                            let font_text = egui::RichText::new((&**name).clone())
                                            .family((*family).clone()).size(12.0);
                            ui.label(font_text);
                        },
                        egui::TextStyle::Monospace => {
                            let egui::FontId {size: _, family} = font_id;
                            // I don't really understand this dereference syntax with the Arc...
                            let font_text = egui::RichText::new("Default Monospace")
                                            .family((*family).clone()).size(12.0);
                            ui.label(font_text);
                        }
                        _ => (),
                    }
                }

            ctx.set_visuals(visuals);
    });
}
