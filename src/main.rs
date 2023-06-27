use tracing;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    tracing_subscriber::fmt().with_max_level(tracing::Level::DEBUG).init();

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Iron Coder",
        native_options,
        Box::new(|cc| Box::new(iron_coder::IronCoderApp::new(cc))),
    )
}

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                "the_canvas_id", // hardcode it
                web_options,
                Box::new(|cc| Box::new(iron_coder::IronCoderApp::new(cc))),
            )
            .await
            .expect("failed to start eframe");
    });
}