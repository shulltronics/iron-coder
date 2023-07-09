//! This module provides functionality for development boards

use log::{info, warn, debug};

use std::path::{Path, PathBuf};
use std::fs;
use std::vec::Vec;
use std::fmt;
use std::cmp;

use serde::{Serialize, Deserialize};

// use ra_ap_rust_analyzer::cli::load_cargo::load_workspace_at;
use ra_ap_ide;

pub mod pinout;
pub mod display;

/// This function recursively reads the boards directory and returns a vector of boards.
/// This includes searching for template directories, examples, and local BSPs for each
/// board.
pub fn get_boards(boards_dir: &Path) -> Vec<Board> {
    let mut r = Vec::new();
    if let Ok(entries) = fs::read_dir(boards_dir) {
        for entry in entries {
            let entry = entry.expect("error with entry");
            if entry.file_type().expect("error parsing file type").is_dir() {
                // if the entry is a directory, recursively go get the files
                // don't recurse into the examples
                if entry.path().ends_with("examples") {
                    continue;
                }
                r.append(&mut get_boards(&entry.path()));
            } else if entry.path().extension().unwrap_or_default() == "toml" {
                // otherwise, if the entry is a file ending in "toml" try to parse it
                // as a board file. unwrap_or_default works well here as the default 
                // ("") for &str will never match "toml"
                match Board::load_from_toml(&entry.path()) {
                    Ok(mut board) => {
                        let parent = entry.path().parent().unwrap().canonicalize().unwrap();
                        // look for a template directory
                        let template_dir = parent.join("template");
                        if let Ok(true) = template_dir.try_exists() {
                            debug!("found template dir for board <{}> at {:?}", board.name.clone(), entry.path().parent().unwrap().canonicalize().unwrap().join("template"));
                            board.template_dir = Some(template_dir);
                        } else {
                            debug!("no template directory found for board <{}>", board.name.clone());
                        }
                        // look for a local BSP
                        let bsp_dir = parent.join("bsp");
                        if let Ok(true) = bsp_dir.try_exists() {
                            info!("found local bsp crate for board {}", board.name.clone());
                            board.bsp_dir = Some(bsp_dir.clone());
                            let bsp_string = fs::read_to_string(bsp_dir.join("src/lib.rs")).unwrap();
                            let (analysis, fid) = ra_ap_ide::Analysis::from_single_file(bsp_string);
                            board.ra_values = analysis.file_structure(fid).unwrap();
                            // info!("syntax tree is: \n{:?}", analysis.file_structure(fid));
                            // for s in analysis.file_structure(fid).unwrap() {
                            //     println!("{:?}: {:?}: {:?}", s.label, s.kind, s.parent);
                            //     // let pos = ra_ap_ide::FilePosition { file_id: fid, offset: s.navigation_range.end() };
                            //     // println!("  {:?}", analysis.signature_help(pos).unwrap());
                            // }
                        } else {
                            debug!("no bsp directory found for board <{}>", board.name.clone());
                        }
                        r.push(board);
                    },
                    Err(e) => {
                        warn!("error loading board from {}: {:?}", entry.path().display().to_string(), e);
                    },
                }
            }
        }
    }
    return r;
}

/// These are the various standard development board form factors
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum BoardStandards {
    Feather,
    Arduino,
    RaspberryPi,
    ThingPlus,
    MicroMod,
}

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

/// The board struct defines a board type
#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct Board {
    /// The name of the board
    name: String,
    /// The board manufacturer
    manufacturer: String,
    /// Whether or not the board has a processor that can run code
    is_main_board: bool,
    /// A possible form factor that the board adheres to
    standard: Option<BoardStandards>,
    cpu: Option<String>,
    ram: Option<isize>,
    flash: Option<isize>,
    /// A list of the interfaces available on the board
    interfaces: Vec<pinout::Interface>,
    #[serde(skip)]
    pub ra_values: Vec<ra_ap_ide::StructureNode>,
    #[serde(skip)]
    examples: Vec<PathBuf>,
    #[serde(skip)]
    template_dir: Option<PathBuf>,
    #[serde(skip)]
    bsp_dir: Option<PathBuf>,
    #[serde(skip)]
    pic: Option<egui::ColorImage>,
    required_crates: Option<Vec<String>>,
    related_crates: Option<Vec<String>>,
}

impl Default for Board {
    fn default() -> Self {
        let mut i = Vec::new();
        i.push(pinout::Interface::I2C(pinout::InterfaceDirection::Controller));
        i.push(pinout::Interface::I2C(pinout::InterfaceDirection::Peripheral));
        i.push(pinout::Interface::ADC);
        Self {
            name: "".to_string(),
            manufacturer: "".to_string(),
            is_main_board: false,
            standard: None,
            cpu: None,
            ram: None,
            flash: None,
            interfaces: i,
            ra_values: Vec::new(),
            examples: Vec::new(),
            template_dir: None,
            bsp_dir: None,
            pic: None,
            required_crates: None,
            related_crates: None,
        }
    }
}
// Define some thin wrappers around Board so we can display a Board with the
//   Widget trait in multiple ways
//   see https://doc.rust-lang.org/book/ch19-03-advanced-traits.html#using-the-newtype-pattern-to-implement-external-traits-on-external-types
//   for the Newtype pattern
pub struct BoardSelectorWidget(pub Board);
pub struct BoardMiniWidget(pub Board);

impl fmt::Debug for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Board {}\n", self.name)?;
        write!(f, "  is main board? {}\n", self.is_main_board)?;
        write!(f, "  num examples: {}\n", self.examples.len())?;
        write!(f, "  num required crates: {}\n", self.required_crates.clone().unwrap_or_default().len())?;
        write!(f, "  num related crates: {}\n", self.related_crates.clone().unwrap_or_default().len())?;
        write!(f, "  has pic: {}\n", self.pic.is_some())?;
        write!(f, "  has template: {}", self.template_dir.is_some())?;
        write!(f, "  has local bsp: {}", self.bsp_dir.is_some())?;
        Ok(())
    }
}

// Boards should be uniquely identified by their name, and thus comparable.
impl cmp::PartialEq for Board {
    // Boards are equal if their names are equal
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

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
        if let Ok(pic_path) = path.with_extension("png").canonicalize() {
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

    pub fn get_interfaces(&self) -> Vec<pinout::Interface> {
        self.interfaces.clone()
    }

    pub fn required_crates(&self) -> Option<Vec<String>> {
        self.required_crates.clone()
    }

    pub fn related_crates(&self) -> Option<Vec<String>> {
        self.related_crates.clone()
    }

    pub fn is_main_board(&self) -> bool {
        self.is_main_board
    }

    pub fn get_template_dir(&self) -> Option<PathBuf> {
        return self.template_dir.clone();
    }
}
