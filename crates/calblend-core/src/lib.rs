//! Calblend Core - Unified calendar integration library
//!
//! This crate provides the core functionality for integrating with multiple
//! calendar providers (Google, Outlook, iOS, Android) through a unified API.

pub mod models;
pub mod providers;
pub mod error;
pub mod auth;
pub mod sync;

pub use models::*;
pub use error::{CalblendError, Result};
pub use auth::TokenStorage;

use async_trait::async_trait;
use chrono::{DateTime, Utc};

/// Main trait that all calendar providers must implement
#[async_trait]
pub trait CalendarProvider: Send + Sync {
    /// Get provider name
    fn name(&self) -> &'static str;
    
    /// List calendars accessible by the user
    async fn list_calendars(&self) -> Result<Vec<Calendar>>;
    
    /// Get events from a specific calendar
    async fn list_events(
        &self,
        calendar_id: &str,
        start: Option<DateTime<Utc>>,
        end: Option<DateTime<Utc>>,
    ) -> Result<Vec<UnifiedCalendarEvent>>;
    
    /// Create a new event
    async fn create_event(
        &self,
        calendar_id: &str,
        event: UnifiedCalendarEvent,
    ) -> Result<UnifiedCalendarEvent>;
    
    /// Update an existing event
    async fn update_event(
        &self,
        calendar_id: &str,
        event_id: &str,
        event: UnifiedCalendarEvent,
    ) -> Result<UnifiedCalendarEvent>;
    
    /// Delete an event
    async fn delete_event(
        &self,
        calendar_id: &str,
        event_id: &str,
    ) -> Result<()>;
    
    /// Get free/busy information
    async fn get_free_busy(
        &self,
        calendar_ids: &[String],
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<FreeBusyPeriod>>;
}

/// Calendar metadata
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Calendar {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub is_primary: bool,
    pub can_write: bool,
    pub source: CalendarSource,
}

/// Free/busy time period
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FreeBusyPeriod {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub status: BusyStatus,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum BusyStatus {
    Free,
    Busy,
    Tentative,
    OutOfOffice,
}

/// Client configuration
#[derive(Debug, Clone)]
pub struct CalblendConfig {
    pub user_agent: String,
    pub timeout_secs: u64,
    pub max_retries: u32,
}

impl Default for CalblendConfig {
    fn default() -> Self {
        Self {
            user_agent: format!("Calblend/{}", env!("CARGO_PKG_VERSION")),
            timeout_secs: 30,
            max_retries: 3,
        }
    }
}