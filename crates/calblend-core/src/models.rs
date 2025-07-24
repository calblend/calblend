//! Unified calendar data models

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

/// Participant in an event (attendee, organizer, resource)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Participant {
    pub id: Option<String>,
    pub email: Option<String>,
    pub name: Option<String>,
    pub optional: Option<bool>,
    pub response_status: Option<ParticipantStatus>,
    #[serde(rename = "self")]
    pub is_self: Option<bool>,
    pub resource: Option<bool>,
    pub organizer: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ParticipantStatus {
    Accepted,
    Tentative,
    Declined,
    NeedsAction,
}

/// Alarm/reminder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reminder {
    pub minutes_before: i32,
    pub method: Option<ReminderMethod>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ReminderMethod {
    Popup,
    Email,
    Sms,
}

/// Conference/online-meeting link
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConferenceLink {
    pub url: Option<String>,
    pub provider: Option<String>,
}

/// Core unified event
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
    pub created: Option<DateTime<FixedOffset>>,
    pub updated: Option<DateTime<FixedOffset>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMoment {
    pub date_time: DateTime<FixedOffset>,
    pub time_zone: Option<String>,
    pub all_day: Option<bool>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum CalendarSource {
    Google,
    Outlook,
    Ios,
    Android,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum EventStatus {
    Confirmed,
    Tentative,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum EventVisibility {
    Default,
    Public,
    Private,
    Confidential,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum ShowAs {
    Busy,
    Free,
    Oof,
    WorkingElsewhere,
    Unknown,
}

impl UnifiedCalendarEvent {
    /// Create a new event with minimal required fields
    pub fn new(id: String, source: CalendarSource, start: EventMoment, end: EventMoment) -> Self {
        Self {
            id,
            source,
            calendar_id: None,
            title: None,
            description: None,
            location: None,
            color: None,
            start,
            end,
            recurrence_rule: None,
            recurrence_exceptions: None,
            organizer: None,
            attendees: None,
            status: None,
            visibility: None,
            show_as: None,
            reminders: None,
            conference: None,
            raw: None,
            created: None,
            updated: None,
        }
    }
}