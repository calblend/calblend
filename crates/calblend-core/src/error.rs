//! Error types for Calblend

use thiserror::Error;

pub type Result<T> = std::result::Result<T, CalblendError>;

#[derive(Error, Debug)]
pub enum CalblendError {
    #[error("Authentication failed: {0}")]
    Authentication(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),
    
    #[error("Invalid data: {0}")]
    InvalidData(String),
    
    #[error("Provider error: {0}")]
    Provider(String),
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    
    #[error("Calendar not found: {0}")]
    CalendarNotFound(String),
    
    #[error("Event not found: {0}")]
    EventNotFound(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("Token storage error: {0}")]
    TokenStorageError(String),
    
    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("HTTP error: {0}")]
    Http(String),
    
    #[error("Deserialization error: {0}")]
    Deserialization(String),
}

impl CalblendError {
    /// Check if this error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            CalblendError::NetworkError(_) | CalblendError::RateLimitExceeded
        )
    }
    
    /// Get the error code for FFI boundary
    pub fn error_code(&self) -> i32 {
        match self {
            CalblendError::Authentication(_) => 1001,
            CalblendError::PermissionDenied(_) => 1002,
            CalblendError::NetworkError(_) => 2001,
            CalblendError::InvalidData(_) => 3001,
            CalblendError::Provider(_) => 4001,
            CalblendError::RateLimitExceeded => 4002,
            CalblendError::CalendarNotFound(_) => 5001,
            CalblendError::EventNotFound(_) => 5002,
            CalblendError::SerializationError(_) => 6001,
            CalblendError::TokenStorageError(_) => 7001,
            CalblendError::UnsupportedOperation(_) => 8001,
            CalblendError::InternalError(_) => 9001,
            CalblendError::Configuration(_) => 10001,
            CalblendError::Http(_) => 10002,
            CalblendError::Deserialization(_) => 10003,
        }
    }
}