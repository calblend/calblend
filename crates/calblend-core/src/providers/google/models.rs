//! Google Calendar API models

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{
    Calendar, CalendarSource, ConferenceLink, EventMoment, EventStatus,
    EventVisibility, Participant, ParticipantStatus, Reminder, ReminderMethod, Result,
    ShowAs, UnifiedCalendarEvent,
};

/// Google Calendar representation
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GoogleCalendar {
    pub id: String,
    pub summary: String,
    pub description: Option<String>,
    #[serde(rename = "backgroundColor")]
    pub background_color: Option<String>,
    pub primary: Option<bool>,
    #[serde(rename = "accessRole")]
    pub access_role: String,
}

impl From<GoogleCalendar> for Calendar {
    fn from(gc: GoogleCalendar) -> Self {
        Self {
            id: gc.id,
            name: gc.summary,
            description: gc.description,
            color: gc.background_color,
            is_primary: gc.primary.unwrap_or(false),
            can_write: matches!(gc.access_role.as_str(), "owner" | "writer"),
            source: CalendarSource::Google,
        }
    }
}

/// Google Event representation
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GoogleEvent {
    pub id: Option<String>,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub location: Option<String>,
    #[serde(rename = "colorId")]
    pub color_id: Option<String>,
    pub start: Option<GoogleEventTime>,
    pub end: Option<GoogleEventTime>,
    pub recurrence: Option<Vec<String>>,
    #[serde(rename = "recurringEventId")]
    pub recurring_event_id: Option<String>,
    pub status: Option<String>,
    pub visibility: Option<String>,
    pub transparency: Option<String>,
    pub creator: Option<GooglePerson>,
    pub organizer: Option<GooglePerson>,
    pub attendees: Option<Vec<GoogleAttendee>>,
    pub reminders: Option<GoogleReminders>,
    #[serde(rename = "conferenceData")]
    pub conference_data: Option<GoogleConferenceData>,
    pub created: Option<String>,
    pub updated: Option<String>,
    #[serde(rename = "htmlLink")]
    pub html_link: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GoogleEventTime {
    #[serde(rename = "dateTime")]
    pub date_time: Option<String>,
    pub date: Option<String>,
    #[serde(rename = "timeZone")]
    pub time_zone: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GooglePerson {
    pub email: Option<String>,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    #[serde(rename = "self")]
    pub is_self: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GoogleAttendee {
    pub email: Option<String>,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    pub optional: Option<bool>,
    #[serde(rename = "responseStatus")]
    pub response_status: Option<String>,
    #[serde(rename = "self")]
    pub is_self: Option<bool>,
    pub resource: Option<bool>,
    pub organizer: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GoogleReminders {
    #[serde(rename = "useDefault")]
    pub use_default: Option<bool>,
    pub overrides: Option<Vec<GoogleReminder>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GoogleReminder {
    pub method: String,
    pub minutes: i32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GoogleConferenceData {
    #[serde(rename = "entryPoints")]
    pub entry_points: Option<Vec<GoogleEntryPoint>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GoogleEntryPoint {
    #[serde(rename = "entryPointType")]
    pub entry_point_type: String,
    pub uri: String,
}

/// Free/busy request
#[derive(Debug, Serialize)]
pub struct GoogleFreeBusyRequest {
    #[serde(rename = "timeMin")]
    pub time_min: String,
    #[serde(rename = "timeMax")]
    pub time_max: String,
    pub items: Vec<GoogleFreeBusyItem>,
}

#[derive(Debug, Serialize)]
pub struct GoogleFreeBusyItem {
    pub id: String,
}

/// Free/busy response
#[derive(Debug, Deserialize)]
pub struct GoogleFreeBusyResponse {
    pub calendars: HashMap<String, CalendarFreeBusy>,
}

#[derive(Debug, Deserialize)]
pub struct CalendarFreeBusy {
    pub busy: Vec<TimePeriod>,
}

#[derive(Debug, Deserialize)]
pub struct TimePeriod {
    pub start: String,
    pub end: String,
}

impl GoogleEvent {
    /// Convert from unified format to Google format
    pub fn from_unified(event: &UnifiedCalendarEvent) -> Result<Self> {
        Ok(Self {
            id: Some(event.id.clone()),
            summary: event.title.clone(),
            description: event.description.clone(),
            location: event.location.clone(),
            color_id: None, // TODO: Map color to colorId
            start: Some(GoogleEventTime {
                date_time: Some(event.start.date_time.to_rfc3339()),
                date: None,
                time_zone: event.start.time_zone.clone(),
            }),
            end: Some(GoogleEventTime {
                date_time: Some(event.end.date_time.to_rfc3339()),
                date: None,
                time_zone: event.end.time_zone.clone(),
            }),
            recurrence: event.recurrence_rule.as_ref().map(|r| vec![format!("RRULE:{}", r)]),
            recurring_event_id: None,
            status: event.status.as_ref().map(|s| match s {
                EventStatus::Confirmed => "confirmed",
                EventStatus::Tentative => "tentative",
                EventStatus::Cancelled => "cancelled",
            }.to_string()),
            visibility: event.visibility.as_ref().map(|v| match v {
                EventVisibility::Default => "default",
                EventVisibility::Public => "public",
                EventVisibility::Private => "private",
                EventVisibility::Confidential => "confidential",
            }.to_string()),
            transparency: event.show_as.as_ref().map(|s| match s {
                ShowAs::Busy => "opaque",
                ShowAs::Free => "transparent",
                _ => "opaque",
            }.to_string()),
            creator: None,
            organizer: event.organizer.as_ref().map(|p| GooglePerson {
                email: p.email.clone(),
                display_name: p.name.clone(),
                is_self: p.is_self,
            }),
            attendees: event.attendees.as_ref().map(|attendees| {
                attendees.iter().map(|a| GoogleAttendee {
                    email: a.email.clone(),
                    display_name: a.name.clone(),
                    optional: a.optional,
                    response_status: a.response_status.as_ref().map(|s| match s {
                        ParticipantStatus::Accepted => "accepted",
                        ParticipantStatus::Tentative => "tentative",
                        ParticipantStatus::Declined => "declined",
                        ParticipantStatus::NeedsAction => "needsAction",
                    }.to_string()),
                    is_self: a.is_self,
                    resource: a.resource,
                    organizer: a.organizer,
                }).collect()
            }),
            reminders: event.reminders.as_ref().map(|reminders| GoogleReminders {
                use_default: Some(false),
                overrides: Some(reminders.iter().map(|r| GoogleReminder {
                    method: match &r.method {
                        Some(ReminderMethod::Email) => "email",
                        Some(ReminderMethod::Sms) => "sms",
                        _ => "popup",
                    }.to_string(),
                    minutes: r.minutes_before,
                }).collect()),
            }),
            conference_data: event.conference.as_ref().and_then(|c| c.url.as_ref().map(|url| {
                GoogleConferenceData {
                    entry_points: Some(vec![GoogleEntryPoint {
                        entry_point_type: "video".to_string(),
                        uri: url.clone(),
                    }]),
                }
            })),
            created: None,
            updated: None,
            html_link: None,
        })
    }

    /// Convert to unified format
    pub fn into_unified(self) -> UnifiedCalendarEvent {
        let parse_time = |time: Option<GoogleEventTime>| -> EventMoment {
            if let Some(t) = time {
                if let Some(date_time) = t.date_time {
                    EventMoment {
                        date_time: DateTime::parse_from_rfc3339(&date_time)
                            .unwrap_or_else(|_| DateTime::<FixedOffset>::from(chrono::Utc::now())),
                        time_zone: t.time_zone,
                        all_day: Some(false),
                    }
                } else if let Some(date) = t.date {
                    // All-day event
                    EventMoment {
                        date_time: DateTime::parse_from_str(&format!("{}T00:00:00Z", date), "%Y-%m-%dT%H:%M:%SZ")
                            .unwrap_or_else(|_| DateTime::<FixedOffset>::from(chrono::Utc::now())),
                        time_zone: t.time_zone,
                        all_day: Some(true),
                    }
                } else {
                    EventMoment {
                        date_time: DateTime::<FixedOffset>::from(chrono::Utc::now()),
                        time_zone: None,
                        all_day: Some(false),
                    }
                }
            } else {
                EventMoment {
                    date_time: DateTime::<FixedOffset>::from(chrono::Utc::now()),
                    time_zone: None,
                    all_day: Some(false),
                }
            }
        };

        UnifiedCalendarEvent {
            id: self.id.clone().unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            source: CalendarSource::Google,
            calendar_id: None,
            title: self.summary.clone(),
            description: self.description.clone(),
            location: self.location.clone(),
            color: self.color_id.clone(),
            start: parse_time(self.start.clone()),
            end: parse_time(self.end.clone()),
            recurrence_rule: self.recurrence.as_ref().and_then(|rules| {
                rules.first().map(|r| r.strip_prefix("RRULE:").unwrap_or(r).to_string())
            }),
            recurrence_exceptions: None,
            organizer: self.organizer.as_ref().map(|p| Participant {
                id: None,
                email: p.email.clone(),
                name: p.display_name.clone(),
                optional: Some(false),
                response_status: None,
                is_self: p.is_self,
                resource: Some(false),
                organizer: Some(true),
            }),
            attendees: self.attendees.as_ref().map(|attendees| {
                attendees.iter().map(|a| Participant {
                    id: None,
                    email: a.email.clone(),
                    name: a.display_name.clone(),
                    optional: a.optional,
                    response_status: a.response_status.as_ref().and_then(|s| match s.as_str() {
                        "accepted" => Some(ParticipantStatus::Accepted),
                        "tentative" => Some(ParticipantStatus::Tentative),
                        "declined" => Some(ParticipantStatus::Declined),
                        "needsAction" => Some(ParticipantStatus::NeedsAction),
                        _ => None,
                    }),
                    is_self: a.is_self,
                    resource: a.resource,
                    organizer: a.organizer,
                }).collect()
            }),
            status: self.status.as_ref().and_then(|s| match s.as_str() {
                "confirmed" => Some(EventStatus::Confirmed),
                "tentative" => Some(EventStatus::Tentative),
                "cancelled" => Some(EventStatus::Cancelled),
                _ => None,
            }),
            visibility: self.visibility.as_ref().and_then(|v| match v.as_str() {
                "default" => Some(EventVisibility::Default),
                "public" => Some(EventVisibility::Public),
                "private" => Some(EventVisibility::Private),
                "confidential" => Some(EventVisibility::Confidential),
                _ => None,
            }),
            show_as: self.transparency.as_ref().and_then(|t| match t.as_str() {
                "transparent" => Some(ShowAs::Free),
                "opaque" => Some(ShowAs::Busy),
                _ => None,
            }),
            reminders: self.reminders.as_ref().and_then(|r| r.overrides.as_ref().map(|overrides| {
                overrides.iter().map(|o| Reminder {
                    minutes_before: o.minutes,
                    method: match o.method.as_str() {
                        "email" => Some(ReminderMethod::Email),
                        "sms" => Some(ReminderMethod::Sms),
                        _ => Some(ReminderMethod::Popup),
                    },
                }).collect()
            })),
            conference: self.conference_data.as_ref().and_then(|cd| {
                cd.entry_points.as_ref().and_then(|eps| eps.first().map(|ep| ConferenceLink {
                    url: Some(ep.uri.clone()),
                    provider: Some("Google Meet".to_string()),
                }))
            }),
            raw: serde_json::to_value(&self).ok(),
            created: self.created.as_ref().and_then(|c| DateTime::parse_from_rfc3339(c).ok()),
            updated: self.updated.as_ref().and_then(|u| DateTime::parse_from_rfc3339(u).ok()),
        }
    }
}