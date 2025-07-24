//! Error handling for FFI boundary

use napi::bindgen_prelude::*;

/// Convert Calblend errors to N-API errors
pub fn to_napi_error(err: calblend_core::CalblendError) -> Error {
    let message = err.to_string();
    
    let status = match &err {
        calblend_core::CalblendError::Authentication(_) => Status::GenericFailure,
        calblend_core::CalblendError::PermissionDenied(_) => Status::GenericFailure,
        calblend_core::CalblendError::InvalidData(_) => Status::InvalidArg,
        calblend_core::CalblendError::CalendarNotFound(_) => Status::ObjectExpected,
        calblend_core::CalblendError::EventNotFound(_) => Status::ObjectExpected,
        _ => Status::GenericFailure,
    };
    
    Error::new(status, message)
}