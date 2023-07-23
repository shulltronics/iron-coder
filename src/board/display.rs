//! This module contains code related to displaying Boards and related types in egui.

use log::{info, debug};
use crate::board::Board;
use egui::{
    Color32,
    Ui,
    Response,
    FontFamily,
    FontId,
};
use egui::text::{
    TextFormat,
    LayoutJob,
};
use egui::widgets::Widget;
use egui_extras::RetainedImage;

/// Construct a LayoutJob with a bold heading, followed by a colon,
/// followed by some content, all with custom colors.
fn make_field_widget_text(heading: &str,
                          hcolor: Color32,
                          content: &str,
                          ccolor: Color32) -> LayoutJob {
    let mut job = LayoutJob::default();
    job.append(
        heading,
        0.0,
        TextFormat {
            font_id: FontId::new(12.0, FontFamily::Name("MonospaceBold".into())),
            color: hcolor,
            ..Default::default()
        },
    );
    job.append(
        content,
        0.0,
        TextFormat {
            font_id: FontId::new(12.0, FontFamily::Monospace),
            color: ccolor,
            ..Default::default()
        },
    );
    return job;
}

/// Normal view for the board widget
impl Widget for Board {
    // How to display a board as a widget
    fn ui(self, ui: &mut Ui) -> Response {
        let response: egui::Response;
        if let Some(color_image) = self.pic {
            // Use a frame to display multiple widgets within our widget,
            // with an inner margin
            response = egui::Frame::none()
            .inner_margin(egui::Margin::same(10.0))
            .show(ui, |ui| {
                // center all text
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    // let label = egui::RichText::new(self.name).strong();
                    ui.label(make_field_widget_text(
                        "Board: ",
                        ui.style().visuals.warn_fg_color,
                        self.name.as_str(),
                        ui.style().visuals.window_stroke.color,
                    ));
                    // ui.label(label);
                    let retained_image = RetainedImage::from_color_image(
                        "pic",
                        color_image,
                    );
                    retained_image.show_max_size(ui, egui::vec2(150.0, 150.0));
                });
                ui.horizontal(|ui| {
                    ui.label(make_field_widget_text(
                        "Manufacturer: ",
                        ui.style().visuals.warn_fg_color,
                        self.manufacturer.as_str(),
                        ui.style().visuals.window_stroke.color,
                    ));
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
                ui.separator();
                // Show the examples
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    let label = egui::RichText::new("Examples").underline();
                    ui.label(label);
                });
                for e in self.examples {
                    ui.horizontal(|ui| {
                        if ui.link(e.file_name().unwrap().to_str().unwrap()).clicked() {
                            info!("TODO - open the example!")
                        };
                    });
                }
                ui.separator();
                // show the interfaces
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    let label = egui::RichText::new("Pinout").underline();
                    ui.label(label);
                });
                ui.label(format!("{:?}", self.pinout));

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

/// Display the board for use in the Board selector window
pub struct BoardSelectorWidget(pub Board);
impl Widget for BoardSelectorWidget {
    fn ui(self, ui: &mut Ui) -> Response {
        let this_board = self.0;
        let response: egui::Response;
        if let Some(color_image) = this_board.clone().pic {
            // Use a frame to display multiple widgets within our widget,
            // with an inner margin
            response = egui::Frame::none()
            .inner_margin(egui::Margin::same(10.0))
            .show(ui, |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    // let label = egui::RichText::new(this_board.name).strong();
                    ui.label(make_field_widget_text(
                        "Board: ",
                        ui.style().visuals.warn_fg_color,
                        this_board.name.as_str(),
                        ui.style().visuals.window_stroke.color,
                    ));
                    // ui.label(label);
                    let retained_image = RetainedImage::from_color_image(
                        "pic",
                        color_image,
                    );
                    retained_image.show_max_size(ui, egui::vec2(150.0, 150.0));

                });
                ui.horizontal(|ui| {
                    ui.label(make_field_widget_text(
                        "Manufacturer: ",
                        ui.style().visuals.warn_fg_color,
                        this_board.manufacturer.as_str(),
                        ui.style().visuals.window_stroke.color,
                    ));
                });
                ui.horizontal(|ui| {
                    ui.label("Ecosystem: ");
                    if let Some(standard) = this_board.clone().standard {
                        ui.label(standard.to_string());
                    } else {
                        ui.label("none");
                    }
                });
            }).response.interact(egui::Sense::click());

            
            // draw a bounding box for main boards
            if this_board.clone().is_main_board() {
                ui.painter().rect_stroke(response.rect, 0.0, (1.0, egui::Color32::WHITE));
            }

        } else {
            response = ui.allocate_response(egui::vec2(128.0, 128.0), egui::Sense::click());
        }
        return response;
    }
}

/// Display to Board as an image with pin overlays that respond to hover events
pub struct BoardEditorWidget(pub Board);
impl Widget for BoardEditorWidget {

    fn ui(self, ui: &mut Ui) -> Response {

        let this_board = self.0;
        let response: egui::Response;
        let scale: f32 = 0.5;
        if let Some(color_image) = this_board.clone().pic {
            let retained_image = RetainedImage::from_color_image(
                "pic",
                color_image,
            );
            response = retained_image.show_scaled(ui, scale).interact(egui::Sense::click());
            // draw a bounding box
            ui.painter().rect_stroke(response.rect, 0.0, (1.0, egui::Color32::WHITE));
            // iterate through the pin_nodes of the board, and check if their rects (properly scaled and translated) contain the pointer.
            // if so, actually draw the stuff there.
            for mut pin_rect in this_board.pin_nodes {
                // scale the rects the same amount that the board image was scaled
                pin_rect.min.x *= scale;
                pin_rect.min.y *= scale;
                pin_rect.max.x *= scale;
                pin_rect.max.y *= scale;
                // translate the rects so they are in absolute coordinates
                pin_rect = pin_rect.translate(response.rect.left_top().to_vec2());
                let r = ui.allocate_rect(pin_rect, egui::Sense::hover());
                r.clone().on_hover_text("test");
                if r.hovered() { 
                    ui.painter().circle_filled(r.rect.center(), r.rect.height()/2.0, Color32::GREEN);
                }
            }
        } else {
            // if there is no image, show an empty box (TODO there probably should be some default image)
            response = ui.allocate_response(egui::vec2(128.0, 128.0), egui::Sense::click());
        }

        return response; 

    }

}


/// Display the Board as a "mini widget"
pub struct BoardMiniWidget(pub Board);
impl Widget for BoardMiniWidget {
    fn ui(self, ui: &mut Ui) -> Response {
        let this_board = self.0;
        let response: egui::Response;
        if let Some(color_image) = this_board.clone().pic {
            // Use a frame to display multiple widgets within our widget,
            // with an inner margin
            response = egui::Frame::none()
            .inner_margin(egui::Margin::same(5.0))
            .show(ui, |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    ui.label(this_board.clone().name);
                    let retained_image = RetainedImage::from_color_image(
                        "pic",
                        color_image,
                    );
                    retained_image.show_max_size(ui, egui::vec2(96.0, 96.0));
                });
            }).response.interact(egui::Sense::click());
            if this_board.clone().is_main_board() {
                // draw a bounding box
                ui.painter().rect_stroke(response.rect, 0.0, (1.0, egui::Color32::WHITE));
            }
        } else {
            debug!("could not find color_image when rendering BoardMiniWidget");
            response = ui.allocate_response(egui::vec2(128.0, 128.0), egui::Sense::click());
        }
        return response;
    }
}
