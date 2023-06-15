use std::string::String;
use egui::Ui;
// use egui::containers::scroll_area::ScrollArea;
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

// for invoking external programs
use std::process::Command;

use crate::colorscheme::ColorScheme;

/// This module contains functionality for the code editor.
/// The namesake struct CodeEditor contains the state of the editor,
/// which includes:
/// * multiple tabs of source files
///
/// most of the code for syntaxt highlighting was adapted from the code_editor of
/// the egui demo app: https://github.com/emilk/egui/blob/master/crates/egui_demo_lib/src/syntax_highlighting.rs


// A CodeFile is some code in memory, it's path in the filesystem,
// and its file descriptor.
struct CodeFile {
    code: String,
    path: Option<PathBuf>,
    #[serde(skip)]
    file: Option<fs::File>,
}

impl Default for CodeFile {
    fn default() -> Self {
        Self {
            code: String::new(),
            path: None,
            file: None,
        }
    }
}

impl CodeFile {
    // Load some code from a path
    fn load_from_file(&mut self, file_path: &Path) -> std::io::Result<()> {
        let CodeFile { code, .. } = self;
        code.clear();
        self.path = Some(file_path.canonicalize()?);
        self.file = Some(
            fs::OpenOptions::new()
                            .read(true)
                            .write(true)
                            .open(file_path)?
                        );
        println!("{:?}", self.file);
        if let Some(file) = &mut self.file {
            file.read_to_string(code)?;
        }
        Ok(())
    }

    pub fn save(&mut self) -> std::io::Result<()> {
        if let Some(file) = &mut self.file {
            file.rewind()?;
            file.set_len(0)?;
            file.write(self.code.as_bytes())?;
            file.sync_all()?;
        }
        Ok(())
    }

    pub fn build_code(&mut self) {
        println!("Saving code...");
        // Save file first
        match self.save() {
            Ok(()) => (),
            Err(e) => println!("error saving file: {:?}", e),
        }
        // Make sure we have a valid path
        println!("{:?}", &self.path);
        let path = match &self.path {
            Some(p) => {
                let p = p.parent().unwrap().parent().unwrap().to_str().unwrap();
                println!("{:?}", p);
                p
            }
            None    => {
                println!("no valid path for cargo build!");
                return;
            }
        };
        let args = ["-Z", "unstable-options", "-C", path, "build"];
        // let args = ["version"];
        let mut build_command = Command::new("cargo");
        build_command.args(args);
        if let Ok(output) = build_command.output() {
            // println!("cargo version is: {:?}", cargo_v.stdout);
            std::io::stdout().write_all(&output.stdout).unwrap();
            std::io::stderr().write_all(&output.stderr).unwrap();
        } else {
            println!("error executing cargo build!");
        }
    }
}

// #[derive(serde::Deserialize, serde::Serialize)]
pub struct CodeEditor {
    tabs: Vec<CodeFile>,
    active_tab: Option<usize>,
    // #[serde(skip)]
    ps: SyntaxSet,
    // #[serde(skip)]
    ts: ThemeSet,
    cs: ColorScheme,
}

impl Default for CodeEditor {
    fn default() -> Self {
        Self {
            tabs: Vec::new(),
            active_tab: None,
            ps: SyntaxSet::load_defaults_newlines(),
            ts: ThemeSet::load_defaults(),
            cs: ColorScheme::default(),
        }
    }
}

impl CodeEditor {

    // Loads a CodeFile and pushes it onto the Vec of tabs
    pub fn load_from_file(&mut self, file_path: &Path) -> std::io::Result<()> {
        let mut code_file = CodeFile::default();
        code_file.load_from_file(file_path)?;
        self.tabs.push(code_file);
        self.active_tab = Some(self.tabs.len() - 1);
        Ok(())
    }

    pub fn set_colorscheme(&mut self, cs: ColorScheme) {
        self.cs = cs;
    }

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

    // I would prefer to implement the CodeEditor display
    // via the Widget trait I think (see below -- commented out).
    // But I was fighting the borrow checker too much. This seems to work.
    // TODO -- optimizations (and opportunities for benchmarking) 
    // regarding the syntax highlighting, as well as error checking and 
    // bug fixes
    pub fn display(&mut self, ctx: &egui::Context, _ui: &mut Ui) {
        // control pane for editor actions
        egui::TopBottomPanel::bottom("editor_control_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Buttons for various code actions, like compilation
                if ui.button("BUILD").clicked() {
                    if let Some(i) = self.active_tab {
                        self.tabs[i].build_code();
                    } else {
                        println!("no active editor tab!");
                    }
                };
                if ui.button("LOAD").clicked() {
                    println!("Todo -- load onto board based on bootloader/degubber type (uf2, etc)");
                };
            });
        });

        egui::TopBottomPanel::top("editor_tabs_pane").show(ctx, |ui| {
            // ui.label("tabs will go here...");
            ui.horizontal(|ui| {
                for (i, code_file) in self.tabs.iter().enumerate() {
                    let p = code_file.path.clone().unwrap();
                    let fname = p.as_path().file_name().unwrap();
                    let fname = fname.to_str().unwrap();
                    let mut text = RichText::new(fname);
                    if let Some(at) = self.active_tab {
                        if at == i {
                            text = text.strong();
                        }
                    }
                    let label = Label::new(text).sense(Sense::click());
                    if ui.add(label).clicked() {
                        self.active_tab = Some(i);
                    }
                    ui.separator();
                }
            });
        });

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

        let frame = egui::Frame::canvas(&ctx.style());
        egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
            // ui.set_style(ui.ctx().style());
            egui::containers::scroll_area::ScrollArea::both().show(ui, |ui| {
                // ui.style().code_bg_color = egui::Color32::RED;
                ui.add(
                    egui::TextEdit::multiline(&mut tabs[i].code)
                        .font(egui::TextStyle::Name("EditorFont".into()))
                        .code_editor()
                        .lock_focus(true)
                        .desired_width(f32::INFINITY)
                        .frame(false)
                        .layouter(&mut layouter),
                );
            });
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

pub fn highlight(ctx: &egui::Context, code: &str, language: &str) -> LayoutJob {
    // Implement this trait for the CodeEditor struct
    // should this be moved outside of the function?
    impl egui::util::cache::ComputerMut<(&str, &str), LayoutJob> for CodeEditor {
        fn compute(&mut self, (code, lang): (&str, &str)) -> LayoutJob {
            self.highlight(code, lang)
        }
    }

    type HighlightCache = egui::util::cache::FrameCache<LayoutJob, CodeEditor>;

    ctx.memory_mut(|mem| {
        mem.caches
            .cache::<HighlightCache>()
            .get((code, language))
    })
}

// impl Widget for CodeEditor {
//     fn ui(&mut self, ui: &mut Ui) -> Response {
//         let CodeEditor { code } = self;
//         ui.add(
//             egui::TextEdit::multiline(code)
//                 .font(egui::TextStyle::Monospace) // for cursor height
//                 .code_editor()
//                 .desired_rows(10)
//                 .lock_focus(true)
//                 .desired_width(f32::INFINITY)
//                 .frame(false),
//                 // .layouter(&mut layouter),
//         )
//     }
// }