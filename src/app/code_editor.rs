//! Title: Iron Coder App Module - Code Editor
//! Description: Handles all the code editing functionality for the app.

use std::string::String;
use std::fmt;

use log::info;

use egui::Ui;
use egui::containers::scroll_area::ScrollArea;
use egui::text::LayoutJob;
use egui::Label;
use egui::Sense;
use egui::widget_text::RichText;

use syntect::easy::HighlightLines;
use syntect::parsing::SyntaxSet;
use syntect::highlighting::{ThemeSet, FontStyle};
use syntect::util::LinesWithEndings;

// Imports for reading & writing to/from files and opening via Path
use std::path::{Path, PathBuf};
use std::fs;
use std::io::{Read, Write, Seek};

use std::sync::Arc;
use crate::app::icons::IconSet;
// use crate::app::colorscheme::ColorScheme;

/// This module contains functionality for the code editor.
/// The namesake struct CodeEditor contains the state of the editor,
/// which includes:
/// * multiple tabs of source files
///
/// most of the code for syntaxt highlighting was adapted from the code_editor of
/// the egui demo app: https://github.com/emilk/egui/blob/master/crates/egui_demo_lib/src/syntax_highlighting.rs


// A CodeFile is some code in memory, its path in the filesystem,
// and its file descriptor.
#[derive(Debug)]
struct CodeFile {
    code: String,
    path: Option<PathBuf>,
    file: Option<fs::File>,
    synced: bool,   // represents whether the code buffer is synced to the filesystem
}

impl Default for CodeFile {
    fn default() -> Self {
        Self {
            code: String::new(),
            path: None,
            file: None,
            synced: false,
        }
    }
}

impl CodeFile {
    // Load some code from a path
    fn load_from_file(&mut self, file_path: &Path) -> std::io::Result<()> {
        let CodeFile { code, .. } = self;
        code.clear();
        self.path = Some(file_path.canonicalize()?);
        self.file = Some(fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(file_path)?
        );
        if let Some(file) = &mut self.file {
            file.read_to_string(code)?;
            self.synced = true;
        }
        Ok(())
    }

    pub fn save(&mut self) -> std::io::Result<()> {
        if let Some(file) = &mut self.file {
            file.rewind()?;
            file.set_len(0)?;
            file.write(self.code.as_bytes())?;
            file.sync_all()?;
            self.synced = true;
        }
        Ok(())
    }
}

pub struct CodeEditor {
    tabs: Vec<CodeFile>,
    active_tab: Option<usize>,
    ps: SyntaxSet,
    ts: ThemeSet,
    // cs: ColorScheme,
}

impl fmt::Debug for CodeEditor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "num tabs: {:?}", self.tabs.len())?;
        write!(f, "active_tab: {:?}", self.active_tab)?;
        Ok(())
    }
}

impl Default for CodeEditor {
    fn default() -> Self {
        Self {
            tabs: Vec::new(),
            active_tab: None,
            ps: SyntaxSet::load_defaults_newlines(),
            ts: ThemeSet::load_defaults(),
            // cs: ColorScheme::default(),
        }
    }
}

impl CodeEditor {

    // Loads a CodeFile and pushes it onto the Vec of tabs,
    // unless file_path already exists in one of the tabs, in which case
    // the active tab is switched to that tab
    pub fn load_from_file(&mut self, file_path: &Path) -> std::io::Result<()> {
        let predicate = |elem: &CodeFile| {
            // elem is a CodeFile, see it the path matches the arg to this method
            if let Some(path) = &elem.path {
                // use canonicalize() here to keep things consistent
                return path.as_path() == file_path.canonicalize().unwrap();
            } else {
                return false;
            }
        };
        if let Some(i) = self.tabs.iter().position(predicate) {
            self.active_tab = Some(i);
        } else {
            let mut code_file = CodeFile::default();
            code_file.load_from_file(file_path)?;
            self.tabs.push(code_file);
            self.active_tab = Some(self.tabs.len() - 1);
        }
        Ok(())
    }

    // iterates through and saves all open tabs
    pub fn save_all(&mut self) -> std::io::Result<()> {
        for tab in self.tabs.iter_mut() {
            tab.save()?;
        }
        Ok(())
    }

    // pub fn set_colorscheme(&mut self, cs: ColorScheme) {
    //     self.cs = cs;
    // }

    // This method computes the syntax highlighting.
    // The module function `highlight` caches the result and should
    // only call this method if the code changes
    fn highlight(&mut self, text: &str, language: &str) -> LayoutJob {
        // Destructure, and do the highlighting
        let CodeEditor {
            ps,
            ts,
            ..
        } = self;

        let syntax = ps.find_syntax_by_extension(language).unwrap();

        // Choose the syntext colorscheme by parsing the current GUI colorscheme
        // let theme = match 

        let mut h = HighlightLines::new(syntax, &ts.themes["Solarized (dark)"]);

        use egui::text::{LayoutSection, TextFormat};

        let mut job = LayoutJob {
            text: text.to_string(),
            ..Default::default()
        };

        for line in LinesWithEndings::from(text) {
            for (style, range) in h.highlight_line(line, &ps).ok().unwrap() {
                let fg = style.foreground;
                let text_color = egui::Color32::from_rgb(fg.r, fg.g, fg.b);
                let italics = style.font_style.contains(FontStyle::ITALIC);
                let underline = style.font_style.contains(FontStyle::ITALIC);
                let underline = if underline {
                    egui::Stroke::new(1.0, text_color)
                } else {
                    egui::Stroke::NONE
                };
                job.sections.push(LayoutSection {
                    leading_space: 0.0,
                    byte_range: as_byte_range(text, range),
                    format: TextFormat {
                        font_id: egui::FontId::new(12.0, egui::FontFamily::Name("EditorFont".into())),
                        color: text_color,
                        italics,
                        underline,
                        ..Default::default()
                    },
                });
            }
        }

        return job;
    }

    // TODO -- optimizations (and opportunities for benchmarking) 
    // regarding the syntax highlighting, as well as error checking and 
    // bug fixes
    pub fn display_code(&mut self, ctx: &egui::Context, ui: &mut Ui) {

        // First, get some data from the object, and detect if any tabs are open
        // (if not, just return)
        let CodeEditor { tabs, active_tab, .. } = self;
        let i: usize;
        if *active_tab == None {
            return;
        } else {
            i = active_tab.unwrap();
        }

        let mut layouter = |ui: &egui::Ui, string: &str, _wrap_width: f32| {
            // Call the highlight function (below), which is a memoized version
            // of this struct's highlight method
            let layout_job = highlight(ctx, string, "rs");
            ui.fonts(|f| f.layout_job(layout_job))
        };

        ScrollArea::both().auto_shrink([false; 2]).show(ui, |ui| {
            let former_contents = tabs[i].code.clone();
            let resp = ui.add(
                egui::TextEdit::multiline(&mut tabs[i].code)
                    .font(egui::TextStyle::Name("EditorFont".into()))
                    .code_editor()
                    .lock_focus(true)
                    .desired_width(f32::INFINITY)
                    .frame(false)
                    .layouter(&mut layouter),
            );
            // check if the code has changed, so we can set the synced flag
            if tabs[i].synced && tabs[i].code != former_contents {
                tabs[i].synced = false;
            }
            // See if a code snippet was released over the editor.
            // TODO -- if so, insert it on the proper line
            ctx.memory_mut(|mem| {
                let id = egui::Id::new("released_code_snippet");
                let data: Option<String> = mem.data.get_temp(id);
                if let Some(value) = data {
                    if resp.hovered() {
                        info!("found a released code snippet!");
                        mem.data.remove::<String>(id);
                        tabs[i].code += &value;
                    }
                }
            });
        });
    }

    pub fn close_all_tabs(&mut self) {
        self.active_tab = None;
        self.tabs.clear();
    }

    pub fn get_active_tab(&self) -> Option<usize> {
        return self.active_tab;
    }
    pub fn close_tab(&mut self, i: usize) {
        if self.tabs.len() == 0 {
            return;
        }

        let _ = self.tabs.remove(i);
        let mut at = i;

        if self.tabs.len() == 0 {
            self.active_tab = None;
        } else {
            if at >= self.tabs.len() {
                at -= 1;
            }

            if (self.active_tab == None || self.active_tab == Some(i)) {
                self.active_tab = Some(at);
            }
            else {
                if self.active_tab > Some(i) {
                    self.active_tab = Some(self.active_tab.unwrap() - 1);
                }
            }
        }
    }
    pub fn display_editor_tabs(&mut self, ctx: &egui::Context, ui: &mut Ui) {
        let icons_ref: Arc<IconSet> = ctx.data_mut(|data| {
            data.get_temp("icons".into()).expect("error loading shared icon map!")
        });
        let icons = icons_ref.clone();
        ui.horizontal(|ui| {
            let mut idx_to_remove: Option<usize> = None;
            for (i, code_file) in self.tabs.iter().enumerate() {
                // display the close icon
                let x_icon = egui::widgets::ImageButton::new(
                    icons.get("quit_icon").unwrap().clone()
                ).frame(true);
                if ui.add(x_icon).clicked() {
                    // we'll remove the tab right after this loop
                    idx_to_remove = Some(i);
                }
                // extract the file name for the tab and display it
                // as a clickable label
                let p = code_file.path.clone().unwrap();
                let fname = p.as_path().file_name().unwrap();
                let fname = fname.to_str().unwrap();
                let mut text = RichText::new(fname);
                // active tab should be hightlighted
                if let Some(at) = self.active_tab {
                    if at == i {
                        text = text.underline();
                    }
                }
                if !self.tabs[i].synced {
                    text = text.color(egui::Color32::RED);
                }
                let label = Label::new(text).sense(Sense::click());
                if ui.add(label).clicked() {
                    self.active_tab = Some(i);
                }
                ui.separator();
            }

            if idx_to_remove.is_some() {
                self.close_tab(idx_to_remove.unwrap());
            }
        });
    }

}

fn as_byte_range(whole: &str, range: &str) -> std::ops::Range<usize> {
    let whole_start = whole.as_ptr() as usize;
    let range_start = range.as_ptr() as usize;
    assert!(whole_start <= range_start);
    assert!(range_start + range.len() <= whole_start + whole.len());
    let offset = range_start - whole_start;
    offset..(offset + range.len())
}

// Implementation of egui's ComputerMut trait to cache the syntax highlighting.
// This function will only call the compute function if the  (code, lang) tuple
// changes (i.e. the code was modified), or the language type changed (not  yet
// supported in Iron Coder). Otherwise, the cached LayoutJob will be returned.
pub fn highlight(ctx: &egui::Context, code: &str, language: &str) -> LayoutJob {
    // ComputerMut trait
    impl egui::util::cache::ComputerMut<(&str, &str), LayoutJob> for CodeEditor {
        fn compute(&mut self, (code, lang): (&str, &str)) -> LayoutJob {
            self.highlight(code, lang)
        }
    }
    type HighlightCache = egui::util::cache::FrameCache<LayoutJob, CodeEditor>;
    // either return cached LayoutJob, or compute the new one
    ctx.memory_mut(|mem| {
        mem.caches
            .cache::<HighlightCache>()
            .get((code, language))
    })
    
}