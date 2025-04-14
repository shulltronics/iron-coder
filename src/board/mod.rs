//! This module provides functionality for development boards

use log::{warn, info, debug};

use std::path::{Path, PathBuf};
use std::fs;
use std::vec::Vec;
use std::fmt;
use std::cmp;
use std::cmp::Ordering;
use std::fmt::Display;
use std::hash::{Hash, Hasher};
use egui::{Button, Color32, TextBuffer, Vec2};
use serde::{Serialize, Deserialize};

use ra_ap_ide;
use tracing_subscriber::fmt::writer::EitherWriter::B;

pub mod svg_reader;
use svg_reader::SvgBoardInfo;

pub mod display;

pub mod pinout;
use pinout::Pinout;

pub mod parsing;
mod test;

use parsing::BspParseInfo;
use crate::board::pinout::{InterfaceDirection, InterfaceType};

/// These are the various standard development board form factors
#[non_exhaustive]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub enum BoardStandards {
    #[default]
    None,
    Feather,
    Arduino,
    RaspberryPi,
    ThingPlus,
    MicroMod,
}

impl fmt::Display for BoardStandards {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BoardStandards::None => write!(f, "None"),
            BoardStandards::Feather => write!(f, "Feather"),
            BoardStandards::Arduino => write!(f, "Arduino"),
            BoardStandards::RaspberryPi => write!(f, "RaspberryPi"),
            BoardStandards::ThingPlus => write!(f, "ThingPlus"),
            BoardStandards::MicroMod => write!(f, "MicroMod"),
            // _ => write!(f, "Unknown Dev Board Standard"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub enum BoardType {
    #[default]
    Main,
    Peripheral,
    Discrete,
}

impl fmt::Display for BoardType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BoardType::Main => write!(f, "Main"),
            BoardType::Peripheral => write!(f, "Peripheral"),
            BoardType::Discrete => write!(f, "Discrete"),
        }
    }
}

/// The board struct defines a board type
#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct Board {
    /// The name of the board
    name: String,
    /// The board manufacturer
    manufacturer: String,
    /// Whether or not the board has a processor that can run code
    board_type: BoardType,
    /// A possible form factor that the board adheres to
    standard: Option<BoardStandards>,
    cpu: Option<String>,
    ram: Option<isize>,
    flash: Option<isize>,
    /// A list of the interfaces available on the board
    pub pinout: Pinout,
    /// A list of the Syntax Nodes of the BSP calculated by Rust Analyzer
    #[serde(skip)]
    pub ra_values: Vec<ra_ap_ide::StructureNode>,
    /// A list of examples
    #[serde(skip)]
    examples: Vec<PathBuf>,
    /// An local path of a project template
    #[serde(skip)]
    template_dir: Option<PathBuf>,
    /// The name of a BSP crate
    pub bsp: Option<String>,
    /// An optional path to a local BSP (if None, means the BSP should be on crates.io)
    #[serde(skip)]
    pub bsp_path: Option<PathBuf>,
    /// A syntax tree representation of the BSP
    #[serde(skip)]
    pub bsp_parse_info: Option<BspParseInfo>,
    /// Possible image loaded from an SVG file, along with size info and pin locations
    #[serde(skip)]
    pub svg_board_info: Option<SvgBoardInfo>,
    /// A list of required crates
    required_crates: Option<Vec<String>>,
    /// A list of related, optional crates
    related_crates: Option<Vec<String>>,
}

impl fmt::Debug for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Board {}\n", self.name)?;
        write!(f, "  is main board? {}\n", self.is_main_board())?;
        write!(f, "  num examples: {}\n", self.examples.len())?;
        write!(f, "  num required crates: {}\n", self.required_crates.clone().unwrap_or_default().len())?;
        write!(f, "  num related crates: {}\n", self.related_crates.clone().unwrap_or_default().len())?;
        write!(f, "  has svg info: {}\n", self.svg_board_info.is_some())?;
        write!(f, "  has template: {}\n", self.template_dir.is_some())?;
        write!(f, "  bsp crate name: {:?}\n", self.bsp)?;
        write!(f, "  has local bsp: {:?}\n", self.bsp_path)?;
        write!(f, "  has some syntax loaded: {:?}\n", self.bsp_parse_info.is_some())?;
        Ok(())
    }
}

/// Boards are uniquely identified by their name, and thus comparable.
impl cmp::PartialEq for Board {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}
impl cmp::Eq for Board {}

/// Boards can be sorted by name and type (main, peripheral, discrete)
impl PartialOrd<Self> for Board {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl cmp::Ord for Board{
    fn cmp(&self, other: &Self) -> Ordering {
        if self.is_main_board() && !other.is_main_board() {
            Ordering::Less
        } else if !self.is_main_board() && other.is_main_board() {
            Ordering::Greater
        } else if self.is_discrete() && !other.is_discrete() {
            Ordering::Greater
        } else if !self.is_discrete() && other.is_discrete() {
            Ordering::Less
        } else {
            self.name.cmp(&other.name)
        }
    }
}

/// Boards are uniquely identified by their name, and thus hashable.
impl Hash for Board {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

/// Basic implementation, including loading boards from the filesystem, and retrieving certain
/// information about them.
impl Board {

    /// Loads a board from its toml description
    fn load_from_toml(path: &Path) -> std::io::Result<Self> {
        
        let toml_str = fs::read_to_string(path)?;
        let mut b: Board = match toml::from_str(&toml_str) {
            Ok(b) => b,
            Err(_) => {
                return Err(std::io::Error::other("load from toml failed"));
            },
        };

        // See if there is an image
        if let Ok(pic_path) = path.with_extension("svg").canonicalize() {
            // BASED ON SVG WORK
            match SvgBoardInfo::from_path(&pic_path) {
                Ok(svg_board_info) => {
                    info!("successfully decoded SVG for board {}. Board has physical size: {:?}", b.get_name(), svg_board_info.physical_size);
                    b.svg_board_info = Some(svg_board_info);
                },
                Err(e) => {
                    warn!("error with svg parsing! {:?}", e);
                    return Err(std::io::Error::other("unable to parse board SVG file."));
                },
            };
        } else {
            warn!("no svg file for board {}", b.get_name());
            return Err(std::io::Error::other("no SVG file for board."));

        }

        // See if there are any examples
        if let Ok(examples_path) = path.parent().unwrap().join("examples").canonicalize() {
            for (_i, e) in examples_path.read_dir().unwrap().enumerate() {
                let example_path = e.unwrap().path();
                b.examples.push(example_path);
            }
        }

        return Ok(b);
    }

    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    pub fn get_pinout(&self) -> Pinout {
        self.pinout.clone()
    }

    pub fn required_crates(&self) -> Option<Vec<String>> {
        self.required_crates.clone()
    }

    pub fn related_crates(&self) -> Option<Vec<String>> {
        self.related_crates.clone()
    }

    pub fn is_main_board(&self) -> bool {
        match self.board_type {
            BoardType::Main => true,
            BoardType::Peripheral => false,
            BoardType::Discrete => false,
        }
    }

    pub fn is_discrete(&self) -> bool {
        match self.board_type {
            BoardType::Main => false,
            BoardType::Peripheral => false,
            BoardType::Discrete => true,
        }
    }

    pub fn get_template_dir(&self) -> Option<PathBuf> {
        return self.template_dir.clone();
    }

}

/// Iteratively gather the Boards from the filesystem.
pub fn get_boards(boards_dir: &Path) -> Vec<Board> {
    let mut r = Vec::new();
    if let Ok(manufacturers) = fs::read_dir(boards_dir) {
        // first tier of organization is by manufacturer
        for manufacturer in manufacturers {
            let manufacturer = manufacturer.expect("error with manufacturer directory");
            if manufacturer.file_type().expect("error parsing file type").is_file() {
                continue;
            }
            let boards = fs::read_dir(manufacturer.path()).expect("error iterating over files in manufacturer directory");
            for board in boards {
                let board = board.expect("error with Board directory");
                if board.file_type().expect("error parsing file type within board dir").is_file() {
                    continue;
                }
                let files = fs::read_dir(board.path()).expect("error iterating over files in board directory");
                for file in files {
                    let file = file.expect("error reading file within board directory");
                    if file.path().extension().unwrap_or_default() == "toml" {
                        match Board::load_from_toml(&file.path()) {
                            Ok(mut board) => {
                                let parent = file.path().parent().unwrap().canonicalize().unwrap();
                                // look for a template directory
                                let template_dir = parent.join("template");
                                if let Ok(true) = template_dir.try_exists() {
                                    debug!("found template dir for board <{}> at {:?}", board.name.clone(), file.path().parent().unwrap().canonicalize().unwrap().join("template"));
                                    board.template_dir = Some(template_dir);
                                } else {
                                    debug!("no template directory found for board <{}>", board.name.clone());
                                }
                                // look for a local BSP, and do things related to it if needed
                                let bsp_dir = parent.join("bsp");
                                if let Ok(true) = bsp_dir.try_exists() {
                                    info!("found local bsp crate for board {}", board.name.clone());
                                    board.bsp_path = Some(bsp_dir.clone());
                                    // let bsp_string = fs::read_to_string(bsp_dir.join("src/lib.rs")).unwrap();
                                    // let (analysis, fid) = ra_ap_ide::Analysis::from_single_file(bsp_string);
                                    // board.ra_values = analysis.file_structure(fid).unwrap();
                                    match board.load_bsp_info() {
                                        Ok(_) => (),
                                        Err(e) => warn!("error parsing BSP for board {}: {:?}", board.get_name(), e),
                                    };
                                } else {
                                    debug!("no bsp directory found for board <{}>", board.name.clone());
                                }
                                r.push(board);
                            },
                            Err(e) => {
                                warn!("error loading board from {}: {:?}", file.path().display().to_string(), e);
                            },
                        }
                    }
                }
            }
        }
    }
    return r;
}

/// A struct that holds the decoded toml information for new board generation
#[derive(Default, Clone)]
pub struct BoardTomlInfo {
    pub name : String,
    pub manufacturer : String,
    pub board_type : BoardType,
    pub standard : String,
    pub cpu : String,
    pub ram : i32,
    pub flash : i32,
    pub required_crates : Vec<String>,
    pub related_crates : Vec<String>,

    pub bsp : String,

    pub pinouts : Vec<PinoutTomlInfo>,
}

impl BoardTomlInfo {
    /// Convert fields to TOML string
    pub fn get_name(&self) -> String { String::from("name = \"") + self.name.as_str() + "\"\n" }

    pub fn get_manufacturer(&self) -> String { String::from("manufacturer = \"") + self.manufacturer.as_str() + "\"\n" }

    pub fn get_board_type(&self) -> String { String::from("board_type = \"") + self.board_type.to_string().as_str() + "\"\n" }

    pub fn get_standard(&self) -> String { String::from("standard = \"") + self.standard.as_str() + "\"\n" }

    pub fn get_cpu(&self) -> String { String::from("cpu = \"") + self.cpu.as_str() + "\"\n" }

    pub fn get_ram(&self) -> String { String::from("ram = ") + self.ram.to_string().as_str() + "\n" }

    pub fn get_flash(&self) -> String { String::from("flash = ") + self.flash.to_string().as_str() + "\n" }

    pub fn get_required_crates(&self) -> String {
        let mut required_string : String = String::from("required_crates = [");

        for req_crate in &self.required_crates {
            required_string =  required_string + String::from("\"").as_str() + req_crate.as_str() + "\", ";
        }

        if !self.required_crates.is_empty() {
            required_string.pop();
            required_string.pop();
        }

        required_string =  required_string + String::from("]\n").as_str();


        required_string
    }

    pub fn get_related_crates(&self) -> String {
        let mut related_string : String = String::from("related_crates = [");

        for rel_crate in &self.related_crates {
            related_string =  related_string + String::from("\"").as_str() + rel_crate.as_str() + "\", ";
        }

        if !self.related_crates.is_empty() {
            related_string.pop();
            related_string.pop();
        }

        related_string =  related_string + String::from("]\n\n").as_str();


        related_string
    }

    pub fn get_bsp(&self) -> String { String::from("bsp = \"iron-coder-") + self.name.replace(" ", "-").as_str() + "-bsp\"\n\n"}

    pub fn get_pinouts(&self) -> String {
        let mut pinout_str :String = String::from("");

        for pinout in &self.pinouts {
            pinout_str = pinout_str + String::from("[[pinout]]\n").as_str();
            pinout_str = pinout_str + pinout.get_pins().as_str();
            pinout_str = pinout_str + pinout.get_interface().as_str();
        }

        pinout_str
    }

    pub fn get_all_pin_names(& self) -> Vec<String> {
        let mut pin_names : Vec<String> = Vec::new();
        for pinout in &self.pinouts{
            for pin in &pinout.pins{
                pin_names.push(String::from(pin));
            }
        }
        pin_names
    }

    pub fn display_required_message(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, flag_id: &str){
        let field_required_id = egui::Id::new(flag_id);
        let mut display_name_required : bool = ctx.data_mut(|data| {
            data.get_temp_mut_or(field_required_id, false).clone()
        });

        if display_name_required {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Field is required.").color(Color32::RED));
            });
        }
    }

    pub fn update_general_form_UI(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Component Name:");
            egui::TextEdit::singleline(&mut self.name)
                .hint_text("Enter here").show(ui);
        });

        let duplicate_id = egui::Id::new("name_duplicated");
        let name_duplicated : bool = ctx.data_mut(|data| {
            data.get_temp_mut_or(duplicate_id, false).clone()
        });

        if name_duplicated {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Board already exists.").color(Color32::RED));
            });
        }

        self.display_required_message(ctx, ui, "name_required");

        ui.horizontal(|ui| {
            ui.label("Manufacturer:");
            egui::TextEdit::singleline(&mut self.manufacturer)
                .hint_text("Enter here").show(ui);
        });

        self.display_required_message(ctx, ui, "manufacturer_required");

        ui.horizontal(|ui | {
            ui.label("Select Board Type:");
            ui.selectable_value(&mut self.board_type, BoardType::Main, "Main");
            ui.selectable_value(&mut self.board_type, BoardType::Peripheral, "Peripheral");
            ui.selectable_value(&mut self.board_type, BoardType::Discrete, "Discrete");
        });

        ui.horizontal(|ui| {
            ui.label("Standard:");
            if self.board_type == BoardType::Discrete || self.board_type == BoardType::Peripheral{
                self.standard = BoardStandards::None.to_string();
                ui.selectable_value(&mut self.standard, BoardStandards::None.to_string(), "None");
            } else {
                ui.selectable_value(&mut self.standard, BoardStandards::None.to_string(), "None");
                ui.selectable_value(&mut self.standard, BoardStandards::Arduino.to_string(), "Arduino");
                ui.selectable_value(&mut self.standard, BoardStandards::RaspberryPi.to_string(), "RaspberryPi");
                ui.selectable_value(&mut self.standard, BoardStandards::Feather.to_string(), "Feather");
                ui.selectable_value(&mut self.standard, BoardStandards::ThingPlus.to_string(), "ThingPlus");
                ui.selectable_value(&mut self.standard, BoardStandards::MicroMod.to_string(), "MicroMod");
            }
        });

        self.display_required_message(ctx, ui, "standard_required");

        if self.board_type == BoardType::Main {
            ui.horizontal(|ui| {
                ui.label("CPU:");
                egui::TextEdit::singleline(&mut self.cpu)
                    .hint_text("Enter here").show(ui);
            });

            self.display_required_message(ctx, ui, "cpu_required");

            ui.horizontal(|ui| {
                ui.label("Flash (kb):");
                ui.add(egui::Slider::new(&mut self.flash, 0..=8000));
            });

            self.display_required_message(ctx, ui, "flash_required");

            ui.horizontal(|ui| {
                ui.label("RAM (kb):");
                ui.add(egui::Slider::new(&mut self.ram, 0..=4000));
            });

            self.display_required_message(ctx, ui, "ram_required");
        }

        ui.horizontal(|ui| {
            if self.required_crates.is_empty() {
                self.required_crates.push("".parse().unwrap());
            }
            ui.label("Required Crates:");

            ui.vertical(|ui| {
                let mut n : usize = 0;
                while n < self.required_crates.len() {
                    ui.horizontal(|ui| {
                        egui::TextEdit::singleline(&mut self.required_crates[n])
                            .hint_text("Enter here").show(ui);
                        if self.required_crates.len() != 1 {
                            if ui.button("Delete Crate").clicked() {
                                self.required_crates.remove(n);
                                if n != 0 {
                                    n = n - 1;
                                }
                            }
                        }
                    });
                    if n == self.required_crates.len() -1 && ui.button("Add Required Crate").clicked() {
                        self.required_crates.push("".parse().unwrap());
                        break;
                    }
                    n = n + 1;
                }
            });
        });

        let req_crates_required_id = egui::Id::new("req_crates_required");
        let mut display_crate_required : bool = ctx.data_mut(|data| {
            data.get_temp_mut_or(req_crates_required_id, false).clone()
        });

        if display_crate_required {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Field is required. Delete empty crates").color(Color32::RED));
            });
        }

        ui.horizontal(|ui| {
            if self.related_crates.is_empty() {
                self.related_crates.push("".parse().unwrap());
            }
            ui.label("Related Crates:");

            ui.vertical(|ui| {
                let mut n : usize = 0;
                while n < self.related_crates.len() {
                    ui.horizontal(|ui| {
                        egui::TextEdit::singleline(&mut self.related_crates[n])
                            .hint_text("Enter here").show(ui);
                        if self.related_crates.len() != 1 {
                            if ui.button("Delete Crate").clicked() {
                                self.related_crates.remove(n);
                                if n != 0 {
                                    n = n - 1;
                                }
                            }
                        }
                    });
                    if n == self.related_crates.len() -1 && ui.button("Add Related Crate").clicked() {
                        self.related_crates.push("".parse().unwrap());
                        break;
                    }
                    n = n + 1;
                }
            });
        });

        let rel_required_id = egui::Id::new("rel_crates_required");
        let mut display_rel_required : bool = ctx.data_mut(|data| {
            data.get_temp_mut_or(rel_required_id, false).clone()
        });

        if display_rel_required {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Field is required. Delete empty crates").color(Color32::RED));
            });
        }
    }

    pub fn update_pinout_form_UI(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        let pins_required_id = egui::Id::new("pins_required");
        let mut display_pins_required : bool = ctx.data_mut(|data| {
            data.get_temp_mut_or(pins_required_id, false).clone()
        });

        ui.horizontal(|ui| {
            if self.pinouts.is_empty() {
                self.pinouts.push(PinoutTomlInfo::default());
            }

            ui.vertical(|ui| {
                let mut n : usize = 0;
                while n < self.pinouts.len() {
                    ui.label(format!("PINOUT #{}", n + 1));
                    ui.horizontal(|ui| {
                        PinoutTomlInfo::update_form_UI(&mut self.pinouts[n], ctx, ui);
                        if self.pinouts.len() != 1 {
                            if ui.add(Button::new("Delete Pinout").min_size(Vec2::new(20.0, 50.0 + (25 * self.pinouts[n].pins.len()) as f32))).clicked() {
                                self.pinouts.remove(n);
                                if n != 0 {
                                    n = n - 1;
                                }
                            }
                        }
                    });
                    if display_pins_required && self.pinouts[n].pins.contains(&String::new()) {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Field is required. Replace or delete empty pins").color(Color32::RED));
                        });
                    }
                    if n == self.pinouts.len() -1 && ui.button("Add Pinout").clicked() {
                        self.pinouts.push(PinoutTomlInfo::default());
                        break;
                    }
                    n = n + 1;
                }
            });
        });
    }

    pub fn generate_toml_string(&self) -> String {
        if self.board_type != BoardType::Discrete {
            self.get_name() + self.get_manufacturer().as_str() + self.get_board_type().as_str()
                + self.get_standard().as_str() + self.get_cpu().as_str() + self.get_ram().as_str()
                + self.get_flash().as_str() + self.get_required_crates().as_str()
                + self.get_related_crates().as_str() + self.get_bsp().as_str()
                + self.get_pinouts().as_str()
        } else {
            self.get_name() + self.get_manufacturer().as_str() + self.get_board_type().as_str()
                + self.get_standard().as_str()
                + self.get_required_crates().as_str()
                + self.get_related_crates().as_str() + self.get_bsp().as_str()
                + self.get_pinouts().as_str()
        }
    }

    pub fn cleanup(&self, file_path: &Path) -> () {
        // TODO reb Delete files if necessary
    }
}

/// A struct that holds the decoded pinout information for creating board toml file
#[derive(Default, Clone)]
pub struct PinoutTomlInfo {
    pub pins : Vec<String>,
    pub iface_type : InterfaceType,
    pub direction : InterfaceDirection,
}

impl PinoutTomlInfo {
    /// Convert fields to TOML string
    pub fn get_pins(&self) -> String {
        let mut pins_str = String::from("pins = [");
        for pin in &self.pins{
            pins_str = pins_str + "\"" + pin.as_str() + "\", ";
        }

        if !self.pins.is_empty(){
            pins_str.pop();
            pins_str.pop();
        }

        pins_str = pins_str + "]\n";

        pins_str
    }

    pub fn get_interface(&self) -> String {
        let mut interface_str = String::from("interface = { iface_type = \"");

        interface_str = interface_str + self.iface_type.to_string().as_str() + "\", direction = \"";
        interface_str = interface_str + self.direction.to_string().as_str() + "\" }\n\n";

        interface_str
    }



    pub fn update_form_UI(&mut self, ctx: &egui::Context, ui: &mut egui::Ui){

        ui.horizontal(|ui| {
            if self.pins.is_empty() {
                self.pins.push("".parse().unwrap());
            }
            ui.label("Pin Names:");

            ui.vertical(|ui| {
                let mut n : usize = 0;
                while n < self.pins.len() {
                    ui.horizontal(|ui| {
                        egui::TextEdit::singleline(&mut self.pins[n])
                            .hint_text("Enter here").show(ui);
                        if self.pins.len() != 1 {
                            if ui.button("Delete Pin").clicked() {
                                self.pins.remove(n);
                                if n != 0 {
                                    n = n - 1;
                                }
                            }
                        }
                    });
                    if n == self.pins.len() -1 && ui.button("Add Pin").clicked() {
                        self.pins.push("".parse().unwrap());
                        break;
                    }
                    n = n + 1;
                }

                ui.horizontal(|ui | {
                    ui.label("Select Interface Type:");
                    ui.selectable_value(&mut self.iface_type, InterfaceType::NONE, "None");
                    ui.selectable_value(&mut self.iface_type, InterfaceType::GPIO, "GPIO");
                    ui.selectable_value(&mut self.iface_type, InterfaceType::ADC, "ADC");
                    ui.selectable_value(&mut self.iface_type, InterfaceType::PWM, "PWM");
                    ui.selectable_value(&mut self.iface_type, InterfaceType::UART, "UART");
                    ui.selectable_value(&mut self.iface_type, InterfaceType::I2C, "I2C");
                    ui.selectable_value(&mut self.iface_type, InterfaceType::SPI, "SPI");
                });

                ui.horizontal(|ui | {
                    ui.label("Select Direction:");
                    ui.selectable_value(&mut self.direction, InterfaceDirection::Unknown, "Unknown");
                    ui.selectable_value(&mut self.direction, InterfaceDirection::Input, "Input");
                    ui.selectable_value(&mut self.direction, InterfaceDirection::Output, "Output");
                    ui.selectable_value(&mut self.direction, InterfaceDirection::Bidirectional, "Bidirectional");
                });
            });
        });

    }
}
