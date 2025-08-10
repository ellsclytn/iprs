use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("IO error: `{0}`")]
    Io(#[from] std::io::Error),
    #[error("Failed to parse `{0}` as interface")]
    AddrParse(String),
    #[error("Prefix length error: `{0}`")]
    PrefixLen(#[from] ipnet::PrefixLenError),
    #[error("Split mask ({0}) must be greater than the input prefix length ({1})")]
    SplitSmallerThanPrefixLen(u8, u8),
    #[error("Split mask cannot be greater than {0}. Supplied: {1}")]
    SplitTooBig(u8, u8),
}

/// Type alias for the standard [`Result`] type.
pub type Result<T> = std::result::Result<T, Error>;
