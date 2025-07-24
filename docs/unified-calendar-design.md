# Unified Calendar Object Design

This document defines the core data model for Calblend's unified calendar API.

## Core Design Principles

1. **Single Unified Type**: One `UnifiedCalendarEvent` type that works across all providers
2. **FFI-Ready**: Uses `#[napi(object)]` for automatic TypeScript generation
3. **Lossless**: Preserves provider-specific data in `raw` field
4. **Type-Safe**: Full type safety from Rust through to TypeScript

## Rust Implementation

```rust
use chrono::{DateTime, FixedOffset};
use napi_derive::napi;
use serde::{Deserialize, Serialize};

/// Participant in an event (attendee, organiser, resource…)
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Participant {
    pub id:      Option<String>,
    pub email:   Option<String>,
    pub name:    Option<String>,
    pub optional: Option<bool>,
    pub response_status: Option<ParticipantStatus>,
    pub r#self:  Option<bool>,
    pub resource: Option<bool>,
    pub organizer: Option<bool>,
}

#[napi]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParticipantStatus {
    Accepted,
    Tentative,
    Declined,
    NeedsAction,
}

/// Alarm / reminder
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reminder {
    pub minutes_before: i32,
    pub method: Option<ReminderMethod>,
}

#[napi]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReminderMethod {
    Popup,
    Email,
    Sms,
}

/// Conference / online-meeting link
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConferenceLink {
    pub url: Option<String>,
    pub provider: Option<String>, // "googleMeet" | "teams" | "zoom" | …
}

/// Core unified event
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedCalendarEvent {
    // ---------- identity ----------
    pub id: String,
    pub source: CalendarSource,
    pub calendar_id: Option<String>,

    // ---------- content ----------
    pub title:       Option<String>,
    pub description: Option<String>,
    pub location:    Option<String>,
    pub color:       Option<String>,

    // ---------- timing ----------
    pub start: EventMoment,
    pub end:   EventMoment,
    pub recurrence_rule:       Option<String>,
    pub recurrence_exceptions: Option<Vec<String>>,

    // ---------- participation ----------
    pub organizer: Option<Participant>,
    pub attendees: Option<Vec<Participant>>,

    // ---------- status / visibility ----------
    pub status:     Option<EventStatus>,
    pub visibility: Option<EventVisibility>,
    pub show_as:    Option<ShowAs>,

    // ---------- extras ----------
    pub reminders:  Option<Vec<Reminder>>,
    pub conference: Option<ConferenceLink>,

    // ---------- provider metadata ----------
    pub raw:     Option<serde_json::Value>,
    pub created: Option<DateTime<FixedOffset>>,
    pub updated: Option<DateTime<FixedOffset>>,
}

#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMoment {
    pub date_time: DateTime<FixedOffset>,
    pub time_zone: Option<String>,
    pub all_day:   Option<bool>,
}

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
```

## Provider Mapping Reference

| Unified field | Google Calendar API v3 | Microsoft Graph | Apple EventKit | Android CalendarContract |
|--------------|------------------------|-----------------|----------------|-------------------------|
| id | id | id | eventIdentifier | _ID |
| calendarId | calendarId | calendar.id | calendar.calendarIdentifier | CALENDAR_ID |
| title | summary | subject | title | TITLE |
| description | description | body.content | notes | DESCRIPTION |
| location | location | location.displayName | location | EVENT_LOCATION |
| color | colorId | categories[0] | calendar.cgColor | EVENT_COLOR_KEY |
| start.* | start.{dateTime,date} | start.dateTime | startDate | DTSTART |
| end.* | end.{dateTime,date} | end.dateTime | endDate | DTEND |
| recurrenceRule | recurrence[0] | recurrence.pattern* | recurrenceRules[].rruleString | RRULE |
| attendees[] | attendees[] | attendees[] | attendees[] | Attendees table |
| status | status | responseStatus.response | status | STATUS_* |
| visibility | visibility | sensitivity | – | ACCESS_LEVEL |
| showAs | transparency | showAs | availability | AVAILABILITY |
| reminders | reminders.overrides[] | reminderMinutesBeforeStart | alarms[] | Reminders table |
| conference.url | hangoutLink | onlineMeetingUrl | custom | custom |
| created | created | createdDateTime | creationDate | CREATED |
| updated | updated | lastModifiedDateTime | lastModifiedDate | LAST_DATE |

## Key Benefits

1. **Consistent API**: Single interface for all calendar providers
2. **Type Safety**: Automatic TypeScript generation via napi-rs
3. **No Data Loss**: Provider-specific data preserved in `raw` field
4. **FFI Optimized**: Zero-copy strings, efficient enum representation
5. **Extensible**: Easy to add new fields or providers

## Usage Example

```typescript
// Creating an event
const event: UnifiedCalendarEvent = {
  id: generateId(),
  source: 'Google',
  title: 'Team Meeting',
  start: {
    dateTime: '2024-01-15T10:00:00-08:00',
    allDay: false
  },
  end: {
    dateTime: '2024-01-15T11:00:00-08:00',
    allDay: false
  },
  attendees: [{
    email: 'team@example.com',
    responseStatus: 'NeedsAction'
  }]
};

// Accessing provider-specific data
if (event.source === 'Google' && event.raw) {
  const googleSpecific = event.raw as GoogleEventData;
  console.log(googleSpecific.colorId);
}
```