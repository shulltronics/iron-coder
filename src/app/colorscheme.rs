//! Title: Iron Coder App Module - ColorScheme
//! Description: This module defines the ColorScheme struct and some built-in color schemes for the app.

use egui::Color32;
use serde::{Serialize, Deserialize};

use std::borrow::Cow;

#[derive(PartialEq, Clone, Serialize, Deserialize)]
pub struct ColorScheme {
    // pub name: &'static str,
    pub name: Cow<'static, str>,
    is_dark: bool,
    colors: [Color32; 4],
}

impl Default for ColorScheme {
    fn default() -> Self {
        SOLARIZED_DARK
    }
}

pub fn set_colorscheme(ctx: &egui::Context, cs: ColorScheme) {
    // get current style
    let mut style = (*ctx.style()).clone();

    style.visuals.dark_mode = cs.is_dark;

    // TODO style.dark_mode = false;
    // TODO style.widgets.... all widget styling

    // Background colors
    style.visuals.extreme_bg_color = cs.colors[0];
    style.visuals.faint_bg_color = cs.colors[1];
    style.visuals.code_bg_color = cs.colors[1];
    style.visuals.panel_fill = cs.colors[1];
    style.visuals.window_fill = cs.colors[1];

    // Foreground colors
    // TODO style.selection = ;
    style.visuals.hyperlink_color = cs.colors[3];
    style.visuals.window_stroke.color = cs.colors[2];
    style.visuals.warn_fg_color = cs.colors[3];
    style.visuals.error_fg_color = cs.colors[3];

    ctx.set_style(style);
}

// TODO -- make these serializable in a toml file for addition of new ones
//         without re-complilation (but maybe there also are some built-in ones)
pub const SOLARIZED_DARK: ColorScheme = ColorScheme {
    name: Cow::Borrowed("Solarized Dark"),
    is_dark: true,
    colors: [
        Color32::from_rgb(  0,  43,  54),   // Base 03 (background)
        Color32::from_rgb(  7,  54,  66),   // Base 02 (background highlights)
        Color32::from_rgb( 88, 110, 117),   // Base 01 (secondary text)
        Color32::from_rgb(131, 148, 150),   // Base 0 (body text)
    ],
};

pub const SOLARIZED_LIGHT: ColorScheme = ColorScheme {
    name: Cow::Borrowed("Solarized Light"),
    is_dark: false,
    colors: [
        Color32::from_rgb(253, 246, 227),   // Base 3 (background)
        Color32::from_rgb(238, 232, 213),   // Base 2 (background highlights)
        Color32::from_rgb(147, 161, 161),   // Base 1 (secondary text)
        Color32::from_rgb(101, 123, 131),   // Base 00 (body text)
    ],
};

pub const INDUSTRIAL_DARK: ColorScheme = ColorScheme {
    name: Cow::Borrowed("Industrial Dark"),
    is_dark: true,
    colors: [
        Color32::from_rgb(31,   31,  31),   // Base 3 (background)
        Color32::from_rgb(42,   42,  42),   // Base 2 (background highlights)
        Color32::from_rgb(204, 204, 204),   // Base 1 (secondary text)
        Color32::from_rgb(248,  81,  73),   // Base 00 (body text)
    ]
};

pub const SYSTEM_COLORSCHEMES: [ColorScheme; 3] = [
    SOLARIZED_DARK,
    SOLARIZED_LIGHT,
    INDUSTRIAL_DARK,
];