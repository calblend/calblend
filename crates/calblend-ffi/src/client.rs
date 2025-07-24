//! FFI client implementation

use napi::bindgen_prelude::*;
use napi::JsObject;
use napi_derive::napi;
use std::sync::Arc;

use crate::models::*;
use crate::token_storage::JsTokenStorage;

/// Main calendar client exposed to JavaScript
#[napi]
pub struct CalendarClient {
    // We'll implement the actual client logic later
    // For now, just the structure
    #[allow(dead_code)]
    token_storage: Arc<JsTokenStorage>,
}

#[napi]
impl CalendarClient {
    #[napi(constructor)]
    pub fn new(_token_storage: JsObject) -> Result<Self> {
        // For now, use the in-memory storage until we implement JS bridge
        Ok(Self {
            token_storage: Arc::new(JsTokenStorage::new()),
        })
    }

    /// List all calendars accessible by the user
    #[napi]
    pub async fn list_calendars(&self, _provider: CalendarSource) -> Result<Vec<Calendar>> {
        // TODO: Implement actual calendar listing
        Ok(vec![])
    }

    /// List events from a calendar
    #[napi]
    pub async fn list_events(
        &self,
        _provider: CalendarSource,
        _calendar_id: String,
        _start_date: Option<String>,
        _end_date: Option<String>,
    ) -> Result<Vec<UnifiedCalendarEvent>> {
        // TODO: Implement actual event listing
        Ok(vec![])
    }

    /// Create a new event
    #[napi]
    pub async fn create_event(
        &self,
        _provider: CalendarSource,
        _calendar_id: String,
        event: UnifiedCalendarEvent,
    ) -> Result<UnifiedCalendarEvent> {
        // TODO: Implement actual event creation
        Ok(event)
    }

    /// Update an existing event
    #[napi]
    pub async fn update_event(
        &self,
        _provider: CalendarSource,
        _calendar_id: String,
        _event_id: String,
        event: UnifiedCalendarEvent,
    ) -> Result<UnifiedCalendarEvent> {
        // TODO: Implement actual event update
        Ok(event)
    }

    /// Delete an event
    #[napi]
    pub async fn delete_event(
        &self,
        _provider: CalendarSource,
        _calendar_id: String,
        _event_id: String,
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