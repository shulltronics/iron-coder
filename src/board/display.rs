use log::info;
use crate::board::{
    Board,
    BoardSelectorWidget,
};
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

/// module board::display contains code related to displaying Boards
/// and related types in egui

// This function will construct a LayoutJob with a bold heading
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

impl Widget for Board {
    // How to display a board as a widget
    fn ui(self, ui: &mut Ui) -> Response {
        let response: egui::Response;
        if let Some(color_image) = self.pic {
            // Use a frame to display multiple widgets within our widget,
            // with an inner margin
            response = egui::Frame::none()
            // .inner_margin(egui::Margin::same(10.0))
            // .outer_margin(egui::Margin::same(3.0))
            .show(ui, |ui| {
                egui::CollapsingHeader::new(self.name.as_str())
                .default_open(true)
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
                    // show the related crates
                    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                        let label = egui::RichText::new("Related Crates").underline();
                        ui.label(label);
                    });
                    if let Some(related_crates) = self.related_crates {
                        for rc in related_crates.iter() {
                            ui.horizontal(|ui| {
                                if ui.link(rc).clicked() {
                                    info!("TODO - deal with the related crate!")
                                };
                            });
                        }
                    }
                });
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

impl Widget for BoardSelectorWidget {
    fn ui(self, ui: &mut Ui) -> Response {
        let this_board = self.0;
        let response: egui::Response;
        if let Some(color_image) = this_board.pic {
            // Use a frame to display multiple widgets within our widget,
            // with an inner margin
            response = egui::Frame::none()
            // .inner_margin(egui::Margin::same(10.0))
            // .outer_margin(egui::Margin::same(3.0))
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
                    if let Some(standard) = this_board.standard {
                        ui.label(standard.to_string());
                    } else {
                        ui.label("none");
                    }
                });
            }).response.interact(egui::Sense::click());

            if ui.rect_contains_pointer(response.rect) {
                // draw a bounding box
                ui.painter().rect_stroke(response.rect, 0.0, (1.0, egui::Color32::WHITE));
            }

        } else {
            response = ui.allocate_response(egui::vec2(128.0, 128.0), egui::Sense::click());
        }
        return response;
    }
}