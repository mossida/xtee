use ractor::Actor;
use tracing::Level;
use tracing_appender::non_blocking::{NonBlockingBuilder, WorkerGuard};
use tracing_subscriber::{fmt::format::FmtSpan, prelude::*, EnvFilter};

use crate::{
    api::router::{router, RouterContext},
    core::components::master::Master,
};

pub fn setup_logging() -> (WorkerGuard, WorkerGuard) {
    let (stdout, stdout_guard) = NonBlockingBuilder::default()
        .lossy(true)
        .thread_name("xtee-log-stdout")
        .finish(std::io::stdout());

    let (stderr, stderr_guard) = NonBlockingBuilder::default()
        .lossy(true)
        .thread_name("xtee-log-stderr")
        .finish(std::io::stderr());

    let filter = EnvFilter::new("xtee_lib");
    let writer = stderr.with_max_level(Level::WARN).or_else(stdout);
    let logger = tracing_subscriber::fmt::layer()
        .compact()
        .with_ansi(true)
        .with_file(false)
        .with_target(true)
        .with_line_number(false)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_span_events(FmtSpan::NONE)
        .with_writer(writer)
        .with_filter(filter);

    tracing_subscriber::registry().with(logger).init();

    (stdout_guard, stderr_guard)
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
