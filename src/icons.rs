use std::path::Path;
use std::collections::HashMap;

use image;
use egui::ColorImage;
use egui_extras::RetainedImage;

/// icons module
/// defines an IconSet type, which is a mapping of static str
///   to RetainedImages.
/// defines functionality to load icons from the filesystem,
///   and exposes them to the app via a const

pub type IconSet = HashMap<&'static str, RetainedImage>;
pub const ICON_DIR: &'static str = "assets/icons/pack/white/";

// This function returns a mapping of icon names to RetainedImages 
pub fn load_icons(icon_path: &Path) -> HashMap<&'static str, RetainedImage> {

    let mut icon_map = HashMap::new();

    let icon_names_and_files: [(&str, &str); 12] = [
        ("settings_icon", "gear.png"),
        ("boards_icon", "chip.png"),
        ("about_icon", "005b_13.gif"),
        ("folder_icon", "005b_43.gif"),
        ("save_icon", "005b_23.gif"),
        ("build_icon", "005b_35.gif"),
        ("load_icon", "005b_56.gif"),
        ("menu_icon", "005b_44.gif"),
        ("quit_icon", "005b_75.gif"),
        ("folder_closed_icon", "005b_49.gif"),
        ("folder_open_icon", "005b_50.gif"),
        ("file_icon", "005b_65.gif"),
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