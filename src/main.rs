fn main() -> eframe::Result<()> {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    tracing_subscriber::fmt::init();

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Iron Coder",
        native_options,
        Box::new(|cc| Box::new(iron_coder::IronCoderApp::new(cc))),
    )
}