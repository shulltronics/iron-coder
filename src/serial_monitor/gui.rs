use core::f32;
use std::cmp::max;
use std::ops::RangeInclusive;
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, RwLock};
use std::fmt;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::error::Error;
use csv::{WriterBuilder};

use crate::serial::{clear_serial_settings, save_serial_settings, Device, SerialDevices};
use crate::{APP_INFO, PREFERENCES_KEY};
use eframe::egui::panel::Side;
use eframe::egui::{
    Align2, CollapsingHeader, Color32, FontFamily, FontId, InnerResponse, Pos2, Sense, Ui, Vec2, Visuals,
    vec2, Response, Stroke,
};
use eframe::epaint::StrokeKind;
use eframe::{egui, Storage};
use egui::ThemePreference;
use egui_theme_switch::ThemeSwitch;
use egui_file_dialog::information_panel::InformationPanel;
use egui_file_dialog::FileDialog;
use egui_plot::{log_grid_spacer, GridMark, Legend, Line, Plot, PlotPoint, PlotPoints};
use preferences::Preferences;
use serde::{Deserialize, Serialize};
use serialport::{DataBits, FlowControl, Parity, StopBits};

const DEFAULT_FONT_ID: FontId = FontId::new(14.0, FontFamily::Monospace);
pub const RIGHT_PANEL_WIDTH: f32 = 350.0;
const BAUD_RATES: &[u32] = &[
    300, 1200, 2400, 4800, 9600, 19200, 38400, 57600, 74880, 115200, 116500, 230400, 128000, 460800,
    576000, 921600,
];

#[derive(Clone)]
pub enum FileDialogState {
    //Open,
    Save,
    //SavePlot,
    None,
}
#[derive(PartialEq)]
pub enum WindowFeedback {
    None,
    Waiting,
    Clear,
    Cancel,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct GuiSettingsContainer {
    pub device: String,
    pub baud: u32,
    pub debug: bool,
    pub x: f32,
    pub y: f32,
    pub save_absolute_time: bool,
    pub dark_mode: bool,
    pub theme_preference: ThemePreference,
}

impl Default for GuiSettingsContainer {
    fn default() -> Self {
        Self {
            device: "".to_string(),
            baud: 115_200,
            debug: true,
            x: 1600.0,
            y: 900.0,
            save_absolute_time: false,
            dark_mode: true,
            theme_preference: ThemePreference::Dark,
        }
    }
}

pub fn load_gui_settings() -> GuiSettingsContainer {
    GuiSettingsContainer::load(&APP_INFO, PREFERENCES_KEY).unwrap_or_else(|_| {
        let gui_settings = GuiSettingsContainer::default();
        if gui_settings.save(&APP_INFO, PREFERENCES_KEY).is_err() {
            log::error!("failed to save gui_settings");
        }
        gui_settings
    })
}

#[derive(Clone, Debug, PartialEq)]
pub enum SerialDirection {
    Send,
    Receive,
}

impl fmt::Display for SerialDirection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SerialDirection::Send => write!(f, "SEND"),
            SerialDirection::Receive => write!(f, "RECV"),
        }
    }
}

pub fn get_epoch_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

#[derive(Clone, Debug)]
pub struct Packet {
    pub relative_time: f64,
    pub absolute_time: f64,
    pub direction: SerialDirection,
    pub payload: String,
}

impl Default for Packet {
    fn default() -> Packet {
        Packet {
            relative_time: 0.0,
            absolute_time: get_epoch_ms() as f64,
            direction: SerialDirection::Send,
            payload: "".to_string(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct DataContainer {
    pub time: Vec<f64>,
    pub absolute_time: Vec<f64>,
    pub dataset: Vec<Vec<f32>>,
    pub raw_traffic: Vec<Packet>,
    pub loaded_from_file: bool,
}

impl Default for DataContainer {
    fn default() -> DataContainer {
        DataContainer {
            time: vec![],
            absolute_time: vec![],
            dataset: vec![vec![]],
            raw_traffic: vec![],
            loaded_from_file: false,
        }
    }
}

/// A set of options for saving data to a CSV file.
#[derive(Debug)]
pub struct FileOptions {
    pub file_path: PathBuf,
    pub save_absolute_time: bool,
    pub save_raw_traffic: bool,
    pub names: Vec<String>,
}

pub fn save_to_csv(data: &DataContainer, csv_options: &FileOptions) -> Result<(), Box<dyn Error>> {
    let mut wtr = WriterBuilder::new()
        .has_headers(false)
        .from_path(&csv_options.file_path)?;
    let mut header = vec!["Time [ms]".to_string()];
    header.extend_from_slice(&csv_options.names);
    wtr.write_record(header)?;
    for j in 0..data.dataset[0].len() {
        let time = if csv_options.save_absolute_time {
            data.absolute_time[j].to_string()
        } else {
            data.time[j].to_string()
        };
        let mut data_to_write = vec![time];
        for value in data.dataset.iter() {
            data_to_write.push(value[j].to_string());
        }
        wtr.write_record(&data_to_write)?;
    }
    wtr.flush()?;
    if csv_options.save_raw_traffic {
        let mut path = csv_options.file_path.clone();
        let mut file_name = path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string()
            .replace(".csv", "");
        file_name += "raw.csv";
        path.set_file_name(file_name);
        save_raw(data, &path)?
    }
    Ok(())
}

pub fn save_raw(data: &DataContainer, path: &PathBuf) -> Result<(), Box<dyn Error>> {
    let mut wtr = WriterBuilder::new().has_headers(false).from_path(path)?;
    let header = vec![
        "Time [ms]".to_string(),
        "Abs Time [ms]".to_string(),
        "Raw Traffic".to_string(),
    ];
    wtr.write_record(header)?;

    for j in 0..data.dataset[0].len() {
        let mut data_to_write = vec![data.time[j].to_string(), data.absolute_time[j].to_string()];
        data_to_write.push(data.raw_traffic[j].payload.clone());
        wtr.write_record(&data_to_write)?;
    }
    wtr.flush()?;
    Ok(())
}

pub enum ColorWindow {
    NoShow,
    ColorIndex(usize),
}

pub const COLORS: [Color32; 10] = [
    Color32::WHITE,                   // White
    Color32::from_rgb(230, 159, 0),   // Orange
    Color32::from_rgb(86, 180, 233),  // Blue
    Color32::from_rgb(0, 158, 115),   // Turquoise
    Color32::from_rgb(240, 228, 66),  // Yellow
    Color32::from_rgb(0, 114, 178),   // Blue
    Color32::from_rgb(213, 94, 0),    // Red-orange
    Color32::from_rgb(204, 121, 167), // Purple
    Color32::from_rgb(121, 94, 56),   // Brown
    Color32::from_rgb(0, 204, 204),   // Cyan
];

pub struct SerialMonitor {
    connected_to_device: bool,
    command: String,
    device: String,
    old_device: String,
    device_idx: usize,
    serial_devices: SerialDevices,
    plotting_range: usize,
    plot_serial_display_ratio: f32,
    picked_path: PathBuf,
    plot_location: Option<egui::Rect>,
    data: DataContainer,
    file_dialog_state: FileDialogState,
    file_dialog: FileDialog,
    information_panel: InformationPanel,
    file_opened: bool,
    settings_window_open: bool,
    connection_error_window_open: bool,
    gui_conf: GuiSettingsContainer,
    device_lock: Arc<RwLock<Device>>,
    devices_lock: Arc<RwLock<Vec<String>>>,
    connected_lock: Arc<RwLock<bool>>,
    data_lock: Arc<RwLock<DataContainer>>,
    save_tx: Sender<FileOptions>,
    load_tx: Sender<PathBuf>,
    load_names_rx: Receiver<Vec<String>>,
    send_tx: Sender<String>,
    clear_tx: Sender<bool>,
    history: Vec<String>,
    index: usize,
    colors: Vec<Color32>,
    color_vals: Vec<f32>,
    labels: Vec<String>,
    show_color_window: ColorWindow,
    show_sent_cmds: bool,
    show_timestamps: bool,
    save_raw: bool,
    show_warning_window: WindowFeedback,
    do_not_show_clear_warning: bool,
    init: bool,
}

#[allow(clippy::too_many_arguments)]
impl SerialMonitor {
    pub fn new(
        cc: &eframe::CreationContext,
        data_lock: Arc<RwLock<DataContainer>>,
        device_lock: Arc<RwLock<Device>>,
        devices_lock: Arc<RwLock<Vec<String>>>,
        devices: SerialDevices,
        connected_lock: Arc<RwLock<bool>>,
        gui_conf: GuiSettingsContainer,
        save_tx: Sender<FileOptions>,
        load_tx: Sender<PathBuf>,
        load_names_rx: Receiver<Vec<String>>,
        send_tx: Sender<String>,
        clear_tx: Sender<bool>,
    ) -> Self {
        let mut file_dialog = FileDialog::default()
            .default_file_name("measurement.csv")
            .default_size([600.0, 400.0])
            .set_file_icon(
                "🖹",
                Arc::new(|path| path.extension().unwrap_or_default().to_ascii_lowercase() == "md"),
            )
            .set_file_icon(
                "",
                Arc::new(|path| {
                    path.file_name().unwrap_or_default().to_ascii_lowercase() == ".gitignore"
                }),
            )
            .add_file_filter(
                "CSV files",
                Arc::new(|p| p.extension().unwrap_or_default().to_ascii_lowercase() == "csv"),
            );
        if let Some(storage) = cc.storage {
            *file_dialog.storage_mut() =
                eframe::get_value(storage, "file_dialog_storage").unwrap_or_default()
        }

        Self {
            connected_to_device: false,
            picked_path: PathBuf::new(),
            device: "".to_string(),
            old_device: "".to_string(),
            data: DataContainer::default(),
            file_dialog_state: FileDialogState::None,
            file_dialog,
            information_panel: InformationPanel::default().add_file_preview("csv", |ui, item| {
                ui.label("CSV preview:");
                if let Some(mut content) = item.content() {
                    egui::ScrollArea::vertical()
                        .max_height(ui.available_height())
                        .show(ui, |ui| {
                            ui.add(egui::TextEdit::multiline(&mut content).code_editor());
                        });
                }
            }),
            connected_lock,
            device_lock,
            devices_lock,
            device_idx: 0,
            serial_devices: devices,
            gui_conf,
            data_lock,
            save_tx,
            load_tx,
            load_names_rx,
            send_tx,
            clear_tx,
            plotting_range: usize::MAX,
            plot_serial_display_ratio: 0.8,
            command: "".to_string(),
            show_sent_cmds: true,
            show_timestamps: true,
            save_raw: false,
            colors: vec![COLORS[0]],
            color_vals: vec![0.0],
            labels: vec!["Dataset 1".to_string()],
            history: vec![],
            index: 0,
            plot_location: None,
            do_not_show_clear_warning: false,
            show_warning_window: WindowFeedback::None,
            init: false,
            show_color_window: ColorWindow::NoShow,
            file_opened: false,
            settings_window_open: false,
            connection_error_window_open: false,
        }
    }

    pub fn clear_warning_window(&mut self, ctx: &egui::Context) -> WindowFeedback {
        let mut window_feedback = WindowFeedback::Waiting;
        egui::Window::new("Attention!")
            .fixed_pos(Pos2 { x: 800.0, y: 450.0 })
            .fixed_size(Vec2 { x: 400.0, y: 200.0 })
            .anchor(Align2::CENTER_CENTER, Vec2 { x: 0.0, y: 0.0 })
            .collapsible(false)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(20.0);
                    ui.label("Changing devices will clear all data.");
                    ui.label("How do you want to proceed?");
                    ui.add_space(20.0);
                    ui.horizontal(|ui| {
                        ui.add_space(130.0);
                        if ui.button("Continue & Clear").clicked() {
                            window_feedback = WindowFeedback::Clear;
                        }
                        if ui.button("Cancel").clicked() {
                            window_feedback = WindowFeedback::Cancel;
                        }
                    });
                    ui.add_space(5.0);
                });
            });
        window_feedback
    }

    pub fn color_picker_widget(
        ui: &mut Ui,
        label: &str,
        color: &mut [Color32],
        index: usize,
    ) -> Response {
        ui.horizontal(|ui| {
            let square_size = ui.spacing().interact_size.y * 0.8;
            let (rect, response) =
                ui.allocate_exact_size(egui::vec2(square_size, square_size), Sense::click());
            let stroke = if response.hovered() {
                Stroke::new(2.0, Color32::WHITE)
            } else {
                Stroke::NONE
            };
    
            ui.painter()
                .rect(rect, 2.0, color[index], stroke, StrokeKind::Middle);
            ui.label(label);
            response
        })
        .inner
    }
    pub fn color_picker_window(ctx: &egui::Context, color: &mut Color32, value: &mut f32) -> bool {
        let mut save_button = false;
    
        let _window_response = egui::Window::new("Color Menu")
            .fixed_size(Vec2 { x: 100.0, y: 100.0 })
            .anchor(Align2::CENTER_CENTER, Vec2 { x: 0.0, y: 0.0 })
            .collapsible(false)
            .show(ctx, |ui| {
                // Show five color options in each row
                let square_size = ui.spacing().interact_size.y * 0.8;
    
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        for color_option in &COLORS[0..5] {
                            let (rect, response) = ui.allocate_exact_size(
                                egui::vec2(square_size, square_size),
                                Sense::click(),
                            );
    
                            if response.clicked() {
                                *color = *color_option;
                            }
    
                            let stroke = if response.hovered() {
                                Stroke::new(2.0, Color32::WHITE)
                            } else {
                                Stroke::NONE
                            };
    
                            ui.painter()
                                .rect(rect, 2.0, *color_option, stroke, StrokeKind::Middle);
                        }
                    });
    
                    ui.horizontal(|ui| {
                        for color_option in &COLORS[5..10] {
                            let (rect, response) = ui.allocate_exact_size(
                                egui::vec2(square_size, square_size),
                                Sense::click(),
                            );
    
                            if response.clicked() {
                                *color = *color_option;
                            }
    
                            let stroke = if response.hovered() {
                                Stroke::new(2.0, Color32::WHITE)
                            } else {
                                Stroke::NONE
                            };
    
                            ui.painter()
                                .rect(rect, 2.0, *color_option, stroke, StrokeKind::Middle);
                        }
                    });
    
                    ui.add_space(25.0);
                    ui.centered_and_justified(|ui| {
                        if ui.button("Exit").clicked() {
                            save_button = true;
                        }
                    });
                });
            });
    
        save_button
    }

    fn show_connection_error(
        ctx: &egui::Context,
        connection_error_window_open: &mut bool,
    ) -> Option<InnerResponse<Option<()>>> {
        egui::Window::new("Connection error!")
            .fixed_pos(Pos2 { x: 800.0, y: 450.0 })
            .fixed_size(Vec2 { x: 400.0, y: 200.0 })
            .anchor(Align2::CENTER_CENTER, Vec2 { x: 0.0, y: 0.0 })
            .collapsible(false)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(20.0);
                    ui.label("No device selected.");
                    ui.label("Please select a device for connection.");
                    ui.add_space(20.0);
                    ui.add_space(5.0);
                    ui.add_space(5.0);
                    ui.horizontal(|ui| {
                        ui.horizontal(|ui| {
                            ui.add_space(190.0);
                            if ui.button("Exit").clicked() {
                                *connection_error_window_open = false;
                            }
                        });
                    });
                    ui.add_space(5.0);
                });
            })
    }

    fn show_settings_window(
        ctx: &egui::Context,
        gui_conf: &mut GuiSettingsContainer,
        settings_window_open: &mut bool,
    ) -> Option<InnerResponse<Option<()>>> {
        egui::Window::new("Settings")
            .fixed_size(Vec2 { x: 600.0, y: 200.0 })
            .anchor(Align2::CENTER_CENTER, Vec2 { x: 0.0, y: 0.0 })
            .collapsible(false)
            .show(ctx, |ui| {
                egui::Grid::new("theme settings")
                    .striped(true)
                    .show(ui, |ui| {
                        if ui
                            .add(ThemeSwitch::new(&mut gui_conf.theme_preference))
                            .changed()
                        {
                            ui.ctx().set_theme(gui_conf.theme_preference);
                        };
                        gui_conf.dark_mode = ui.visuals() == &Visuals::dark();
    
                        ui.end_row();
                        ui.end_row();
                    });
                ui.add_space(5.0);
                ui.horizontal(|ui| {
                    ui.horizontal(|ui| {
                        if ui.button("Exit Settings").clicked() {
                            *settings_window_open = false;
                        }
                    });
    
                });
            })
    }

    fn draw_central_panel(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let left_border = 10.0;
            let width = ui.available_size().x - 2.0 * left_border - RIGHT_PANEL_WIDTH;
            let top_spacing = 5.0;
            let panel_height = ui.available_size().y;
            let mut plot_height: f32 = 0.0;

            if self.serial_devices.number_of_plots[self.device_idx] > 0 {
                let height = ui.available_size().y * self.plot_serial_display_ratio;
                plot_height = height;
                plot_height = plot_height
                    / (self.serial_devices.number_of_plots[self.device_idx] as f32)
                    - 12.0;
            }

            let mut plot_ui_heigh: f32 = 0.0;

            ui.add_space(top_spacing);
            ui.horizontal(|ui| {
                ui.add_space(left_border);
                ui.vertical(|ui| {
                    if let Ok(read_guard) = self.data_lock.read() {
                        self.data = read_guard.clone();
                    }

                    if self.data.loaded_from_file && self.file_opened {
                        if let Ok(labels) =
                            self.load_names_rx.recv_timeout(Duration::from_millis(10))
                        {
                            self.labels = labels;
                            self.colors = (0..max(self.labels.len(), 1))
                                .map(|i| COLORS[i % COLORS.len()])
                                .collect();
                            self.color_vals = (0..max(self.labels.len(), 1)).map(|_| 0.0).collect();
                        }
                    }
                    if self.serial_devices.number_of_plots[self.device_idx] > 0 {
                        if self.data.dataset.len() != self.labels.len() && !self.file_opened {
                            self.labels = (1..max(self.data.dataset.len(), 1))
                                .map(|i| format!("Dataset {i}"))
                                .collect();
                            self.colors = (0..max(self.data.dataset.len(), 1))
                                .map(|i| COLORS[i % COLORS.len()])
                                .collect();
                            self.color_vals =
                                (0..max(self.data.dataset.len(), 1)).map(|_| 0.0).collect();
                        }

                        let mut graphs: Vec<Vec<PlotPoint>> = vec![vec![]; self.data.dataset.len()];

                        let window = self.data.dataset[0]
                            .len()
                            .saturating_sub(self.plotting_range);

                        for (i, time) in self.data.time[window..].iter().enumerate() {
                            let x = *time / 1000.0;
                            for (graph, data) in graphs.iter_mut().zip(&self.data.dataset) {
                                if self.data.time.len() == data.len() {
                                    if let Some(y) = data.get(i + window) {
                                        graph.push(PlotPoint { x, y: *y as f64 });
                                    }
                                }
                            }
                        }

                        let t_fmt = |x: GridMark, _range: &RangeInclusive<f64>| {
                            format!("{:4.2} s", x.value)
                        };

                        let plots_ui = ui.vertical(|ui| {
                            for graph_idx in 0..self.serial_devices.number_of_plots[self.device_idx]
                            {
                                if graph_idx != 0 {
                                    ui.separator();
                                }

                                let signal_plot = Plot::new(format!("data-{graph_idx}"))
                                    .height(plot_height)
                                    .width(width)
                                    .legend(Legend::default());
                                    //.x_grid_spacer(log_grid_spacer(10))
                                    //.y_grid_spacer(log_grid_spacer(10));
                                    //.x_axis_formatter(t_fmt);

                                let plot_inner = signal_plot.show(ui, |signal_plot_ui| {
                                    let data_points: egui_plot::PlotPoints = [0,0,0,0,0,0,0,0,0,0,1,1,1,1,1,1,1,1,1,1]
                                        .into_iter()
                                        .enumerate()
                                        .map(|(i, value)| [i as f64, value as f64])
                                        .collect();
                                    if (self.connected_to_device){
                                        signal_plot_ui.line(
                                            Line::new(data_points)
                                                .name(&self.labels[0])
                                                .color(self.colors[0]),
                                        );
                                    }
                                    /*
                                    for (i, graph) in graphs.iter().enumerate() {
                                        if i < self.labels.len() {
                                            signal_plot_ui.line(
                                                Line::new(PlotPoints::Owned(graph.to_vec()))
                                                    .name(&self.labels[i])
                                                    .color(self.colors[i]),
                                            );
                                        }
                                    }
                                    */
                                });

                                self.plot_location = Some(plot_inner.response.rect);
                            }
                            let separator_response = ui.separator();
                            let separator = ui
                                .interact(
                                    separator_response.rect,
                                    separator_response.id,
                                    Sense::click_and_drag(),
                                )
                                .on_hover_cursor(egui::CursorIcon::ResizeVertical);

                            let resize_y = separator.drag_delta().y;

                            if separator.double_clicked() {
                                self.plot_serial_display_ratio = 0.8;
                            }
                            self.plot_serial_display_ratio = (self.plot_serial_display_ratio
                                + resize_y / panel_height)
                                .clamp(0.1, 0.9);

                            ui.add_space(top_spacing);
                        });
                        plot_ui_heigh = plots_ui.response.rect.height();
                    } else {
                        plot_ui_heigh = 0.0;
                    }

                    let serial_height =
                        panel_height - plot_ui_heigh - left_border * 2.0 - top_spacing;

                    let num_rows = self.data.raw_traffic.len();
                    let row_height = ui.text_style_height(&egui::TextStyle::Body);

                    let color = if self.gui_conf.dark_mode {
                        Color32::WHITE
                    } else {
                        Color32::BLACK
                    };

                    let mut text_edit_size = ui.available_size();
                    text_edit_size.x = width;
                });
                ui.add_space(left_border);
            });
        });
    }

    fn draw_serial_settings(&mut self, ctx: &egui::Context, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.heading("Serial Monitor");
            self.paint_connection_indicator(ui);
        });

        let devices: Vec<String> = if let Ok(read_guard) = self.devices_lock.read() {
            read_guard.clone()
        } else {
            vec![]
        };

        if !devices.contains(&self.device) {
            self.device.clear();
        }
        if let Ok(dev) = self.device_lock.read() {
            if !dev.name.is_empty() {
                self.device = dev.name.clone();
            }
        }
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            ui.label("Device");
            ui.add_space(130.0);
            ui.label("Baud");
        });

        let old_name = self.device.clone();
        ui.horizontal(|ui| {
            if self.file_opened {
                ui.disable();
            }
            let dev_text = self.device.replace("/dev/tty.", "");
            ui.horizontal(|ui| {
                if self.connected_to_device {
                    ui.disable();
                }
                let _response = egui::ComboBox::from_id_salt("Device")
                    .selected_text(dev_text)
                    .width(RIGHT_PANEL_WIDTH * 0.92 - 155.0)
                    .show_ui(ui, |ui| {
                        devices
                            .into_iter()
                            .filter(|dev| !dev.contains("/dev/cu."))
                            .for_each(|dev| {
                                let dev_text = dev.replace("/dev/tty.", "");
                                ui.selectable_value(&mut self.device, dev, dev_text);
                            });
                    })
                    .response;

                if old_name != self.device {
                    if !self.data.time.is_empty() {
                        self.show_warning_window = WindowFeedback::Waiting;
                        self.old_device = old_name;
                    } else {
                        self.show_warning_window = WindowFeedback::Clear;
                    }
                }
            });
            match self.show_warning_window {
                WindowFeedback::None => {}
                WindowFeedback::Waiting => {
                    self.show_warning_window = self.clear_warning_window(ctx);
                }
                WindowFeedback::Clear => {
                    let mut device_is_already_saved = false;
                    for (idx, dev) in self.serial_devices.devices.iter().enumerate() {
                        if dev.name == self.device {
                            self.device = dev.name.clone();
                            self.device_idx = idx;
                            self.init = true;
                            device_is_already_saved = true;
                        }
                    }
                    if !device_is_already_saved {
                        let mut device = Device::default();
                        device.name = self.device.clone();
                        self.serial_devices.devices.push(device);
                        self.serial_devices.number_of_plots.push(1);
                        self.serial_devices
                            .labels
                            .push(vec!["Dataset 1".to_string()]);
                        self.device_idx = self.serial_devices.devices.len() - 1;
                        save_serial_settings(&self.serial_devices);
                    }
                    self.clear_tx
                        .send(true)
                        .expect("failed to send clear after choosing new device");
                    self.data = DataContainer::default();
                    self.show_warning_window = WindowFeedback::None;
                }
                WindowFeedback::Cancel => {
                    self.device = self.old_device.clone();
                    self.show_warning_window = WindowFeedback::None;
                }
            }
            egui::ComboBox::from_id_salt("Baud Rate")
                .selected_text(format!(
                    "{}",
                    self.serial_devices.devices[self.device_idx].baud_rate
                ))
                .width(80.0)
                .show_ui(ui, |ui| {
                    if self.connected_to_device {
                        ui.disable();
                    }
                    BAUD_RATES.iter().for_each(|baud_rate| {
                        ui.selectable_value(
                            &mut self.serial_devices.devices[self.device_idx].baud_rate,
                            *baud_rate,
                            baud_rate.to_string(),
                        );
                    });
                });
            let connect_text = if self.connected_to_device {
                "Disconnect"
            } else {
                "Connect"
            };
            if ui.button(connect_text).clicked() {
                if let Ok(mut device) = self.device_lock.write() {
                    if self.connected_to_device {
                        device.name.clear();
                    } else {
                        device.name = self.serial_devices.devices[self.device_idx].name.clone();
                        device.baud_rate = self.serial_devices.devices[self.device_idx].baud_rate;
                    }
                }
                if self.device.as_str() == "" {
                    self.connection_error_window_open = true;
                }
            }
        });
        if self.connection_error_window_open {
            Self::show_connection_error(
                ui.ctx(),
                &mut self.connection_error_window_open,
            );
        }
        ui.add_space(5.0);
        ui.horizontal(|ui| {
            ui.label("Data Bits");
            ui.add_space(5.0);
            ui.label("Parity");
            ui.add_space(20.0);
            ui.label("Stop Bits");
            ui.label("Flow Control");
            ui.label("Timeout");
        });
        ui.horizontal(|ui| {
            if self.connected_to_device {
                ui.disable();
            }
            egui::ComboBox::from_id_salt("Data Bits")
                .selected_text(
                    self.serial_devices.devices[self.device_idx]
                        .data_bits
                        .to_string(),
                )
                .width(30.0)
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.serial_devices.devices[self.device_idx].data_bits,
                        DataBits::Eight,
                        DataBits::Eight.to_string(),
                    );
                    ui.selectable_value(
                        &mut self.serial_devices.devices[self.device_idx].data_bits,
                        DataBits::Seven,
                        DataBits::Seven.to_string(),
                    );
                    ui.selectable_value(
                        &mut self.serial_devices.devices[self.device_idx].data_bits,
                        DataBits::Six,
                        DataBits::Six.to_string(),
                    );
                    ui.selectable_value(
                        &mut self.serial_devices.devices[self.device_idx].data_bits,
                        DataBits::Five,
                        DataBits::Five.to_string(),
                    );
                });
            egui::ComboBox::from_id_salt("Parity")
                .selected_text(
                    self.serial_devices.devices[self.device_idx]
                        .parity
                        .to_string(),
                )
                .width(30.0)
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.serial_devices.devices[self.device_idx].parity,
                        Parity::None,
                        Parity::None.to_string(),
                    );
                    ui.selectable_value(
                        &mut self.serial_devices.devices[self.device_idx].parity,
                        Parity::Odd,
                        Parity::Odd.to_string(),
                    );
                    ui.selectable_value(
                        &mut self.serial_devices.devices[self.device_idx].parity,
                        Parity::Even,
                        Parity::Even.to_string(),
                    );
                });
            egui::ComboBox::from_id_salt("Stop Bits")
                .selected_text(
                    self.serial_devices.devices[self.device_idx]
                        .stop_bits
                        .to_string(),
                )
                .width(30.0)
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.serial_devices.devices[self.device_idx].stop_bits,
                        StopBits::One,
                        StopBits::One.to_string(),
                    );
                    ui.selectable_value(
                        &mut self.serial_devices.devices[self.device_idx].stop_bits,
                        StopBits::Two,
                        StopBits::Two.to_string(),
                    );
                });
            egui::ComboBox::from_id_salt("Flow Control")
                .selected_text(
                    self.serial_devices.devices[self.device_idx]
                        .flow_control
                        .to_string(),
                )
                .width(75.0)
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.serial_devices.devices[self.device_idx].flow_control,
                        FlowControl::None,
                        FlowControl::None.to_string(),
                    );
                    ui.selectable_value(
                        &mut self.serial_devices.devices[self.device_idx].flow_control,
                        FlowControl::Hardware,
                        FlowControl::Hardware.to_string(),
                    );
                    ui.selectable_value(
                        &mut self.serial_devices.devices[self.device_idx].flow_control,
                        FlowControl::Software,
                        FlowControl::Software.to_string(),
                    );
                });
            egui::ComboBox::from_id_salt("Timeout")
                .selected_text(
                    self.serial_devices.devices[self.device_idx]
                        .timeout
                        .as_millis()
                        .to_string(),
                )
                .width(55.0)
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.serial_devices.devices[self.device_idx].timeout,
                        Duration::from_millis(0),
                        "0",
                    );
                    ui.selectable_value(
                        &mut self.serial_devices.devices[self.device_idx].timeout,
                        Duration::from_millis(10),
                        "10",
                    );
                    ui.selectable_value(
                        &mut self.serial_devices.devices[self.device_idx].timeout,
                        Duration::from_millis(100),
                        "100",
                    );
                    ui.selectable_value(
                        &mut self.serial_devices.devices[self.device_idx].timeout,
                        Duration::from_millis(1000),
                        "1000",
                    );
                });
        });
        ui.add_space(5.0);
        ui.horizontal(|ui| {
            if self.connected_to_device {
                ui.disable();
            }
        });
    }
    fn draw_export_settings(&mut self, _ctx: &egui::Context, ui: &mut Ui) {
        egui::Grid::new("export_settings")
            .num_columns(2)
            .spacing(Vec2 { x: 10.0, y: 10.0 })
            .striped(true)
            .show(ui, |ui| {
                if ui
                    .button(egui::RichText::new(format!(
                        "{} Save CSV",
                        egui_phosphor::regular::FLOPPY_DISK
                    )))
                    .on_hover_text("Save Plot Data to CSV.")
                    .clicked()
                {
                    self.file_dialog_state = FileDialogState::Save;
                    self.file_dialog.save_file();
                }
                ui.end_row();
            });
    }

    fn draw_global_settings(&mut self, ui: &mut Ui) {
        ui.add_space(20.0);

        if ui
            .button(format!("{} Display Settings", egui_phosphor::regular::GEAR_FINE))
            .clicked()
        {
            self.settings_window_open = true;
        }
        if self.settings_window_open {
            Self::show_settings_window(
                ui.ctx(),
                &mut self.gui_conf,
                &mut self.settings_window_open,
            );
        }

        ui.add_space(20.0);

        if ui
            .button(egui::RichText::new(format!(
                "{} Clear Data",
                egui_phosphor::regular::X
            )))
            .on_hover_text("Clear Data from Plot.")
            .clicked()
        {
            log::info!("Cleared recorded Data");
            if let Err(err) = self.clear_tx.send(true) {
                log::error!("clear_tx thread send failed: {:?}", err);
            }
            self.data = DataContainer::default();
        }
        ui.add_space(5.0);
        ui.horizontal(|ui| {
            if ui.button("Clear Device History").clicked() {
                self.serial_devices = SerialDevices::default();
                self.device.clear();
                self.device_idx = 0;
                clear_serial_settings();
            }
            if ui.button("Reset Fields").clicked() {
                // TODO: put default values back
            }
        });
    }

    fn draw_plot_settings(&mut self, ui: &mut Ui) {
        if self.labels.len() == 1 {
            ui.label("Dataset:");
        } else {
            ui.label(format!("Detected {} Datasets:", self.labels.len()));
        }
        ui.add_space(5.0);
        for i in 0..self.labels.len().min(10) {
            if self.init {
                self.init = false;
            }

            if self.labels.len() <= i {
                break;
            }
            ui.horizontal(|ui| {
                let response = Self::color_picker_widget(ui, "", &mut self.colors, i);

                if response.clicked() {
                    self.show_color_window = ColorWindow::ColorIndex(i);
                };

                ui.add(
                        egui::TextEdit::singleline(&mut self.labels[i])
                            .desired_width(0.95 * RIGHT_PANEL_WIDTH),
                    )
                    .on_hover_text("Use custom names for your Datasets.")
                    .changed()
            });
        }
        match self.show_color_window {
            ColorWindow::NoShow => {}
            ColorWindow::ColorIndex(index) => {
                if Self::color_picker_window(
                    ui.ctx(),
                    &mut self.colors[index],
                    &mut self.color_vals[index],
                ) {
                    self.show_color_window = ColorWindow::NoShow;
                }
            }
        }
    }

    fn draw_side_panel(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::new(Side::Right, "settings panel")
            .min_width(RIGHT_PANEL_WIDTH)
            .max_width(RIGHT_PANEL_WIDTH)
            .resizable(false)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add_enabled_ui(true, |ui| {
                        self.draw_serial_settings(ctx, ui);

                        self.draw_global_settings(ui);
                        ui.add_space(10.0);
                        CollapsingHeader::new("Plot Settings")
                            .default_open(true)
                            .show(ui, |ui| {
                                self.draw_plot_settings(ui);
                            });

                        CollapsingHeader::new("Export Settings")
                            .default_open(true)
                            .show(ui, |ui| {
                                self.draw_export_settings(ctx, ui);
                            });
                    });
                    ui.add_space(20.0);
                    ui.separator();
                    ui.collapsing("Debug logs:", |ui| {
                        egui_logger::logger_ui().show(ui);
                    });

                    match self.file_dialog_state {
                        FileDialogState::Save => {
                            if let Some(path) = self.file_dialog.update(ctx).picked() {
                                self.picked_path = path.to_path_buf();
                                self.file_dialog_state = FileDialogState::None;
                                self.picked_path.set_extension("csv");

                                if let Err(e) = self.save_tx.send(FileOptions {
                                    file_path: self.picked_path.clone(),
                                    save_absolute_time: self.gui_conf.save_absolute_time,
                                    save_raw_traffic: self.save_raw,
                                    names: self.labels.clone(),
                                }) {
                                    log::error!("save_tx thread send failed: {:?}", e);
                                }
                            }
                        }
                        FileDialogState::None => {}
                    }
                });
            });
    }

    fn paint_connection_indicator(&self, ui: &mut egui::Ui) {
        let (color, color_stroke) = if !self.connected_to_device {
            ui.add(egui::Spinner::new());
            (Color32::DARK_RED, Color32::RED)
        } else {
            (Color32::DARK_GREEN, Color32::GREEN)
        };

        let radius = ui.spacing().interact_size.y * 0.375;
        let center = egui::pos2(
            ui.next_widget_position().x + ui.spacing().interact_size.x * 0.5,
            ui.next_widget_position().y,
        );
        ui.painter()
            .circle(center, radius, color, egui::Stroke::new(1.0, color_stroke));
    }
}

impl eframe::App for SerialMonitor {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if let Ok(read_guard) = self.connected_lock.read() {
            self.connected_to_device = *read_guard;
        }
        self.draw_central_panel(ctx);
        self.draw_side_panel(ctx, frame);

        self.gui_conf.x = ctx.used_size().x;
        self.gui_conf.y = ctx.used_size().y;
    }

    fn save(&mut self, _storage: &mut dyn Storage) {
        save_serial_settings(&self.serial_devices);
        if let Err(err) = self.gui_conf.save(&APP_INFO, PREFERENCES_KEY) {
            log::error!("gui settings save failed: {:?}", err);
        }
    }
}
