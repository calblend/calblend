//! Google Calendar provider implementation

mod auth;
mod api;
mod models;

#[cfg(test)]
mod tests;

pub use auth::GoogleAuth;
pub use api::GoogleCalendarApi;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tracing::{debug, instrument};

use crate::{
    CalendarProvider, Result, UnifiedCalendarEvent,
    Calendar, FreeBusyPeriod, TokenStorage, CalblendConfig, http::HttpClient,
};

use self::models::GoogleEvent;

/// Google Calendar provider
pub struct GoogleCalendarProvider {
    auth: Arc<GoogleAuth>,
    api: Arc<GoogleCalendarApi>,
    token_storage: Arc<dyn TokenStorage>,
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
        })
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
}

#[async_trait]
impl CalendarProvider for GoogleCalendarProvider {
    fn name(&self) -> &'static str {
        "Google Calendar"
    }
    
    #[instrument(skip(self))]
    async fn list_calendars(&self) -> Result<Vec<Calendar>> {
        debug!("Listing Google calendars");
        let calendars = self.api.list_calendars().await?;
        Ok(calendars.into_iter().map(|c| c.into()).collect())
    }
    
    #[instrument(skip(self))]
    async fn list_events(
        &self,
        calendar_id: &str,
        start: Option<DateTime<Utc>>,
        end: Option<DateTime<Utc>>,
    ) -> Result<Vec<UnifiedCalendarEvent>> {
        debug!("Listing events for calendar: {}", calendar_id);
        let events = self.api.list_events(calendar_id, start, end).await?;
        Ok(events.into_iter().map(|e| self.convert_to_unified(e)).collect())
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
        Ok(self.convert_to_unified(updated))
    }
    
    #[instrument(skip(self))]
    async fn delete_event(
        &self,
        calendar_id: &str,
        event_id: &str,
    ) -> Result<()> {
        debug!("Deleting event {} from calendar: {}", event_id, calendar_id);
        self.api.delete_event(calendar_id, event_id).await
    }
    
    #[instrument(skip(self))]
    async fn get_free_busy(
        &self,
        calendar_ids: &[String],
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<FreeBusyPeriod>> {
        debug!("Getting free/busy for {} calendars", calendar_ids.len());
        self.api.get_free_busy(calendar_ids, start, end).await
    }
}