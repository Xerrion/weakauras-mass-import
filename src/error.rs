//! Error types for the WeakAura importer

use thiserror::Error;

#[derive(Error, Debug)]
pub enum WeakAuraError {
    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    #[error("Lua parse error: {0}")]
    LuaParseError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("File not found: {0}")]
    FileNotFound(String),
}

pub type Result<T> = std::result::Result<T, WeakAuraError>;
