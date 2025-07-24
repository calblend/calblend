//! FFI-safe model definitions that map to TypeScript

use chrono::{DateTime, FixedOffset};
use napi::bindgen_prelude::*;
use napi_derive::napi;
use serde::{Deserialize, Serialize};

// Re-export the enums and types from core with N-API attributes

#[napi]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CalendarSource {
    Google,
    Outlook,
    Ios,
    Android,
}

#[napi]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParticipantStatus {
    Accepted,
    Tentative,
    Declined,
    NeedsAction,
}

#[napi]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReminderMethod {
    Popup,
    Email,
    Sms,
}

#[napi]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventStatus {
    Confirmed,
    Tentative,
    Cancelled,
}

#[napi]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventVisibility {
    Default,
    Public,
    Private,
    Confidential,
}

#[napi]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShowAs {
    Busy,
    Free,
    Oof,
    WorkingElsewhere,
    Unknown,
}

#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Participant {
    pub id: Option<String>,
    pub email: Option<String>,
    pub name: Option<String>,
    pub optional: Option<bool>,
    pub response_status: Option<ParticipantStatus>,
    #[napi(js_name = "self")]
    pub is_self: Option<bool>,
    pub resource: Option<bool>,
    pub organizer: Option<bool>,
}

#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reminder {
    pub minutes_before: i32,
    pub method: Option<ReminderMethod>,
}

#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConferenceLink {
    pub url: Option<String>,
    pub provider: Option<String>,
}

#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMoment {
    pub date_time: String, // RFC3339 string for JS compatibility
    pub time_zone: Option<String>,
    pub all_day: Option<bool>,
}

#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedCalendarEvent {
    // Identity
    pub id: String,
    pub source: CalendarSource,
    pub calendar_id: Option<String>,

    // Content
    pub title: Option<String>,
    pub description: Option<String>,
    pub location: Option<String>,
    pub color: Option<String>,

    // Timing
    pub start: EventMoment,
    pub end: EventMoment,
    pub recurrence_rule: Option<String>,
    pub recurrence_exceptions: Option<Vec<String>>,

    // Participation
    pub organizer: Option<Participant>,
    pub attendees: Option<Vec<Participant>>,

    // Status/visibility
    pub status: Option<EventStatus>,
    pub visibility: Option<EventVisibility>,
    pub show_as: Option<ShowAs>,

    // Extras
    pub reminders: Option<Vec<Reminder>>,
    pub conference: Option<ConferenceLink>,

    // Provider metadata
    pub raw: Option<serde_json::Value>,
    pub created: Option<String>, // RFC3339 string
    pub updated: Option<String>, // RFC3339 string
}

// Conversion helpers between core types and FFI types
impl From<calblend_core::CalendarSource> for CalendarSource {
    fn from(source: calblend_core::CalendarSource) -> Self {
        match source {
            calblend_core::CalendarSource::Google => CalendarSource::Google,
            calblend_core::CalendarSource::Outlook => CalendarSource::Outlook,
            calblend_core::CalendarSource::Ios => CalendarSource::Ios,
            calblend_core::CalendarSource::Android => CalendarSource::Android,
        }
    }
}

impl From<CalendarSource> for calblend_core::CalendarSource {
    fn from(source: CalendarSource) -> Self {
        match source {
            CalendarSource::Google => calblend_core::CalendarSource::Google,
            CalendarSource::Outlook => calblend_core::CalendarSource::Outlook,
            CalendarSource::Ios => calblend_core::CalendarSource::Ios,
            CalendarSource::Android => calblend_core::CalendarSource::Android,
        }
    }
}

// Helper to convert DateTime to/from strings for FFI
pub fn datetime_to_string(dt: &DateTime<FixedOffset>) -> String {
    dt.to_rfc3339()
}

pub fn string_to_datetime(s: &str) -> Result<DateTime<FixedOffset>> {
    DateTime::parse_from_rfc3339(s)
        .map_err(|e| Error::new(Status::InvalidArg, format!("Invalid datetime: {}", e)))
}