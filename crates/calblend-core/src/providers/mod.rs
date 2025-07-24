//! Calendar provider implementations

pub mod google;
pub mod outlook;

// Conditional compilation for mobile platforms
#[cfg(target_os = "ios")]
pub mod ios;

#[cfg(target_os = "android")]
pub mod android;