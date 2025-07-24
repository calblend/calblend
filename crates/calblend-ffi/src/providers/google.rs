//! Google Calendar provider FFI bindings

use napi_derive::napi;
use napi::{Error, Result, Status};
use std::sync::Arc;

use calblend_core::{
    providers::google::{GoogleCalendarProvider as CoreGoogleProvider, WatchChannel, PushNotification},
    CalblendConfig, TokenStorage, CalendarProvider,
};

use crate::models::{Calendar, UnifiedCalendarEvent};
use crate::token_storage::JsTokenStorage;

/// Google Calendar provider for Node.js
#[napi]
pub struct GoogleCalendarProvider {
    inner: Arc<CoreGoogleProvider>,
    has_webhooks: bool,
}

#[napi]
impl GoogleCalendarProvider {
    /// Create a new Google Calendar provider
    #[napi(constructor)]
    pub fn new(
        client_id: String,
        client_secret: String,
        redirect_uri: String,
        token_storage: &JsTokenStorage,
        webhook_endpoint: Option<String>,
    ) -> Result<Self> {
        let config = CalblendConfig::default()
            .with_timeout_seconds(30)
            .with_max_retries(3);

        let token_storage: Arc<dyn TokenStorage> = Arc::new(token_storage.clone());

        let mut provider = CoreGoogleProvider::new(
            client_id,
            client_secret,
            redirect_uri,
            token_storage,
            config,
        )
        .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;

        let has_webhooks = webhook_endpoint.is_some();
        
        if let Some(endpoint) = webhook_endpoint {
            provider = provider.with_webhook_endpoint(endpoint);
        }

        Ok(Self {
            inner: Arc::new(provider),
            has_webhooks,
        })
    }

    /// Get the authorization URL for OAuth flow
    #[napi]
    pub async fn get_auth_url(&self) -> Result<String> {
        self.inner
            .get_auth_url()
            .await
            .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))
    }

    /// Exchange authorization code for tokens
    #[napi]
    pub async fn exchange_code(&self, code: String) -> Result<()> {
        self.inner
            .exchange_code(code)
            .await
            .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))
    }

    /// List all calendars
    #[napi]
    pub async fn list_calendars(&self) -> Result<Vec<Calendar>> {
        let calendars = self.inner
            .list_calendars()
            .await
            .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;

        Ok(calendars.into_iter().map(Into::into).collect())
    }

    /// List events in a calendar
    #[napi]
    pub async fn list_events(
        &self,
        calendar_id: String,
        start_date: Option<String>,
        end_date: Option<String>,
    ) -> Result<Vec<UnifiedCalendarEvent>> {
        let start = start_date
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc));
        
        let end = end_date
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc));

        let events = self.inner
            .list_events(&calendar_id, start, end)
            .await
            .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;

        Ok(events.into_iter().map(Into::into).collect())
    }

    /// Create a new event
    #[napi]
    pub async fn create_event(
        &self,
        calendar_id: String,
        event: UnifiedCalendarEvent,
    ) -> Result<UnifiedCalendarEvent> {
        let core_event = event.try_into()
            .map_err(|e: String| Error::new(Status::InvalidArg, e))?;

        let created = self.inner
            .create_event(&calendar_id, core_event)
            .await
            .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;

        Ok(created.into())
    }

    /// Update an existing event
    #[napi]
    pub async fn update_event(
        &self,
        calendar_id: String,
        event_id: String,
        event: UnifiedCalendarEvent,
    ) -> Result<UnifiedCalendarEvent> {
        let core_event = event.try_into()
            .map_err(|e: String| Error::new(Status::InvalidArg, e))?;

        let updated = self.inner
            .update_event(&calendar_id, &event_id, core_event)
            .await
            .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;

        Ok(updated.into())
    }

    /// Delete an event
    #[napi]
    pub async fn delete_event(
        &self,
        calendar_id: String,
        event_id: String,
    ) -> Result<()> {
        self.inner
            .delete_event(&calendar_id, &event_id)
            .await
            .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))
    }

    /// Check if webhook support is enabled
    #[napi]
    pub fn has_webhook_support(&self) -> bool {
        self.has_webhooks
    }

    /// Watch a calendar for changes (webhooks)
    #[napi]
    pub async fn watch_calendar(
        &self,
        calendar_id: String,
        token: Option<String>,
        ttl_hours: Option<i32>,
    ) -> Result<GoogleWatchChannel> {
        if !self.has_webhooks {
            return Err(Error::new(
                Status::GenericFailure,
                "Webhook endpoint not configured"
            ));
        }

        let channel = self.inner
            .watch_calendar(&calendar_id, token, ttl_hours.map(|h| h as i64))
            .await
            .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;

        Ok(GoogleWatchChannel::from(channel))
    }

    /// Stop watching a calendar
    #[napi]
    pub async fn stop_watch(
        &self,
        channel_id: String,
        resource_id: String,
    ) -> Result<()> {
        if !self.has_webhooks {
            return Err(Error::new(
                Status::GenericFailure,
                "Webhook endpoint not configured"
            ));
        }

        self.inner
            .stop_watch(&channel_id, &resource_id)
            .await
            .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))
    }

    /// Process a webhook notification
    #[napi]
    pub async fn process_notification(
        &self,
        channel_id: String,
        channel_token: Option<String>,
        channel_expiration: Option<String>,
        resource_id: String,
        resource_state: String,
        resource_uri: String,
        message_number: Option<String>,
        expected_token: Option<String>,
    ) -> Result<Vec<UnifiedCalendarEvent>> {
        if !self.has_webhooks {
            return Err(Error::new(
                Status::GenericFailure,
                "Webhook endpoint not configured"
            ));
        }

        let notification = PushNotification {
            channel_id,
            channel_token,
            channel_expiration,
            resource_id,
            resource_state,
            resource_uri,
            message_number,
        };

        let events = self.inner
            .process_notification(&notification, expected_token.as_deref())
            .await
            .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;

        Ok(events.into_iter().map(Into::into).collect())
    }
}

/// Google Calendar watch channel for webhooks
#[napi(object)]
#[derive(Debug, Clone)]
pub struct GoogleWatchChannel {
    pub id: String,
    pub resource_id: String,
    pub resource_uri: String,
    pub token: Option<String>,
    pub expiration: String,
}

impl From<WatchChannel> for GoogleWatchChannel {
    fn from(channel: WatchChannel) -> Self {
        Self {
            id: channel.id,
            resource_id: channel.resource_id,
            resource_uri: channel.resource_uri,
            token: channel.token,
            expiration: channel.expiration.to_rfc3339(),
        }
    }
}