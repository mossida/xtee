use ractor::Actor;

use crate::components::master::Master;

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
    let handle = app.handle().to_owned();

    tauri::async_runtime::spawn(async move {
        let (_, handle) = Actor::spawn(Some("master".to_owned()), Master, handle)
            .await
            .expect("Failed to spawn master");

        handle.await.expect("Master failed");
    });

    Ok(())
}
