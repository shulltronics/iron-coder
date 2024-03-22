//! Title: Iron Coder App Module - Icons
//! Description: This module defines the IconSet type, which is a mapping of static str
//!   to RetainedImages. It also defines functionality to load icons from the filesystem,

use log::error;

use std::path::Path;
use std::collections::HashMap;

use image;
use egui::{Vec2, Image};

pub type IconSet<'a> = HashMap<&'static str, Image<'a>>;
pub const ICON_DIR: &'static str = "file://assets/icons/pack/white/";
pub const SMALL_ICON_SIZE: Vec2 = Vec2::new(8.0, 8.0);
pub const DEFAULT_ICON_SIZE: Vec2 = Vec2::new(12.0, 12.0);

// This function returns a mapping of icon names to RetainedImages 
pub fn load_icons(icon_path: &Path) -> HashMap<&'static str, Image> {

    let mut icon_map = HashMap::new();

    let icon_names_and_files: [(&str, &str); 16] = [
        ("settings_icon", "gear.png"),
        ("boards_icon", "chip.png"),
        ("about_icon", "005b_13.gif"),
        ("trash_icon", "005b_15.gif"),
        ("folder_icon", "005b_43.gif"),
        ("save_icon", "005b_23.gif"),
        ("build_icon", "005b_35.gif"),
        ("load_icon", "005b_56.gif"),
        ("menu_icon", "005b_44.gif"),
        ("quit_icon", "005b_75.gif"),
        ("folder_closed_icon", "005b_49.gif"),
        ("folder_open_icon", "005b_50.gif"),
        ("file_icon", "005b_65.gif"),
        ("edit_icon", "005b_19.gif"),
        ("plus_icon", "005b_39.gif"),
        ("right_arrow_icon", "005b_53.gif"),
    ];
    for (icon_name, icon_file) in icon_names_and_files.into_iter() {
        let p = icon_path.join(icon_file).as_path().to_str().unwrap().to_string();
        // attempt to open the icon image file
        let image = Image::new(p).fit_to_exact_size(DEFAULT_ICON_SIZE);
        icon_map.insert(icon_name, image);
    }
    return icon_map;
}