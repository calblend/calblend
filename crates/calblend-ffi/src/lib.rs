//! N-API FFI bindings for Calblend

use napi::bindgen_prelude::*;
use napi_derive::napi;

mod models;
mod error;
mod client;
mod providers;
mod token_storage;
mod auth;
mod conversions;

pub use models::{
    CalendarSource, ParticipantStatus, ReminderMethod, EventStatus, 
    EventVisibility, ShowAs, Participant, Reminder, ConferenceLink,
    EventMoment, UnifiedCalendarEvent, Calendar
};
pub use error::*;
pub use client::*;
pub use providers::google::*;

/// Initialize the Calblend library (called automatically by N-API)
#[napi]
pub fn init_calblend() -> Result<()> {
    // Initialize logging if needed
    Ok(())
}