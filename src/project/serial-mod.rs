use std::cmp::max;
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc, RwLock};
use std::time::Duration;
use std::{env, thread};

use crate::serial_monitor::{DataContainer, Packet};
use crate::serial_monitor::{load_gui_settings, SerialMonitor, RIGHT_PANEL_WIDTH};
use crate::serial_monitor::{save_to_csv, FileOptions};
use crate::serial::{load_serial_settings, serial_thread, Device};
use eframe::egui::{vec2, ViewportBuilder, Visuals};
use eframe::{egui};
use preferences::AppInfo;

mod serial_monitor;
mod serial;

const PREFERENCES_KEY: &str = "config/gui";
const PREFERENCES_KEY_SERIAL: &str = "config/serial_devices";

fn split(payload: &str) -> Vec<f32> {
    let mut split_data: Vec<&str> = vec![];
    for s in payload.split(':') {
        split_data.extend(s.split(','));
    }
    split_data
        .iter()
        .map(|x| x.trim())
        .flat_map(|x| x.parse::<f32>())
        .collect()
}

fn main_thread(
    sync_tx: Sender<bool>,
    data_lock: Arc<RwLock<DataContainer>>,
    raw_data_rx: Receiver<Packet>,
    save_rx: Receiver<FileOptions>,
    load_rx: Receiver<PathBuf>,
    load_names_tx: Sender<Vec<String>>,
    clear_rx: Receiver<bool>,
) {
    // reads data from mutex, samples and saves if needed
    let mut data = DataContainer::default();
    let mut failed_format_counter = 0;

    let mut file_opened = false;

    loop {
        if let Ok(cl) = clear_rx.recv_timeout(Duration::from_millis(1)) {
            if cl {
                data = DataContainer::default();
                failed_format_counter = 0;
            }
        }
        if !file_opened {
            if let Ok(packet) = raw_data_rx.recv_timeout(Duration::from_millis(1)) {
                data.loaded_from_file = false;
                if !packet.payload.is_empty() {
                    sync_tx.send(true).expect("unable to send sync tx");
                    data.raw_traffic.push(packet.clone());
                    let split_data = split(&packet.payload);
                    if data.dataset.is_empty() || failed_format_counter > 10 {
                        // resetting dataset
                        data.dataset = vec![vec![]; max(split_data.len(), 1)];
                        failed_format_counter = 0;
                    } else if split_data.len() == data.dataset.len() {
                        // appending data
                        for (i, set) in data.dataset.iter_mut().enumerate() {
                            set.push(split_data[i]);
                            failed_format_counter = 0;
                        }
                        data.time.push(packet.relative_time);
                        data.absolute_time.push(packet.absolute_time);
                        if data.time.len() != data.dataset[0].len() {
                            // resetting dataset
                            data.time = vec![];
                            data.dataset = vec![vec![]; max(split_data.len(), 1)];
                        }
                    } else {
                        // not same length
                        failed_format_counter += 1;
                    }
                }
            }

        }
        if let Ok(fp) = load_rx.recv_timeout(Duration::from_millis(10)) {
            if let Some(file_ending) = fp.extension() {
                match file_ending.to_str().unwrap() {
                    "csv" => {
                        file_opened = true;
                        let mut file_options = FileOptions {
                            file_path: fp.clone(),
                            save_absolute_time: false,
                            save_raw_traffic: false,
                            names: vec![],
                        };
                    }
                    _ => {
                        file_opened = false;
                        continue;
                    }
                }
            } else {
                file_opened = false;
            }
        } else {
            file_opened = false;
        }

        if let Ok(mut write_guard) = data_lock.write() {
            *write_guard = data.clone();
        }

        if let Ok(csv_options) = save_rx.recv_timeout(Duration::from_millis(1)) {
            match save_to_csv(&data, &csv_options) {
                Ok(_) => {
                    log::info!("saved data file to {:?} ", csv_options.file_path);
                }
                Err(e) => {
                    log::error!(
                        "failed to save file to {:?}: {:?}",
                        csv_options.file_path,
                        e
                    );
                }
            }
        }
    }
}

fn show_serial_monitor() {
    let gui_settings = load_gui_settings();
    let saved_serial_device_configs = load_serial_settings();

    let device_lock = Arc::new(RwLock::new(Device::default()));
    let devices_lock = Arc::new(RwLock::new(vec![gui_settings.device.clone()]));
    let data_lock = Arc::new(RwLock::new(DataContainer::default()));
    let connected_lock = Arc::new(RwLock::new(false));

    let (save_tx, save_rx): (Sender<FileOptions>, Receiver<FileOptions>) = mpsc::channel();
    let (load_tx, load_rx): (Sender<PathBuf>, Receiver<PathBuf>) = mpsc::channel();
    let (loaded_names_tx, loaded_names_rx): (Sender<Vec<String>>, Receiver<Vec<String>>) =
        mpsc::channel();
    let (send_tx, send_rx): (Sender<String>, Receiver<String>) = mpsc::channel();
    let (clear_tx, clear_rx): (Sender<bool>, Receiver<bool>) = mpsc::channel();
    let (raw_data_tx, raw_data_rx): (Sender<Packet>, Receiver<Packet>) = mpsc::channel();
    let (sync_tx, sync_rx): (Sender<bool>, Receiver<bool>) = mpsc::channel();

    let serial_device_lock = device_lock.clone();
    let serial_devices_lock = devices_lock.clone();
    let serial_connected_lock = connected_lock.clone();

    let _serial_thread_handler = thread::spawn(|| {
        serial_thread(
            send_rx,
            raw_data_tx,
            serial_device_lock,
            serial_devices_lock,
            serial_connected_lock,
        );
    });

    let main_data_lock = data_lock.clone();

    let _main_thread_handler = thread::spawn(|| {
        main_thread(
            sync_tx,
            main_data_lock,
            raw_data_rx,
            save_rx,
            load_rx,
            loaded_names_tx,
            clear_rx,
        );
    });

    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        load_tx
            .send(PathBuf::from(&args[1]))
            .expect("failed to send file");
    }

    let gui_data_lock = data_lock;
    let gui_device_lock = device_lock;
    let gui_devices_lock = devices_lock;
    let gui_connected_lock = connected_lock;

    /*
    SerialMonitor::new(
        ctx,
        gui_data_lock,
        gui_device_lock,
        gui_devices_lock,
        saved_serial_device_configs,
        gui_connected_lock,
        gui_settings,
        save_tx,
        load_tx,
        loaded_names_rx,
        send_tx,
        clear_tx,
    )
    */
}

fn main(){
    show_serial_monitor();
}