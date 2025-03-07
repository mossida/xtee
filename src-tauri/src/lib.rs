use setup::setup_app;
use tracing::error;

mod api;
mod core;
mod setup;
mod utils;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default();
    let (outg, errg) = setup::setup_logging();

    let result = builder
        .plugin(tauri_plugin_store::Builder::new().build())
        .setup(setup_app)
        .run(tauri::generate_context!());

    if let Err(e) = result {
        error!("Error while running tauri application: {e}");
    }

    drop(outg);
    drop(errg);
}
