//! This module provides functionality for development boards

use log::{warn, info, debug};

use std::path::{Path, PathBuf};
use std::fs;
use std::vec::Vec;
use std::fmt;
use std::cmp;
use std::hash::{Hash, Hasher};

use serde::{Serialize, Deserialize};

use ra_ap_ide;

mod svg_reader;
use svg_reader::SvgBoardInfo;

pub mod display;

pub mod pinout;
use pinout::Pinout;

pub mod parsing;
mod test;

use parsing::BspParseInfo;

/// These are the various standard development board form factors
#[non_exhaustive]
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
            // _ => write!(f, "Unknown Dev Board Standard"),
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
    is_main_board: bool,
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
        write!(f, "  is main board? {}\n", self.is_main_board)?;
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
        self.is_main_board
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
