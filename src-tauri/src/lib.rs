use setup::setup_app;

mod api;
mod core;
mod setup;
mod utils;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default();
    let (outg, errg) = setup::setup_logging();

    builder
        .plugin(tauri_plugin_store::Builder::new().build())
        .setup(setup_app)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    drop(outg);
    drop(errg);
}
