use thiserror::Error;

#[derive(Debug, Error)]
pub enum PluginApiError {
    #[error("API version mismatch: expected version {expected}, found {actual}")]
    ApiVersionMismatch { expected: u32, actual: u32 },

    #[error("Adapter does not handle path or URI: {0}")]
    UnsupportedPathOrUri(String),

    #[error("Failed to instantiate plugin for '{0}' (adapter returned null)")]
    InstantiationFailed(String),

    #[error("Invalid null-terminated C string: {0}")]
    InvalidCString(#[from] std::ffi::NulError),

    #[error("UTF-8 decoding error in plugin metadata: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),

    #[error("Unexpected null pointer encountered in C-ABI structure '{0}'")]
    NullPointer(&'static str),
}
