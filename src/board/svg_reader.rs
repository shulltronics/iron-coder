//! A module to help parse and SVG file with an embedded image, and various
//! SVG paths that should be converted into egui elements.
//!
//! A few things are assumed about the SVG files:
//!   * The units should be in mm
//!   * All transforms have been removed and the units of all elements are absolute
//!   * All paths that should be displayed in Iron Coder have element id's that are also in the board manifest pinouts section.

use usvg::{
    Options,
    TreeParsing,
    Tree,
    NodeKind,
    ImageKind,
};
use std::io::Cursor;
use std::path::Path;
use std::fs;
use std::vec::Vec;

use std::borrow::Borrow;

use image;
use egui::{
    ColorImage,
    Pos2,
    Rect,
    Vec2,
};

/// A struct that holds the decoded SVG for use in egui.
#[derive(Default, Clone)]
pub struct SvgBoardInfo {
    /// The SVG size (should be in mm)
    pub physical_size: Vec2,
    /// The egui ColorImage of the board. This can be any size in px.
    pub image: ColorImage,
    /// A vector of egui Rects that represent the pin locations on the Board
    pub pin_rects: Vec<(String, Rect)>,
}

impl SvgBoardInfo {

    /// Parse an Iron Coder SVG Board image from the filesystem.
    pub fn from_path(path: &Path) -> Result<SvgBoardInfo, Error> {

        let mut svg_board_info = SvgBoardInfo::default();

        let svg_string = match fs::read_to_string(path) {
            Ok(string) => string,
            Err(e) => return Err(Error::FsError(e)),
        };
        
        let options = Options::default();
        let tree = match Tree::from_str(&svg_string.as_str(), &options) {
            Ok(t) => t,
            Err(_e) => return Err(Error::OtherError),
        };
    
        // At this point we have a valid SVG tree to work with
        
        svg_board_info.physical_size = Vec2 {
            x: tree.view_box.rect.width(),
            y: tree.view_box.rect.height(),
        };
    
        // iterate through the svg looking for elements
        let mut board_image: Option<ColorImage> = None;
        for node in tree.root.descendants() {
            // first, look for the image
            match node.borrow().clone() {
                NodeKind::Image(img) => {
                    if let ImageKind::PNG(png_bytes) = img.kind.clone() {
                        //let size = [img.view_box.rect.width().round() as usize, img.view_box.rect.height() as usize];
                        let borrowed_bytes: &Vec<u8> = png_bytes.borrow();
                        let png = match image::io::Reader::new(Cursor::new(borrowed_bytes)).with_guessed_format() {
                            Ok(ok) => ok,
                            Err(_e) => return Err(Error::ImageDecodeError),
                        };
                        let image = png.decode().unwrap();
                        // get the image size from the PNG itself
                        let size = [image.width() as usize, image.height() as usize];
                        let image_bytes = image.to_rgba8();
                        let color_image = ColorImage::from_rgba_unmultiplied(
                            size,
                            &image_bytes,
                        );
                        board_image = Some(color_image);
                    }
                },
                NodeKind::Path(path) => {
                    let id = path.id;
                    let bounds = path.data.bounds(); 
                    let min = Pos2 {
                        x: bounds.left(),
                        y: bounds.top(),
                    };
                    let max = Pos2 {
                        x: bounds.right(),
                        y: bounds.bottom(),
                    };
                    let rect = Rect::from_min_max(min, max);
                    svg_board_info.pin_rects.push((String::from(id), rect));
                },
                _ => {},
            }
        }
    
        if let Some(board_image) = board_image {
            svg_board_info.image = board_image;
        } else {
            return Err(Error::NoImage);
        }
    
        return Ok(svg_board_info);
    }

}

#[derive(Debug)]
pub enum Error {
    FsError(std::io::Error),
    ImageDecodeError,
    ArcError,
    NoImage,
    OtherError,
}