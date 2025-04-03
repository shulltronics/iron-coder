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
use std::io::{BufWriter, Cursor};
use std::path::Path;
use std::fs;
use std::vec::Vec;

use std::borrow::Borrow;

use image;
use base64::{Engine as _, engine::general_purpose};
use egui::{
    ColorImage,
    Pos2,
    Rect,
    Vec2,
};
use image::codecs::png::CompressionType;
use image::codecs::png::FilterType::Adaptive;

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

    /// Create an SVG file from a PNG file, return SVG contents as a String
    pub fn from_png(input_path: &Path) -> Result<String, Error> {
        // Load the PNG image
        let img = match image::open(input_path){
            Ok(image) => image,
            Err(e) => {return Err(Error::ImageError(e))}
        };

        // Convert the image to a byte vector
        let mut img_bytes = Vec::new();
        let cursor = Cursor::new(&mut img_bytes);
        let writer = BufWriter::new(cursor);
        let encoder = image::codecs::png::PngEncoder::new_with_quality(writer, CompressionType::Fast, Adaptive);
        match img.write_with_encoder(encoder){
            Ok(_) => {}
            Err(e) => {return Err(Error::ImageError(e))}
        }

        // Base64 encode the PNG bytes
        let encoded_img = general_purpose::STANDARD_NO_PAD.encode(&img_bytes);

        // Set the width and height, so it can be displayed in the editor
        let mut image_width = img.width() as f64;
        let mut image_height = img.height() as f64;
        while image_width > 64.0 || image_height > 50.0 {
            image_width = image_width / 2.0;
            image_height = image_height / 2.0;
        }

        // Create the SVG content with the base64 encoded PNG image
        let svg_content = format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">
    <g
     id="g1">
        <image
            x="0"
            y="0"
            id="image1"
            width="{width}"
            height="{height}"
            href="data:image/png;base64,{encoded_img}" />
    </g>
</svg>"#,
            width = image_width,
            height = image_height,
            encoded_img = encoded_img
        );

        Ok(svg_content)
    }

    /// Parse an Iron Coder SVG Board image from a string
    pub fn from_path(path: &Path) -> Result<SvgBoardInfo, Error>{
        let svg_string = match fs::read_to_string(path) {
            Ok(string) => string,
            Err(e) => return Err(Error::FsError(e)),
        };

        Self::from_string(svg_string)
    }

    /// Parse an Iron Coder SVG Board image from the filesystem.
    pub fn from_string(svg_string: String) -> Result<SvgBoardInfo, Error> {

        let mut svg_board_info = SvgBoardInfo::default();
        
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
                    } else {
                        return Err(Error::ImageNotPNG);
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
    ImageError(image::error::ImageError),
    ImageDecodeError,
    ArcError,
    NoImage,
    ImageNotPNG,
    OtherError,
}