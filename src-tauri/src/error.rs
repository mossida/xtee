#[derive(Debug, thiserror::Error)]
pub enum ControllerError {
    #[error("Option mismatch")]
    OptionMismatch,
    #[error("CRC mismatch")]
    CRCMismatch,
    #[error("Packet error")]
    PacketError,
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
}
