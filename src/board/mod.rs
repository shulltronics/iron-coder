use log::{warn, debug};

use std::path::{Path, PathBuf};
use std::fs;
use std::vec::Vec;
use std::fmt;
use std::cmp;

use serde::{Serialize, Deserialize};

pub mod display;

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
                r.append(&mut get_boards(&entry.path()));
            // otherwise, if the entry is a file ending in "toml" try to parse it
            // as a board file. unwrap_or_default works well here as the default 
            // ("") for &str will never match "toml"
            } else if entry.path().extension().unwrap_or_default() == "toml" {
                if let Ok(mut board) = Board::load_from_toml(&entry.path()) {
                    let parent = entry.path().parent().unwrap().canonicalize().unwrap();
                    let template_dir = parent.join("template");
                    if let Ok(true) = template_dir.try_exists() {
                        debug!("found template dir for board <{}> at {:?}", board.name.clone(), entry.path().parent().unwrap().canonicalize().unwrap().join("template"));
                        board.template_dir = Some(template_dir);
                    } else {
                        debug!("not template directory found for board <{}>", board.name.clone());
                    }
                    r.push(board);
                } else {
                    warn!("error loading board from {}", entry.path().display().to_string());
                }
            }
        }
    }
    return r;
}

// These are the various standard development board specifications
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

// The board struct defines a board type
#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct Board {
    name: String,
    manufacturer: String,
    is_main_board: bool,            // in Iron Coder, a project can only have one "main board" -- the one that the firmware targets
    standard: Option<BoardStandards>,
    cpu: Option<String>,
    ram: Option<isize>,
    flash: Option<isize>,
    #[serde(skip)]                  //
    examples: Vec<PathBuf>,         //\__ all of these fields are populated
    #[serde(skip)]                  ///   via file hierarchy, hence no serde
    pic: Option<egui::ColorImage>,  //
    required_crates: Option<Vec<String>>,
    related_crates: Option<Vec<String>>,
    #[serde(skip)]                  // we'll populate this dynamically when loading boards
    template_dir: Option<PathBuf>,
}

// A board shell template
impl Default for Board {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            manufacturer: "".to_string(),
            is_main_board: false,
            standard: None,
            cpu: None,
            ram: None,
            flash: None,
            examples: Vec::new(),
            pic: None,
            required_crates: None,
            related_crates: None,
            template_dir: None,
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
