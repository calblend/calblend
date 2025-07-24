//! Google Calendar provider implementation

mod auth;
mod api;
mod models;
mod webhooks;

#[cfg(test)]
mod tests;

pub use auth::GoogleAuth;
pub use api::GoogleCalendarApi;
pub use webhooks::{GoogleWebhookManager, WatchChannel, PushNotification};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tracing::{debug, instrument};

use crate::{
    CalendarProvider, Result, UnifiedCalendarEvent, CalblendError,
    Calendar, FreeBusyPeriod, TokenStorage, CalblendConfig, http::HttpClient,
    cache::CalendarCache,
};

use self::models::GoogleEvent;

/// Google Calendar provider
pub struct GoogleCalendarProvider {
    auth: Arc<GoogleAuth>,
    api: Arc<GoogleCalendarApi>,
    token_storage: Arc<dyn TokenStorage>,
    webhook_manager: Option<Arc<GoogleWebhookManager>>,
    cache: Option<CalendarCache>,
}

impl GoogleCalendarProvider {
    /// Create a new Google Calendar provider
    pub fn new(
        client_id: String,
        client_secret: String,
        redirect_uri: String,
        token_storage: Arc<dyn TokenStorage>,
        config: CalblendConfig,
    ) -> Result<Self> {
        let http_client = HttpClient::new(&config)?;
        let auth = Arc::new(GoogleAuth::new(
            client_id,
            client_secret,
            redirect_uri,
            Arc::clone(&token_storage),
            http_client.clone(),
        ));
        let api = Arc::new(GoogleCalendarApi::new(
            Arc::clone(&auth),
            http_client,
        ));

        Ok(Self {
            auth,
            api,
            token_storage,
            webhook_manager: None,
            cache: Some(CalendarCache::new(60)), // 60 minute default TTL
        })
    }

    /// Enable webhook support
    pub fn with_webhook_endpoint(mut self, webhook_endpoint: String) -> Self {
        self.webhook_manager = Some(Arc::new(GoogleWebhookManager::new(
            Arc::clone(&self.auth),
            self.api.http.clone(),
            webhook_endpoint,
        )));
        self
    }

    /// Disable caching
    pub fn without_cache(mut self) -> Self {
        self.cache = None;
        self
    }

    /// Set cache TTL in minutes
    pub fn with_cache_ttl(mut self, ttl_minutes: i64) -> Self {
        self.cache = Some(CalendarCache::new(ttl_minutes));
        self
    }

    /// Get the authorization URL for OAuth flow
    pub async fn get_auth_url(&self) -> Result<String> {
        self.auth.get_authorization_url().await
    }

    /// Exchange authorization code for tokens
    pub async fn exchange_code(&self, code: String) -> Result<()> {
        self.auth.exchange_code(code).await
    }

    /// Convert Google event to unified format
    fn convert_to_unified(&self, google_event: GoogleEvent) -> UnifiedCalendarEvent {
        google_event.into_unified()
    }

    /// Watch a calendar for changes
    pub async fn watch_calendar(
        &self,
        calendar_id: &str,
        token: Option<String>,
        ttl_hours: Option<i64>,
    ) -> Result<WatchChannel> {
        let manager = self.webhook_manager
            .as_ref()
            .ok_or_else(|| CalblendError::Configuration(
                "Webhook endpoint not configured. Use with_webhook_endpoint()".to_string()
            ))?;
        
        manager.watch_calendar(calendar_id, token, ttl_hours).await
    }

    /// Stop watching a calendar
    pub async fn stop_watch(
        &self,
        channel_id: &str,
        resource_id: &str,
    ) -> Result<()> {
        let manager = self.webhook_manager
            .as_ref()
            .ok_or_else(|| CalblendError::Configuration(
                "Webhook endpoint not configured. Use with_webhook_endpoint()".to_string()
            ))?;
        
        manager.stop_watch(channel_id, resource_id).await
    }

    /// Process a webhook notification
    pub async fn process_notification(
        &self,
        notification: &PushNotification,
        expected_token: Option<&str>,
    ) -> Result<Vec<UnifiedCalendarEvent>> {
        let manager = self.webhook_manager
            .as_ref()
            .ok_or_else(|| CalblendError::Configuration(
                "Webhook endpoint not configured. Use with_webhook_endpoint()".to_string()
            ))?;

        // Verify the notification
        if !manager.verify_notification(notification, expected_token) {
            return Err(CalblendError::Authentication("Invalid webhook token".to_string()));
        }

        // Extract calendar ID from resource URI
        // Format: https://www.googleapis.com/calendar/v3/calendars/{calendarId}/events
        let calendar_id = notification.resource_uri
            .split("/calendars/")
            .nth(1)
            .and_then(|s| s.split("/events").next())
            .ok_or_else(|| CalblendError::Provider(
                "Invalid resource URI format".to_string()
            ))?;

        // For sync event, fetch recent changes
        if notification.resource_state == "sync" {
            debug!("Received sync notification for calendar: {}", calendar_id);
            return Ok(vec![]);
        }

        // Fetch recent events (last 24 hours)
        let start = Some(Utc::now() - chrono::Duration::hours(24));
        let end = Some(Utc::now() + chrono::Duration::hours(24));
        
        self.list_events(calendar_id, start, end).await
    }

    /// Check if webhook support is enabled
    pub fn has_webhook_support(&self) -> bool {
        self.webhook_manager.is_some()
    }
}

#[async_trait]
impl CalendarProvider for GoogleCalendarProvider {
    fn name(&self) -> &'static str {
        "Google Calendar"
    }
    
    #[instrument(skip(self))]
    async fn list_calendars(&self) -> Result<Vec<Calendar>> {
        debug!("Listing Google calendars");
        
        // Check cache first
        if let Some(cache) = &self.cache {
            if let Some(cached) = cache.get_calendars().await {
                debug!("Returning cached calendars");
                return Ok(cached);
            }
        }
        
        // Fetch from API
        let calendars = self.api.list_calendars().await?;
        let result: Vec<Calendar> = calendars.into_iter().map(|c| c.into()).collect();
        
        // Cache the result
        if let Some(cache) = &self.cache {
            cache.set_calendars(result.clone()).await;
        }
        
        Ok(result)
    }
    
    #[instrument(skip(self))]
    async fn list_events(
        &self,
        calendar_id: &str,
        start: Option<DateTime<Utc>>,
        end: Option<DateTime<Utc>>,
    ) -> Result<Vec<UnifiedCalendarEvent>> {
        debug!("Listing events for calendar: {}", calendar_id);
        
        // Check cache first
        if let Some(cache) = &self.cache {
            if let Some(cached) = cache.get_events(calendar_id, start, end).await {
                debug!("Returning cached events");
                return Ok(cached);
            }
        }
        
        // Fetch from API
        let events = self.api.list_events(calendar_id, start, end).await?;
        let result: Vec<UnifiedCalendarEvent> = events.into_iter()
            .map(|e| self.convert_to_unified(e))
            .collect();
        
        // Cache the result
        if let Some(cache) = &self.cache {
            cache.set_events(calendar_id, start, end, result.clone()).await;
        }
        
        Ok(result)
    }
    
    #[instrument(skip(self, event))]
    async fn create_event(
        &self,
        calendar_id: &str,
        event: UnifiedCalendarEvent,
    ) -> Result<UnifiedCalendarEvent> {
        debug!("Creating event in calendar: {}", calendar_id);
        let google_event = GoogleEvent::from_unified(&event)?;
        let created = self.api.create_event(calendar_id, google_event).await?;
        
        // Invalidate events cache for this calendar
        if let Some(cache) = &self.cache {
            cache.invalidate_events(calendar_id).await;
        }
        
        Ok(self.convert_to_unified(created))
    }
    
    #[instrument(skip(self, event))]
    async fn update_event(
        &self,
        calendar_id: &str,
        event_id: &str,
        event: UnifiedCalendarEvent,
    ) -> Result<UnifiedCalendarEvent> {
        debug!("Updating event {} in calendar: {}", event_id, calendar_id);
        let google_event = GoogleEvent::from_unified(&event)?;
        let updated = self.api.update_event(calendar_id, event_id, google_event).await?;
        
        // Invalidate events cache for this calendar
        if let Some(cache) = &self.cache {
            cache.invalidate_events(calendar_id).await;
        }
        
        Ok(self.convert_to_unified(updated))
    }
    
    #[instrument(skip(self))]
    async fn delete_event(
        &self,
        calendar_id: &str,
        event_id: &str,
    ) -> Result<()> {
        debug!("Deleting event {} from calendar: {}", event_id, calendar_id);
        self.api.delete_event(calendar_id, event_id).await?;
        
        // Invalidate events cache for this calendar
        if let Some(cache) = &self.cache {
            cache.invalidate_events(calendar_id).await;
        }
        
        Ok(())
    }
    
    #[instrument(skip(self))]
    async fn get_free_busy(
        &self,
        calendar_ids: &[String],
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<FreeBusyPeriod>> {
        debug!("Getting free/busy for {} calendars", calendar_ids.len());
        
        // Check cache first
        if let Some(cache) = &self.cache {
            if let Some(cached) = cache.get_free_busy(calendar_ids, start, end).await {
                debug!("Returning cached free/busy data");
                return Ok(cached);
            }
        }
        
        // Fetch from API
        let result = self.api.get_free_busy(calendar_ids, start, end).await?;
        
        // Cache the result
        if let Some(cache) = &self.cache {
            cache.set_free_busy(calendar_ids, start, end, result.clone()).await;
        }
        
        Ok(result)
    }
}