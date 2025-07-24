//! Caching layer for calendar data

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{Calendar, UnifiedCalendarEvent, FreeBusyPeriod};

/// Cache entry with expiration
#[derive(Debug, Clone)]
struct CacheEntry<T> {
    data: T,
    expires_at: DateTime<Utc>,
}

impl<T> CacheEntry<T> {
    fn new(data: T, ttl: Duration) -> Self {
        Self {
            data,
            expires_at: Utc::now() + ttl,
        }
    }

    fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
}

/// Calendar cache implementation
#[derive(Clone)]
pub struct CalendarCache {
    calendars: Arc<RwLock<Option<CacheEntry<Vec<Calendar>>>>>,
    events: Arc<RwLock<HashMap<String, CacheEntry<Vec<UnifiedCalendarEvent>>>>>,
    free_busy: Arc<RwLock<HashMap<String, CacheEntry<Vec<FreeBusyPeriod>>>>>,
    default_ttl: Duration,
}

impl CalendarCache {
    /// Create a new cache with default TTL
    pub fn new(default_ttl_minutes: i64) -> Self {
        Self {
            calendars: Arc::new(RwLock::new(None)),
            events: Arc::new(RwLock::new(HashMap::new())),
            free_busy: Arc::new(RwLock::new(HashMap::new())),
            default_ttl: Duration::minutes(default_ttl_minutes),
        }
    }

    /// Get cached calendars
    pub async fn get_calendars(&self) -> Option<Vec<Calendar>> {
        let cache = self.calendars.read().await;
        cache.as_ref()
            .filter(|entry| !entry.is_expired())
            .map(|entry| entry.data.clone())
    }

    /// Cache calendars
    pub async fn set_calendars(&self, calendars: Vec<Calendar>) {
        let mut cache = self.calendars.write().await;
        *cache = Some(CacheEntry::new(calendars, self.default_ttl));
    }

    /// Get cached events for a calendar
    pub async fn get_events(
        &self,
        calendar_id: &str,
        start: Option<DateTime<Utc>>,
        end: Option<DateTime<Utc>>,
    ) -> Option<Vec<UnifiedCalendarEvent>> {
        let cache = self.events.read().await;
        
        // Create cache key based on calendar ID and date range
        let cache_key = format!(
            "{}_{}_{}", 
            calendar_id,
            start.map(|d| d.timestamp()).unwrap_or(0),
            end.map(|d| d.timestamp()).unwrap_or(0)
        );
        
        cache.get(&cache_key)
            .filter(|entry| !entry.is_expired())
            .map(|entry| entry.data.clone())
    }

    /// Cache events for a calendar
    pub async fn set_events(
        &self,
        calendar_id: &str,
        start: Option<DateTime<Utc>>,
        end: Option<DateTime<Utc>>,
        events: Vec<UnifiedCalendarEvent>,
    ) {
        let mut cache = self.events.write().await;
        
        let cache_key = format!(
            "{}_{}_{}", 
            calendar_id,
            start.map(|d| d.timestamp()).unwrap_or(0),
            end.map(|d| d.timestamp()).unwrap_or(0)
        );
        
        // Use shorter TTL for events (5 minutes)
        let ttl = Duration::minutes(5);
        cache.insert(cache_key, CacheEntry::new(events, ttl));
    }

    /// Invalidate events cache for a calendar
    pub async fn invalidate_events(&self, calendar_id: &str) {
        let mut cache = self.events.write().await;
        
        // Remove all entries for this calendar
        cache.retain(|key, _| !key.starts_with(calendar_id));
    }

    /// Get cached free/busy data
    pub async fn get_free_busy(
        &self,
        calendar_ids: &[String],
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Option<Vec<FreeBusyPeriod>> {
        let cache = self.free_busy.read().await;
        
        let cache_key = format!(
            "{}_{}_{}", 
            calendar_ids.join(","),
            start.timestamp(),
            end.timestamp()
        );
        
        cache.get(&cache_key)
            .filter(|entry| !entry.is_expired())
            .map(|entry| entry.data.clone())
    }

    /// Cache free/busy data
    pub async fn set_free_busy(
        &self,
        calendar_ids: &[String],
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        free_busy: Vec<FreeBusyPeriod>,
    ) {
        let mut cache = self.free_busy.write().await;
        
        let cache_key = format!(
            "{}_{}_{}", 
            calendar_ids.join(","),
            start.timestamp(),
            end.timestamp()
        );
        
        // Use shorter TTL for free/busy (5 minutes)
        let ttl = Duration::minutes(5);
        cache.insert(cache_key, CacheEntry::new(free_busy, ttl));
    }

    /// Clear all caches
    pub async fn clear_all(&self) {
        let mut calendars = self.calendars.write().await;
        let mut events = self.events.write().await;
        let mut free_busy = self.free_busy.write().await;
        
        *calendars = None;
        events.clear();
        free_busy.clear();
    }

    /// Get cache statistics
    pub async fn get_stats(&self) -> CacheStats {
        let calendars = self.calendars.read().await;
        let events = self.events.read().await;
        let free_busy = self.free_busy.read().await;
        
        CacheStats {
            has_calendars: calendars.is_some() && !calendars.as_ref().unwrap().is_expired(),
            event_entries: events.len(),
            free_busy_entries: free_busy.len(),
            total_entries: (if calendars.is_some() { 1 } else { 0 }) + events.len() + free_busy.len(),
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub has_calendars: bool,
    pub event_entries: usize,
    pub free_busy_entries: usize,
    pub total_entries: usize,
}