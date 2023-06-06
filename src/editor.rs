// This module contains functionality for the code editor.
// The namesake struct CodeEditor contains the state of the editor,
// which includes:
// * multiple tabs of source files

use std::string::String;
use serde;
use egui::Ui;
use egui::containers::scroll_area::ScrollArea;
use egui::text::LayoutJob;

use syntect::easy::HighlightLines;
use syntect::parsing::SyntaxSet;
use syntect::highlighting::{ThemeSet, Style, FontStyle};
use syntect::util::{as_24_bit_terminal_escaped, LinesWithEndings};

#[derive(serde::Deserialize, serde::Serialize)]
pub struct CodeEditor {
    code: String,
    #[serde(skip)]
    ps: SyntaxSet,
    #[serde(skip)]
    ts: ThemeSet,
}

impl Default for CodeEditor {
    fn default() -> Self {
        Self {
            code: "// welcome to Iron Coder!\n".to_string(),
            ps: SyntaxSet::load_defaults_newlines(),
            ts: ThemeSet::load_defaults(),
        }
    }
}

impl CodeEditor {

    fn highlight(&mut self, text: &str, language: &str) -> LayoutJob {
        // Destructure, and do the highlighting
        let CodeEditor { code, ps, ts } = self;

        let syntax = ps.find_syntax_by_extension(language).unwrap();

        let mut h = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);

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
                        font_id: egui::FontId::monospace(12.0),
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

    // TODO -- Lots of optimizations (and opportunities for benchmarking)
    // regarding the syntax highlighting, as well as error checking and 
    // bug fixes

    pub fn display(&mut self, ctx: &egui::Context, ui: &mut Ui) {
        // control pane for editor actions
        egui::TopBottomPanel::bottom("editor_control_panel").show(ctx, |ui| {
            ui.label("TODO -- editor control pane");
        });

        let CodeEditor { code, ps, ts } = self;

        let mut layouter = |ui: &egui::Ui, string: &str, _wrap_width: f32| {
            // Call the highlight function (below), which is a memoized version
            // of this struct's highlight method
            let layout_job = highlight(ctx, string, "rs");
            ui.fonts(|f| f.layout_job(layout_job))
        };

        ui.add(
            egui::TextEdit::multiline(code)
                .font(egui::TextStyle::Monospace)
                .code_editor()
                .desired_rows(1)
                .lock_focus(true)
                .desired_width(f32::INFINITY)
                .frame(false)
                .layouter(&mut layouter),
        );

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