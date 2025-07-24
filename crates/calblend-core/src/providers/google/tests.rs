//! Tests for Google Calendar provider

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::{
        auth::{test_utils::InMemoryTokenStorage, TokenData},
        CalendarSource, EventMoment,
    };
    use chrono::{DateTime, Utc};
    use std::sync::Arc;
    use wiremock::{Mock, MockServer, ResponseTemplate};
    use wiremock::matchers::{method, path, bearer_token};

    async fn setup_mock_provider() -> (GoogleCalendarProvider, MockServer) {
        let mock_server = MockServer::start().await;
        let token_storage = Arc::new(InMemoryTokenStorage::default());
        
        // Store a test token
        let token = TokenData {
            access_token: "test_access_token".to_string(),
            refresh_token: Some("test_refresh_token".to_string()),
            expires_at: Some(Utc::now() + chrono::Duration::hours(1)),
            token_type: "Bearer".to_string(),
            scope: Some("https://www.googleapis.com/auth/calendar".to_string()),
        };
        token_storage.save_token(CalendarSource::Google, token).await.unwrap();

        let config = CalblendConfig::default();
        let provider = GoogleCalendarProvider::new(
            "test_client_id".to_string(),
            "test_client_secret".to_string(),
            "http://localhost:8080/callback".to_string(),
            token_storage,
            config,
        ).unwrap();

        (provider, mock_server)
    }

    #[tokio::test]
    async fn test_list_calendars() {
        let (provider, mock_server) = setup_mock_provider().await;

        Mock::given(method("GET"))
            .and(path("/calendar/v3/users/me/calendarList"))
            .and(bearer_token("test_access_token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "items": [
                    {
                        "id": "primary",
                        "summary": "My Primary Calendar",
                        "description": "Main calendar",
                        "backgroundColor": "#4285F4",
                        "primary": true,
                        "accessRole": "owner"
                    },
                    {
                        "id": "work@example.com",
                        "summary": "Work Calendar",
                        "backgroundColor": "#DB4437",
                        "primary": false,
                        "accessRole": "writer"
                    }
                ]
            })))
            .mount(&mock_server)
            .await;

        // Override the base URL for testing
        // Note: In a real implementation, we'd make the base URL configurable

        let calendars = provider.list_calendars().await.unwrap();
        assert_eq!(calendars.len(), 2);
        
        let primary = &calendars[0];
        assert_eq!(primary.id, "primary");
        assert_eq!(primary.name, "My Primary Calendar");
        assert!(primary.is_primary);
        assert!(primary.can_write);
        
        let work = &calendars[1];
        assert_eq!(work.id, "work@example.com");
        assert_eq!(work.name, "Work Calendar");
        assert!(!work.is_primary);
        assert!(work.can_write);
    }

    #[tokio::test]
    async fn test_create_event() {
        let (provider, mock_server) = setup_mock_provider().await;

        let new_event = UnifiedCalendarEvent::new(
            "temp_id".to_string(),
            CalendarSource::Google,
            EventMoment {
                date_time: DateTime::parse_from_rfc3339("2024-01-20T10:00:00-08:00").unwrap(),
                time_zone: Some("America/Los_Angeles".to_string()),
                all_day: Some(false),
            },
            EventMoment {
                date_time: DateTime::parse_from_rfc3339("2024-01-20T11:00:00-08:00").unwrap(),
                time_zone: Some("America/Los_Angeles".to_string()),
                all_day: Some(false),
            },
        );

        Mock::given(method("POST"))
            .and(path("/calendar/v3/calendars/primary/events"))
            .and(bearer_token("test_access_token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": "generated_event_id",
                "summary": null,
                "start": {
                    "dateTime": "2024-01-20T10:00:00-08:00",
                    "timeZone": "America/Los_Angeles"
                },
                "end": {
                    "dateTime": "2024-01-20T11:00:00-08:00",
                    "timeZone": "America/Los_Angeles"
                },
                "status": "confirmed",
                "created": "2024-01-15T12:00:00Z",
                "updated": "2024-01-15T12:00:00Z"
            })))
            .mount(&mock_server)
            .await;

        let created = provider.create_event("primary", new_event).await.unwrap();
        assert_eq!(created.id, "generated_event_id");
        assert_eq!(created.source, CalendarSource::Google);
    }

    #[tokio::test]
    async fn test_auth_url_generation() {
        let token_storage = Arc::new(InMemoryTokenStorage::default());
        let config = CalblendConfig::default();
        let provider = GoogleCalendarProvider::new(
            "test_client_id".to_string(),
            "test_client_secret".to_string(),
            "http://localhost:8080/callback".to_string(),
            token_storage,
            config,
        ).unwrap();

        let auth_url = provider.get_auth_url().await.unwrap();
        
        // Verify the URL contains expected components
        assert!(auth_url.contains("https://accounts.google.com/o/oauth2/v2/auth"));
        assert!(auth_url.contains("client_id=test_client_id"));
        assert!(auth_url.contains("redirect_uri=http%3A%2F%2Flocalhost%3A8080%2Fcallback"));
        assert!(auth_url.contains("response_type=code"));
        assert!(auth_url.contains("scope="));
        assert!(auth_url.contains("code_challenge=")); // PKCE
    }
}