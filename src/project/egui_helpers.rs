//! Title: Iron Coder Project Module - Egui Helpers
//! Description: This module contains helper functions for drawing egui elements in the project module.

use std::sync::Arc;

use crate::project::Project;
use crate::app::icons::{
    IconSet,
};

use egui::{
    Align,
    Layout,
    Response,
    Ui,
    Context,
    RichText,
};

use egui::widgets::{
    Label,
    ImageButton,
};

impl Project {
    // this function will draw the project name on the left of the Ui, and an "edit" icon on the right.
    pub fn label_with_action(&mut self, ctx: &Context, ui: &mut Ui) -> Response {
        // Grab the icons for a moment from the shared data
        let icons_ref: Arc<IconSet> = ctx.data_mut(|data| {
            data.get_temp("icons".into()).expect("couldn't load icons!")
        });
        let icons = icons_ref.clone();
        ui.with_layout(Layout::top_down(Align::Center), |ui| {
            let color = ui.style().visuals.window_stroke.color;
            let mut height: f32 = 0.0;
            // prepare the project name label
            let text = RichText::new(self.borrow_name().clone()).underline().italics();
            let project_label = Label::new(text);
            // create a column for the text and a column for the buttons
            ui.columns(2, |columns| {
                let resp = columns[0].add(project_label).on_hover_text(self.get_location());
                // capture the height of drawn text
                height = (resp.rect.max - resp.rect.min).y;
                let button = ImageButton::new(
                    icons.get("edit_icon").unwrap().clone(),
                ).frame(false).tint(color);
                columns[1].with_layout(Layout::top_down(Align::RIGHT), |ui| {
                    // slightly confusing, but we leave the semi-colons off here so that
                    // the inner response contains the response from this Ui addition 
                    ui.add(button).on_hover_text("edit project")
                })
            }).inner    //\\__ return the response from the button
        }).inner          //
    }
}