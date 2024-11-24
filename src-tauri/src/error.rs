#[derive(Debug, thiserror::Error)]
pub enum ControllerError {
    #[error("Config error")]
    ConfigError,
    #[error("Packet error")]
    PacketError,
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("Serial error: {0}")]
    SerialError(#[from] tokio_serial::Error),
    #[error("Generic error: {0}")]
    Any(#[from] anyhow::Error),
}
