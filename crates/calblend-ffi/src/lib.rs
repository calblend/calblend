//! N-API FFI bindings for Calblend

use napi::bindgen_prelude::*;
use napi_derive::napi;

mod models;
mod error;
mod client;

pub use models::*;
pub use error::*;
pub use client::*;

/// Initialize the Calblend library (called automatically by N-API)
#[napi]
pub fn init_calblend() -> Result<()> {
    // Initialize logging if needed
    Ok(())
}