//! Google Calendar API client

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, instrument};

use crate::{
    CalblendError, Result, FreeBusyPeriod, BusyStatus,
    http::{HttpClient, RateLimiter, map_google_error},
};

use super::auth::GoogleAuth;
use super::models::{GoogleCalendar, GoogleEvent, GoogleFreeBusyRequest, GoogleFreeBusyResponse, GoogleFreeBusyItem};

/// Google Calendar API client
pub struct GoogleCalendarApi {
    auth: Arc<GoogleAuth>,
    pub(crate) http: HttpClient,
    rate_limiter: RateLimiter,
}

impl GoogleCalendarApi {
    const BASE_URL: &'static str = "https://www.googleapis.com/calendar/v3";
    
    /// Google API rate limits: 1,000,000 quota units per day
    /// Most read operations cost 1 unit, writes cost 50 units
    /// We'll limit to 100 requests per second to be safe
    const RATE_LIMIT_MAX_REQUESTS: u32 = 100;
    const RATE_LIMIT_WINDOW_SECS: u64 = 1;

    pub fn new(auth: Arc<GoogleAuth>, http_client: HttpClient) -> Self {
        Self {
            auth,
            http: http_client,
            rate_limiter: RateLimiter::new(
                Self::RATE_LIMIT_MAX_REQUESTS,
                Self::RATE_LIMIT_WINDOW_SECS,
            ),
        }
    }

    /// Make an authenticated GET request
    #[instrument(skip(self))]
    async fn get<T: for<'de> Deserialize<'de>>(&self, url: &str) -> Result<T> {
        self.rate_limiter.check_rate_limit().await;
        
        let access_token = self.auth.get_access_token().await?;
        let response = self.http.client()
            .get(url)
            .bearer_auth(&access_token)
            .send()
            .await
            .map_err(|e| CalblendError::InternalError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(map_google_error(status, &body));
        }

        response
            .json()
            .await
            .map_err(|e| CalblendError::InternalError(e.to_string()))
    }

    /// Make an authenticated POST request
    #[instrument(skip(self, body))]
    async fn post<T: Serialize, R: for<'de> Deserialize<'de>>(
        &self,
        url: &str,
        body: &T,
    ) -> Result<R> {
        self.rate_limiter.check_rate_limit().await;
        
        let access_token = self.auth.get_access_token().await?;
        let response = self.http.client()
            .post(url)
            .bearer_auth(&access_token)
            .json(body)
            .send()
            .await
            .map_err(|e| CalblendError::InternalError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(map_google_error(status, &body));
        }

        response
            .json()
            .await
            .map_err(|e| CalblendError::InternalError(e.to_string()))
    }

    /// Make an authenticated PUT request
    #[instrument(skip(self, body))]
    async fn put<T: Serialize, R: for<'de> Deserialize<'de>>(
        &self,
        url: &str,
        body: &T,
    ) -> Result<R> {
        self.rate_limiter.check_rate_limit().await;
        
        let access_token = self.auth.get_access_token().await?;
        let response = self.http.client()
            .put(url)
            .bearer_auth(&access_token)
            .json(body)
            .send()
            .await
            .map_err(|e| CalblendError::InternalError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(map_google_error(status, &body));
        }

        response
            .json()
            .await
            .map_err(|e| CalblendError::InternalError(e.to_string()))
    }

    /// Make an authenticated DELETE request
    #[instrument(skip(self))]
    async fn delete(&self, url: &str) -> Result<()> {
        self.rate_limiter.check_rate_limit().await;
        
        let access_token = self.auth.get_access_token().await?;
        let response = self.http.client()
            .delete(url)
            .bearer_auth(&access_token)
            .send()
            .await
            .map_err(|e| CalblendError::InternalError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(map_google_error(status, &body));
        }

        Ok(())
    }

    /// List user's calendars
    #[instrument(skip(self))]
    pub async fn list_calendars(&self) -> Result<Vec<GoogleCalendar>> {
        let url = format!("{}/users/me/calendarList", Self::BASE_URL);
        
        #[derive(Deserialize)]
        struct CalendarListResponse {
            items: Vec<GoogleCalendar>,
            #[serde(rename = "nextPageToken")]
            next_page_token: Option<String>,
        }

        let mut calendars = Vec::new();
        let mut page_token: Option<String> = None;

        loop {
            let mut url = url.clone();
            if let Some(token) = &page_token {
                url.push_str(&format!("?pageToken={}", token));
            }

            let response: CalendarListResponse = self.get(&url).await?;
            calendars.extend(response.items);

            match response.next_page_token {
                Some(token) => page_token = Some(token),
                None => break,
            }
        }

        debug!("Listed {} calendars", calendars.len());
        Ok(calendars)
    }

    /// List events from a calendar
    #[instrument(skip(self))]
    pub async fn list_events(
        &self,
        calendar_id: &str,
        start: Option<DateTime<Utc>>,
        end: Option<DateTime<Utc>>,
    ) -> Result<Vec<GoogleEvent>> {
        let mut url = format!("{}/calendars/{}/events", Self::BASE_URL, calendar_id);
        let mut params = Vec::new();

        if let Some(start) = start {
            params.push(format!("timeMin={}", start.to_rfc3339()));
        }
        if let Some(end) = end {
            params.push(format!("timeMax={}", end.to_rfc3339()));
        }
        params.push("singleEvents=true".to_string());
        params.push("orderBy=startTime".to_string());

        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }

        #[derive(Deserialize)]
        struct EventListResponse {
            items: Vec<GoogleEvent>,
            #[serde(rename = "nextPageToken")]
            next_page_token: Option<String>,
        }

        let mut events = Vec::new();
        let mut page_token: Option<String> = None;

        loop {
            let mut paginated_url = url.clone();
            if let Some(token) = &page_token {
                paginated_url.push_str(&format!("&pageToken={}", token));
            }

            let response: EventListResponse = self.get(&paginated_url).await?;
            events.extend(response.items);

            match response.next_page_token {
                Some(token) => page_token = Some(token),
                None => break,
            }
        }

        debug!("Listed {} events", events.len());
        Ok(events)
    }

    /// Create a new event
    #[instrument(skip(self, event))]
    pub async fn create_event(
        &self,
        calendar_id: &str,
        event: GoogleEvent,
    ) -> Result<GoogleEvent> {
        let url = format!("{}/calendars/{}/events", Self::BASE_URL, calendar_id);
        self.post(&url, &event).await
    }

    /// Update an existing event
    #[instrument(skip(self, event))]
    pub async fn update_event(
        &self,
        calendar_id: &str,
        event_id: &str,
        event: GoogleEvent,
    ) -> Result<GoogleEvent> {
        let url = format!("{}/calendars/{}/events/{}", Self::BASE_URL, calendar_id, event_id);
        self.put(&url, &event).await
    }

    /// Delete an event
    #[instrument(skip(self))]
    pub async fn delete_event(&self, calendar_id: &str, event_id: &str) -> Result<()> {
        let url = format!("{}/calendars/{}/events/{}", Self::BASE_URL, calendar_id, event_id);
        self.delete(&url).await
    }

    /// Get free/busy information
    #[instrument(skip(self))]
    pub async fn get_free_busy(
        &self,
        calendar_ids: &[String],
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<FreeBusyPeriod>> {
        let url = format!("{}/freeBusy", Self::BASE_URL);
        
        let request = GoogleFreeBusyRequest {
            time_min: start.to_rfc3339(),
            time_max: end.to_rfc3339(),
            items: calendar_ids
                .iter()
                .map(|id| GoogleFreeBusyItem { id: id.clone() })
                .collect(),
        };

        let response: GoogleFreeBusyResponse = self.post(&url, &request).await?;

        let mut periods = Vec::new();
        for (_calendar_id, calendar_data) in response.calendars {
            for busy in calendar_data.busy {
                periods.push(FreeBusyPeriod {
                    start: busy.start.parse().map_err(|e| {
                        CalblendError::InvalidData(format!("Invalid date format: {}", e))
                    })?,
                    end: busy.end.parse().map_err(|e| {
                        CalblendError::InvalidData(format!("Invalid date format: {}", e))
                    })?,
                    status: BusyStatus::Busy,
                });
            }
        }

        Ok(periods)
    }
}