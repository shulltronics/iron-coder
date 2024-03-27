use log::info;
use clap::Parser;
use std::str::FromStr;

use iron_coder::IronCoderOptions;

fn main() -> eframe::Result<()> {

    let app_options = IronCoderOptions::parse();

    // Setup the subscriber with a logging level.
    let debug_level: tracing::Level = if let Some(verbosity) = app_options.clone().verbosity {
        tracing::Level::from_str(&verbosity).unwrap_or_else(|_| {
            println!("Unknown debug level, using INFO instead.");
            tracing::Level::INFO
        })
    } else {
        tracing::Level::INFO
    };
    tracing_subscriber::fmt().with_max_level(debug_level).init();

    info!("Running Iron Coder with options:\n{:?}", app_options);

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Iron Coder",
        native_options,
        Box::new(|cc| Box::new(iron_coder::IronCoderApp::with_options(cc, app_options))),
    )
}
