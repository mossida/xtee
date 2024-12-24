use ractor::Actor;

use crate::{
    api::router::{router, RouterContext},
    core::components::master::Master,
};

#[cfg(not(debug_assertions))]
pub fn setup_logging() {
    let builder = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_file(false)
        .with_line_number(false)
        .with_target(false);

    builder.json().init();
}

pub fn setup_app(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let app_handle = app.handle().to_owned();

    tauri::async_runtime::spawn(async move {
        let (actor, handle) = Actor::spawn(Some("master".to_owned()), Master, app_handle.clone())
            .await
            .expect("Failed to spawn master");

        // Inject the master actor into the router
        app_handle
            .plugin(rspc_tauri::plugin(router().arced(), move |_| {
                RouterContext {
                    master: actor.clone(),
                }
            }))
            .expect("Failed to plugin rspc");

        handle.await.expect("Master failed");
    });

    Ok(())
}
