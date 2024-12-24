use setup::setup_app;

mod api;
mod core;
mod setup;
mod utils;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(debug_assertions)]
    let devtools = tauri_plugin_devtools::init();
    let mut builder = tauri::Builder::default();

    #[cfg(debug_assertions)]
    {
        builder = builder.plugin(devtools);
    }

    #[cfg(not(debug_assertions))]
    {
        setup::setup_logging();
    }

    builder
        .plugin(tauri_plugin_store::Builder::new().build())
        .setup(setup_app)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
