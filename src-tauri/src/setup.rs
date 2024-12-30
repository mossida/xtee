use std::str::FromStr;

use ractor::Actor;
use tracing::{Level, Subscriber};
use tracing_appender::non_blocking::{NonBlocking, NonBlockingBuilder, WorkerGuard};
use tracing_subscriber::{
    filter::Targets,
    fmt::{format::FmtSpan, writer::MakeWriterExt},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    Layer,
};

use crate::{
    api::router::{router, RouterContext},
    core::components::master::Master,
};

pub fn setup_logger<S>(
    filter: Targets,
    stdout: NonBlocking,
    stderr: NonBlocking,
) -> Box<dyn Layer<S> + Send + Sync>
where
    S: Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a> + Send + Sync,
{
    let writer = stderr.with_max_level(Level::WARN).or_else(stdout);

    #[cfg(debug_assertions)]
    {
        return tracing_subscriber::fmt::layer()
            .pretty()
            .with_file(false)
            .with_line_number(false)
            .with_thread_ids(false)
            .with_thread_names(false)
            .with_span_events(FmtSpan::NONE)
            .with_writer(writer)
            .with_filter(filter)
            .boxed();
    }

    #[cfg(not(debug_assertions))]
    {
        return tracing_subscriber::fmt::layer()
            .compact()
            .with_file(false)
            .with_line_number(false)
            .with_thread_ids(false)
            .with_thread_names(false)
            .with_span_events(FmtSpan::NONE)
            .with_writer(writer)
            .with_filter(filter)
            .boxed();
    }
}

pub fn setup_logging() -> (WorkerGuard, WorkerGuard) {
    let (stdout, stdout_guard) = NonBlockingBuilder::default()
        .lossy(true)
        .thread_name("xtee-log-stdout")
        .finish(std::io::stdout());

    let (stderr, stderr_guard) = NonBlockingBuilder::default()
        .lossy(true)
        .thread_name("xtee-log-stderr")
        .finish(std::io::stderr());

    let env = std::env::var("XTEE_LOG").unwrap_or("info".to_string());
    let level = Level::from_str(&env).unwrap_or(Level::INFO);
    let package = env!("CARGO_PKG_NAME").replace("-", "_");

    let filter = Targets::default()
        .with_target(package, level)
        .with_target("tokio", level)
        .with_target("ractor", level)
        .with_target("rspc", level);

    let logger = setup_logger(filter, stdout, stderr);

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
