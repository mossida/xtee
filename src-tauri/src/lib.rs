use router::{router, RouterContext};
use setup::{setup_app, setup_logging};

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
    setup_logging();

    tauri::Builder::default()
        .plugin(rspc_tauri::plugin(router().arced(), |handle| {
            RouterContext { handle }
        }))
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .setup(setup_app)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
