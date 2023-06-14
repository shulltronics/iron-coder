use std::path::{Path, PathBuf};
use std::fs;
use std::vec::Vec;

use serde::Deserialize;

use egui_extras::RetainedImage;

use egui::{Ui, Response};
use egui::widgets::Widget;

// this function reads the boards directory and returns a Vec in RAM
// the boards directory is structured as:
// boards/
// -- manufacturer/
// -- -- board/
// -- -- -- <name>.toml
// -- -- -- <name>.png
pub fn get_boards(boards_dir: &Path) -> Vec<Board> {
    let mut r = Vec::new();
    if let Ok(entries) = fs::read_dir(boards_dir) {
        for entry in entries {
            let entry = entry.expect("error with entry");
            // if the entry is a directory, recursively go get the files
            if entry.file_type().expect("error parsing file type").is_dir() {
                // don't recurse into the examples
                if entry.path().ends_with("examples") {
                    continue;
                }
                // println!("recursing.. {:?}", entry.path());
                r.append(&mut get_boards(&entry.path()));
            // otherwise, if the entry is a file ending in "toml" try to parse it
            // as a board file
            } else if entry.path().extension().unwrap() == "toml" {
                r.push(Board::load_from_toml(&entry.path()));
            }
        }
    }
    return r;
}

// These are the various standard development board specifications
#[derive(Deserialize, Debug, Clone)]
pub enum BoardStandards {
    Feather,
    Arduino,
    RaspberryPi,
    ThingPlus,
    MicroMod,
}

use std::fmt;
impl fmt::Display for BoardStandards {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BoardStandards::Feather => write!(f, "Feather"),
            BoardStandards::Arduino => write!(f, "Arduino"),
            BoardStandards::RaspberryPi => write!(f, "RaspberryPi"),
            BoardStandards::ThingPlus => write!(f, "ThingPlus"),
            BoardStandards::MicroMod => write!(f, "MicroMod"),
        }
    }
}

// The board struct defines a board type
#[derive(Deserialize, Clone)]
pub struct Board {
    name: String,
    manufacturer: String,
    standard: Option<BoardStandards>,
    #[serde(skip)]
    examples: Vec<PathBuf>,
    #[serde(skip)]
    pic: Option<egui::ColorImage>,
    cpu: Option<String>,
    ram: Option<isize>,   // in kb
    flash: Option<isize>,
}

impl Board {

    fn load_from_toml(path: &Path) -> Self {
        
        let toml_str = fs::read_to_string(path).unwrap();
        let mut b: Board = toml::from_str(&toml_str).unwrap();

        // See if there is an image
        if let Ok(pic_path) = path.with_extension("png").canonicalize() {
            // println!("picture at {:?}", pic_path);
            let image = image::io::Reader::open(pic_path).unwrap().decode().unwrap();
            let size = [image.width() as _, image.height() as _];
            let image_buffer = image.to_rgba8();
            let pixels = image_buffer.as_flat_samples();
            let color_image = egui::ColorImage::from_rgba_unmultiplied(
                size,
                pixels.as_slice(),
            );
            b.pic = Some(color_image);
        }  

        // See if there are any examples
        if let Ok(examples_path) = path.parent().unwrap().join("examples").canonicalize() {
            // b.examples = ...;
            println!("found examples dir for board: {}", b.name);
            for (i, e) in examples_path.read_dir().unwrap().enumerate() {
                let example_path = e.unwrap().path();
                println!("  {}: {:?}", i, example_path.clone().file_name());
                b.examples.push(example_path);
            }
        }

        return b;
    }

    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }
}

// I might want to use this idea in the future:

// pub struct BoardSelectorWidget;

// impl BoardSelectorWidget {
//     pub fn new() -> BoardSelectorWidget {
//         return Self;
//     }
// }

// impl Widget for BoardSelectorWidget {
    
//     fn ui(self, ui: &mut Ui) -> Response {
//         let response = ui.allocate_response(egui::vec2(100.0, 200.0), egui::Sense::click());
//         ui.painter().rect_stroke(response.rect, 0.0, (1.0, egui::Color32::WHITE));
//         return response;
//     }
// }

impl Widget for Board {
    // How to display a board as a widget
    fn ui(self, ui: &mut Ui) -> Response {

        egui::Frame::none().show(ui, |ui| {

        });

        let response: egui::Response;
        if let Some(color_image) = self.pic {
            // Use a frame to display multiple widgets within our widget,
            // with an inner margin
            response = egui::Frame::none()
            .inner_margin(egui::Margin::same(10.0))
            .outer_margin(egui::Margin::same(3.0))
            .show(ui, |ui| {
                // center all text
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    let label = egui::RichText::new(self.name).underline();
                    ui.label(label);
                    let retained_image = RetainedImage::from_color_image(
                        "pic",
                        color_image,
                    );
                    retained_image.show_max_size(ui, egui::vec2(150.0, 150.0));
                });
                ui.horizontal(|ui| {
                    ui.label("Manufacturer: ");
                    ui.label(self.manufacturer);
                // TODO -- make the manufacturer logos an app-wide resource
                    // let p = Path::new("./assets/images/Adafruit_logo_small.png");
                    // let image = image::io::Reader::open(p).unwrap().decode().unwrap();
                    // let size = [image.width() as _, image.height() as _];
                    // let image_buffer = image.to_rgba8();
                    // let pixels = image_buffer.as_flat_samples();
                    // let color_image = egui::ColorImage::from_rgba_unmultiplied(
                    //     size,
                    //     pixels.as_slice(),
                    // );
                    // let ri = egui_extras::RetainedImage::from_color_image("logo", color_image);
                    // let image = egui::widgets::Image::new(
                    //     ri.texture_id(ui.ctx()),
                    //     egui::Vec2::new(47.0, 16.0)
                    // ).tint(egui::Color32::GREEN);   // TODO: replace with a val from current colorscheme
                    // ui.add(image);
                });
                ui.horizontal(|ui| {
                    ui.label("Ecosystem: ");
                    if let Some(standard) = self.standard {
                        ui.label(standard.to_string());
                    } else {
                        ui.label("none");
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("CPU: ");
                    if let Some(cpu) = self.cpu {
                        ui.label(cpu);
                    } else {
                        ui.label("unknown");
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("RAM Amount (in kb): ");
                    if let Some(ram) = self.ram {
                        ui.label(ram.to_string());
                    } else {
                        ui.label("unknown");
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Flash Amount (in kb): ");
                    if let Some(flash) = self.flash {
                        ui.label(flash.to_string());
                    } else {
                        ui.label("unknown");
                    }
                });
                // Show the examples
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    let label = egui::RichText::new("Examples").underline();
                    ui.label(label);
                });
                for e in self.examples {
                    ui.horizontal(|ui| {
                        if ui.link(e.file_name().unwrap().to_str().unwrap()).clicked() {
                            println!("TODO - open the example!")
                        };
                    });
                }
            }).response.interact(egui::Sense::click());

            if ui.rect_contains_pointer(response.rect) {
                // draw a bounding box
                ui.painter().rect_stroke(response.rect, 0.0, (1.0, egui::Color32::WHITE));
            }
            
            //another way of doing it; clicking works but scaling is off
            // let th = ui.ctx().load_texture(
            //     "pic",
            //     color_image,
            //     Default::default(),
            // );
            // let i = egui::Image::new(&th, egui::vec2(128.0, 128.0)).sense(egui::Sense::click());
            // response = ui.add(i);
        } else {
            response = ui.allocate_response(egui::vec2(128.0, 128.0), egui::Sense::click());
        }
        return response;
    }

}

