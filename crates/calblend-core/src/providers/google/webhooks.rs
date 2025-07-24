//! Google Calendar webhooks/push notifications support

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info, warn, instrument};
use uuid::Uuid;
use http::HeaderMap;

use crate::{CalblendError, Result, http::HttpClient};
use super::auth::GoogleAuth;

/// Google Calendar push notification channel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchChannel {
    pub id: String,
    pub resource_id: String,
    pub resource_uri: String,
    pub token: Option<String>,
    pub expiration: DateTime<Utc>,
}

/// Request to watch a calendar for changes
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct WatchRequest {
    id: String,
    #[serde(rename = "type")]
    channel_type: String,
    address: String,
    token: Option<String>,
    expiration: Option<i64>,
}

/// Response from watch request
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WatchResponse {
    kind: String,
    id: String,
    resource_id: String,
    resource_uri: String,
    token: Option<String>,
    expiration: String,
}

/// Push notification from Google
#[derive(Debug, Clone, Deserialize)]
pub struct PushNotification {
    pub channel_id: String,
    pub channel_token: Option<String>,
    pub channel_expiration: Option<String>,
    pub resource_id: String,
    pub resource_state: String,
    pub resource_uri: String,
    pub message_number: Option<String>,
}

/// Webhook manager for Google Calendar
pub struct GoogleWebhookManager {
    auth: Arc<GoogleAuth>,
    http: HttpClient,
    webhook_endpoint: String,
}

impl GoogleWebhookManager {
    /// Create a new webhook manager
    pub fn new(
        auth: Arc<GoogleAuth>,
        http: HttpClient,
        webhook_endpoint: String,
    ) -> Self {
        Self {
            auth,
            http,
            webhook_endpoint,
        }
    }

    /// Start watching a calendar for changes
    #[instrument(skip(self, token))]
    pub async fn watch_calendar(
        &self,
        calendar_id: &str,
        token: Option<String>,
        ttl_hours: Option<i64>,
    ) -> Result<WatchChannel> {
        debug!("Setting up webhook for calendar: {}", calendar_id);

        let access_token = self.auth.get_valid_token().await?;
        
        // Generate unique channel ID
        let channel_id = Uuid::new_v4().to_string();
        
        // Calculate expiration (max 1 week for Google Calendar)
        let ttl = ttl_hours.unwrap_or(24).min(168); // Cap at 1 week
        let expiration = Utc::now() + Duration::hours(ttl);
        
        let request = WatchRequest {
            id: channel_id.clone(),
            channel_type: "web_hook".to_string(),
            address: self.webhook_endpoint.clone(),
            token: token.clone(),
            expiration: Some(expiration.timestamp_millis()),
        };

        let url = format!(
            "https://www.googleapis.com/calendar/v3/calendars/{}/events/watch",
            urlencoding::encode(calendar_id)
        );

        let response = self.http
            .client()
            .post(&url)
            .bearer_auth(access_token)
            .json(&request)
            .send()
            .await
            .map_err(|e| CalblendError::Http(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            warn!("Failed to create webhook: {} - {}", status, error_text);
            return Err(CalblendError::Provider(
                format!("Failed to create webhook: {} - {}", status, error_text)
            ));
        }

        let watch_response: WatchResponse = response
            .json()
            .await
            .map_err(|e| CalblendError::Deserialization(e.to_string()))?;

        let expiration = DateTime::parse_from_rfc3339(&watch_response.expiration)
            .map_err(|e| CalblendError::Deserialization(e.to_string()))?
            .with_timezone(&Utc);

        info!("Created webhook channel {} for calendar {}", channel_id, calendar_id);

        Ok(WatchChannel {
            id: watch_response.id,
            resource_id: watch_response.resource_id,
            resource_uri: watch_response.resource_uri,
            token: watch_response.token,
            expiration,
        })
    }

    /// Stop watching a calendar
    #[instrument(skip(self))]
    pub async fn stop_watch(
        &self,
        channel_id: &str,
        resource_id: &str,
    ) -> Result<()> {
        debug!("Stopping webhook channel: {}", channel_id);

        let access_token = self.auth.get_valid_token().await?;

        let stop_request = serde_json::json!({
            "id": channel_id,
            "resourceId": resource_id,
        });

        let response = self.http
            .client()
            .post("https://www.googleapis.com/calendar/v3/channels/stop")
            .bearer_auth(access_token)
            .json(&stop_request)
            .send()
            .await
            .map_err(|e| CalblendError::Http(e.to_string()))?;

        if !response.status().is_success() && response.status() != 404 {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            warn!("Failed to stop webhook: {} - {}", status, error_text);
            return Err(CalblendError::Provider(
                format!("Failed to stop webhook: {} - {}", status, error_text)
            ));
        }

        info!("Stopped webhook channel: {}", channel_id);
        Ok(())
    }

    /// Verify webhook notification (validate token)
    pub fn verify_notification(
        &self,
        notification: &PushNotification,
        expected_token: Option<&str>,
    ) -> bool {
        match (expected_token, &notification.channel_token) {
            (Some(expected), Some(actual)) => expected == actual,
            (None, None) => true,
            _ => false,
        }
    }

    /// Check if a channel needs renewal (within 24 hours of expiry)
    pub fn needs_renewal(channel: &WatchChannel) -> bool {
        let time_until_expiry = channel.expiration.signed_duration_since(Utc::now());
        time_until_expiry < Duration::hours(24)
    }

    /// Parse webhook headers into notification
    pub fn parse_notification_headers(
        headers: &HeaderMap,
    ) -> Result<PushNotification> {
        let channel_id = headers
            .get("x-goog-channel-id")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| CalblendError::Provider("Missing channel ID".to_string()))?
            .to_string();

        let resource_id = headers
            .get("x-goog-resource-id")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| CalblendError::Provider("Missing resource ID".to_string()))?
            .to_string();

        let resource_state = headers
            .get("x-goog-resource-state")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| CalblendError::Provider("Missing resource state".to_string()))?
            .to_string();

        let resource_uri = headers
            .get("x-goog-resource-uri")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| CalblendError::Provider("Missing resource URI".to_string()))?
            .to_string();

        Ok(PushNotification {
            channel_id,
            channel_token: headers
                .get("x-goog-channel-token")
                .and_then(|v| v.to_str().ok())
                .map(String::from),
            channel_expiration: headers
                .get("x-goog-channel-expiration")
                .and_then(|v| v.to_str().ok())
                .map(String::from),
            resource_id,
            resource_state,
            resource_uri,
            message_number: headers
                .get("x-goog-message-number")
                .and_then(|v| v.to_str().ok())
                .map(String::from),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_needs_renewal() {
        let channel = WatchChannel {
            id: "test".to_string(),
            resource_id: "test".to_string(),
            resource_uri: "test".to_string(),
            token: None,
            expiration: Utc::now() + Duration::hours(12),
        };
        
        assert!(GoogleWebhookManager::needs_renewal(&channel));

        let channel = WatchChannel {
            id: "test".to_string(),
            resource_id: "test".to_string(),
            resource_uri: "test".to_string(),
            token: None,
            expiration: Utc::now() + Duration::hours(48),
        };
        
        assert!(!GoogleWebhookManager::needs_renewal(&channel));
    }

    #[test]
    fn test_verify_notification() {
        let manager = GoogleWebhookManager::new(
            Arc::new(GoogleAuth::new(
                "".to_string(),
                "".to_string(),
                "".to_string(),
                Arc::new(crate::auth::test_utils::InMemoryTokenStorage::new()),
                HttpClient::new(&crate::CalblendConfig::default()).unwrap(),
            )),
            HttpClient::new(&crate::CalblendConfig::default()).unwrap(),
            "http://localhost/webhook".to_string(),
        );

        let notification = PushNotification {
            channel_id: "test".to_string(),
            channel_token: Some("secret".to_string()),
            channel_expiration: None,
            resource_id: "test".to_string(),
            resource_state: "exists".to_string(),
            resource_uri: "test".to_string(),
            message_number: None,
        };

        assert!(manager.verify_notification(&notification, Some("secret")));
        assert!(!manager.verify_notification(&notification, Some("wrong")));
        assert!(!manager.verify_notification(&notification, None));
    }
}