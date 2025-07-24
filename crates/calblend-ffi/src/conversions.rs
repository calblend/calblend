//! Conversion implementations between FFI and core types

use crate::models::*;
use chrono::DateTime;

impl From<calblend_core::Calendar> for Calendar {
    fn from(cal: calblend_core::Calendar) -> Self {
        Self {
            id: cal.id,
            name: cal.name,
            description: cal.description,
            color: cal.color,
            is_primary: cal.is_primary,
            can_write: cal.can_write,
            source: cal.source.into(),
        }
    }
}

impl From<calblend_core::UnifiedCalendarEvent> for UnifiedCalendarEvent {
    fn from(event: calblend_core::UnifiedCalendarEvent) -> Self {
        Self {
            id: event.id,
            source: event.source.into(),
            calendar_id: event.calendar_id,
            title: event.title,
            description: event.description,
            location: event.location,
            color: event.color,
            start: EventMoment {
                date_time: event.start.date_time.to_rfc3339(),
                time_zone: event.start.time_zone,
                all_day: event.start.all_day,
            },
            end: EventMoment {
                date_time: event.end.date_time.to_rfc3339(),
                time_zone: event.end.time_zone,
                all_day: event.end.all_day,
            },
            recurrence_rule: event.recurrence_rule,
            recurrence_exceptions: event.recurrence_exceptions,
            organizer: event.organizer.map(|p| Participant {
                id: p.id,
                email: p.email,
                name: p.name,
                optional: p.optional,
                response_status: p.response_status.map(|s| match s {
                    calblend_core::ParticipantStatus::Accepted => ParticipantStatus::Accepted,
                    calblend_core::ParticipantStatus::Tentative => ParticipantStatus::Tentative,
                    calblend_core::ParticipantStatus::Declined => ParticipantStatus::Declined,
                    calblend_core::ParticipantStatus::NeedsAction => ParticipantStatus::NeedsAction,
                }),
                is_self: p.is_self,
                resource: p.resource,
                organizer: p.organizer,
            }),
            attendees: event.attendees.map(|attendees| {
                attendees.into_iter().map(|p| Participant {
                    id: p.id,
                    email: p.email,
                    name: p.name,
                    optional: p.optional,
                    response_status: p.response_status.map(|s| match s {
                        calblend_core::ParticipantStatus::Accepted => ParticipantStatus::Accepted,
                        calblend_core::ParticipantStatus::Tentative => ParticipantStatus::Tentative,
                        calblend_core::ParticipantStatus::Declined => ParticipantStatus::Declined,
                        calblend_core::ParticipantStatus::NeedsAction => ParticipantStatus::NeedsAction,
                    }),
                    is_self: p.is_self,
                    resource: p.resource,
                    organizer: p.organizer,
                }).collect()
            }),
            status: event.status.map(|s| match s {
                calblend_core::EventStatus::Confirmed => EventStatus::Confirmed,
                calblend_core::EventStatus::Tentative => EventStatus::Tentative,
                calblend_core::EventStatus::Cancelled => EventStatus::Cancelled,
            }),
            visibility: event.visibility.map(|v| match v {
                calblend_core::EventVisibility::Default => EventVisibility::Default,
                calblend_core::EventVisibility::Public => EventVisibility::Public,
                calblend_core::EventVisibility::Private => EventVisibility::Private,
                calblend_core::EventVisibility::Confidential => EventVisibility::Confidential,
            }),
            show_as: event.show_as.map(|s| match s {
                calblend_core::ShowAs::Busy => ShowAs::Busy,
                calblend_core::ShowAs::Free => ShowAs::Free,
                calblend_core::ShowAs::Oof => ShowAs::Oof,
                calblend_core::ShowAs::WorkingElsewhere => ShowAs::WorkingElsewhere,
                calblend_core::ShowAs::Unknown => ShowAs::Unknown,
            }),
            reminders: event.reminders.map(|reminders| {
                reminders.into_iter().map(|r| Reminder {
                    minutes_before: r.minutes_before,
                    method: r.method.map(|m| match m {
                        calblend_core::ReminderMethod::Popup => ReminderMethod::Popup,
                        calblend_core::ReminderMethod::Email => ReminderMethod::Email,
                        calblend_core::ReminderMethod::Sms => ReminderMethod::Sms,
                    }),
                }).collect()
            }),
            conference: event.conference.map(|c| ConferenceLink {
                url: c.url,
                provider: c.provider,
            }),
            raw: event.raw.map(|v| v.to_string()),
            created: event.created.map(|dt| dt.to_rfc3339()),
            updated: event.updated.map(|dt| dt.to_rfc3339()),
        }
    }
}

impl TryFrom<UnifiedCalendarEvent> for calblend_core::UnifiedCalendarEvent {
    type Error = String;

    fn try_from(event: UnifiedCalendarEvent) -> Result<Self, Self::Error> {
        let parse_datetime = |s: &str| {
            DateTime::parse_from_rfc3339(s)
                .map_err(|e| format!("Invalid datetime: {}", e))
        };

        Ok(calblend_core::UnifiedCalendarEvent {
            id: event.id,
            source: event.source.into(),
            calendar_id: event.calendar_id,
            title: event.title,
            description: event.description,
            location: event.location,
            color: event.color,
            start: calblend_core::EventMoment {
                date_time: parse_datetime(&event.start.date_time)?,
                time_zone: event.start.time_zone,
                all_day: event.start.all_day,
            },
            end: calblend_core::EventMoment {
                date_time: parse_datetime(&event.end.date_time)?,
                time_zone: event.end.time_zone,
                all_day: event.end.all_day,
            },
            recurrence_rule: event.recurrence_rule,
            recurrence_exceptions: event.recurrence_exceptions,
            organizer: event.organizer.map(|p| calblend_core::Participant {
                id: p.id,
                email: p.email,
                name: p.name,
                optional: p.optional,
                response_status: p.response_status.map(|s| match s {
                    ParticipantStatus::Accepted => calblend_core::ParticipantStatus::Accepted,
                    ParticipantStatus::Tentative => calblend_core::ParticipantStatus::Tentative,
                    ParticipantStatus::Declined => calblend_core::ParticipantStatus::Declined,
                    ParticipantStatus::NeedsAction => calblend_core::ParticipantStatus::NeedsAction,
                }),
                is_self: p.is_self,
                resource: p.resource,
                organizer: p.organizer,
            }),
            attendees: event.attendees.map(|attendees| {
                attendees.into_iter().map(|p| calblend_core::Participant {
                    id: p.id,
                    email: p.email,
                    name: p.name,
                    optional: p.optional,
                    response_status: p.response_status.map(|s| match s {
                        ParticipantStatus::Accepted => calblend_core::ParticipantStatus::Accepted,
                        ParticipantStatus::Tentative => calblend_core::ParticipantStatus::Tentative,
                        ParticipantStatus::Declined => calblend_core::ParticipantStatus::Declined,
                        ParticipantStatus::NeedsAction => calblend_core::ParticipantStatus::NeedsAction,
                    }),
                    is_self: p.is_self,
                    resource: p.resource,
                    organizer: p.organizer,
                }).collect()
            }),
            status: event.status.map(|s| match s {
                EventStatus::Confirmed => calblend_core::EventStatus::Confirmed,
                EventStatus::Tentative => calblend_core::EventStatus::Tentative,
                EventStatus::Cancelled => calblend_core::EventStatus::Cancelled,
            }),
            visibility: event.visibility.map(|v| match v {
                EventVisibility::Default => calblend_core::EventVisibility::Default,
                EventVisibility::Public => calblend_core::EventVisibility::Public,
                EventVisibility::Private => calblend_core::EventVisibility::Private,
                EventVisibility::Confidential => calblend_core::EventVisibility::Confidential,
            }),
            show_as: event.show_as.map(|s| match s {
                ShowAs::Busy => calblend_core::ShowAs::Busy,
                ShowAs::Free => calblend_core::ShowAs::Free,
                ShowAs::Oof => calblend_core::ShowAs::Oof,
                ShowAs::WorkingElsewhere => calblend_core::ShowAs::WorkingElsewhere,
                ShowAs::Unknown => calblend_core::ShowAs::Unknown,
            }),
            reminders: event.reminders.map(|reminders| {
                reminders.into_iter().map(|r| calblend_core::Reminder {
                    minutes_before: r.minutes_before,
                    method: r.method.map(|m| match m {
                        ReminderMethod::Popup => calblend_core::ReminderMethod::Popup,
                        ReminderMethod::Email => calblend_core::ReminderMethod::Email,
                        ReminderMethod::Sms => calblend_core::ReminderMethod::Sms,
                    }),
                }).collect()
            }),
            conference: event.conference.map(|c| calblend_core::ConferenceLink {
                url: c.url,
                provider: c.provider,
            }),
            raw: event.raw.and_then(|s| serde_json::from_str(&s).ok()),
            created: event.created.and_then(|s| DateTime::parse_from_rfc3339(&s).ok()),
            updated: event.updated.and_then(|s| DateTime::parse_from_rfc3339(&s).ok()),
        })
    }
}