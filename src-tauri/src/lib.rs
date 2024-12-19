use router::{router, RouterContext};
use setup::setup_app;

mod cmd;
mod components;
mod error;
mod filter;
mod protocol;
mod router;
mod setup;
mod store;
mod tuner;

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
        .plugin(rspc_tauri::plugin(router().arced(), |handle| {
            RouterContext { handle }
        }))
        .plugin(tauri_plugin_store::Builder::new().build())
        .setup(setup_app)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
