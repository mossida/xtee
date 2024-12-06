use ractor::Actor;
use time::macros::{format_description, offset};
use tracing::Level;
use tracing_subscriber::fmt::time::OffsetTime;

use crate::components::master::Master;

pub fn setup_logging() {
    let fmt = if cfg!(debug_assertions) {
        format_description!("[hour]:[minute]:[second].[subsecond digits:3]")
    } else {
        format_description!("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]")
    };

    let timer = OffsetTime::new(offset!(+8), fmt);

    let builder = tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .with_file(false)
        .with_line_number(false)
        .with_target(false)
        .with_timer(timer);

    if cfg!(debug_assertions) {
        builder.init();
    } else {
        builder.json().init();
    }
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
