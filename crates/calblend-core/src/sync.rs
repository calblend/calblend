//! Synchronization and caching functionality

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{CalendarSource, UnifiedCalendarEvent};

/// Sync token for incremental synchronization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncToken {
    pub provider: CalendarSource,
    pub calendar_id: String,
    pub token: String,
    pub last_sync: DateTime<Utc>,
}

/// Sync status for a calendar
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatus {
    pub calendar_id: String,
    pub last_sync: Option<DateTime<Utc>>,
    pub sync_token: Option<String>,
    pub events_synced: usize,
    pub errors: Vec<String>,
}

/// Sync configuration
#[derive(Debug, Clone)]
pub struct SyncConfig {
    /// Enable incremental sync where supported
    pub incremental: bool,
    /// Sync window in days (past and future)
    pub window_days: i64,
    /// Maximum events per sync batch
    pub batch_size: usize,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            incremental: true,
            window_days: 365,
            batch_size: 100,
        }
    }
}

/// Cache for event data
#[derive(Debug, Default)]
pub struct EventCache {
    events: HashMap<String, UnifiedCalendarEvent>,
    last_update: HashMap<String, DateTime<Utc>>,
}

impl EventCache {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn insert(&mut self, event: UnifiedCalendarEvent) {
        let key = format!("{}:{}", event.calendar_id.as_deref().unwrap_or(""), event.id);
        self.last_update.insert(key.clone(), Utc::now());
        self.events.insert(key, event);
    }
    
    pub fn get(&self, calendar_id: &str, event_id: &str) -> Option<&UnifiedCalendarEvent> {
        let key = format!("{}:{}", calendar_id, event_id);
        self.events.get(&key)
    }
    
    pub fn remove(&mut self, calendar_id: &str, event_id: &str) -> Option<UnifiedCalendarEvent> {
        let key = format!("{}:{}", calendar_id, event_id);
        self.last_update.remove(&key);
        self.events.remove(&key)
    }
    
    pub fn clear(&mut self) {
        self.events.clear();
        self.last_update.clear();
    }
}