//! FFI auth types

use napi_derive::napi;
use serde::{Deserialize, Serialize};

/// Token data for FFI
#[napi(object)]
#[derive(Debug, Serialize, Deserialize)]
pub struct TokenData {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<String>, // RFC3339 string
    pub token_type: String,
    pub scope: Option<String>,
}