use setup::setup_app;
use tauri::Manager;
use tracing::error;

use crate::core::store::StoreKey;

mod api;
mod core;
mod setup;
mod utils;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default();

    #[cfg(debug_assertions)]
    let (outg, errg) = setup::setup_logging();

    let result = builder
        .plugin(tauri_plugin_store::Builder::new().build())
        .on_page_load(|window, _payload| {
            let app = window.app_handle();

            if let Some(zoom) = core::store::store(&app)
                .ok()
                .and_then(|store| store.get(StoreKey::InterfaceZoom))
                .and_then(|value| value.as_f64())
            {
                let _ = window.set_zoom(zoom);
            }
        })
        .setup(setup_app)
        .run(tauri::generate_context!());

    if let Err(e) = result {
        error!("Error while running tauri application: {e}");
    }

    #[cfg(debug_assertions)]
    {
        drop(outg);
        drop(errg);
    }
}
