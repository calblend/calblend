//! FFI client implementation

use napi::bindgen_prelude::*;
use napi::{JsObject};
use napi_derive::napi;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::models::*;

/// Token storage wrapper for JavaScript objects
pub struct JsTokenStorage {
    inner: JsObject,
}

impl JsTokenStorage {
    pub fn new(storage: JsObject) -> Self {
        Self { inner: storage }
    }
}

/// Main calendar client exposed to JavaScript
#[napi]
pub struct CalendarClient {
    // We'll implement the actual client logic later
    // For now, just the structure
    token_storage: Arc<Mutex<JsTokenStorage>>,
}

#[napi]
impl CalendarClient {
    #[napi(constructor)]
    pub fn new(token_storage: JsObject) -> Result<Self> {
        Ok(Self {
            token_storage: Arc::new(Mutex::new(JsTokenStorage::new(token_storage))),
        })
    }

    /// List all calendars accessible by the user
    #[napi]
    pub async fn list_calendars(&self, provider: CalendarSource) -> Result<Vec<Calendar>> {
        // TODO: Implement actual calendar listing
        Ok(vec![])
    }

    /// List events from a calendar
    #[napi]
    pub async fn list_events(
        &self,
        provider: CalendarSource,
        calendar_id: String,
        start_date: Option<String>,
        end_date: Option<String>,
    ) -> Result<Vec<UnifiedCalendarEvent>> {
        // TODO: Implement actual event listing
        Ok(vec![])
    }

    /// Create a new event
    #[napi]
    pub async fn create_event(
        &self,
        provider: CalendarSource,
        calendar_id: String,
        event: UnifiedCalendarEvent,
    ) -> Result<UnifiedCalendarEvent> {
        // TODO: Implement actual event creation
        Ok(event)
    }

    /// Update an existing event
    #[napi]
    pub async fn update_event(
        &self,
        provider: CalendarSource,
        calendar_id: String,
        event_id: String,
        event: UnifiedCalendarEvent,
    ) -> Result<UnifiedCalendarEvent> {
        // TODO: Implement actual event update
        Ok(event)
    }

    /// Delete an event
    #[napi]
    pub async fn delete_event(
        &self,
        provider: CalendarSource,
        calendar_id: String,
        event_id: String,
    ) -> Result<()> {
        // TODO: Implement actual event deletion
        Ok(())
    }
}

#[napi(object)]
#[derive(Debug, Clone)]
pub struct Calendar {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub is_primary: bool,
    pub can_write: bool,
    pub source: CalendarSource,
}