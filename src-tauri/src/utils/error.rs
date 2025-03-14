#[derive(Debug, thiserror::Error)]
#[error("Error: {0}")]
pub enum Error {
    #[error("Config error")]
    Config,
    #[error("Ancestor missing")]
    MissingAncestor,
    #[error("Invalid store")]
    InvalidStore,
    #[error("Missing mux")]
    MissingMux,
    #[error("Packet error")]
    Packet,
    #[error("Timeout: {0}")]
    Timeout(String),
    IO(#[from] std::io::Error),
    Serial(#[from] tokio_serial::Error),
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Any(#[from] anyhow::Error),
}
