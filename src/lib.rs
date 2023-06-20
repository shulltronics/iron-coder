#![feature(file_create_new)]    // TODO - track progress of this feature
#![feature(io_error_other)]
mod app;
mod board;
mod editor;
mod project;
mod colorscheme;
mod icons;
pub use app::IronCoderApp;