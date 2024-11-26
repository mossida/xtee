use setup::{setup_app, setup_logging};

mod actor;
mod cmd;
mod error;
mod event;
mod filter;
mod protocol;
mod setup;
mod store;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    setup_logging();

    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .setup(setup_app)
        .invoke_handler(tauri::generate_handler![
            cmd::actuator_load,
            cmd::actuator_keep,
            cmd::actuator_move,
            cmd::actuator_stop,
            cmd::get_controllers,
            cmd::restart,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
