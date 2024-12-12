#[derive(Debug, thiserror::Error)]
#[error("Error: {0}")]
pub enum Error {
    #[error("Config error")]
    ConfigError,
    #[error("Invalid store")]
    InvalidStore,
    #[error("Missing mux")]
    MissingMux,
    #[error("Packet error")]
    PacketError,
    IOError(#[from] std::io::Error),
    SerialError(#[from] tokio_serial::Error),
    JSONError(#[from] serde_json::Error),
    #[error(transparent)]
    Any(#[from] anyhow::Error),
}
